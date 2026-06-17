#!/usr/bin/env bash
set -euo pipefail

# V080-007 authenticated read-only gate.
# Default mode is a CI-safe preflight: synthetic env-only credentials are
# present, but NTPRO_ALLOW_TESTNET_NETWORK is intentionally unset, so no socket
# can be opened and no real Binance credential is required.
#
# Real online proof is explicit only:
#
#   NTPRO_V08_MANUAL_ONLINE=1 \
#   NTPRO_ALLOW_TESTNET_NETWORK=1 \
#   BINANCE_TESTNET_API_KEY=... \
#   BINANCE_TESTNET_API_SECRET=... \
#   scripts/ai/verify_v08_authenticated_readonly_gate.sh
#
# The real mode is still read-only. It only allows authenticated GET
# /api/v3/account on Binance testnet and must not submit, cancel, replace, or
# amend orders.

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

GATE_ROOT="${NTPRO_V08_AUTH_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v08-auth-readonly.XXXXXX")}"
OUTPUT_DIR="$GATE_ROOT/command-output"
WORKFLOW_DIR="$GATE_ROOT/workflows/v080-auth-readonly"
mkdir -p "$OUTPUT_DIR"

assert_no_mutation_artifacts() {
  python3 - "$WORKFLOW_DIR" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])


def read_json(relative: str):
    return json.loads((root / relative).read_text())


def require(condition, message):
    if not condition:
        raise SystemExit(message)


manifest = read_json("manifest.json")
summary = read_json("summary.json")
boundary = read_json("boundary.json")
policy = read_json("testnet/credential_policy.json")
auth_probe = read_json("testnet/authenticated_readonly_probe.json")
lifecycle = read_json("orders/testnet_dry_run_lifecycle.json")
reconciliation = read_json("orders/reconciliation.json")

require(manifest["artifact_count"] == 12, manifest)
require(summary["production_venue_connection"] is False, summary)
require(summary["real_orders_submitted"] is False, summary)
require(boundary["production_venue_connection"] is False, boundary)
require(boundary["real_funds"] is False, boundary)
require(boundary["production_trading"] is False, boundary)
require(boundary["real_orders_submitted"] is False, boundary)
require(policy["values_recorded"] is False, policy)
require(policy["api_key_value_recorded"] is False, policy)
require(policy["api_secret_value_recorded"] is False, policy)
require(policy["secrets_redacted"] is True, policy)
require(auth_probe["schema_version"] == "ntpro.v08_binance_testnet_authenticated_readonly_probe.v1", auth_probe)
require(auth_probe["endpoint_kind"] == "authenticated_http_read_only", auth_probe)
require(auth_probe["request_method"] == "GET", auth_probe)
require(auth_probe["request_target"] == "/api/v3/account", auth_probe)
require(auth_probe["query_shape"] == "timestamp=<ms>&recvWindow=<ms>&signature=<redacted>", auth_probe)
require(auth_probe["api_key_header_value_recorded"] is False, auth_probe)
require(auth_probe["signature_recorded"] is False, auth_probe)
require(auth_probe["signed_query_recorded"] is False, auth_probe)
require(auth_probe["signed_url_recorded"] is False, auth_probe)
require(auth_probe["raw_response_recorded"] is False, auth_probe)
require(auth_probe["balances_recorded"] is False, auth_probe)
require(auth_probe["uid_recorded"] is False, auth_probe)
require(auth_probe["account_mutation"] is False, auth_probe)
require(auth_probe["order_submission"] == "disabled", auth_probe)
require(auth_probe["real_orders_submitted"] is False, auth_probe)
require(auth_probe["production_venue_connection"] is False, auth_probe)
require(auth_probe["real_funds"] is False, auth_probe)
require(auth_probe["production_trading"] is False, auth_probe)
require(lifecycle["real_orders_submitted"] is False, lifecycle)
require(reconciliation["real_orders_submitted"] is False, reconciliation)
PY
}

if [[ "${NTPRO_V08_MANUAL_ONLINE:-0}" != "1" ]]; then
  synthetic_api_key="${NTPRO_V08_SYNTHETIC_API_KEY:-FAKE_BINANCE_TESTNET_API_KEY_SHOULD_NOT_APPEAR}"
  synthetic_api_secret="${NTPRO_V08_SYNTHETIC_API_SECRET:-FAKE_BINANCE_TESTNET_API_SECRET_SHOULD_NOT_APPEAR}"
  synthetic_signature="${NTPRO_V08_SYNTHETIC_SIGNATURE:-FAKE_BINANCE_SIGNATURE_SHOULD_NOT_APPEAR}"

  env \
    -u NTPRO_ALLOW_TESTNET_NETWORK \
    BINANCE_TESTNET_API_KEY="$synthetic_api_key" \
    BINANCE_TESTNET_API_SECRET="$synthetic_api_secret" \
    NTPRO_V08_SYNTHETIC_SIGNATURE="$synthetic_signature" \
    "$NAUTILUS_BIN" workflow run \
      --workflow binance-testnet \
      --mode connectivity-probe \
      --allow-testnet-network \
      --config "$CONFIG" \
      --run-id v080-auth-readonly-preflight \
      --output "$WORKFLOW_DIR" \
      >"$OUTPUT_DIR/preflight.stdout.log" \
      2>"$OUTPUT_DIR/preflight.stderr.log"

  assert_no_mutation_artifacts
  python3 - "$WORKFLOW_DIR" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
summary = json.loads((root / "summary.json").read_text())
policy = json.loads((root / "testnet/credential_policy.json").read_text())
auth_probe = json.loads((root / "testnet/authenticated_readonly_probe.json").read_text())


def require(condition, message):
    if not condition:
        raise SystemExit(message)


require(policy["api_key_present"] is True, policy)
require(policy["api_secret_present"] is True, policy)
require(summary["network_permission_requested"] is True, summary)
require(summary["network_attempted"] is False, summary)
require(summary["testnet_connection"] is False, summary)
require(auth_probe["network_gate_status"] == "blocked", auth_probe)
require(auth_probe["network_gate_reasons"] == ["NTPRO_ALLOW_TESTNET_NETWORK=1 is not set"], auth_probe)
require(auth_probe["api_key_present"] is True, auth_probe)
require(auth_probe["api_secret_present"] is True, auth_probe)
require(auth_probe["network_attempted"] is False, auth_probe)
require(auth_probe["testnet_connection"] is False, auth_probe)
require(auth_probe["status"] == "authenticated_readonly_probe_deferred", auth_probe)
require(auth_probe["error_code"] == "network_gate_blocked", auth_probe)
require(auth_probe["response_shape"] == "binance_account_readonly_redacted_v1", auth_probe)
require(auth_probe["response_shape_validated"] is False, auth_probe)

print(
    "v08_authenticated_readonly_gate status=preflight_blocked "
    "manual_online=false network_attempted=false "
    "synthetic_credentials_present=true secrets_redacted=true"
)
PY

  NTPRO_V08_SYNTHETIC_API_KEY="$synthetic_api_key" \
    NTPRO_V08_SYNTHETIC_API_SECRET="$synthetic_api_secret" \
    NTPRO_V08_SYNTHETIC_SIGNATURE="$synthetic_signature" \
    scripts/ai/scan_v08_synthetic_secret_leaks.sh "$GATE_ROOT"

  echo "v08_authenticated_readonly_gate status=preflight_blocked root=$GATE_ROOT"
  exit 0
fi

if [[ "${NTPRO_ALLOW_TESTNET_NETWORK:-0}" != "1" ]]; then
  echo "manual authenticated read-only proof requires NTPRO_ALLOW_TESTNET_NETWORK=1" >&2
  exit 1
fi
if [[ -z "${BINANCE_TESTNET_API_KEY:-}" || -z "${BINANCE_TESTNET_API_SECRET:-}" ]]; then
  echo "manual authenticated read-only proof requires BINANCE_TESTNET_API_KEY and BINANCE_TESTNET_API_SECRET" >&2
  exit 1
fi

"$NAUTILUS_BIN" workflow run \
  --workflow binance-testnet \
  --mode connectivity-probe \
  --allow-testnet-network \
  --config "$CONFIG" \
  --run-id v080-auth-readonly-manual-online \
  --output "$WORKFLOW_DIR" \
  >"$OUTPUT_DIR/manual-online.stdout.log" \
  2>"$OUTPUT_DIR/manual-online.stderr.log"

assert_no_mutation_artifacts
python3 - "$WORKFLOW_DIR" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
policy = json.loads((root / "testnet/credential_policy.json").read_text())
auth_probe = json.loads((root / "testnet/authenticated_readonly_probe.json").read_text())


def require(condition, message):
    if not condition:
        raise SystemExit(message)


stable_errors = {
    "none",
    "timeout",
    "connect_error",
    "decode_error",
    "request_error",
    "body_error",
    "unknown_http_error",
    "http_client_build_failed",
    "authenticated_probe_thread_panicked",
    "http_status_not_success",
    "response_shape_invalid",
    "signed_request_builder_failed",
}

require(policy["api_key_present"] is True, policy)
require(policy["api_secret_present"] is True, policy)
require(auth_probe["network_gate_status"] == "allowed", auth_probe)
require(auth_probe["network_gate_reasons"] == [], auth_probe)
require(auth_probe["network_attempted"] is True, auth_probe)
require(auth_probe["api_key_present"] is True, auth_probe)
require(auth_probe["api_secret_present"] is True, auth_probe)
require(auth_probe["error_code"] in stable_errors, auth_probe)

if auth_probe["error_code"] != "none":
    print(
        "v08_authenticated_readonly_gate status=classified_failure "
        "authenticated_connectivity_proof=false "
        f"error_code={auth_probe['error_code']} "
        f"response_shape_validated={str(auth_probe.get('response_shape_validated')).lower()}",
        file=sys.stderr,
    )
    raise SystemExit({
        "reason": "stable_failure_is_classification_only_not_authenticated_connectivity_proof",
        "authenticated_probe": {
            "error_code": auth_probe["error_code"],
            "network_attempted": auth_probe["network_attempted"],
            "testnet_connection": auth_probe.get("testnet_connection"),
            "response_shape": auth_probe.get("response_shape"),
            "response_shape_validated": auth_probe.get("response_shape_validated"),
        },
    })

require(auth_probe["status"] == "authenticated_readonly_probe_ok", auth_probe)
require(auth_probe["testnet_connection"] is True, auth_probe)
require(auth_probe["response_status_code"] is not None, auth_probe)
require(auth_probe["response_shape"] == "binance_account_readonly_redacted_v1", auth_probe)
require(auth_probe["response_shape_validated"] is True, auth_probe)

print(
    "v08_authenticated_readonly_gate status=ok "
    "authenticated_connectivity_proof=true "
    "response_shape_validated=true real_orders_submitted=false secrets_redacted=true"
)
PY

echo "v08_authenticated_readonly_gate status=ok root=$GATE_ROOT"
