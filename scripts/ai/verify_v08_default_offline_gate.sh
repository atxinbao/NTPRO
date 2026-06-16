#!/usr/bin/env bash
set -euo pipefail

# V080-005 default offline gate.
# Runs the Binance testnet workflow with synthetic env-only credentials while
# the network gate remains closed, then scans all generated outputs for secret
# leakage. This is CI-safe: no real credentials, no sockets, no orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V08_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V08_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V08_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
CONFIG="${NTPRO_V08_CONFIG:-$ROOT_DIR/examples/rust/binance/testnet_dry_run.toml}"

if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi
if [[ ! -f "$CONFIG" ]]; then
  echo "missing Binance testnet config: $CONFIG" >&2
  exit 1
fi

SYNTHETIC_API_KEY="${NTPRO_V08_SYNTHETIC_API_KEY:-FAKE_BINANCE_TESTNET_API_KEY_SHOULD_NOT_APPEAR}"
SYNTHETIC_API_SECRET="${NTPRO_V08_SYNTHETIC_API_SECRET:-FAKE_BINANCE_TESTNET_API_SECRET_SHOULD_NOT_APPEAR}"
SYNTHETIC_SIGNATURE="${NTPRO_V08_SYNTHETIC_SIGNATURE:-FAKE_BINANCE_SIGNATURE_SHOULD_NOT_APPEAR}"

GATE_ROOT="${NTPRO_V08_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v08-default-offline.XXXXXX")}"
RUN_ID="${NTPRO_V08_RUN_ID:-v080-secret-leak-scan}"
WORKFLOW_DIR="$GATE_ROOT/workflows/$RUN_ID"
OUTPUT_DIR="$GATE_ROOT/command-output"
DASHBOARD_DIR="$GATE_ROOT/dashboard"
LOG_DIR="$GATE_ROOT/logs"
EVIDENCE_DIR="$GATE_ROOT/pr-evidence"

mkdir -p "$OUTPUT_DIR" "$DASHBOARD_DIR" "$LOG_DIR" "$EVIDENCE_DIR"

env \
  -u NTPRO_ALLOW_TESTNET_NETWORK \
  BINANCE_TESTNET_API_KEY="$SYNTHETIC_API_KEY" \
  BINANCE_TESTNET_API_SECRET="$SYNTHETIC_API_SECRET" \
  NTPRO_V08_SYNTHETIC_SIGNATURE="$SYNTHETIC_SIGNATURE" \
  "$NAUTILUS_BIN" workflow run \
    --workflow binance-testnet \
    --mode dry-run \
    --config "$CONFIG" \
    --run-id "$RUN_ID" \
    --output "$WORKFLOW_DIR" \
    >"$OUTPUT_DIR/workflow.stdout.log" \
    2>"$OUTPUT_DIR/workflow.stderr.log"

python3 - "$WORKFLOW_DIR" "$DASHBOARD_DIR" "$LOG_DIR" "$EVIDENCE_DIR" "$RUN_ID" <<'PY'
import json
import sys
from pathlib import Path

workflow_dir = Path(sys.argv[1])
dashboard_dir = Path(sys.argv[2])
log_dir = Path(sys.argv[3])
evidence_dir = Path(sys.argv[4])
run_id = sys.argv[5]


def read_json(relative: str):
    return json.loads((workflow_dir / relative).read_text())


def require(condition, message):
    if not condition:
        raise SystemExit(message)


manifest = read_json("manifest.json")
summary = read_json("summary.json")
boundary = read_json("boundary.json")
policy = read_json("testnet/credential_policy.json")
auth_probe = read_json("testnet/authenticated_readonly_probe.json")

require(manifest["run_id"] == run_id, manifest)
require(manifest["artifact_count"] == 12, manifest)
require(summary["network_attempted"] is False, summary)
require(summary["testnet_connection"] is False, summary)
require(summary["real_orders_submitted"] is False, summary)
require(boundary["real_funds"] is False, boundary)
require(boundary["production_trading"] is False, boundary)
require(policy["api_key_present"] is True, policy)
require(policy["api_secret_present"] is True, policy)
require(policy["values_recorded"] is False, policy)
require(policy["api_key_value_recorded"] is False, policy)
require(policy["api_secret_value_recorded"] is False, policy)
require(policy["secrets_redacted"] is True, policy)
require(auth_probe["api_key_present"] is True, auth_probe)
require(auth_probe["api_secret_present"] is True, auth_probe)
require(auth_probe["network_attempted"] is False, auth_probe)
require(auth_probe["testnet_connection"] is False, auth_probe)
require(auth_probe["api_key_header_value_recorded"] is False, auth_probe)
require(auth_probe["signature_recorded"] is False, auth_probe)
require(auth_probe["signed_query_recorded"] is False, auth_probe)
require(auth_probe["signed_url_recorded"] is False, auth_probe)
require(auth_probe["raw_response_recorded"] is False, auth_probe)
require(auth_probe["balances_recorded"] is False, auth_probe)
require(auth_probe["uid_recorded"] is False, auth_probe)
require(auth_probe["real_orders_submitted"] is False, auth_probe)
require(auth_probe["production_trading"] is False, auth_probe)

dashboard_snapshot = {
    "schema_version": "ntpro.v08_dashboard_secret_scan_fixture.v1",
    "run_id": run_id,
    "workflow": manifest["workflow"],
    "runtime_status": manifest["runtime_status"],
    "artifact_count": manifest["artifact_count"],
    "credential_policy": policy["policy"],
    "api_key_present": policy["api_key_present"],
    "api_secret_present": policy["api_secret_present"],
    "values_recorded": policy["values_recorded"],
    "authenticated_readonly_probe_status": auth_probe["status"],
    "raw_response_recorded": auth_probe["raw_response_recorded"],
    "balances_recorded": auth_probe["balances_recorded"],
    "uid_recorded": auth_probe["uid_recorded"],
    "real_orders_submitted": auth_probe["real_orders_submitted"],
    "production_trading": auth_probe["production_trading"],
}
dashboard_dir.joinpath("snapshot.json").write_text(
    json.dumps(dashboard_snapshot, indent=2, sort_keys=True) + "\n"
)
log_dir.joinpath("workflow-events-summary.log").write_text(
    "v08 default offline gate retained no secret values, no network, no orders\n"
)
evidence_dir.joinpath("pr-evidence-snippet.md").write_text(
    "\n".join(
        [
            "# V080-005 synthetic secret leak scan",
            "",
            f"run_id = {run_id}",
            "synthetic credentials = env-only, not printed",
            "network_attempted = false",
            "real_orders_submitted = false",
            "raw account body = not persisted",
            "signature = not persisted",
            "",
        ]
    )
)

print(
    "v08_default_offline_gate_assertions status=ok "
    f"run_id={run_id} artifact_count={manifest['artifact_count']} "
    "synthetic_credentials_present=true values_recorded=false network_attempted=false"
)
PY

NTPRO_V08_SYNTHETIC_API_KEY="$SYNTHETIC_API_KEY" \
NTPRO_V08_SYNTHETIC_API_SECRET="$SYNTHETIC_API_SECRET" \
NTPRO_V08_SYNTHETIC_SIGNATURE="$SYNTHETIC_SIGNATURE" \
  scripts/ai/scan_v08_synthetic_secret_leaks.sh "$GATE_ROOT"

echo "v08_default_offline_gate status=ok root=$GATE_ROOT workflow_dir=$WORKFLOW_DIR"
