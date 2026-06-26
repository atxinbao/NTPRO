#!/usr/bin/env bash
set -euo pipefail

# V180-008: v0.18 cancel recovery incident/audit closeout contract.
# This verifier stays local/offline. It proves the closeout artifact links risk
# gate, owner approval, response redaction, and post-cancel readback evidence
# while preserving no-send, no-network, and no-automatic-remediation markers.

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

GATE_ROOT="${NTPRO_V18_CANCEL_RECOVERY_INCIDENT_AUDIT_CLOSEOUT_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v18-incident-audit-closeout.XXXXXX")}"
POST_ROOT="$GATE_ROOT/post-cancel-readback"
FIXTURE_DIR="$GATE_ROOT/fixtures"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$FIXTURE_DIR" "$OUTPUT_DIR"

NTPRO_V18_SKIP_BUILD=1 \
  NTPRO_V18_NAUTILUS_BIN="$NAUTILUS_BIN" \
  NTPRO_V18_POST_CANCEL_READBACK_ROOT="$POST_ROOT" \
  scripts/ai/verify_v18_post_cancel_readback.sh >/dev/null

VALID_RISK_GATE="$POST_ROOT/cancel-response-redaction/manual-owner-approval/command-output/open-orphan-risk-gate.json"
VALID_APPROVAL="$POST_ROOT/cancel-response-redaction/manual-owner-approval/command-output/manual-owner-approval-valid.json"
VALID_REDACTION="$POST_ROOT/cancel-response-redaction/command-output/cancel-response-redaction-ready.json"
READY_CANCELED_READBACK="$POST_ROOT/command-output/post-cancel-readback-canceled.json"
READY_MISSING_READBACK="$POST_ROOT/command-output/post-cancel-readback-missing.json"

for source in "$VALID_RISK_GATE" "$VALID_APPROVAL" "$VALID_REDACTION" "$READY_CANCELED_READBACK" "$READY_MISSING_READBACK"; do
  if [[ ! -s "$source" ]]; then
    echo "required source artifact missing: $source" >&2
    exit 1
  fi
done

python3 - "$READY_CANCELED_READBACK" "$FIXTURE_DIR/invalid-readback.json" "$FIXTURE_DIR/mismatched-readback.json" <<'PY'
import json
import sys
from pathlib import Path

ready = Path(sys.argv[1])
invalid = Path(sys.argv[2])
mismatched = Path(sys.argv[3])

payload = json.loads(ready.read_text())
invalid_payload = dict(payload)
invalid_payload["status"] = "blocked_source_artifact"
invalid_payload["post_cancel_readback_ready"] = False
invalid.write_text(json.dumps(invalid_payload, indent=2, sort_keys=True) + "\n")

mismatched_payload = dict(payload)
mismatched_payload["order_lineage_id"] = "lineage-other-cancel-candidate"
mismatched.write_text(json.dumps(mismatched_payload, indent=2, sort_keys=True) + "\n")
PY

run_closeout() {
  local name="$1"
  local readback="$2"
  local all_flags="$3"
  local output="$OUTPUT_DIR/incident-audit-closeout-$name.json"
  local cmd=(
    "$NAUTILUS_BIN" live production-mutation-cancel-recovery-incident-audit-closeout
    --run-id v180-production-mutation-cancel-recovery-incident-audit-closeout
    --cancel-risk-gate "$VALID_RISK_GATE"
    --manual-owner-approval-lifecycle "$VALID_APPROVAL"
    --cancel-response-redaction "$VALID_REDACTION"
    --post-cancel-readback "$readback"
    --output "$output"
  )
  if [[ "$all_flags" == "true" ]]; then
    cmd+=(
      --allow-production-mutation-cancel-recovery-incident-audit-closeout
      --confirm-cancel-recovery-lineage
      --confirm-risk-reason-recorded
      --confirm-risk-gate-result-recorded
      --confirm-owner-approval-state-recorded
      --confirm-redaction-contract-state-recorded
      --confirm-readback-state-recorded
      --confirm-terminal-action-recommendation
      --confirm-remaining-risk-recorded
      --confirm-no-mutation
      --confirm-no-cancel
      --confirm-no-network
      --confirm-no-retry
      --confirm-no-remediation
      --confirm-no-automatic-remediation
      --confirm-dashboard-order-controls-disabled
      --confirm-no-secret-persistence
    )
  fi
  "${cmd[@]}" >/dev/null
  printf '%s\n' "$output"
}

READY_CANCELED="$(run_closeout canceled "$READY_CANCELED_READBACK" true)"
READY_MISSING="$(run_closeout missing "$READY_MISSING_READBACK" true)"
MISSING_FLAGS="$(run_closeout missing-flags "$READY_CANCELED_READBACK" false)"
INVALID_SOURCE="$(run_closeout invalid-source "$FIXTURE_DIR/invalid-readback.json" true)"
LINEAGE_MISMATCH="$(run_closeout lineage-mismatch "$FIXTURE_DIR/mismatched-readback.json" true)"

python3 - "$READY_CANCELED" "$READY_MISSING" "$MISSING_FLAGS" "$INVALID_SOURCE" "$LINEAGE_MISMATCH" <<'PY'
import json
import sys
from pathlib import Path

ready_canceled, ready_missing, missing_flags, invalid_source, lineage_mismatch = [
    json.loads(Path(path).read_text()) for path in sys.argv[1:]
]

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def assert_false_boundary(artifact):
    for field in [
        "api_key_value_recorded",
        "api_secret_value_recorded",
        "api_key_header_value_recorded",
        "signature_recorded",
        "signed_query_recorded",
        "signed_url_recorded",
        "raw_exchange_response_recorded",
        "raw_readback_body_recorded",
        "response_body_recorded",
        "response_headers_recorded",
        "unrestricted_payload_recorded",
        "account_balances_recorded",
        "fills_recorded",
        "readback_execution_attempted",
        "order_state_read_attempted",
        "actual_cancel_send_allowed",
        "cancel_attempted",
        "network_attempted",
        "network_readback_endpoint_attempted",
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
    ]:
        require(artifact[field] is False, (field, artifact))
    require(artifact["production_order_state_reads_attempted"] == 0, artifact)
    require(artifact["cancel_requests_sent"] == 0, artifact)
    require(artifact["production_order_mutations_attempted"] == 0, artifact)

def assert_ready(artifact, state, recommendation, remaining_risk, manual_review):
    require(artifact["schema_version"] == "ntpro.v180_cancel_recovery_incident_audit_closeout.v1", artifact)
    require(artifact["artifact_type"] == "cancel_recovery_incident_audit_closeout", artifact)
    require(artifact["status"] == "ready_cancel_recovery_incident_audit_closeout", artifact)
    require(artifact["incident_closeout_ready"] is True, artifact)
    require(artifact["audit_trail_ready"] is True, artifact)
    require(artifact["audit_traceability_ready"] is True, artifact)
    require(artifact["cancel_recovery_lineage_ready"] is True, artifact)
    require(artifact["recovery_needed_reason"] == "orphan_risk_detected", artifact)
    require(artifact["risk_gate_result"] == "ready_owner_approval_required", artifact)
    require(artifact["owner_approval_state"] == "approved", artifact)
    require(artifact["redaction_contract_state"] == "ready_redacted_metadata_only", artifact)
    require(artifact["readback_state"] == state, artifact)
    require(artifact["terminal_action_recommendation"] == recommendation, artifact)
    require(artifact["remaining_risk"] == remaining_risk, artifact)
    require(artifact["remaining_risk_requires_manual_review"] is manual_review, artifact)
    require(artifact["source_artifact_issues"] == [], artifact)
    require(artifact["lineage_issues"] == [], artifact)
    require(artifact["missing_cli_flags"] == [], artifact)
    assert_false_boundary(artifact)

assert_ready(
    ready_canceled,
    "CANCELED",
    "close_incident_cancel_confirmed",
    "none_cancel_confirmed",
    False,
)
assert_ready(
    ready_missing,
    "MISSING",
    "manual_exchange_and_local_ledger_review",
    "exchange_state_missing_manual_review_required",
    True,
)

require(missing_flags["status"] == "blocked_missing_gate", missing_flags)
require(missing_flags["incident_closeout_ready"] is False, missing_flags)
require("--allow-production-mutation-cancel-recovery-incident-audit-closeout" in missing_flags["missing_cli_flags"], missing_flags)
require("--confirm-remaining-risk-recorded" in missing_flags["missing_cli_flags"], missing_flags)
assert_false_boundary(missing_flags)

require(invalid_source["status"] == "blocked_source_artifact", invalid_source)
require(
    "source_post_cancel_readback_post_cancel_readback_status_blocked_source_artifact"
    in invalid_source["source_artifact_issues"],
    invalid_source,
)
require(
    "source_post_cancel_readback_post_cancel_readback_post_cancel_readback_ready_not_true"
    in invalid_source["source_artifact_issues"],
    invalid_source,
)
assert_false_boundary(invalid_source)

require(lineage_mismatch["status"] == "blocked_lineage_mismatch", lineage_mismatch)
require("post_cancel_readback_order_lineage_id_mismatch" in lineage_mismatch["lineage_issues"], lineage_mismatch)
assert_false_boundary(lineage_mismatch)
PY

for output in "$OUTPUT_DIR"/*.json; do
  body="$(cat "$output")"
  for token in "123456789" "owner-approved-v160-single-shot" "X-MBX-APIKEY" "apiSecret must not persist" "raw readback must not persist" "raw response must not persist" "signature=must_not_persist" "signedQuery=" "signedUrl="; do
    if [[ "$body" == *"$token"* ]]; then
      echo "$output contains forbidden token $token" >&2
      exit 1
    fi
  done
done

echo "verify_v18_cancel_recovery_incident_audit_closeout PASS root=$GATE_ROOT incident_closeout_ready=true audit_trail_ready=true network_attempted=false cancel_attempted=false retry_attempted=false remediation_attempted=false automatic_remediation_allowed=false dashboard_cancel_controls_enabled=false"
