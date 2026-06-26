#!/usr/bin/env bash
set -euo pipefail

# V170-003: v0.17 reconciliation classifier.
# This verifier stays local/offline. It classifies local-vs-exchange state
# outcomes from a redacted exchange-readback mapper artifact and proves
# ambiguous outcomes require manual review without retry, cancel, remediation,
# or Dashboard order controls.

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

GATE_ROOT="${NTPRO_V17_RECONCILIATION_CLASSIFIER_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v17-reconciliation-classifier.XXXXXX")}"
FIXTURE_DIR="$GATE_ROOT/fixtures"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$FIXTURE_DIR" "$OUTPUT_DIR"

python3 - "$FIXTURE_DIR" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])

def write(path, payload):
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")

def mapper(
    source_status,
    mapped,
    request_sent,
    exchange_status,
    exchange_state,
    order_found,
    open_observed,
    terminal,
):
    return {
        "schema_version": "ntpro.v170_production_mutation_exchange_readback_mapper.v1",
        "run_id": "v170-production-mutation-exchange-readback-mapper",
        "order_lineage_id": "lineage-v160-single-shot",
        "artifact_type": "production_mutation_exchange_readback_mapper",
        "status": source_status,
        "mode": "single_mutation_candidate_exchange_readback_mapper",
        "capability": "Production Reconciliation And Orphan Recovery Evidence",
        "capability_expansion_from_v16": "reconciliation_evidence_only",
        "lineage_scope": "single_v16_mutation_candidate",
        "default_fail_closed": True,
        "owner_gated_readback_required": True,
        "local_ledger_ready": True,
        "exchange_readback_mapped": mapped,
        "reconciliation_classified": False,
        "orphan_risk_detected": False,
        "known_order_id": "123456789",
        "known_client_order_id": "owner-approved-v160-single-shot",
        "symbol": "BTCUSDT",
        "exchange_order_status": exchange_status,
        "exchange_order_state": exchange_state,
        "open_order_observed": open_observed,
        "terminal_state_observed": terminal,
        "order_found": order_found,
        "open_orders_count": 1 if open_observed else 0,
        "source_artifact_issues": [],
        "malformed_readback_issues": [],
        "missing_cli_flags": [],
        "manual_review_required": False,
        "new_orders_blocked": False,
        "network_attempted": False,
        "request_sent": request_sent,
        "production_order_submission_allowed": False,
        "production_order_mutation_allowed": False,
        "production_order_state_reads_allowed": False,
        "listen_key_lifecycle_allowed": False,
        "duplicate_submit_attempted": False,
        "retry_attempted": False,
        "cancel_attempted": False,
        "replace_attempted": False,
        "amend_attempted": False,
        "flatten_attempted": False,
        "remediation_attempted": False,
        "automatic_cancel_allowed": False,
        "automatic_remediation_allowed": False,
        "dashboard_order_controls_enabled": False,
        "dashboard_cancel_controls_enabled": False,
        "api_key_value_recorded": False,
        "api_secret_value_recorded": False,
        "api_key_header_value_recorded": False,
        "signature_recorded": False,
        "signed_query_recorded": False,
        "signed_url_recorded": False,
        "raw_exchange_response_recorded": False,
        "response_body_recorded": False,
        "response_headers_recorded": False,
    }

cases = {
    "local-sent-exchange-unknown": mapper("ready_exchange_readback_mapped", True, True, "UNKNOWN", "unknown", True, False, False),
    "local-sent-exchange-new": mapper("ready_exchange_readback_mapped", True, True, "NEW", "open", True, True, False),
    "local-sent-exchange-filled": mapper("ready_exchange_readback_mapped", True, True, "FILLED", "filled", True, False, True),
    "local-sent-exchange-canceled": mapper("ready_exchange_readback_mapped", True, True, "CANCELED", "canceled", True, False, True),
    "local-sent-exchange-rejected": mapper("ready_exchange_readback_mapped", True, True, "REJECTED", "rejected", True, False, True),
    "local-sent-exchange-missing": mapper("ready_exchange_readback_mapped", True, True, "MISSING", "missing", False, False, False),
    "local-no-send-exchange-order-seen": mapper("ready_exchange_readback_mapped", True, False, "NEW", "open", True, True, False),
    "readback-failed": mapper("blocked_malformed_exchange_readback", False, True, "MALFORMED", "malformed", True, False, False),
}

for name, payload in cases.items():
    write(root / f"{name}.json", payload)
PY

run_classifier() {
  local mapper="$1"
  local output="$2"
  shift 2
  "$NAUTILUS_BIN" live production-mutation-reconciliation-classifier \
    --run-id v170-production-mutation-reconciliation-classifier \
    --exchange-readback-mapper "$mapper" \
    --output "$output" \
    "$@"
}

run_ready_classifier() {
  run_classifier "$1" "$2" \
    --allow-production-mutation-reconciliation-classifier \
    --confirm-single-v16-mutation-candidate-lineage \
    --confirm-read-only-reconciliation-scope \
    --confirm-no-retry \
    --confirm-no-cancel \
    --confirm-no-remediation \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-secret-persistence
}

run_classifier \
  "$FIXTURE_DIR/local-sent-exchange-new.json" \
  "$OUTPUT_DIR/missing-flags-reconciliation-classifier.json" >/dev/null

for name in \
  local-sent-exchange-unknown \
  local-sent-exchange-new \
  local-sent-exchange-filled \
  local-sent-exchange-canceled \
  local-sent-exchange-rejected \
  local-sent-exchange-missing \
  local-no-send-exchange-order-seen \
  readback-failed
do
  run_ready_classifier \
    "$FIXTURE_DIR/$name.json" \
    "$OUTPUT_DIR/$name.json" >/dev/null
done

python3 - "$OUTPUT_DIR" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])

def assert_source_ref(artifact, field):
    ref = artifact[field]
    assert ref["hash"].startswith("fnv1a64:"), field
    assert ref["sha256"].startswith("sha256:"), field
    assert len(ref["sha256"]) == 71, field
    assert ref["bytes"] > 0, field
    assert ref["source_command"] != "unknown", field
    assert ref["source_commit"] != "unknown", field
    assert ref["source_release_tag"], field

missing_flags = json.loads((root / "missing-flags-reconciliation-classifier.json").read_text())
assert missing_flags["schema_version"] == "ntpro.v170_production_mutation_reconciliation_classifier.v1"
assert missing_flags["status"] == "blocked_missing_gate"
assert missing_flags["reconciliation_classified"] is False
assert missing_flags["reconciliation_outcome"] == "local_sent_exchange_new"
assert "--allow-production-mutation-reconciliation-classifier" in missing_flags["missing_cli_flags"]
assert "--confirm-no-remediation" in missing_flags["missing_cli_flags"]
assert missing_flags["manual_review_required"] is True
assert missing_flags["new_orders_blocked"] is True
assert missing_flags["retry_attempted"] is False
assert missing_flags["cancel_attempted"] is False
assert missing_flags["dashboard_order_controls_enabled"] is False

expected = {
    "local-sent-exchange-unknown": ("local_sent_exchange_unknown", True, True),
    "local-sent-exchange-new": ("local_sent_exchange_new", True, True),
    "local-sent-exchange-filled": ("local_sent_exchange_filled", False, False),
    "local-sent-exchange-canceled": ("local_sent_exchange_canceled", False, False),
    "local-sent-exchange-rejected": ("local_sent_exchange_rejected", False, False),
    "local-sent-exchange-missing": ("local_sent_exchange_missing", True, True),
    "local-no-send-exchange-order-seen": ("local_no_send_exchange_order_seen", True, True),
    "readback-failed": ("readback_failed", True, True),
}
for name, (outcome, manual_review, new_orders_blocked) in expected.items():
    artifact = json.loads((root / f"{name}.json").read_text())
    assert artifact["schema_version"] == "ntpro.v170_production_mutation_reconciliation_classifier.v1"
    assert artifact["status"] == "ready_reconciliation_classified"
    assert artifact["reconciliation_classified"] is True
    assert artifact["orphan_risk_detected"] is False
    assert artifact["order_lineage_id"] == "lineage-v160-single-shot"
    assert artifact["reconciliation_outcome"] == outcome
    assert artifact["manual_review_required"] is manual_review
    assert artifact["new_orders_blocked"] is new_orders_blocked
    assert artifact["source_artifact_issues"] == []
    assert artifact["missing_cli_flags"] == []
    assert_source_ref(artifact, "exchange_readback_mapper_ref")
    for field in [
        "network_attempted",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
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
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "raw_exchange_response_recorded",
        "response_body_recorded",
        "response_headers_recorded",
    ]:
        assert artifact[field] is False, f"{name} {field}"

for output in root.glob("*.json"):
    body = output.read_text()
    forbidden = [
        "X-MBX-APIKEY",
        "signature=",
        "apiSecret",
        "signedQuery=",
        "signedUrl=",
    ]
    for token in forbidden:
        assert token not in body, f"{output.name} contains {token}"
PY

echo "verify_v17_reconciliation_classifier PASS root=$GATE_ROOT"
