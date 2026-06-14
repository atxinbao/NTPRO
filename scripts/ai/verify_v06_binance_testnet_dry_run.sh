#!/usr/bin/env bash
set -euo pipefail

# V06-009: Binance testnet runtime foundation smoke.
# This gate validates the offline dry-run contract only. It does not connect to
# Binance testnet, does not read or persist credential values, and does not
# submit real orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V06_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V06_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V06_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
CONFIG="${NTPRO_V06_CONFIG:-$ROOT_DIR/examples/rust/binance/testnet_dry_run.toml}"

if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi
if [[ ! -f "$CONFIG" ]]; then
  echo "missing Binance testnet dry-run config: $CONFIG" >&2
  exit 1
fi

SMOKE_ROOT="${NTPRO_V06_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v06-testnet.XXXXXX")}"
RUN_ID="${NTPRO_V06_RUN_ID:-v06-smoke}"
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

run_and_capture workflow_run_help "$NAUTILUS_BIN" workflow run --help
assert_output_contains "$OUTPUT_DIR/workflow_run_help.txt" "binance-testnet"
assert_output_contains "$OUTPUT_DIR/workflow_run_help.txt" "connectivity-probe"
assert_output_contains "$OUTPUT_DIR/workflow_run_help.txt" "allow-testnet-network"

run_and_capture workflow_run "$NAUTILUS_BIN" workflow run \
  --workflow binance-testnet \
  --mode dry-run \
  --config "$CONFIG" \
  --run-id "$RUN_ID" \
  --output "$WORKFLOW_DIR"

assert_output_contains "$OUTPUT_DIR/workflow_run.txt" "workflow.run status=ok"
assert_output_contains "$OUTPUT_DIR/workflow_run.txt" "workflow=binance-testnet"
assert_output_contains "$OUTPUT_DIR/workflow_run.txt" "external_venue_connection=false"
assert_output_contains "$OUTPUT_DIR/workflow_run.txt" "real_funds=false"
assert_output_contains "$OUTPUT_DIR/workflow_run.txt" "production_trading=false"
assert_output_contains "$OUTPUT_DIR/workflow_run.txt" "real_orders_submitted=false"
assert_output_contains "$OUTPUT_DIR/workflow_run.txt" "testnet_connection=false"
assert_output_contains "$OUTPUT_DIR/workflow_run.txt" "runtime_status=dry_run_completed"

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
    "orders/reconciliation.json",
    "orders/testnet_dry_run_lifecycle.json",
    "summary.json",
    "testnet/config.json",
    "testnet/connectivity_probe.json",
    "testnet/credential_policy.json",
    "testnet/ws_connectivity_probe.json",
]

for relative in expected_files:
    path = workflow_dir / relative
    if not path.is_file():
        raise SystemExit(f"missing workflow artifact: {path}")

manifest = json.loads((workflow_dir / "manifest.json").read_text())
summary = json.loads((workflow_dir / "summary.json").read_text())
boundary = json.loads((workflow_dir / "boundary.json").read_text())
policy = json.loads((workflow_dir / "testnet/credential_policy.json").read_text())
probe = json.loads((workflow_dir / "testnet/connectivity_probe.json").read_text())
ws_probe = json.loads((workflow_dir / "testnet/ws_connectivity_probe.json").read_text())
lifecycle = json.loads((workflow_dir / "orders/testnet_dry_run_lifecycle.json").read_text())
reconciliation = json.loads((workflow_dir / "orders/reconciliation.json").read_text())
events = [
    json.loads(line)
    for line in (workflow_dir / "events.jsonl").read_text().splitlines()
    if line.strip()
]

def require(condition, message):
    if not condition:
        raise SystemExit(message)

require(manifest["schema_version"] == "ntpro.workflow_manifest.v1", manifest)
require(manifest["workflow"] == "binance-testnet", manifest)
require(manifest["run_id"] == run_id, manifest)
require(manifest["runtime_status"] == "dry_run_completed", manifest)
require(manifest["artifact_count"] == 10, manifest)
require({item["path"] for item in manifest["artifacts"]} == set(expected_files), manifest["artifacts"])

for item in manifest["artifacts"]:
    artifact_path = workflow_dir / item["path"]
    expected_schema = item["schema_version"]
    if item["path"].endswith(".jsonl"):
        records = 0
        for line_number, line in enumerate(artifact_path.read_text().splitlines(), start=1):
            if not line.strip():
                continue
            records += 1
            payload = json.loads(line)
            require(
                payload.get("schema_version") == expected_schema,
                f"{item['path']} line {line_number} schema mismatch: {payload}",
            )
        require(records > 0, f"{item['path']} must contain at least one JSONL record")
    else:
        payload = json.loads(artifact_path.read_text())
        require(
            payload.get("schema_version") == expected_schema,
            f"{item['path']} schema mismatch: {payload}",
        )

for name, payload in [("summary", summary), ("boundary", boundary)]:
    require(payload["sandbox_only"] is True, f"{name}.sandbox_only must be true")
    require(payload["mock_execution"] is True, f"{name}.mock_execution must be true")
    require(payload["external_venue_connection"] is False, f"{name}.external_venue_connection must be false")
    require(payload["real_funds"] is False, f"{name}.real_funds must be false")
    require(payload["production_trading"] is False, f"{name}.production_trading must be false")
    require(payload["real_orders_submitted"] is False, f"{name}.real_orders_submitted must be false")
    require(payload["testnet_connection"] is False, f"{name}.testnet_connection must be false")
    require(payload["network_attempted"] is False, f"{name}.network_attempted must be false")
    require(payload["credential_policy"] == "env-var-only-no-secret-persistence", payload)
    require(payload["connectivity_mode"] == "dry-run", payload)
    require(payload["order_submission_mode"] == "disabled", payload)
    require(payload["reconciliation_mode"] == "artifact-only", payload)

require(policy["values_in_file"] is False, policy)
require(policy["values_recorded"] is False, policy)
require(policy["secrets_redacted"] is True, policy)
require(probe["network_attempted"] is False, probe)
require(probe["testnet_connection"] is False, probe)
require(probe["status"] == "dry_run_completed", probe)
require(ws_probe["schema_version"] == "ntpro.v07_binance_testnet_ws_probe.v1", ws_probe)
require(ws_probe["endpoint_kind"] == "websocket_read_only", ws_probe)
require(ws_probe["websocket_probe_gate"] == "manual-online-only", ws_probe)
require(ws_probe["websocket_attempted"] is False, ws_probe)
require(ws_probe["network_attempted"] is False, ws_probe)
require(ws_probe["testnet_connection"] is False, ws_probe)
require(ws_probe["subscription_attempted"] is False, ws_probe)
require(ws_probe["message_count"] == 0, ws_probe)
require(ws_probe["order_submission"] == "disabled", ws_probe)
require(ws_probe["real_orders_submitted"] is False, ws_probe)
require(ws_probe["values_recorded"] is False, ws_probe)
require(ws_probe["secrets_redacted"] is True, ws_probe)
require(ws_probe["status"] == "websocket_read_only_probe_deferred", ws_probe)
require(ws_probe["error_code"] == "network_gate_blocked", ws_probe)
require(str(ws_probe["generated_at"]).startswith("unix:"), ws_probe)
require(lifecycle["submitted_count"] == 0, lifecycle)
require(lifecycle["real_orders_submitted"] is False, lifecycle)
require(reconciliation["external_account_state_loaded"] is False, reconciliation)
require(reconciliation["real_orders_submitted"] is False, reconciliation)
require(len(events) == 8, f"expected 8 events, got {len(events)}")
require(events[0]["event_type"] == "workflow.testnet_config.ready", events[0])
require(events[-1]["event_type"] == "workflow.events.ready", events[-1])
for event in events:
    require(event["sandbox_only"] is True, event)
    require(event["real_orders_submitted"] is False, event)

print(
    "v06_binance_testnet_dry_run_assertions status=ok "
    f"run_id={run_id} artifact_count={manifest['artifact_count']} events={len(events)} "
    "testnet_connection=false real_orders_submitted=false"
)
PY

echo "v06_binance_testnet_dry_run_smoke status=ok root=$SMOKE_ROOT workflow_dir=$WORKFLOW_DIR"
