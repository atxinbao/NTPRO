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
lock_child="$lock_dir/child.pid"

read_pid_file() {
  local file="$1"
  if [[ -f "$file" ]]; then
    tr -dc '0-9' <"$file"
  fi
}

process_alive() {
  [[ -n "$1" ]] && kill -0 "$1" 2>/dev/null
}

wait_for_process_exit() {
  local pid="$1"
  local attempts="${2:-50}"
  local attempt=0
  while process_alive "$pid" && (( attempt < attempts )); do
    sleep 0.1
    attempt=$((attempt + 1))
  done
  ! process_alive "$pid"
}

acquire_lock() {
  local attempt=0
  local existing_owner=""
  local existing_child=""
  local quarantine=""

  while (( attempt < 3 )); do
    if mkdir "$lock_dir" 2>/dev/null; then
      printf '%s\n' "$$" >"$lock_owner"
      return
    fi
    if [[ -L "$lock_dir" || ! -d "$lock_dir" ]]; then
      fail "运行锁路径不是可信目录：${lock_dir}。请保留现场并联系维护人员。" 73
    fi

    existing_owner="$(read_pid_file "$lock_owner")"
    existing_child="$(read_pid_file "$lock_child")"
    if process_alive "$existing_owner"; then
      fail "该工作区已经有一个 NTPRO 实例在运行（PID ${existing_owner}）。请使用现有页面，或先在原终端按 Ctrl-C 停止。" 73
    fi
    if process_alive "$existing_child" && ! wait_for_process_exit "$existing_child" 50; then
      fail "上次启动器已退出，但 NTPRO 正在安全收口（PID ${existing_child}）。请稍等几秒后重试。" 75
    fi

    # 原子移动确保多个并发启动器中只有一个能接管同一失效锁。
    quarantine="${lock_dir}.stale.$$.$RANDOM"
    if mv "$lock_dir" "$quarantine" 2>/dev/null; then
      existing_owner="$(read_pid_file "$quarantine/owner.pid")"
      existing_child="$(read_pid_file "$quarantine/child.pid")"
      if process_alive "$existing_owner" || process_alive "$existing_child"; then
        mv "$quarantine" "$lock_dir" 2>/dev/null || true
        fail "检测到仍存活的 NTPRO 进程，拒绝接管工作区。请稍等后重试。" 75
      fi
      if mkdir "$lock_dir" 2>/dev/null; then
        printf '%s\n' "$$" >"$lock_owner"
        rm -rf "$quarantine"
        printf 'NTPRO 检测到上次异常退出，已原子接管失效运行锁。\n'
        return
      fi
      rm -rf "$quarantine"
    fi
    attempt=$((attempt + 1))
    sleep 0.1
  done

  fail "另一个启动器正在接管该工作区，请稍等几秒后重试。" 75
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

child_pid=""
guardian_pid=""

request_graceful_shutdown() {
  if process_alive "$child_pid"; then
    kill -INT "$child_pid" 2>/dev/null || true
  fi
}

stop_guardian() {
  if process_alive "$guardian_pid"; then
    kill -KILL "$guardian_pid" 2>/dev/null || true
    wait "$guardian_pid" 2>/dev/null || true
  fi
  guardian_pid=""
}

cleanup_launcher() {
  if process_alive "$child_pid"; then
    kill -INT "$child_pid" 2>/dev/null || true
    wait_for_process_exit "$child_pid" 100 || kill -KILL "$child_pid" 2>/dev/null || true
  fi
  stop_guardian
  cleanup_lock
}
trap cleanup_launcher EXIT

# Bash 的本机 TCP 探测只用于给出更直接的端口占用说明；真正的监听仍由 Axum 完成。
if (exec 3<>"/dev/tcp/${bind_host}/${bind_port}") 2>/dev/null; then
  exec 3>&- 3<&-
  fail "端口 ${bind_port} 已被占用。请关闭占用程序，或设置 NTPRO_BIND=127.0.0.1:其他端口。" 69
fi

trap request_graceful_shutdown INT
trap request_graceful_shutdown TERM
trap request_graceful_shutdown HUP

printf 'NTPRO 正在启动...\n'
printf '工作区：%s\n' "$workspace"
printf '页面入口：服务就绪后，请打开下方 strategy_workbench_url 的完整地址。\n'
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
printf '%s\n' "$child_pid" >"$lock_child"

guardian_parent_pid="$$"
guardian_child_pid="$child_pid"
(
  trap - EXIT INT TERM HUP
  while process_alive "$guardian_parent_pid"; do
    sleep 0.2
  done
  if process_alive "$guardian_child_pid"; then
    kill -INT "$guardian_child_pid" 2>/dev/null || true
    wait_for_process_exit "$guardian_child_pid" 150 \
      || kill -KILL "$guardian_child_pid" 2>/dev/null \
      || true
  fi
  recorded_owner="$(read_pid_file "$lock_owner")"
  recorded_child="$(read_pid_file "$lock_child")"
  if [[ "$recorded_owner" == "$guardian_parent_pid" && "$recorded_child" == "$guardian_child_pid" ]]; then
    rm -rf "$lock_dir"
  fi
) &
guardian_pid="$!"

set +e
wait "$child_pid"
status="$?"
if process_alive "$child_pid"; then
  wait "$child_pid"
  status="$?"
fi
set -e
child_pid=""
stop_guardian
cleanup_lock
trap - EXIT

if (( status != 0 )); then
  printf 'NTPRO 已停止，但启动或运行过程返回错误（状态码 %s）。请查看上方错误；节点日志位于 %s/nodes/%s/logs/。\n' \
    "$status" "$workspace" "$node_id" >&2
  exit "$status"
fi

printf 'NTPRO 已安全停止。运行数据保留在：%s\n' "$workspace"
