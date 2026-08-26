#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf 'NTPRO 启动失败：%s\n' "$1" >&2
  exit "${2:-1}"
}

require_file() {
  local path="$1"
  local label="$2"
  [[ -f "$path" ]] || fail "缺少${label}：${path}。请重新获取完整交付目录。" 66
}

require_executable() {
  local path="$1"
  local label="$2"
  [[ -x "$path" ]] || fail "缺少可执行的${label}：${path}。请重新获取完整交付目录。" 66
}

launcher_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
package_root="$launcher_dir"
nautilus_bin="$package_root/bin/nautilus"
node_bin="$package_root/bin/ntpro-node"
node_config="$package_root/configs/nodes/btc-ema-shadow.toml"
backtest_config="$package_root/configs/backtests/ema-cross-btcusdt-product.toml"
frontend_dist="$package_root/apps/strategy-workbench/dist"

require_executable "$nautilus_bin" "NTPRO 主程序"
require_executable "$node_bin" "NTPRO 节点程序"
require_file "$node_config" "节点配置"
require_file "$backtest_config" "回测配置"
require_file "$frontend_dist/index.html" "策略工作台页面"

case "$(uname -s)" in
  Darwin)
    default_data_root="${HOME}/Library/Application Support/NTPRO"
    ;;
  *)
    default_data_root="${XDG_DATA_HOME:-${HOME}/.local/share}/ntpro"
    ;;
esac

workspace="${NTPRO_WORKSPACE:-${default_data_root}/usable-product-v1}"
bind="${NTPRO_BIND:-127.0.0.1:5173}"
node_id="${NTPRO_NODE_ID:-mvp-node-001}"
startup_timeout_ms="${NTPRO_STARTUP_TIMEOUT_MS:-10000}"
node_max_runtime_ms="${NTPRO_NODE_MAX_RUNTIME_MS:-86400000}"
node_shutdown_timeout_ms="${NTPRO_NODE_SHUTDOWN_TIMEOUT_MS:-10000}"

if [[ ! "$bind" =~ ^(127\.0\.0\.1):([0-9]{1,5})$ ]]; then
  fail "NTPRO_BIND 必须是本机地址，例如 127.0.0.1:5173。" 64
fi
bind_host="${BASH_REMATCH[1]}"
bind_port="${BASH_REMATCH[2]}"
if (( bind_port < 1 || bind_port > 65535 )); then
  fail "端口必须在 1 到 65535 之间。" 64
fi
for value in "$startup_timeout_ms" "$node_max_runtime_ms" "$node_shutdown_timeout_ms"; do
  [[ "$value" =~ ^[1-9][0-9]*$ ]] || fail "运行超时参数必须是大于 0 的整数。" 64
done

mkdir -p "$workspace"
lock_dir="$workspace/.local-delivery.lock"
lock_owner="$lock_dir/owner.pid"

acquire_lock() {
  local existing_pid=""
  if mkdir "$lock_dir" 2>/dev/null; then
    printf '%s\n' "$$" >"$lock_owner"
    return
  fi

  if [[ -f "$lock_owner" ]]; then
    existing_pid="$(tr -dc '0-9' <"$lock_owner")"
  fi
  if [[ -n "$existing_pid" ]] && kill -0 "$existing_pid" 2>/dev/null; then
    fail "该工作区已经有一个 NTPRO 实例在运行（PID ${existing_pid}）。请使用现有页面，或先在原终端按 Ctrl-C 停止。" 73
  fi

  rm -rf "$lock_dir"
  mkdir "$lock_dir" || fail "无法创建运行锁：${lock_dir}。" 73
  printf '%s\n' "$$" >"$lock_owner"
  printf 'NTPRO 检测到上次异常退出，已清理失效运行锁。\n'
}

cleanup_lock() {
  local owner=""
  if [[ -f "$lock_owner" ]]; then
    owner="$(tr -dc '0-9' <"$lock_owner")"
  fi
  if [[ "$owner" == "$$" ]]; then
    rm -rf "$lock_dir"
  fi
}

acquire_lock
trap cleanup_lock EXIT

# Bash 的本机 TCP 探测只用于给出更直接的端口占用说明；真正的监听仍由 Axum 完成。
if (exec 3<>"/dev/tcp/${bind_host}/${bind_port}") 2>/dev/null; then
  exec 3>&- 3<&-
  fail "端口 ${bind_port} 已被占用。请关闭占用程序，或设置 NTPRO_BIND=127.0.0.1:其他端口。" 69
fi

child_pid=""
forward_signal() {
  local signal="$1"
  if [[ -n "$child_pid" ]] && kill -0 "$child_pid" 2>/dev/null; then
    kill -s "$signal" "$child_pid" 2>/dev/null || true
  fi
}
trap 'forward_signal INT' INT
trap 'forward_signal TERM' TERM
trap 'forward_signal HUP' HUP

printf 'NTPRO 正在启动...\n'
printf '工作区：%s\n' "$workspace"
printf '访问地址：http://%s/strategy-workbench/overview\n' "$bind"
printf '停止方式：回到本终端按 Ctrl-C。\n'

"$nautilus_bin" mvp serve \
  --config "$node_config" \
  --workspace "$workspace" \
  --node-id "$node_id" \
  --bind "$bind" \
  --strategy-workbench-dist "$frontend_dist" \
  --ntpro-node-bin "$node_bin" \
  --startup-timeout-ms "$startup_timeout_ms" \
  --node-max-runtime-ms "$node_max_runtime_ms" \
  --node-shutdown-timeout-ms "$node_shutdown_timeout_ms" &
child_pid="$!"
printf '%s\n' "$child_pid" >"$lock_dir/child.pid"

set +e
wait "$child_pid"
status="$?"
if kill -0 "$child_pid" 2>/dev/null; then
  wait "$child_pid"
  status="$?"
fi
set -e
child_pid=""

if (( status != 0 )); then
  printf 'NTPRO 已停止，但启动或运行过程返回错误（状态码 %s）。请查看上方错误；节点日志位于 %s/nodes/%s/logs/。\n' \
    "$status" "$workspace" "$node_id" >&2
  exit "$status"
fi

printf 'NTPRO 已安全停止。运行数据保留在：%s\n' "$workspace"
