#!/usr/bin/env bash
set -euo pipefail

# V091-009: Strategy Runtime Supervisor + Dashboard integration smoke.
# This gate is fully offline: it runs the BTC EMA shadow config through the
# local supervisor, reads dashboard snapshots from loopback, and asserts that no
# real order submission or external venue connection is exposed.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V091_SKIP_BUILD:-0}" != "1" &&
      ( -z "${NTPRO_V091_NAUTILUS_BIN:-}" || -z "${NTPRO_V091_NTPRO_NODE_BIN:-}" ) ]]; then
  cargo build -p nautilus-cli --bin nautilus --bin ntpro-node
fi

NAUTILUS_BIN="${NTPRO_V091_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
NTPRO_NODE_BIN="${NTPRO_V091_NTPRO_NODE_BIN:-$ROOT_DIR/target/debug/ntpro-node}"
CONFIG="${NTPRO_V091_CONFIG:-$ROOT_DIR/configs/nodes/btc-ema-shadow.toml}"

if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi
if [[ ! -x "$NTPRO_NODE_BIN" ]]; then
  echo "missing ntpro-node binary: $NTPRO_NODE_BIN" >&2
  exit 1
fi
if [[ ! -f "$CONFIG" ]]; then
  echo "missing v0.9.1 strategy config: $CONFIG" >&2
  exit 1
fi

SMOKE_ROOT="${NTPRO_V091_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v091-integration.XXXXXX")}"
REGISTRY="$SMOKE_ROOT/supervisor/registry.json"
NODE_ID="${NTPRO_V091_NODE_ID:-btc-ema-shadow-001}"
NODE_ROOT="$SMOKE_ROOT/nodes/$NODE_ID"
OUTPUT_DIR="$SMOKE_ROOT/command-output"
COMMAND_LOG="$SMOKE_ROOT/commands.log"
DASHBOARD_OUTPUT="$OUTPUT_DIR/dashboard.txt"
DASHBOARD_PID=""
DASHBOARD_ADDR=""

mkdir -p "$OUTPUT_DIR"
: > "$COMMAND_LOG"

run_cmd() {
  local name="$1"
  shift
  echo "+ $*" | tee -a "$COMMAND_LOG"
  "$@" | tee "$OUTPUT_DIR/$name.txt"
  local status="${PIPESTATUS[0]}"
  cat "$OUTPUT_DIR/$name.txt" >> "$COMMAND_LOG"
  return "$status"
}

assert_output_contains() {
  local name="$1"
  local expected="$2"
  if ! grep -q "$expected" "$OUTPUT_DIR/$name.txt"; then
    echo "missing expected output '$expected' in $name" >&2
    cat "$OUTPUT_DIR/$name.txt" >&2
    exit 1
  fi
}

cleanup() {
  set +e
  if [[ -n "$DASHBOARD_PID" ]]; then
    kill "$DASHBOARD_PID" >/dev/null 2>&1
    wait "$DASHBOARD_PID" >/dev/null 2>&1
  fi
  if [[ -x "$NAUTILUS_BIN" && -f "$REGISTRY" ]]; then
    "$NAUTILUS_BIN" supervisor stop \
      --registry "$REGISTRY" \
      --node-id "$NODE_ID" \
      --stop-timeout-ms 1000 >/dev/null 2>&1
  fi
}
trap cleanup EXIT

wait_for_dashboard() {
  local deadline=$((SECONDS + 20))
  while (( SECONDS < deadline )); do
    if [[ -f "$DASHBOARD_OUTPUT" ]]; then
      DASHBOARD_ADDR="$(grep -Eo 'bind=127\.0\.0\.1:[0-9]+' "$DASHBOARD_OUTPUT" | head -1 | cut -d= -f2 || true)"
      if [[ -n "$DASHBOARD_ADDR" ]]; then
        if python3 - "$DASHBOARD_ADDR" <<'PY' >/dev/null 2>&1
import json
import sys
import urllib.request

addr = sys.argv[1]
with urllib.request.urlopen(f"http://{addr}/api/snapshot", timeout=1.5) as response:
    json.loads(response.read().decode("utf-8"))
PY
        then
          return 0
        fi
      fi
    fi
    sleep 0.2
  done
  echo "dashboard did not become ready" >&2
  [[ -f "$DASHBOARD_OUTPUT" ]] && cat "$DASHBOARD_OUTPUT" >&2
  exit 1
}

assert_running_artifacts() {
  local cycle="$1"
  python3 - "$NODE_ROOT" "$NODE_ID" "$cycle" <<'PY'
import json
import sys
from pathlib import Path

node_root = Path(sys.argv[1])
node_id = sys.argv[2]
cycle = sys.argv[3]


def require(condition, message):
    if not condition:
        raise SystemExit(message)


def read_json(relative: str):
    return json.loads((node_root / relative).read_text())


status = read_json("status.json")
metrics = read_json("metrics.json")
session = read_json("strategy/session_status.json")
summary = read_json("strategy/summary.json")
manifest = read_json("strategy/manifest.json")

require(status["node_id"] == node_id, status)
require(status["lifecycle_state"] == "running", status)
require(status["external_venue_connection"] is False, status)
require(status["real_orders_submitted"] is False, status)
require(metrics["node_id"] == node_id, metrics)
require(metrics["lifecycle_state"] == "running", metrics)
require(metrics["strategy_signal_count"]["value"] == summary["signal_count"], metrics)
require(metrics["strategy_rejection_count"]["value"] == summary["rejection_count"], metrics)
require(metrics["real_orders_submitted"] is False, metrics)
require(session["session_id"] == node_id, session)
require(session["state"] == "running", session)
require(summary["state"] == "running", summary)
require(summary["signal_count"] > 0, summary)
require(summary["rejection_count"] == summary["risk_decision_count"], summary)
require(summary["actual_submission_count"] == 0, summary)
require(manifest["schema_version"] == "ntpro.v091_strategy_session_manifest.v1", manifest)
require(manifest["session_id"] == node_id, manifest)
require(manifest["state"] == "running", manifest)

print(
    f"running_artifact_assertions status=ok cycle={cycle} "
    f"signals={summary['signal_count']} rejections={summary['rejection_count']}"
)
PY
}

assert_stopped_artifacts() {
  local cycle="$1"
  python3 - "$NODE_ROOT" "$NODE_ID" "$cycle" <<'PY'
import json
import sys
from pathlib import Path

node_root = Path(sys.argv[1])
node_id = sys.argv[2]
cycle = sys.argv[3]


def require(condition, message):
    if not condition:
        raise SystemExit(message)


def read_json(relative: str):
    return json.loads((node_root / relative).read_text())


status = read_json("status.json")
metrics = read_json("metrics.json")
session = read_json("strategy/session_status.json")
summary = read_json("strategy/summary.json")
manifest = read_json("strategy/manifest.json")

require(status["node_id"] == node_id, status)
require(status["lifecycle_state"] == "stopped", status)
require(status["external_venue_connection"] is False, status)
require(status["real_orders_submitted"] is False, status)
require(metrics["lifecycle_state"] == "stopped", metrics)
require(metrics["real_orders_submitted"] is False, metrics)
require(session["state"] == "stopped", session)
require(summary["state"] == "stopped", summary)
require(summary["actual_submission_count"] == 0, summary)
require(manifest["schema_version"] == "ntpro.v091_strategy_session_manifest.v1", manifest)
require(manifest["session_id"] == node_id, manifest)
require(manifest["state"] == "stopped", manifest)

print(f"stopped_artifact_assertions status=ok cycle={cycle}")
PY
}

assert_dashboard_snapshot() {
  local expected_state="$1"
  local cycle="$2"
  python3 - "$DASHBOARD_ADDR" "$NODE_ID" "$expected_state" "$cycle" <<'PY'
import json
import sys
import urllib.request

addr = sys.argv[1]
node_id = sys.argv[2]
expected_state = sys.argv[3]
cycle = sys.argv[4]


def require(condition, message):
    if not condition:
        raise SystemExit(message)


with urllib.request.urlopen(f"http://{addr}/api/snapshot", timeout=3) as response:
    snapshot = json.loads(response.read().decode("utf-8"))

raw = json.dumps(snapshot, sort_keys=True)
for forbidden in [
    '"real_orders_submitted": true',
    "exchange_order_id",
    "venue_order_id",
    "api_key",
    "api_secret",
    "private_key",
]:
    require(forbidden not in raw, f"forbidden token {forbidden!r} in dashboard snapshot")

require(snapshot["overview"]["external_venue_connection"] is False, snapshot["overview"])
require(snapshot["overview"]["real_orders_submitted"] is False, snapshot["overview"])

runtimes = snapshot["strategy_runtime"]
require(len(runtimes) == 1, runtimes)
runtime = runtimes[0]
require(runtime["node_id"] == node_id, runtime)
require(runtime["health"] == "healthy", runtime)
require(runtime["session_state"]["value"] == expected_state, runtime)
require(runtime["strategy_id"]["value"] == "ema_cross_btcusdt_v1", runtime)
require(runtime["actual_submission_count"]["value"] == 0, runtime)
require(runtime["manifest_path"]["value"].endswith("strategy/manifest.json"), runtime)

print(
    f"dashboard_snapshot_assertions status=ok cycle={cycle} "
    f"session_state={expected_state} strategy_health=healthy actual_submission_count=0"
)
PY
}

run_lifecycle_cycle() {
  local cycle="$1"

  run_cmd "start_$cycle" "$NAUTILUS_BIN" supervisor start \
    --registry "$REGISTRY" \
    --node-id "$NODE_ID" \
    --ntpro-node-bin "$NTPRO_NODE_BIN" \
    --startup-timeout-ms 10000 \
    --node-heartbeat-interval-ms 100 \
    --node-max-runtime-ms 60000 \
    --node-shutdown-timeout-ms 5000
  assert_output_contains "start_$cycle" "status=ok"
  assert_output_contains "start_$cycle" "lifecycle_state=running"
  assert_output_contains "start_$cycle" "external_venue_connection=false"
  assert_output_contains "start_$cycle" "real_orders_submitted=false"

  assert_running_artifacts "$cycle"

  run_cmd "status_running_$cycle" "$NAUTILUS_BIN" supervisor status \
    --registry "$REGISTRY" \
    --node-id "$NODE_ID"
  assert_output_contains "status_running_$cycle" "strategy_session_state=running"
  assert_output_contains "status_running_$cycle" "strategy_health=healthy"
  assert_output_contains "status_running_$cycle" "external_venue_connection=false"
  assert_output_contains "status_running_$cycle" "real_orders_submitted=false"

  run_cmd "metrics_running_$cycle" "$NAUTILUS_BIN" supervisor metrics \
    --registry "$REGISTRY" \
    --node-id "$NODE_ID"
  assert_output_contains "metrics_running_$cycle" "strategy_signal_count=2"
  assert_output_contains "metrics_running_$cycle" "strategy_rejection_count=2"
  assert_output_contains "metrics_running_$cycle" "real_orders_submitted=false"

  assert_dashboard_snapshot "running" "$cycle"

  run_cmd "stop_$cycle" "$NAUTILUS_BIN" supervisor stop \
    --registry "$REGISTRY" \
    --node-id "$NODE_ID" \
    --stop-timeout-ms 10000
  assert_output_contains "stop_$cycle" "lifecycle_state=stopped"
  assert_output_contains "stop_$cycle" "external_venue_connection=false"
  assert_output_contains "stop_$cycle" "real_orders_submitted=false"

  assert_stopped_artifacts "$cycle"
  assert_dashboard_snapshot "stopped" "$cycle"
}

run_cmd register "$NAUTILUS_BIN" supervisor register \
  --registry "$REGISTRY" \
  --node-id "$NODE_ID" \
  --config "$CONFIG" \
  --artifact-root "$NODE_ROOT"
assert_output_contains register "status=ok"

"$NAUTILUS_BIN" dashboard serve \
  --registry "$REGISTRY" \
  --bind 127.0.0.1:0 \
  --ntpro-node-bin "$NTPRO_NODE_BIN" >"$DASHBOARD_OUTPUT" 2>&1 &
DASHBOARD_PID="$!"
wait_for_dashboard

run_lifecycle_cycle first
run_lifecycle_cycle restart

echo "v091_strategy_supervisor_dashboard_integration status=ok root=$SMOKE_ROOT registry=$REGISTRY node=$NODE_ID dashboard=http://$DASHBOARD_ADDR/dashboard"
