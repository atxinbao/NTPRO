#!/usr/bin/env bash
set -euo pipefail

# V05-008: local Binance sandbox workflow artifact smoke.
# This gate only validates local fixture/mock artifacts. It must never connect
# to Binance, use real funds, submit real orders, or claim production trading.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V05_WORKFLOW_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V05_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V05_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"

if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

SMOKE_ROOT="${NTPRO_V05_WORKFLOW_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v05-workflow.XXXXXX")}"
RUN_ID="${NTPRO_V05_WORKFLOW_RUN_ID:-v05-smoke}"
WORKFLOW_DIR="$SMOKE_ROOT/workflows/$RUN_ID"
OUTPUT_DIR="$SMOKE_ROOT/command-output"

mkdir -p "$OUTPUT_DIR"

assert_output_contains() {
  local file="$1"
  local expected="$2"
  if ! grep -q "$expected" "$file"; then
    echo "missing expected output '$expected' in $file" >&2
    cat "$file" >&2
    exit 1
  fi
}

run_and_capture() {
  local name="$1"
  shift
  echo "+ $*"
  "$@" | tee "$OUTPUT_DIR/$name.txt"
  local status="${PIPESTATUS[0]}"
  return "$status"
}

run_and_capture workflow_help "$NAUTILUS_BIN" workflow --help
assert_output_contains "$OUTPUT_DIR/workflow_help.txt" "run"

run_and_capture workflow_run_help "$NAUTILUS_BIN" workflow run --help
assert_output_contains "$OUTPUT_DIR/workflow_run_help.txt" "binance-sandbox"
assert_output_contains "$OUTPUT_DIR/workflow_run_help.txt" "run-id"
assert_output_contains "$OUTPUT_DIR/workflow_run_help.txt" "output"

run_and_capture workflow_run "$NAUTILUS_BIN" workflow run \
  --workflow binance-sandbox \
  --run-id "$RUN_ID" \
  --output "$WORKFLOW_DIR"

assert_output_contains "$OUTPUT_DIR/workflow_run.txt" "workflow.run status=ok"
assert_output_contains "$OUTPUT_DIR/workflow_run.txt" "workflow=binance-sandbox"
assert_output_contains "$OUTPUT_DIR/workflow_run.txt" "external_venue_connection=false"
assert_output_contains "$OUTPUT_DIR/workflow_run.txt" "real_funds=false"
assert_output_contains "$OUTPUT_DIR/workflow_run.txt" "production_trading=false"
assert_output_contains "$OUTPUT_DIR/workflow_run.txt" "real_orders_submitted=false"
assert_output_contains "$OUTPUT_DIR/workflow_run.txt" "runtime_status=completed"

python3 - "$WORKFLOW_DIR" "$RUN_ID" <<'PY'
import json
import sys
from pathlib import Path

workflow_dir = Path(sys.argv[1])
run_id = sys.argv[2]

expected_files = [
    "boundary.json",
    "events.jsonl",
    "manifest.json",
    "market/replay.json",
    "orders/mock_lifecycle.json",
    "risk/rejection.json",
    "strategies/ema.json",
    "strategies/rsi.json",
    "summary.json",
]

for relative in expected_files:
    path = workflow_dir / relative
    if not path.is_file():
        raise SystemExit(f"missing workflow artifact: {path}")

manifest = json.loads((workflow_dir / "manifest.json").read_text())
summary = json.loads((workflow_dir / "summary.json").read_text())
boundary = json.loads((workflow_dir / "boundary.json").read_text())
events = [
    json.loads(line)
    for line in (workflow_dir / "events.jsonl").read_text().splitlines()
    if line.strip()
]

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(manifest["schema_version"] == "ntpro.workflow_manifest.v1", manifest)
require(manifest["workflow"] == "binance-sandbox", manifest)
require(manifest["run_id"] == run_id, manifest)
require(manifest["runtime_status"] == "completed", manifest)
require(manifest["artifact_count"] == 9, manifest)
require(len(manifest["artifacts"]) == 9, manifest["artifacts"])
require({item["path"] for item in manifest["artifacts"]} == set(expected_files), manifest["artifacts"])

for name, payload in [("summary", summary), ("boundary", boundary)]:
    require(payload["sandbox_only"] is True, f"{name}.sandbox_only must be true")
    require(payload["fixture_replay"] is True, f"{name}.fixture_replay must be true")
    require(payload["mock_execution"] is True, f"{name}.mock_execution must be true")
    require(payload["external_venue_connection"] is False, f"{name}.external_venue_connection must be false")
    require(payload["real_funds"] is False, f"{name}.real_funds must be false")
    require(payload["production_trading"] is False, f"{name}.production_trading must be false")
    require(payload["real_orders_submitted"] is False, f"{name}.real_orders_submitted must be false")

require(boundary["testnet_connection"] is False, "boundary.testnet_connection must be false")
require(len(events) == 7, f"expected 7 events, got {len(events)}")
require(events[-1]["event_type"] == "workflow.events.ready", events[-1])
for index, event in enumerate(events, start=1):
    require(event["sequence"] == index, f"event sequence mismatch: {event}")
    require(event["status"] == "ok", f"event status mismatch: {event}")
    require(event["sandbox_only"] is True, f"event sandbox_only mismatch: {event}")
    require(event["real_orders_submitted"] is False, f"event real_orders_submitted mismatch: {event}")

print(
    "v05_workflow_artifact_assertions status=ok "
    f"run_id={run_id} artifact_count={manifest['artifact_count']} events={len(events)}"
)
PY

echo "v05_workflow_artifacts_smoke status=ok root=$SMOKE_ROOT workflow_dir=$WORKFLOW_DIR"
