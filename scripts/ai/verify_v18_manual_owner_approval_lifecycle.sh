#!/usr/bin/env bash
set -euo pipefail

# V180-005: v0.18 manual owner approval lifecycle artifact.
# This verifier stays local/offline. It proves one owner approval is scoped to
# one cancel candidate, one-time, non-reusable, expirable, and no-send in v0.18.

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

GATE_ROOT="${NTPRO_V18_MANUAL_OWNER_APPROVAL_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v18-manual-owner-approval.XXXXXX")}"
FIXTURE_DIR="$GATE_ROOT/fixtures"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$FIXTURE_DIR" "$OUTPUT_DIR"

python3 - "$FIXTURE_DIR" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
payload = {
    "schema_version": "ntpro.v170_production_mutation_exchange_readback_mapper.v1",
    "run_id": "v170-exchange-readback-mapper-open-orphan",
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
    "exchange_order_status": "NEW",
    "exchange_order_state": "open",
    "open_order_observed": True,
    "terminal_state_observed": False,
    "order_found": True,
    "open_orders_count": 1,
    "source_artifact_issues": [],
    "malformed_readback_issues": [],
    "missing_cli_flags": [],
    "manual_review_required": False,
    "new_orders_blocked": False,
    "request_sent": True,
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
(root / "open-orphan-mapper.json").write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
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

run_gate() {
  "$NAUTILUS_BIN" live production-mutation-cancel-risk-gate \
    --run-id v180-production-mutation-cancel-risk-gate \
    --cancel-request-preview "$1" \
    --expected-symbol BTCUSDT \
    --expected-account-label prod-account-redacted \
    --output "$2" \
    --allow-production-mutation-cancel-risk-gate \
    --confirm-single-v16-mutation-candidate-lineage \
    --confirm-cancel-request-preview-ready \
    --confirm-orphan-risk-halted \
    --confirm-known-order-identifier-only \
    --confirm-symbol-account-scope \
    --confirm-owner-approval-required \
    --confirm-no-cancel-all-or-bulk \
    --confirm-no-retry \
    --confirm-no-cancel \
    --confirm-no-network \
    --confirm-no-remediation \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-secret-persistence
}

run_approval() {
  local name="$1"
  local state="$2"
  local now_ms="$3"
  local gate="$4"
  local output="$OUTPUT_DIR/manual-owner-approval-$name.json"
  local cmd=(
    "$NAUTILUS_BIN" live production-mutation-manual-owner-approval-lifecycle
    --run-id v180-production-mutation-manual-owner-approval-lifecycle
    --cancel-risk-gate "$gate"
    --approval-state "$state"
    --now-unix-ms "$now_ms"
    --expires-at-unix-ms 1718400060000
    --output "$output"
    --allow-production-mutation-manual-owner-approval-lifecycle
    --confirm-one-order-cancel-candidate
    --confirm-one-time-approval
    --confirm-non-reusable-approval
    --confirm-approval-expiry
    --confirm-no-strategy-auto-approval
    --confirm-no-background-auto-approval
    --confirm-no-dashboard-cancel-approval
    --confirm-no-incident-handler-auto-approval
    --confirm-no-cancel
    --confirm-no-network
    --confirm-dashboard-order-controls-disabled
    --confirm-no-secret-persistence
  )
  if [[ "$state" != "pending" ]]; then
    cmd+=(--manual-approval-id "owner-approval-v180-005-$name" --approved-by owner)
  fi
  "${cmd[@]}" >/dev/null
  printf '%s\n' "$output"
}

run_approval_missing_flags() {
  "$NAUTILUS_BIN" live production-mutation-manual-owner-approval-lifecycle \
    --run-id v180-production-mutation-manual-owner-approval-lifecycle \
    --cancel-risk-gate "$1" \
    --approval-state approved \
    --manual-approval-id owner-approval-v180-005-missing \
    --approved-by owner \
    --now-unix-ms 1718400000000 \
    --expires-at-unix-ms 1718400060000 \
    --output "$2" >/dev/null
}

run_classifier "$FIXTURE_DIR/open-orphan-mapper.json" "$OUTPUT_DIR/open-orphan-classifier.json" >/dev/null
run_orphan_detector "$OUTPUT_DIR/open-orphan-classifier.json" "$OUTPUT_DIR/open-orphan-detector.json" >/dev/null
run_preview "$OUTPUT_DIR/open-orphan-detector.json" "$OUTPUT_DIR/open-orphan-preview.json" >/dev/null
run_gate "$OUTPUT_DIR/open-orphan-preview.json" "$OUTPUT_DIR/open-orphan-risk-gate.json" >/dev/null

VALID_APPROVAL="$(run_approval valid approved 1718400000000 "$OUTPUT_DIR/open-orphan-risk-gate.json")"
PENDING_APPROVAL="$(run_approval pending pending 1718400000000 "$OUTPUT_DIR/open-orphan-risk-gate.json")"
EXPIRED_APPROVAL="$(run_approval expired approved 1718400070000 "$OUTPUT_DIR/open-orphan-risk-gate.json")"
USED_APPROVAL="$(run_approval used used 1718400000000 "$OUTPUT_DIR/open-orphan-risk-gate.json")"
MISSING_FLAGS_APPROVAL="$OUTPUT_DIR/manual-owner-approval-missing-flags.json"
run_approval_missing_flags "$OUTPUT_DIR/open-orphan-risk-gate.json" "$MISSING_FLAGS_APPROVAL"

python3 - "$OUTPUT_DIR/open-orphan-risk-gate.json" "$OUTPUT_DIR/recorded-risk-gate.json" <<'PY'
import json
import sys
from pathlib import Path

source = Path(sys.argv[1])
target = Path(sys.argv[2])
payload = json.loads(source.read_text())
payload["owner_approval_lifecycle_recorded"] = True
target.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
PY
RECORDED_SOURCE_APPROVAL="$(run_approval source-recorded approved 1718400000000 "$OUTPUT_DIR/recorded-risk-gate.json")"

python3 - "$VALID_APPROVAL" "$PENDING_APPROVAL" "$EXPIRED_APPROVAL" "$USED_APPROVAL" "$MISSING_FLAGS_APPROVAL" "$RECORDED_SOURCE_APPROVAL" <<'PY'
import json
import sys
from pathlib import Path

valid, pending, expired, used, missing, recorded = [
    json.loads(Path(path).read_text()) for path in sys.argv[1:]
]

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def assert_false_boundary(artifact):
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
        "strategy_auto_approval_allowed",
        "strategy_auto_approval_attempted",
        "background_auto_approval_allowed",
        "background_auto_approval_attempted",
        "dashboard_auto_approval_allowed",
        "dashboard_auto_approval_attempted",
        "incident_handler_auto_approval_allowed",
        "incident_handler_auto_approval_attempted",
    ]:
        require(artifact[field] is False, (field, artifact))
    require(artifact["cancel_requests_sent"] == 0, artifact)
    require(artifact["approval_consumed"] is False, artifact)
    require(artifact["approval_consumed_before_send"] is False, artifact)
    require(artifact["approval_consumed_after_send"] is False, artifact)

require(valid["schema_version"] == "ntpro.v180_manual_owner_approval_lifecycle.v1", valid)
require(valid["artifact_type"] == "manual_owner_approval_lifecycle", valid)
require(valid["status"] == "approval_lifecycle_recorded_for_cancel_candidate", valid)
require(valid["cancel_risk_gate_ready"] is True, valid)
require(valid["approval_scope"] == "one_order_cancel_candidate", valid)
require(valid["approval_source"] == "owner_manual_action", valid)
require(valid["approval_state"] == "approved", valid)
require(valid["manual_approval_recorded"] is True, valid)
require(valid["approval_expires"] is True, valid)
require(valid["approval_expired"] is False, valid)
require(valid["approval_revoked"] is False, valid)
require(valid["approval_used"] is False, valid)
require(valid["approval_reusable"] is False, valid)
require(valid["one_time_approval"] is True, valid)
require(valid["approval_lifecycle_valid"] is True, valid)
require(valid["owner_approval_required"] is True, valid)
require(valid["owner_approval_lifecycle_recorded"] is True, valid)
require(valid["candidate_count"] == 1, valid)
require(valid["source_artifact_issues"] == [], valid)
require(valid["missing_cli_flags"] == [], valid)
require(valid["lifecycle_issues"] == [], valid)
assert_false_boundary(valid)

require(pending["status"] == "approval_pending", pending)
require("approval_state_pending" in pending["lifecycle_issues"], pending)
require(pending["approval_lifecycle_valid"] is False, pending)
assert_false_boundary(pending)

require(expired["status"] == "approval_expired", expired)
require("approval_expired" in expired["lifecycle_issues"], expired)
require(expired["approval_lifecycle_valid"] is False, expired)
assert_false_boundary(expired)

require(used["status"] == "approval_used", used)
require("approval_used" in used["lifecycle_issues"], used)
require(used["approval_reusable"] is False, used)
require(used["approval_lifecycle_valid"] is False, used)
assert_false_boundary(used)

require(missing["status"] == "blocked_missing_gate", missing)
require("--allow-production-mutation-manual-owner-approval-lifecycle" in missing["missing_cli_flags"], missing)
require("--confirm-no-strategy-auto-approval" in missing["missing_cli_flags"], missing)
require(missing["approval_lifecycle_valid"] is False, missing)
assert_false_boundary(missing)

require(recorded["status"] == "blocked_source_artifact", recorded)
require(
    "cancel_risk_gate_owner_approval_lifecycle_already_recorded" in recorded["source_artifact_issues"],
    recorded,
)
require(recorded["approval_lifecycle_valid"] is False, recorded)
assert_false_boundary(recorded)
PY

for output in "$OUTPUT_DIR"/*.json; do
  body="$(cat "$output")"
  for token in "123456789" "owner-approved-v160-single-shot" "X-MBX-APIKEY" "signature=" "apiSecret" "signedQuery=" "signedUrl="; do
    if [[ "$body" == *"$token"* ]]; then
      echo "$output contains forbidden token $token" >&2
      exit 1
    fi
  done
done

echo "verify_v18_manual_owner_approval_lifecycle PASS root=$GATE_ROOT"
