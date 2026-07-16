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

jq -e '
  [.nodes["sandbox-a"], .nodes["sandbox-b"]] as $nodes
  | .nodes["sandbox-a"].node_id == "sandbox-a"
  and .nodes["sandbox-b"].node_id == "sandbox-b"
  and ($nodes | all(
      .process.state == "running"
      and (.process.pid.value | type == "number" and . > 0)
    ))
  and ($nodes[0].process.pid.value != $nodes[1].process.pid.value)
  and ([ $nodes[] | .artifact_root, .pid_path, .status_path, .metrics_path,
         .stdout_log_path, .stderr_log_path, .events_log_path ] | unique | length == 14)
' "$REGISTRY" >/dev/null

jq '{"sandbox-a": .nodes["sandbox-a"].process.pid.value, "sandbox-b": .nodes["sandbox-b"].process.pid.value}' \
  "$REGISTRY" >"$RUNNING_PIDS"

for node_id in sandbox-a sandbox-b; do
  for field in pid_path status_path metrics_path stdout_log_path stderr_log_path; do
    artifact="$(jq -er --arg node "$node_id" --arg field "$field" '.nodes[$node][$field]' "$REGISTRY")"
    [[ -e "$artifact" ]] || {
      echo "$node_id: missing running artifact $field=$artifact" >&2
      exit 1
    }
  done
  status_path="$(jq -er --arg node "$node_id" '.nodes[$node].status_path' "$REGISTRY")"
  metrics_path="$(jq -er --arg node "$node_id" '.nodes[$node].metrics_path' "$REGISTRY")"
  jq -e --arg node "$node_id" '
    .node_id == $node
    and .lifecycle_state == "running"
    and .process_mode == "spawned_process"
    and .external_venue_connection == false
    and .real_orders_submitted == false
  ' "$status_path" >/dev/null
  jq -e --arg node "$node_id" '
    .node_id == $node
    and .lifecycle_state == "running"
    and .starts_total == 1
    and .stops_total == 0
    and .state_transitions_total == 1
    and .external_venue_connection == false
    and .real_orders_submitted == false
  ' "$metrics_path" >/dev/null
done
echo "running_artifact_assertions status=ok nodes=sandbox-a,sandbox-b"

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

for node_id in sandbox-a sandbox-b; do
  jq -e --arg node "$node_id" '.nodes[$node].process.state == "stopped"' "$REGISTRY" >/dev/null
  pid_path="$(jq -er --arg node "$node_id" '.nodes[$node].pid_path' "$REGISTRY")"
  [[ ! -e "$pid_path" ]] || {
    echo "$node_id: pid artifact should be removed after stop" >&2
    exit 1
  }
  status_path="$(jq -er --arg node "$node_id" '.nodes[$node].status_path' "$REGISTRY")"
  metrics_path="$(jq -er --arg node "$node_id" '.nodes[$node].metrics_path' "$REGISTRY")"
  stdout_path="$(jq -er --arg node "$node_id" '.nodes[$node].stdout_log_path' "$REGISTRY")"
  stderr_path="$(jq -er --arg node "$node_id" '.nodes[$node].stderr_log_path' "$REGISTRY")"
  events_path="$(jq -er --arg node "$node_id" '.nodes[$node].events_log_path' "$REGISTRY")"
  jq -e --arg node "$node_id" '
    .node_id == $node
    and .lifecycle_state == "stopped"
    and .process_mode == "spawned_process"
    and .external_venue_connection == false
    and .real_orders_submitted == false
  ' "$status_path" >/dev/null
  jq -e --arg node "$node_id" '
    .node_id == $node
    and .lifecycle_state == "stopped"
    and .starts_total == 1
    and .stops_total == 1
    and .state_transitions_total == 2
    and .external_venue_connection == false
    and .real_orders_submitted == false
  ' "$metrics_path" >/dev/null
  grep -F "ntpro-node.run status=ok" "$stdout_path" >/dev/null
  [[ ! -s "$stderr_path" ]] || {
    echo "$node_id: stderr log is not empty" >&2
    exit 1
  }
  grep -F "phase=start status=ok" "$events_path" >/dev/null
  grep -F "phase=stop status=ok" "$events_path" >/dev/null
  pid="$(jq -er --arg node "$node_id" '.[$node]' "$RUNNING_PIDS")"
  if kill -0 "$pid" 2>/dev/null; then
    echo "$node_id: process $pid is still alive after supervisor stop" >&2
    exit 1
  fi
done
echo "stopped_artifact_assertions status=ok nodes=sandbox-a,sandbox-b"

echo "v02_two_node_smoke status=ok root=$SMOKE_ROOT registry=$REGISTRY nodes=sandbox-a,sandbox-b"
