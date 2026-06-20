#!/usr/bin/env bash
set -euo pipefail

# V03-010: 本地 Dashboard MVP smoke。
# 只使用 sandbox live-init 配置和本地 supervisor artifacts，不连接真实交易所，也不提交真实订单。

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
PWCLI="$CODEX_HOME/skills/playwright/scripts/playwright_cli.sh"
HAS_PLAYWRIGHT_WRAPPER=0
if [[ -x "$PWCLI" ]]; then
  HAS_PLAYWRIGHT_WRAPPER=1
  if ! command -v npx >/dev/null 2>&1; then
    echo "missing npx; Playwright wrapper requires Node.js/npm" >&2
    exit 1
  fi
elif [[ "${NTPRO_V03_DASHBOARD_REQUIRE_PLAYWRIGHT:-0}" == "1" ]]; then
  echo "missing Playwright wrapper: $PWCLI" >&2
  exit 1
else
  echo "Playwright wrapper unavailable; using API/HTML dashboard smoke fallback: $PWCLI"
fi

if [[ "${NTPRO_V03_010_SKIP_BUILD:-0}" != "1" &&
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

SMOKE_ROOT="${NTPRO_V03_010_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v03-010.XXXXXX")}"
REGISTRY="$SMOKE_ROOT/supervisor/registry.json"
NODE_A_ROOT="$SMOKE_ROOT/nodes/sandbox-a"
NODE_B_ROOT="$SMOKE_ROOT/nodes/sandbox-b"
OUTPUT_DIR="$SMOKE_ROOT/command-output"
ARTIFACT_DIR="$SMOKE_ROOT/browser"
COMMAND_LOG="$SMOKE_ROOT/commands.log"
SERVER_LOG="$SMOKE_ROOT/dashboard-server.log"
DASHBOARD_PID=""
PW_SESSION="v0310$$"
PW_TMPDIR="${NTPRO_PLAYWRIGHT_TMPDIR:-/tmp}"

mkdir -p "$OUTPUT_DIR" "$ARTIFACT_DIR"
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
  if [[ "$HAS_PLAYWRIGHT_WRAPPER" == "1" ]]; then
    TMPDIR="$PW_TMPDIR" "$PWCLI" --session "$PW_SESSION" close >/dev/null 2>&1
    rm -rf "$ROOT_DIR/.playwright-cli"
  fi
  if [[ -n "$DASHBOARD_PID" ]]; then
    kill "$DASHBOARD_PID" >/dev/null 2>&1
    wait "$DASHBOARD_PID" >/dev/null 2>&1
  fi
  if [[ -x "$NAUTILUS_BIN" && -f "$REGISTRY" ]]; then
    "$NAUTILUS_BIN" supervisor stop --registry "$REGISTRY" --node-id sandbox-a --stop-timeout-ms 1000 >/dev/null 2>&1
    "$NAUTILUS_BIN" supervisor stop --registry "$REGISTRY" --node-id sandbox-b --stop-timeout-ms 1000 >/dev/null 2>&1
  fi
}
trap cleanup EXIT

run_pw() {
  if [[ "$HAS_PLAYWRIGHT_WRAPPER" != "1" ]]; then
    echo "Playwright wrapper unavailable: $PWCLI" >&2
    exit 1
  fi
  TMPDIR="$PW_TMPDIR" "$PWCLI" --session "$PW_SESSION" "$@"
}

assert_playwright_output() {
  local path="$1"
  if grep -q '^### Error' "$path"; then
    cat "$path" >&2
    exit 1
  fi
}

choose_port() {
  python3 - <<'PY'
import socket

sock = socket.socket()
sock.bind(("127.0.0.1", 0))
print(sock.getsockname()[1])
sock.close()
PY
}

wait_http() {
  local url="$1"
  local attempts="${2:-80}"
  for _ in $(seq 1 "$attempts"); do
    if python3 - "$url" <<'PY' >/dev/null 2>&1
import sys
import urllib.request

with urllib.request.urlopen(sys.argv[1], timeout=1) as response:
    if response.status < 400:
        raise SystemExit(0)
raise SystemExit(1)
PY
    then
      return 0
    fi
    sleep 0.25
  done
  return 1
}

run_api_dashboard_smoke() {
  local bind="$1"
  local artifact_dir="$2"

  python3 - "$bind" "$artifact_dir" <<'PY'
import json
import sys
import time
import urllib.request
from pathlib import Path

bind = sys.argv[1]
artifact_dir = Path(sys.argv[2])
artifact_dir.mkdir(parents=True, exist_ok=True)
base_url = f"http://{bind}"


def write_json(name, payload):
    (artifact_dir / name).write_text(
        json.dumps(payload, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def request_text(path):
    with urllib.request.urlopen(f"{base_url}{path}", timeout=5) as response:
        return response.read().decode("utf-8")


def request_json(path):
    with urllib.request.urlopen(f"{base_url}{path}", timeout=5) as response:
        return json.load(response)


def post_json(path):
    request = urllib.request.Request(f"{base_url}{path}", method="POST")
    with urllib.request.urlopen(request, timeout=15) as response:
        return response.status, json.load(response)


def require(condition, message):
    if not condition:
        raise SystemExit(message)


def node_states(snapshot):
    return {node["node_id"]: node["lifecycle_state"] for node in snapshot["nodes"]}


def assert_snapshot_contract(snapshot):
    node_ids = sorted(node["node_id"] for node in snapshot["nodes"])
    require(node_ids == ["sandbox-a", "sandbox-b"], f"unexpected node ids: {node_ids}")
    overview = snapshot["overview"]
    require(overview["node_count"] == 2, f"unexpected node count: {overview}")
    require(overview["running_nodes"] == 1, f"unexpected running count: {overview}")
    require(overview["stopped_nodes"] == 1, f"unexpected stopped count: {overview}")
    for section in [
        "data_sources",
        "execution_gateways",
        "runtime_modules",
        "logs",
        "metrics",
        "gaps",
    ]:
        require(
            isinstance(snapshot.get(section), list) and len(snapshot[section]) > 0,
            f"dashboard snapshot missing section {section}",
        )
    require(isinstance(snapshot.get("production_shadow"), list), "dashboard snapshot missing production_shadow section")
    require(snapshot.get("risk", {}).get("health") is not None, "dashboard snapshot missing risk summary")


def assert_initial_controls(snapshot):
    controls = {control["action"]: control for control in snapshot["controls"]}
    expected = {
        "start:sandbox-a": True,
        "stop:sandbox-a": False,
        "pause:sandbox-a": False,
        "resume:sandbox-a": False,
        "reconnect_data:sandbox-a": False,
        "reconnect_execution:sandbox-a": False,
        "start:sandbox-b": False,
        "stop:sandbox-b": True,
        "pause:sandbox-b": True,
        "resume:sandbox-b": False,
        "reconnect_data:sandbox-b": True,
        "reconnect_execution:sandbox-b": True,
    }
    for action, enabled in expected.items():
        control = controls.get(action)
        require(control is not None, f"missing control state {action}")
        require(control["enabled"] is enabled, f"unexpected control enabled {action}: {control}")
        require(control["availability"] == "available", f"unexpected control availability {action}: {control}")


def wait_state(node_id, expected_state, label, timeout_seconds=20):
    deadline = time.time() + timeout_seconds
    last_states = {}
    while time.time() < deadline:
        snapshot = request_json("/api/snapshot")
        last_states = node_states(snapshot)
        if last_states.get(node_id) == expected_state:
            return snapshot
        time.sleep(0.25)
    raise SystemExit(f"{label} timed out; last states: {last_states}")


def run_action(node_id, action, expected_status, expected_state):
    status, payload = post_json(f"/api/nodes/{node_id}/actions/{action}")
    write_json(f"api-action-{action}-{node_id}.json", payload)
    require(status == 200, f"{action} {node_id} returned HTTP {status}: {payload}")
    require(payload["status"] == expected_status, f"{action} {node_id} status mismatch: {payload}")
    require(payload["current_state"] == expected_state, f"{action} {node_id} state mismatch: {payload}")
    return payload


html = request_text("/dashboard")
(artifact_dir / "api-html-shell.html").write_text(html, encoding="utf-8")
for required in [
    "NTPRO 监督器控制台",
    "control-result",
    "dashboard.js",
]:
    require(required in html, f"dashboard shell missing {required}")

dashboard_js = request_text("/assets/dashboard.js")
(artifact_dir / "api-dashboard.js").write_text(dashboard_js, encoding="utf-8")
for required in [
    "/api/snapshot",
    "data-dashboard-action",
    "reconnect_data",
]:
    require(required in dashboard_js, f"dashboard JS missing {required}")

initial = request_json("/api/snapshot")
write_json("api-initial-snapshot.json", initial)
assert_snapshot_contract(initial)
assert_initial_controls(initial)

run_action("sandbox-b", "reconnect_data", "not_supported", "running")
run_action("sandbox-b", "reconnect_execution", "not_supported", "running")
run_action("sandbox-b", "pause", "succeeded", "paused")
wait_state("sandbox-b", "paused", "pause sandbox-b through dashboard API")
run_action("sandbox-b", "resume", "succeeded", "running")
wait_state("sandbox-b", "running", "resume sandbox-b through dashboard API")
run_action("sandbox-a", "start", "succeeded", "running")
wait_state("sandbox-a", "running", "start sandbox-a through dashboard API")
run_action("sandbox-b", "stop", "succeeded", "stopped")
final = wait_state("sandbox-b", "stopped", "stop sandbox-b through dashboard API")
write_json("api-final-snapshot.json", final)

states = node_states(final)
require(states == {"sandbox-a": "running", "sandbox-b": "stopped"}, f"unexpected final states: {states}")
write_json(
    "api-fallback-summary.json",
    {
        "status": "ok",
        "mode": "api-html-fallback",
        "node_states": states,
        "browser_wrapper": "not_available",
    },
)
print(f"api_html_dashboard_smoke status=ok final_states={states}")
PY
}

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

run_cmd start_b "$NAUTILUS_BIN" supervisor start \
  --registry "$REGISTRY" \
  --node-id sandbox-b \
  --ntpro-node-bin "$NTPRO_NODE_BIN" \
  --startup-timeout-ms 5000

PORT="$(choose_port)"
BIND="127.0.0.1:$PORT"
DASHBOARD_URL="http://$BIND/dashboard"

echo "+ $NAUTILUS_BIN dashboard serve --registry $REGISTRY --bind $BIND --ntpro-node-bin $NTPRO_NODE_BIN" | tee -a "$COMMAND_LOG"
"$NAUTILUS_BIN" dashboard serve \
  --registry "$REGISTRY" \
  --bind "$BIND" \
  --ntpro-node-bin "$NTPRO_NODE_BIN" \
  >"$SERVER_LOG" 2>&1 &
DASHBOARD_PID="$!"

if ! wait_http "http://$BIND/api/snapshot" 80; then
  echo "dashboard server did not become ready; log=$SERVER_LOG" >&2
  cat "$SERVER_LOG" >&2 || true
  exit 1
fi

if [[ "$HAS_PLAYWRIGHT_WRAPPER" != "1" ]]; then
  run_api_dashboard_smoke "$BIND" "$ARTIFACT_DIR"
  echo "v03_dashboard_smoke status=ok mode=api-html-fallback root=$SMOKE_ROOT registry=$REGISTRY dashboard_url=$DASHBOARD_URL artifacts=$ARTIFACT_DIR nodes=sandbox-a,sandbox-b"
  exit 0
fi

run_pw open "$DASHBOARD_URL" >"$ARTIFACT_DIR/open.txt"
run_pw resize 1440 1000 >"$ARTIFACT_DIR/resize-desktop.txt"
run_pw snapshot >"$ARTIFACT_DIR/desktop-snapshot.txt"

run_pw eval "$(cat <<'JS'
(async () => {
  const waitFor = async (predicate, label, timeoutMs = 15000) => {
    const start = Date.now();
    let lastError = "";
    while (Date.now() - start < timeoutMs) {
      try {
        if (await predicate()) return;
      } catch (error) {
        lastError = error.message;
      }
      await new Promise((resolve) => setTimeout(resolve, 250));
    }
    throw new Error(`${label} timed out ${lastError}`);
  };

  await waitFor(
    () => document.body.innerText.includes("sandbox-a") && document.body.innerText.includes("sandbox-b"),
    "dashboard node render",
  );

  const requiredText = [
    "概览",
    "节点",
    "控制",
    "数据源",
    "执行网关",
    "风控引擎",
    "日志",
    "指标",
    "运行模块",
    "Production Shadow",
    "待补能力",
  ];
  const bodyText = document.body.innerText;
  const missingText = requiredText.filter((text) => !bodyText.includes(text));
  if (missingText.length > 0) {
    throw new Error(`missing dashboard text: ${missingText.join(", ")}`);
  }

  const snapshot = await fetch("/api/snapshot").then((response) => response.json());
  const nodeIds = snapshot.nodes.map((node) => node.node_id).sort();
  if (JSON.stringify(nodeIds) !== JSON.stringify(["sandbox-a", "sandbox-b"])) {
    throw new Error(`unexpected node ids: ${JSON.stringify(nodeIds)}`);
  }
  if (snapshot.overview.node_count !== 2 || snapshot.overview.running_nodes !== 1 || snapshot.overview.stopped_nodes !== 1) {
    throw new Error(`unexpected overview counts: ${JSON.stringify(snapshot.overview)}`);
  }
  for (const section of ["data_sources", "execution_gateways", "runtime_modules", "logs", "metrics", "gaps"]) {
    if (!Array.isArray(snapshot[section]) || snapshot[section].length === 0) {
      throw new Error(`dashboard snapshot missing section ${section}`);
    }
  }
  if (!Array.isArray(snapshot.production_shadow)) {
    throw new Error("dashboard snapshot missing production_shadow section");
  }
  if (!snapshot.risk || snapshot.risk.health === undefined) {
    throw new Error("dashboard snapshot missing risk summary");
  }

  const controls = new Map(snapshot.controls.map((control) => [control.action, control]));
  const expectedControls = {
    "start:sandbox-a": true,
    "stop:sandbox-a": false,
    "pause:sandbox-a": false,
    "resume:sandbox-a": false,
    "reconnect_data:sandbox-a": false,
    "reconnect_execution:sandbox-a": false,
    "start:sandbox-b": false,
    "stop:sandbox-b": true,
    "pause:sandbox-b": true,
    "resume:sandbox-b": false,
    "reconnect_data:sandbox-b": true,
    "reconnect_execution:sandbox-b": true,
  };
  for (const [action, enabled] of Object.entries(expectedControls)) {
    const control = controls.get(action);
    if (!control || control.enabled !== enabled || control.availability !== "available") {
      throw new Error(`unexpected control state ${action}: ${JSON.stringify(control)}`);
    }
  }

  const clickControl = async (action, nodeId, label, expectedStatus = "成功") => {
    const selector = `button[data-dashboard-action="${action}"][data-node-id="${nodeId}"]`;
    await waitFor(() => {
      const candidate = document.querySelector(selector);
      return Boolean(candidate && !candidate.disabled);
    }, `${label} button enabled`);
    const button = document.querySelector(selector);
    const result = document.getElementById("control-result");
    result.innerHTML = "";
    button.click();
    await waitFor(
      () => {
        const text = result.innerText;
        if (text.includes(expectedStatus)) return true;
        if (text.includes("控制失败") || text.includes("失败") || text.includes("已拒绝")) {
          throw new Error(`result text: ${text}`);
        }
        return false;
      },
      `${label} control result`,
    );
    await waitFor(
      () => !button.isConnected || !button.disabled,
      `${label} dashboard refresh`,
    );
  };

  await clickControl("reconnect_data", "sandbox-b", "reconnect data sandbox-b", "不支持");
  await clickControl("reconnect_execution", "sandbox-b", "reconnect execution sandbox-b", "不支持");

  await clickControl("pause", "sandbox-b", "pause sandbox-b");
  await waitFor(async () => {
    const next = await fetch("/api/snapshot").then((response) => response.json());
    return next.nodes.find((node) => node.node_id === "sandbox-b")?.lifecycle_state === "paused";
  }, "pause sandbox-b through dashboard control");

  await clickControl("resume", "sandbox-b", "resume sandbox-b");
  await waitFor(async () => {
    const next = await fetch("/api/snapshot").then((response) => response.json());
    return next.nodes.find((node) => node.node_id === "sandbox-b")?.lifecycle_state === "running";
  }, "resume sandbox-b through dashboard control");

  await clickControl("start", "sandbox-a", "start sandbox-a");
  await waitFor(async () => {
    const next = await fetch("/api/snapshot").then((response) => response.json());
    return next.nodes.find((node) => node.node_id === "sandbox-a")?.lifecycle_state === "running";
  }, "start sandbox-a through dashboard control");

  await clickControl("stop", "sandbox-b", "stop sandbox-b");
  await waitFor(async () => {
    const next = await fetch("/api/snapshot").then((response) => response.json());
    return next.nodes.find((node) => node.node_id === "sandbox-b")?.lifecycle_state === "stopped";
  }, "stop sandbox-b through dashboard control");

  const after = await fetch("/api/snapshot").then((response) => response.json());
  const states = Object.fromEntries(after.nodes.map((node) => [node.node_id, node.lifecycle_state]));
  if (states["sandbox-a"] !== "running" || states["sandbox-b"] !== "stopped") {
    throw new Error(`unexpected post-control states: ${JSON.stringify(states)}`);
  }

  const visibleButtons = Array.from(document.querySelectorAll("button")).filter((button) => {
    const rect = button.getBoundingClientRect();
    return rect.width > 0 && rect.height > 0;
  });
  const invalidButtons = visibleButtons.filter((button) => button.scrollWidth > Math.ceil(button.clientWidth + 1));
  if (invalidButtons.length > 0) {
    throw new Error(`button text overflow: ${invalidButtons.map((button) => button.textContent.trim()).join(", ")}`);
  }
  return {
    status: "ok",
    initial_nodes: nodeIds,
    final_states: states,
    required_text: requiredText,
  };
})()
JS
)" >"$ARTIFACT_DIR/desktop-assertions.txt"
assert_playwright_output "$ARTIFACT_DIR/desktop-assertions.txt"

run_pw run-code "async (page) => { await page.screenshot({ path: '$ARTIFACT_DIR/desktop.png', fullPage: true }); }" >"$ARTIFACT_DIR/desktop-screenshot.txt"
assert_playwright_output "$ARTIFACT_DIR/desktop-screenshot.txt"

run_pw resize 390 844 >"$ARTIFACT_DIR/resize-mobile.txt"
run_pw snapshot >"$ARTIFACT_DIR/mobile-snapshot.txt"
run_pw eval "$(cat <<'JS'
(() => {
  const width = document.documentElement.clientWidth;
  const overflow = document.documentElement.scrollWidth - width;
  if (overflow > 2) {
    throw new Error(`mobile body horizontal overflow: ${overflow}`);
  }
  const requiredIds = ["overview", "nodes", "controls", "data-sources", "execution-gateways", "risk", "runtime-modules", "production-shadow", "logs-metrics", "gaps"];
  for (const id of requiredIds) {
    const el = document.getElementById(id);
    const rect = el?.getBoundingClientRect();
    if (!el || !rect || rect.width <= 0 || rect.height <= 0) {
      throw new Error(`mobile panel is not visible: ${id}`);
    }
  }
  const buttons = Array.from(document.querySelectorAll("button"));
  const overflowingButtons = buttons.filter((button) => button.scrollWidth > Math.ceil(button.clientWidth + 1));
  if (overflowingButtons.length > 0) {
    throw new Error(`mobile button overflow: ${overflowingButtons.map((button) => button.textContent.trim()).join(", ")}`);
  }
  return {
    status: "ok",
    viewport_width: width,
    body_horizontal_overflow: overflow,
    panel_count: requiredIds.length,
    button_count: buttons.length,
  };
})()
JS
)" >"$ARTIFACT_DIR/mobile-layout.txt"
assert_playwright_output "$ARTIFACT_DIR/mobile-layout.txt"
run_pw run-code "async (page) => { await page.screenshot({ path: '$ARTIFACT_DIR/mobile.png', fullPage: true }); }" >"$ARTIFACT_DIR/mobile-screenshot.txt"
assert_playwright_output "$ARTIFACT_DIR/mobile-screenshot.txt"

python3 - "$BIND" "$ARTIFACT_DIR/final-snapshot.json" <<'PY'
import json
import sys
import urllib.request
from pathlib import Path

bind = sys.argv[1]
target = Path(sys.argv[2])
with urllib.request.urlopen(f"http://{bind}/api/snapshot", timeout=2) as response:
    snapshot = json.load(response)
target.write_text(json.dumps(snapshot, indent=2, ensure_ascii=False) + "\n")

states = {node["node_id"]: node["lifecycle_state"] for node in snapshot["nodes"]}
if states != {"sandbox-a": "running", "sandbox-b": "stopped"}:
    raise SystemExit(f"unexpected final dashboard states: {states}")
print(f"final_dashboard_states={states}")
PY

echo "v03_dashboard_smoke status=ok root=$SMOKE_ROOT registry=$REGISTRY dashboard_url=$DASHBOARD_URL artifacts=$ARTIFACT_DIR nodes=sandbox-a,sandbox-b"
