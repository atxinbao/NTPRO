#!/usr/bin/env bash
set -euo pipefail

# V170-002: v0.17 exchange readback mapper.
# This verifier stays local/offline. It maps redacted GET /api/v3/order and
# GET /api/v3/openOrders metadata into normalized exchange order state and
# proves malformed/missing fixtures remain evidence-only without secrets,
# retries, cancels, remediation, or Dashboard order controls.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh
export NTPRO_SOURCE_COMMIT="${NTPRO_SOURCE_COMMIT:-$(git rev-parse HEAD)}"
export NTPRO_SOURCE_RELEASE_TAG="${NTPRO_SOURCE_RELEASE_TAG:-unreleased-v17-local-gate}"

if [[ "${NTPRO_V17_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V17_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V17_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

GATE_ROOT="${NTPRO_V17_EXCHANGE_READBACK_MAPPER_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v17-readback-mapper.XXXXXX")}"
LEDGER_ROOT="$GATE_ROOT/local-ledger"
FIXTURE_DIR="$GATE_ROOT/fixtures"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$FIXTURE_DIR" "$OUTPUT_DIR"

NTPRO_V17_SKIP_BUILD=1 \
NTPRO_V17_NAUTILUS_BIN="$NAUTILUS_BIN" \
NTPRO_V17_LOCAL_ORDER_LEDGER_ROOT="$LEDGER_ROOT" \
  scripts/ai/verify_v17_local_order_ledger.sh >/dev/null

LOCAL_LEDGER="$LEDGER_ROOT/command-output/ready-local-order-ledger.json"
if [[ ! -f "$LOCAL_LEDGER" ]]; then
  echo "exchange readback mapper setup did not produce expected local ledger" >&2
  exit 1
fi

python3 - "$FIXTURE_DIR" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])

def write(path, payload):
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")

def base_order(status, found=True):
    return {
        "schema_version": "ntpro.v170_redacted_binance_order_readback.v1",
        "artifact_type": "redacted_binance_order_readback",
        "status": "ready_redacted_order_readback_metadata",
        "endpoint": "order",
        "method": "GET",
        "path": "/api/v3/order",
        "order_found": found,
        "symbol": "BTCUSDT",
        "order_id": "123456789",
        "client_order_id": "owner-approved-v160-single-shot",
        "exchange_status": status,
        "response_redacted": True,
        "api_key_value_recorded": False,
        "api_secret_value_recorded": False,
        "api_key_header_value_recorded": False,
        "signature_recorded": False,
        "signed_query_recorded": False,
        "signed_url_recorded": False,
        "raw_exchange_response_recorded": False,
        "response_body_recorded": False,
        "response_headers_recorded": False,
        "request_sent": False,
        "network_attempted": False,
        "retry_attempted": False,
        "cancel_attempted": False,
        "replace_attempted": False,
        "amend_attempted": False,
        "flatten_attempted": False,
        "remediation_attempted": False,
        "dashboard_order_controls_enabled": False,
    }

def base_open_orders(include):
    orders = []
    if include:
        orders.append({
            "symbol": "BTCUSDT",
            "order_id": "123456789",
            "client_order_id": "owner-approved-v160-single-shot",
            "exchange_status": "NEW",
        })
    return {
        "schema_version": "ntpro.v170_redacted_binance_open_orders_readback.v1",
        "artifact_type": "redacted_binance_open_orders_readback",
        "status": "ready_redacted_open_orders_readback_metadata",
        "endpoint": "open_orders",
        "method": "GET",
        "path": "/api/v3/openOrders",
        "symbol": "BTCUSDT",
        "open_orders": orders,
        "response_redacted": True,
        "api_key_value_recorded": False,
        "api_secret_value_recorded": False,
        "api_key_header_value_recorded": False,
        "signature_recorded": False,
        "signed_query_recorded": False,
        "signed_url_recorded": False,
        "raw_exchange_response_recorded": False,
        "response_body_recorded": False,
        "response_headers_recorded": False,
        "request_sent": False,
        "network_attempted": False,
        "retry_attempted": False,
        "cancel_attempted": False,
        "replace_attempted": False,
        "amend_attempted": False,
        "flatten_attempted": False,
        "remediation_attempted": False,
        "dashboard_order_controls_enabled": False,
    }

for status in ["NEW", "FILLED", "CANCELED", "REJECTED"]:
    write(root / f"order-{status}.json", base_order(status, True))
    write(root / f"open-orders-{status}.json", base_open_orders(status == "NEW"))

write(root / "order-missing.json", base_order(None, False))
write(root / "open-orders-empty.json", base_open_orders(False))
write(root / "order-malformed.json", base_order(None, True))
PY

run_mapper() {
  local order_readback="$1"
  local open_orders_readback="$2"
  local output="$3"
  shift 3
  "$NAUTILUS_BIN" live production-mutation-exchange-readback-mapper \
    --run-id v170-production-mutation-exchange-readback-mapper \
    --local-order-ledger "$LOCAL_LEDGER" \
    --order-readback "$order_readback" \
    --open-orders-readback "$open_orders_readback" \
    --output "$output" \
    "$@"
}

run_ready_mapper() {
  run_mapper "$1" "$2" "$3" \
    --allow-production-mutation-exchange-readback-mapper \
    --confirm-redacted-readback-metadata-only \
    --confirm-known-order-identifier-only \
    --confirm-read-only-reconciliation-scope \
    --confirm-no-network \
    --confirm-no-secret-persistence \
    --confirm-no-production-order-mutation \
    --confirm-no-retry \
    --confirm-no-cancel \
    --confirm-dashboard-order-controls-disabled
}

run_mapper \
  "$FIXTURE_DIR/order-NEW.json" \
  "$FIXTURE_DIR/open-orders-NEW.json" \
  "$OUTPUT_DIR/missing-flags-exchange-readback-mapper.json" >/dev/null

for status in NEW FILLED CANCELED REJECTED; do
  run_ready_mapper \
    "$FIXTURE_DIR/order-$status.json" \
    "$FIXTURE_DIR/open-orders-$status.json" \
    "$OUTPUT_DIR/ready-$status.json" >/dev/null
done

run_ready_mapper \
  "$FIXTURE_DIR/order-missing.json" \
  "$FIXTURE_DIR/open-orders-empty.json" \
  "$OUTPUT_DIR/ready-missing.json" >/dev/null

run_ready_mapper \
  "$FIXTURE_DIR/order-malformed.json" \
  "$FIXTURE_DIR/open-orders-empty.json" \
  "$OUTPUT_DIR/malformed.json" >/dev/null

python3 - "$OUTPUT_DIR" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])

def assert_source_refs(artifact, fields):
    for field in fields:
        ref = artifact[field]
        assert ref["hash"].startswith("fnv1a64:"), field
        assert ref["sha256"].startswith("sha256:"), field
        assert len(ref["sha256"]) == 71, field
        assert ref["bytes"] > 0, field
        assert ref["source_command"] != "unknown", field
        assert ref["source_commit"] != "unknown", field
        assert ref["source_release_tag"], field

missing_flags = json.loads((root / "missing-flags-exchange-readback-mapper.json").read_text())
assert missing_flags["schema_version"] == "ntpro.v170_production_mutation_exchange_readback_mapper.v1"
assert missing_flags["status"] == "blocked_missing_gate"
assert missing_flags["exchange_readback_mapped"] is False
assert "--allow-production-mutation-exchange-readback-mapper" in missing_flags["missing_cli_flags"]
assert "--confirm-redacted-readback-metadata-only" in missing_flags["missing_cli_flags"]
assert missing_flags["retry_attempted"] is False
assert missing_flags["cancel_attempted"] is False
assert missing_flags["dashboard_order_controls_enabled"] is False

expected = {
    "NEW": ("open", False, True),
    "FILLED": ("filled", True, False),
    "CANCELED": ("canceled", True, False),
    "REJECTED": ("rejected", True, False),
}
for status, (state, terminal, open_observed) in expected.items():
    artifact = json.loads((root / f"ready-{status}.json").read_text())
    assert artifact["schema_version"] == "ntpro.v170_production_mutation_exchange_readback_mapper.v1"
    assert artifact["status"] == "ready_exchange_readback_mapped"
    assert artifact["exchange_readback_mapped"] is True
    assert artifact["reconciliation_classified"] is False
    assert artifact["orphan_risk_detected"] is False
    assert artifact["order_lineage_id"] == "lineage-v160-single-shot"
    assert artifact["exchange_order_status"] == status
    assert artifact["exchange_order_state"] == state
    assert artifact["terminal_state_observed"] is terminal
    assert artifact["open_order_observed"] is open_observed
    assert artifact["source_artifact_issues"] == []
    assert artifact["malformed_readback_issues"] == []
    assert artifact["missing_cli_flags"] == []
    assert_source_refs(artifact, [
        "local_ledger_ref",
        "order_readback_ref",
        "open_orders_readback_ref",
    ])
    assert artifact["manual_review_required"] is False
    assert artifact["new_orders_blocked"] is False
    for field in [
        "request_sent",
        "network_attempted",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "production_order_state_reads_allowed",
        "listen_key_lifecycle_allowed",
        "duplicate_submit_attempted",
        "retry_attempted",
        "cancel_attempted",
        "replace_attempted",
        "amend_attempted",
        "flatten_attempted",
        "remediation_attempted",
        "automatic_cancel_allowed",
        "automatic_remediation_allowed",
        "dashboard_order_controls_enabled",
        "dashboard_cancel_controls_enabled",
        "api_key_value_recorded",
        "api_secret_value_recorded",
        "api_key_header_value_recorded",
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "raw_exchange_response_recorded",
        "response_body_recorded",
        "response_headers_recorded",
    ]:
        assert artifact[field] is False, (status, field)

missing = json.loads((root / "ready-missing.json").read_text())
assert missing["status"] == "ready_exchange_readback_mapped"
assert missing["exchange_readback_mapped"] is True
assert missing["exchange_order_status"] == "MISSING"
assert missing["exchange_order_state"] == "missing"
assert missing["order_found"] is False
assert missing["open_orders_count"] == 0
assert missing["open_order_observed"] is False
assert missing["terminal_state_observed"] is False

malformed = json.loads((root / "malformed.json").read_text())
assert malformed["status"] == "blocked_malformed_exchange_readback"
assert malformed["exchange_readback_mapped"] is False
assert malformed["exchange_order_state"] == "malformed"
assert malformed["manual_review_required"] is True
assert malformed["new_orders_blocked"] is True
assert "exchange_status_missing" in malformed["malformed_readback_issues"]
assert malformed["retry_attempted"] is False
assert malformed["cancel_attempted"] is False
assert malformed["remediation_attempted"] is False
PY

if grep -R "ntpro_v160005_production_like_api_key_value\\|ntpro_v160005_production_like_api_secret_value\\|ntpro_v160007_api_key_value\\|ntpro_v160007_api_secret_value\\|X-MBX-APIKEY\\|signature=" "$OUTPUT_DIR" >/dev/null; then
  echo "exchange readback mapper artifacts persisted forbidden secret or signed material" >&2
  exit 1
fi

echo "v17_exchange_readback_mapper status=ok root=$GATE_ROOT mapped_statuses=NEW,FILLED,CANCELED,REJECTED missing=true malformed_blocked=true retry_attempted=false cancel_attempted=false dashboard_order_controls_enabled=false"
