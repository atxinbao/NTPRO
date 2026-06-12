#!/usr/bin/env bash
set -euo pipefail

# V03-015: 本地 Supervisor 控制 smoke。
# 只使用 sandbox live-init 配置和本地 artifacts，不连接真实交易所，也不提交真实订单。

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

if [[ "${NTPRO_V03_CONTROL_SKIP_BUILD:-0}" != "1" &&
      ( -z "${NTPRO_V03_NAUTILUS_BIN:-}" || -z "${NTPRO_V03_NODE_BIN:-}" ) ]]; then
  cargo build -p nautilus-cli --bin nautilus --bin ntpro-node
fi

NAUTILUS_BIN="${NTPRO_V03_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
NTPRO_NODE_BIN="${NTPRO_V03_NODE_BIN:-$ROOT_DIR/target/debug/ntpro-node}"
CONFIG="$ROOT_DIR/examples/rust/live/live_init_smoke.toml"

if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi
if [[ ! -x "$NTPRO_NODE_BIN" ]]; then
  echo "missing ntpro-node binary: $NTPRO_NODE_BIN" >&2
  exit 1
fi
if [[ ! -f "$CONFIG" ]]; then
  echo "missing live-init smoke config: $CONFIG" >&2
  exit 1
fi

SMOKE_ROOT="${NTPRO_V03_CONTROL_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v03-control.XXXXXX")}"
REGISTRY="$SMOKE_ROOT/supervisor/registry.json"
NODE_ROOT="$SMOKE_ROOT/nodes/sandbox-a"
OUTPUT_DIR="$SMOKE_ROOT/command-output"
COMMAND_LOG="$SMOKE_ROOT/commands.log"

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

cleanup() {
  set +e
  if [[ -x "$NAUTILUS_BIN" && -f "$REGISTRY" ]]; then
    "$NAUTILUS_BIN" supervisor stop --registry "$REGISTRY" --node-id sandbox-a --stop-timeout-ms 1000 >/dev/null 2>&1
  fi
}
trap cleanup EXIT

assert_output_contains() {
  local name="$1"
  local expected="$2"
  if ! grep -q "$expected" "$OUTPUT_DIR/$name.txt"; then
    echo "missing expected output '$expected' in $name" >&2
    cat "$OUTPUT_DIR/$name.txt" >&2
    exit 1
  fi
}

run_cmd register "$NAUTILUS_BIN" supervisor register \
  --registry "$REGISTRY" \
  --node-id sandbox-a \
  --config "$CONFIG" \
  --artifact-root "$NODE_ROOT"

run_cmd start "$NAUTILUS_BIN" supervisor start \
  --registry "$REGISTRY" \
  --node-id sandbox-a \
  --ntpro-node-bin "$NTPRO_NODE_BIN" \
  --startup-timeout-ms 5000
assert_output_contains start "status=ok"
assert_output_contains start "external_venue_connection=false"
assert_output_contains start "real_orders_submitted=false"

run_cmd pause "$NAUTILUS_BIN" supervisor pause \
  --registry "$REGISTRY" \
  --node-id sandbox-a
assert_output_contains pause "lifecycle_state=paused"

run_cmd reconnect_data "$NAUTILUS_BIN" supervisor reconnect-data \
  --registry "$REGISTRY" \
  --node-id sandbox-a
assert_output_contains reconnect_data "status=not_supported"
assert_output_contains reconnect_data "data_connection=not_supported"
assert_output_contains reconnect_data "real_orders_submitted=false"

run_cmd reconnect_execution "$NAUTILUS_BIN" supervisor reconnect-execution \
  --registry "$REGISTRY" \
  --node-id sandbox-a
assert_output_contains reconnect_execution "status=not_supported"
assert_output_contains reconnect_execution "execution_connection=not_supported"
assert_output_contains reconnect_execution "real_orders_submitted=false"

run_cmd connections "$NAUTILUS_BIN" supervisor connections \
  --registry "$REGISTRY" \
  --node-id sandbox-a
assert_output_contains connections "data_connection=not_supported"
assert_output_contains connections "execution_connection=not_supported"
assert_output_contains connections "external_venue_connection=false"
assert_output_contains connections "real_orders_submitted=false"

run_cmd execution "$NAUTILUS_BIN" supervisor execution \
  --registry "$REGISTRY" \
  --node-id sandbox-a
assert_output_contains execution "connection=not_supported"

run_cmd resume "$NAUTILUS_BIN" supervisor resume \
  --registry "$REGISTRY" \
  --node-id sandbox-a
assert_output_contains resume "lifecycle_state=running"

python3 - "$NODE_ROOT/logs/events.log" <<'PY'
import sys
from pathlib import Path

events = Path(sys.argv[1]).read_text()
for marker in [
    "phase=pause status=ok",
    "phase=reconnect_data status=not_supported",
    "phase=reconnect_execution status=not_supported",
    "phase=resume status=ok",
]:
    if marker not in events:
        raise SystemExit(f"missing pre-stop event marker: {marker}")

print("pre_stop_event_assertions=ok")
PY

run_cmd stop "$NAUTILUS_BIN" supervisor stop \
  --registry "$REGISTRY" \
  --node-id sandbox-a \
  --stop-timeout-ms 5000
assert_output_contains stop "lifecycle_state=stopped"
assert_output_contains stop "external_venue_connection=false"
assert_output_contains stop "real_orders_submitted=false"

python3 - "$NODE_ROOT/status.json" "$NODE_ROOT/metrics.json" "$NODE_ROOT/logs/events.log" <<'PY'
import json
import sys
from pathlib import Path

status = json.loads(Path(sys.argv[1]).read_text())
metrics = json.loads(Path(sys.argv[2]).read_text())
events = Path(sys.argv[3]).read_text()

if status["external_venue_connection"] is not False:
    raise SystemExit("status.external_venue_connection is not false")
if status["real_orders_submitted"] is not False:
    raise SystemExit("status.real_orders_submitted is not false")
if metrics["external_venue_connection"] is not False:
    raise SystemExit("metrics.external_venue_connection is not false")
if metrics["real_orders_submitted"] is not False:
    raise SystemExit("metrics.real_orders_submitted is not false")
for marker in [
    "phase=stop status=ok",
]:
    if marker not in events:
        raise SystemExit(f"missing event marker: {marker}")

print("artifact_assertions=ok")
PY

echo "v03_supervisor_control_smoke status=ok root=$SMOKE_ROOT registry=$REGISTRY node=sandbox-a"
