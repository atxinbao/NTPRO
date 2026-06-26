#!/usr/bin/env bash
set -euo pipefail

# V180-003: v0.18 cancel request preview artifact.
# This verifier stays local/offline. It builds a preview from v0.17 orphan-risk
# evidence and proves the artifact remains redacted and no-send/no-network only.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh
export NTPRO_SOURCE_COMMIT="${NTPRO_SOURCE_COMMIT:-$(git rev-parse HEAD)}"
export NTPRO_SOURCE_RELEASE_TAG="${NTPRO_SOURCE_RELEASE_TAG:-unreleased-v18-local-gate}"

if [[ "${NTPRO_V18_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V18_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V18_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

GATE_ROOT="${NTPRO_V18_CANCEL_REQUEST_PREVIEW_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v18-cancel-preview.XXXXXX")}"
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

def mapper(name, request_sent, status, state, order_found, open_observed, terminal):
    payload = {
        "schema_version": "ntpro.v170_production_mutation_exchange_readback_mapper.v1",
        "run_id": f"v170-exchange-readback-mapper-{name}",
        "order_lineage_id": "lineage-v160-single-shot",
        "artifact_type": "production_mutation_exchange_readback_mapper",
        "status": "ready_exchange_readback_mapped",
        "mode": "single_mutation_candidate_exchange_readback_mapper",
        "capability": "Production Reconciliation And Orphan Recovery Evidence",
        "capability_expansion_from_v16": "reconciliation_evidence_only",
        "lineage_scope": "single_v16_mutation_candidate",
        "default_fail_closed": True,
        "owner_gated_readback_required": True,
        "local_ledger_ready": True,
        "exchange_readback_mapped": True,
        "reconciliation_classified": False,
        "orphan_risk_detected": False,
        "known_order_id": "123456789",
        "known_client_order_id": "owner-approved-v160-single-shot",
        "symbol": "BTCUSDT",
        "exchange_order_status": status,
        "exchange_order_state": state,
        "open_order_observed": open_observed,
        "terminal_state_observed": terminal,
        "order_found": order_found,
        "open_orders_count": 1 if open_observed else 0,
        "source_artifact_issues": [],
        "malformed_readback_issues": [],
        "missing_cli_flags": [],
        "manual_review_required": False,
        "new_orders_blocked": False,
        "request_sent": request_sent,
    }
    for field in [
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
        payload[field] = False
    write(root / f"{name}-mapper.json", payload)

mapper("open-orphan", True, "NEW", "open", True, True, False)
mapper("clean-terminal", True, "FILLED", "filled", True, False, True)
PY

run_classifier() {
  "$NAUTILUS_BIN" live production-mutation-reconciliation-classifier \
    --run-id v170-production-mutation-reconciliation-classifier \
    --exchange-readback-mapper "$1" \
    --output "$2" \
    --allow-production-mutation-reconciliation-classifier \
    --confirm-single-v16-mutation-candidate-lineage \
    --confirm-read-only-reconciliation-scope \
    --confirm-no-retry \
    --confirm-no-cancel \
    --confirm-no-remediation \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-secret-persistence
}

run_orphan_detector() {
  "$NAUTILUS_BIN" live production-mutation-orphan-order-detector \
    --run-id v170-production-mutation-orphan-order-detector \
    --reconciliation-classifier "$1" \
    --output "$2" \
    --allow-production-mutation-orphan-order-detector \
    --confirm-single-v16-mutation-candidate-lineage \
    --confirm-read-only-reconciliation-scope \
    --confirm-no-retry \
    --confirm-no-cancel \
    --confirm-no-remediation \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-secret-persistence
}

run_preview() {
  "$NAUTILUS_BIN" live production-mutation-cancel-request-preview \
    --run-id v180-production-mutation-cancel-request-preview \
    --orphan-order-detector "$1" \
    --account-label prod-account-redacted \
    --output "$2" \
    --allow-production-mutation-cancel-request-preview \
    --confirm-single-v16-mutation-candidate-lineage \
    --confirm-orphan-risk-halted \
    --confirm-manual-review-required \
    --confirm-known-order-identifier-only \
    --confirm-no-retry \
    --confirm-no-cancel \
    --confirm-no-network \
    --confirm-no-remediation \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-secret-persistence
}

run_preview_missing_flags() {
  "$NAUTILUS_BIN" live production-mutation-cancel-request-preview \
    --run-id v180-production-mutation-cancel-request-preview \
    --orphan-order-detector "$1" \
    --account-label prod-account-redacted \
    --output "$2"
}

for name in open-orphan clean-terminal; do
  run_classifier \
    "$FIXTURE_DIR/$name-mapper.json" \
    "$OUTPUT_DIR/$name-classifier.json" >/dev/null
  run_orphan_detector \
    "$OUTPUT_DIR/$name-classifier.json" \
    "$OUTPUT_DIR/$name-orphan-detector.json" >/dev/null
done

run_preview \
  "$OUTPUT_DIR/open-orphan-orphan-detector.json" \
  "$OUTPUT_DIR/open-orphan-preview.json" >/dev/null
run_preview_missing_flags \
  "$OUTPUT_DIR/open-orphan-orphan-detector.json" \
  "$OUTPUT_DIR/missing-flags-preview.json" >/dev/null
run_preview \
  "$OUTPUT_DIR/clean-terminal-orphan-detector.json" \
  "$OUTPUT_DIR/clean-terminal-preview.json" >/dev/null

python3 - "$OUTPUT_DIR" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])

def load(name):
    return json.loads((root / name).read_text())

def assert_false_fields(artifact):
    for field in [
        "actual_cancel_send_allowed",
        "cancel_attempted",
        "network_attempted",
        "network_cancel_endpoint_attempted",
        "retry_attempted",
        "replace_attempted",
        "amend_attempted",
        "flatten_attempted",
        "remediation_attempted",
        "automatic_cancel_allowed",
        "automatic_remediation_allowed",
        "production_order_mutation_allowed",
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
        assert artifact[field] is False, field
    assert artifact["cancel_requests_sent"] == 0

def assert_source_ref(artifact, field):
    ref = artifact[field]
    assert ref["hash"].startswith("fnv1a64:"), field
    assert ref["sha256"].startswith("sha256:"), field
    assert len(ref["sha256"]) == 71, field
    assert ref["bytes"] > 0, field
    assert ref["source_command"] != "unknown", field
    assert ref["source_commit"] != "unknown", field
    assert ref["source_release_tag"], field

ready = load("open-orphan-preview.json")
assert ready["schema_version"] == "ntpro.v180_cancel_request_preview.v1"
assert ready["artifact_type"] == "cancel_request_preview"
assert ready["status"] == "ready_cancel_request_preview"
assert ready["cancel_request_preview_ready"] is True
assert ready["capability"] == "Owner-Approved Cancel Recovery Preview"
assert ready["capability_expansion"] == "preview_gate_approval_only"
assert ready["lineage_scope"] == "single_v16_mutation_candidate"
assert ready["cancel_candidate_source"] == "production_mutation_orphan_order_detector"
assert ready["orphan_risk_detected"] is True
assert ready["risk_halted"] is True
assert ready["manual_review_required"] is True
assert ready["new_orders_blocked"] is True
assert ready["order_identifier_known"] is True
assert ready["candidate_count"] == 1
assert ready["known_order_id"].startswith("order_id:sha256:")
assert ready["known_client_order_id"].startswith("client_order_id:sha256:")
assert ready["symbol"] == "BTCUSDT"
assert ready["account_label"] == "prod-account-redacted"
assert ready["source_artifact_issues"] == []
assert ready["missing_cli_flags"] == []
assert_source_ref(ready, "orphan_order_detector_ref")
assert_source_ref(ready, "reconciliation_classifier_ref")
assert_source_ref(ready, "exchange_readback_mapper_ref")
assert_false_fields(ready)

missing = load("missing-flags-preview.json")
assert missing["status"] == "blocked_missing_gate"
assert missing["cancel_request_preview_ready"] is False
assert "--allow-production-mutation-cancel-request-preview" in missing["missing_cli_flags"]
assert "--confirm-no-network" in missing["missing_cli_flags"]
assert missing["source_artifact_issues"] == []
assert_false_fields(missing)

clean = load("clean-terminal-preview.json")
assert clean["status"] == "blocked_source_artifact"
assert clean["cancel_request_preview_ready"] is False
assert clean["orphan_risk_detected"] is False
assert clean["risk_halted"] is False
assert clean["manual_review_required"] is False
assert clean["new_orders_blocked"] is False
assert "orphan_order_detector_orphan_risk_detected_not_true" in clean["source_artifact_issues"]
assert "orphan_order_detector_risk_halted_not_true" in clean["source_artifact_issues"]
assert clean["missing_cli_flags"] == []
assert_false_fields(clean)

for output in root.glob("*.json"):
    body = output.read_text()
    for token in [
        "X-MBX-APIKEY",
        "signature=",
        "apiSecret",
        "signedQuery=",
        "signedUrl=",
    ]:
        assert token not in body, f"{output.name} contains {token}"
for output in ["open-orphan-preview.json", "missing-flags-preview.json", "clean-terminal-preview.json"]:
    body = (root / output).read_text()
    assert "123456789" not in body, output
    assert "owner-approved-v160-single-shot" not in body, output
PY

echo "verify_v18_cancel_request_preview PASS root=$GATE_ROOT"
