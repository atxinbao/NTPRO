#!/usr/bin/env bash
set -euo pipefail

# V02-009: 本地两节点 supervisor smoke。
# 只使用 sandbox live-init smoke 配置，不连接真实交易所，也不提交真实订单。

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_RELEASE_GATE:-0}" == "1" && "${NTPRO_V02_009_SKIP_BUILD:-0}" == "1" ]]; then
  echo "NTPRO_RELEASE_GATE=1 forbids NTPRO_V02_009_SKIP_BUILD=1" >&2
  exit 1
fi

if [[ "${NTPRO_V02_009_SKIP_BUILD:-0}" != "1" ]]; then
  cargo build -p nautilus-cli --bin nautilus --bin ntpro-node
fi

NAUTILUS_BIN="$ROOT_DIR/target/debug/nautilus"
NTPRO_NODE_BIN="$ROOT_DIR/target/debug/ntpro-node"
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

SMOKE_ROOT="${NTPRO_V02_009_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v02-009.XXXXXX")}"
REGISTRY="$SMOKE_ROOT/supervisor/registry.json"
NODE_A_ROOT="$SMOKE_ROOT/nodes/sandbox-a"
NODE_B_ROOT="$SMOKE_ROOT/nodes/sandbox-b"
OUTPUT_DIR="$SMOKE_ROOT/command-output"
COMMAND_LOG="$SMOKE_ROOT/commands.log"
RUNNING_PIDS="$SMOKE_ROOT/running-pids.json"

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
    "$NAUTILUS_BIN" supervisor stop --registry "$REGISTRY" --node-id sandbox-a --stop-timeout-ms 500 >/dev/null 2>&1
    "$NAUTILUS_BIN" supervisor stop --registry "$REGISTRY" --node-id sandbox-b --stop-timeout-ms 500 >/dev/null 2>&1
  fi
}
trap cleanup EXIT

run_cmd register_a "$NAUTILUS_BIN" supervisor register \
  --registry "$REGISTRY" \
  --node-id sandbox-a \
  --config "$CONFIG" \
  --artifact-root "$NODE_A_ROOT"

run_cmd register_b "$NAUTILUS_BIN" supervisor register \
  --registry "$REGISTRY" \
  --node-id sandbox-b \
  --config "$CONFIG" \
  --artifact-root "$NODE_B_ROOT"

run_cmd list_registered "$NAUTILUS_BIN" supervisor list --registry "$REGISTRY"

run_cmd start_a "$NAUTILUS_BIN" supervisor start \
  --registry "$REGISTRY" \
  --node-id sandbox-a \
  --ntpro-node-bin "$NTPRO_NODE_BIN" \
  --startup-timeout-ms 5000

run_cmd start_b "$NAUTILUS_BIN" supervisor start \
  --registry "$REGISTRY" \
  --node-id sandbox-b \
  --ntpro-node-bin "$NTPRO_NODE_BIN" \
  --startup-timeout-ms 5000

for node_id in sandbox-a sandbox-b; do
  run_cmd "${node_id}_status_running" "$NAUTILUS_BIN" supervisor status --registry "$REGISTRY" --node-id "$node_id"
  run_cmd "${node_id}_connections_running" "$NAUTILUS_BIN" supervisor connections --registry "$REGISTRY" --node-id "$node_id"
  run_cmd "${node_id}_execution_running" "$NAUTILUS_BIN" supervisor execution --registry "$REGISTRY" --node-id "$node_id"
  run_cmd "${node_id}_risk_running" "$NAUTILUS_BIN" supervisor risk --registry "$REGISTRY" --node-id "$node_id"
  run_cmd "${node_id}_logs_running" "$NAUTILUS_BIN" supervisor logs --registry "$REGISTRY" --node-id "$node_id"
  run_cmd "${node_id}_metrics_running" "$NAUTILUS_BIN" supervisor metrics --registry "$REGISTRY" --node-id "$node_id"
done

python3 - "$REGISTRY" "$RUNNING_PIDS" <<'PY'
import json
import os
import sys
from pathlib import Path

registry_path = Path(sys.argv[1])
running_pids_path = Path(sys.argv[2])
registry = json.loads(registry_path.read_text())
nodes = registry["nodes"]
expected = ["sandbox-a", "sandbox-b"]
missing = [node_id for node_id in expected if node_id not in nodes]
if missing:
    raise SystemExit(f"missing registry nodes: {missing}")

artifact_fields = [
    "artifact_root",
    "pid_path",
    "status_path",
    "metrics_path",
    "stdout_log_path",
    "stderr_log_path",
    "events_log_path",
]
seen_paths = {}
pids = {}

for node_id in expected:
    record = nodes[node_id]
    if record["node_id"] != node_id:
        raise SystemExit(f"{node_id}: registry node_id mismatch")
    if record["process"]["state"] != "running":
        raise SystemExit(f"{node_id}: expected running process state")
    pid = record["process"]["pid"].get("value")
    if not isinstance(pid, int) or pid <= 0:
        raise SystemExit(f"{node_id}: expected positive pid")
    pids[node_id] = pid

    for field in artifact_fields:
        value = record[field]
        if value in seen_paths:
            raise SystemExit(f"artifact path collision: {field}={value} also used by {seen_paths[value]}")
        seen_paths[value] = f"{node_id}.{field}"

    for field in ["pid_path", "status_path", "metrics_path", "stdout_log_path", "stderr_log_path"]:
        if not os.path.exists(record[field]):
            raise SystemExit(f"{node_id}: missing running artifact {field}={record[field]}")

    status = json.loads(Path(record["status_path"]).read_text())
    metrics = json.loads(Path(record["metrics_path"]).read_text())
    if status["node_id"] != node_id:
        raise SystemExit(
            f"{node_id}: runtime status identity mismatch: {status['node_id']}"
        )
    if metrics["node_id"] != node_id:
        raise SystemExit(
            f"{node_id}: runtime metrics identity mismatch: {metrics['node_id']}"
        )
    if status["lifecycle_state"] != "running":
        raise SystemExit(f"{node_id}: expected running status artifact")
    if status["process_mode"] != "spawned_process":
        raise SystemExit(f"{node_id}: expected spawned process status")
    if status["external_venue_connection"] or status["real_orders_submitted"]:
        raise SystemExit(f"{node_id}: running status claims external venue or real orders")
    if metrics["lifecycle_state"] != "running":
        raise SystemExit(f"{node_id}: expected running metrics artifact")
    if metrics["starts_total"] != 1 or metrics["stops_total"] != 0 or metrics["state_transitions_total"] != 1:
        raise SystemExit(f"{node_id}: unexpected running metric counters")
    if metrics["external_venue_connection"] or metrics["real_orders_submitted"]:
        raise SystemExit(f"{node_id}: running metrics claims external venue or real orders")

if len(set(pids.values())) != len(pids):
    raise SystemExit(f"pid collision: {pids}")

running_pids_path.write_text(json.dumps(pids, indent=2) + "\n")
print("running_artifact_assertions status=ok nodes=sandbox-a,sandbox-b")
PY

run_cmd stop_a "$NAUTILUS_BIN" supervisor stop \
  --registry "$REGISTRY" \
  --node-id sandbox-a \
  --stop-timeout-ms 5000

run_cmd stop_b "$NAUTILUS_BIN" supervisor stop \
  --registry "$REGISTRY" \
  --node-id sandbox-b \
  --stop-timeout-ms 5000

for node_id in sandbox-a sandbox-b; do
  run_cmd "${node_id}_status_stopped" "$NAUTILUS_BIN" supervisor status --registry "$REGISTRY" --node-id "$node_id"
  run_cmd "${node_id}_metrics_stopped" "$NAUTILUS_BIN" supervisor metrics --registry "$REGISTRY" --node-id "$node_id"
done

python3 - "$REGISTRY" "$RUNNING_PIDS" <<'PY'
import json
import os
import sys
from pathlib import Path

registry_path = Path(sys.argv[1])
running_pids_path = Path(sys.argv[2])
registry = json.loads(registry_path.read_text())
nodes = registry["nodes"]
running_pids = json.loads(running_pids_path.read_text())

for node_id in ["sandbox-a", "sandbox-b"]:
    record = nodes[node_id]
    if record["process"]["state"] != "stopped":
        raise SystemExit(f"{node_id}: expected stopped process state")
    if os.path.exists(record["pid_path"]):
        raise SystemExit(f"{node_id}: pid artifact should be removed after stop")

    status = json.loads(Path(record["status_path"]).read_text())
    metrics = json.loads(Path(record["metrics_path"]).read_text())
    stdout = Path(record["stdout_log_path"]).read_text()
    stderr = Path(record["stderr_log_path"]).read_text()
    events = Path(record["events_log_path"]).read_text()

    if status["node_id"] != node_id:
        raise SystemExit(
            f"{node_id}: stopped status identity mismatch: {status['node_id']}"
        )
    if metrics["node_id"] != node_id:
        raise SystemExit(
            f"{node_id}: stopped metrics identity mismatch: {metrics['node_id']}"
        )
    if status["lifecycle_state"] != "stopped":
        raise SystemExit(f"{node_id}: expected stopped status artifact")
    if status["process_mode"] != "spawned_process":
        raise SystemExit(f"{node_id}: expected spawned process final status")
    if status["external_venue_connection"] or status["real_orders_submitted"]:
        raise SystemExit(f"{node_id}: stopped status claims external venue or real orders")
    if metrics["lifecycle_state"] != "stopped":
        raise SystemExit(f"{node_id}: expected stopped metrics artifact")
    if metrics["starts_total"] != 1 or metrics["stops_total"] != 1 or metrics["state_transitions_total"] != 2:
        raise SystemExit(f"{node_id}: unexpected stopped metric counters")
    if metrics["external_venue_connection"] or metrics["real_orders_submitted"]:
        raise SystemExit(f"{node_id}: stopped metrics claims external venue or real orders")
    if "ntpro-node.run status=ok" not in stdout:
        raise SystemExit(f"{node_id}: stdout log missing ntpro-node completion")
    if stderr.strip():
        raise SystemExit(f"{node_id}: stderr log is not empty")
    if "phase=start status=ok" not in events or "phase=stop status=ok" not in events:
        raise SystemExit(f"{node_id}: events log missing start/stop phases")

    pid = running_pids[node_id]
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        pass
    except PermissionError as error:
        raise SystemExit(f"{node_id}: process {pid} still exists but cannot be inspected: {error}")
    else:
        raise SystemExit(f"{node_id}: process {pid} is still alive after supervisor stop")

print("stopped_artifact_assertions status=ok nodes=sandbox-a,sandbox-b")
PY

echo "v02_two_node_smoke status=ok root=$SMOKE_ROOT registry=$REGISTRY nodes=sandbox-a,sandbox-b"
