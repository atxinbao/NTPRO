#!/usr/bin/env bash
set -euo pipefail

# V070-006: default offline gate.
# This gate must be safe for local development and normal CI: it unsets the
# testnet network opt-in env var, opens no sockets, and proves offline artifacts
# keep network_attempted=false.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V07_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V07_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V07_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
CONFIG="${NTPRO_V07_CONFIG:-$ROOT_DIR/examples/rust/binance/testnet_dry_run.toml}"

if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi
if [[ ! -f "$CONFIG" ]]; then
  echo "missing Binance testnet config: $CONFIG" >&2
  exit 1
fi

GATE_ROOT="${NTPRO_V07_OFFLINE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v07-offline.XXXXXX")}"
OUTPUT_DIR="$GATE_ROOT/command-output"
DRY_RUN_DIR="$GATE_ROOT/workflows/v070-offline-dry-run"
BLOCKED_PROBE_DIR="$GATE_ROOT/workflows/v070-offline-blocked-probe"

mkdir -p "$OUTPUT_DIR"

run_and_capture() {
  local name="$1"
  shift
  echo "+ $*"
  "$@" | tee "$OUTPUT_DIR/$name.txt"
  local status="${PIPESTATUS[0]}"
  return "$status"
}

run_and_capture workflow_run_help "$NAUTILUS_BIN" workflow run --help
grep -q "binance-testnet" "$OUTPUT_DIR/workflow_run_help.txt"
grep -q "connectivity-probe" "$OUTPUT_DIR/workflow_run_help.txt"
grep -q "allow-testnet-network" "$OUTPUT_DIR/workflow_run_help.txt"

env -u NTPRO_ALLOW_TESTNET_NETWORK "$NAUTILUS_BIN" workflow run \
  --workflow binance-testnet \
  --mode dry-run \
  --config "$CONFIG" \
  --run-id v070-offline-dry-run \
  --output "$DRY_RUN_DIR" \
  | tee "$OUTPUT_DIR/offline_dry_run.txt"

env -u NTPRO_ALLOW_TESTNET_NETWORK "$NAUTILUS_BIN" workflow run \
  --workflow binance-testnet \
  --mode connectivity-probe \
  --config "$CONFIG" \
  --run-id v070-offline-blocked-probe \
  --output "$BLOCKED_PROBE_DIR" \
  | tee "$OUTPUT_DIR/offline_blocked_probe.txt"

python3 - "$DRY_RUN_DIR" "$BLOCKED_PROBE_DIR" <<'PY'
import json
import sys
from pathlib import Path

dry_run_dir = Path(sys.argv[1])
blocked_probe_dir = Path(sys.argv[2])


def read_json(root: Path, relative: str):
    return json.loads((root / relative).read_text())


def require(condition, message):
    if not condition:
        raise SystemExit(message)


def assert_no_network(root: Path, expected_run_id: str, expected_runtime: str, expected_mode: str):
    manifest = read_json(root, "manifest.json")
    summary = read_json(root, "summary.json")
    boundary = read_json(root, "boundary.json")
    probe = read_json(root, "testnet/connectivity_probe.json")
    http_probe = read_json(root, "testnet/http_connectivity_probe.json")
    ws_probe = read_json(root, "testnet/ws_connectivity_probe.json")
    policy = read_json(root, "testnet/credential_policy.json")
    lifecycle = read_json(root, "orders/testnet_dry_run_lifecycle.json")
    reconciliation = read_json(root, "orders/reconciliation.json")

    require(manifest["run_id"] == expected_run_id, manifest)
    require(manifest["runtime_status"] == expected_runtime, manifest)
    require(manifest["artifact_count"] == 11, manifest)
    require(summary["runtime_status"] == expected_runtime, summary)
    require(summary["requested_mode"] == expected_mode, summary)
    require(probe["requested_mode"] == expected_mode, probe)
    require(probe["network_gate_status"] == "blocked", probe)
    require(probe["env_network_permission"] is False, probe)
    require(probe["network_attempted"] is False, probe)
    require(probe["testnet_connection"] is False, probe)
    require(http_probe["schema_version"] == "ntpro.v07_binance_testnet_http_probe.v1", http_probe)
    require(http_probe["endpoint_kind"] == "http_read_only", http_probe)
    require(http_probe["request_method"] == "GET", http_probe)
    require(http_probe["request_target"] == "/api/v3/time", http_probe)
    require(http_probe["response_shape"] == "binance_server_time_v1", http_probe)
    require(http_probe["response_shape_validated"] is False, http_probe)
    require(http_probe["network_gate_status"] == "blocked", http_probe)
    require(http_probe["network_attempted"] is False, http_probe)
    require(http_probe["testnet_connection"] is False, http_probe)
    require(http_probe["real_orders_submitted"] is False, http_probe)
    require(ws_probe["network_gate_status"] == "blocked", ws_probe)
    require(ws_probe["websocket_attempted"] is False, ws_probe)
    require(ws_probe["network_attempted"] is False, ws_probe)
    require(ws_probe["testnet_connection"] is False, ws_probe)
    require(ws_probe["subscription_attempted"] is False, ws_probe)
    require(ws_probe["message_count"] == 0, ws_probe)
    require(ws_probe["real_orders_submitted"] is False, ws_probe)
    require(ws_probe["values_recorded"] is False, ws_probe)
    require(ws_probe["secrets_redacted"] is True, ws_probe)
    require(policy["values_recorded"] is False, policy)
    require(policy["secrets_redacted"] is True, policy)
    require(lifecycle["real_orders_submitted"] is False, lifecycle)
    require(reconciliation["real_orders_submitted"] is False, reconciliation)

    for label, payload in {
        "summary": summary,
        "boundary": boundary,
        "manifest.summary": manifest["summary"],
    }.items():
        require(payload["network_attempted"] is False, f"{label}.network_attempted must be false")
        require(payload["testnet_connection"] is False, f"{label}.testnet_connection must be false")
        require(payload["external_venue_connection"] is False, f"{label}.external_venue_connection must be false")
        require(payload["production_venue_connection"] is False, f"{label}.production_venue_connection must be false")
        require(payload["testnet_public_network_connection"] is False, f"{label}.testnet_public_network_connection must be false")
        require(payload["external_network_attempted"] is False, f"{label}.external_network_attempted must be false")
        require(payload["real_funds"] is False, f"{label}.real_funds must be false")
        require(payload["production_trading"] is False, f"{label}.production_trading must be false")
        require(payload["real_orders_submitted"] is False, f"{label}.real_orders_submitted must be false")

    return probe


dry_probe = assert_no_network(
    dry_run_dir,
    "v070-offline-dry-run",
    "dry_run_completed",
    "dry-run",
)
blocked_probe = assert_no_network(
    blocked_probe_dir,
    "v070-offline-blocked-probe",
    "offline_probe_validated",
    "connectivity-probe",
)
require("missing --allow-testnet-network" in blocked_probe["network_gate_reasons"], blocked_probe)
require("NTPRO_ALLOW_TESTNET_NETWORK=1 is not set" in blocked_probe["network_gate_reasons"], blocked_probe)

print(
    "v07_default_offline_gate status=ok "
    "dry_run_network_attempted=false blocked_probe_network_attempted=false "
    f"dry_run_status={dry_probe['status']} blocked_probe_status={blocked_probe['status']}"
)
PY

echo "v07_default_offline_gate status=ok root=$GATE_ROOT"
