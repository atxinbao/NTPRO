// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

use nautilus_risk::{
    v20_owner_approval::OwnerApprovalState,
    v20_pre_submit_gate::{
        V20_ORDER_LIFECYCLE_CONTRACT_ID, V20_REQUIRED_RELEASE_GATE, V20_REQUIRED_RELEASE_TAG,
    },
    v20_submit_candidate::{
        GuardedSubmitCandidateCode, GuardedSubmitCandidateEvidence, GuardedSubmitCandidateState,
        GuardedSubmitMode, V20_GUARDED_SUBMIT_CANDIDATE_SCHEMA_VERSION,
    },
    v20_submit_response_redaction::{
        SubmitResponseKind, SubmitResponseRedactionCode, SubmitResponseRedactionRequest,
        SubmitResponseRedactionState, V20_SUBMIT_RESPONSE_REDACTION_SCHEMA_VERSION,
        redact_production_submit_response,
    },
};

const NOW_NS: u64 = 1_780_000_000_000_000_000;
const REQUEST_DIGEST: &str = "request-digest-v200-007";
const V20_RELEASE_COMMIT: &str = "d29a764a2fb6b3f9c187d2af17337b08b40d794b";

#[test]
fn redacts_accepted_response_for_readback_correlation() {
    let request = accepted_response();
    let first = redact_production_submit_response(&request, &submitted_attempt());
    let second = redact_production_submit_response(&request, &submitted_attempt());

    assert_eq!(
        first.schema_version,
        V20_SUBMIT_RESPONSE_REDACTION_SCHEMA_VERSION
    );
    assert_eq!(first.state, SubmitResponseRedactionState::Accepted);
    assert_eq!(first.code, SubmitResponseRedactionCode::Accepted);
    assert_eq!(first.code.as_str(), "v200_submit_response_accepted");
    assert_eq!(first.order_id.as_deref(), Some("venue-order-007"));
    assert_eq!(first.client_order_id.as_deref(), Some("O-V200-007"));
    assert_eq!(first.venue_status.as_deref(), Some("NEW"));
    assert_eq!(first.request_digest.as_deref(), Some(REQUEST_DIGEST));
    assert_eq!(first.response_digest, second.response_digest);
    assert!(first.response_redacted);
    assert!(first.redacted_evidence_ready);
    assert!(first.readback_correlation_ready);
    assert!(!first.readback_success_inferred);
    assert!(!first.manual_review_required);
    assert!(!first.raw_exchange_response_recorded);
    assert!(!first.response_headers_recorded);
    assert!(!first.unrestricted_payload_recorded);
    assert!(!first.credential_material_recorded);
    assert!(!first.signature_material_recorded);
    assert!(!first.token_value_recorded);
    assert!(!first.signed_query_recorded);
    assert!(!first.signed_url_recorded);
    assert!(!first.dashboard_raw_response_enabled);
    assert!(!first.dashboard_order_controls_enabled);
}

#[test]
fn redacts_rejected_response_with_stable_reason_codes() {
    let mut request = accepted_response();
    request.response_id = "response-rejected-v200-007".to_string();
    request.response_kind = SubmitResponseKind::Rejected;
    request.http_status = Some(400);
    request.venue_status = Some("REJECTED".to_string());
    request.order_id = None;
    request.reject_code = Some("MIN_NOTIONAL".to_string());
    request.reject_reason_code = Some("min_notional".to_string());

    let evidence = redact_production_submit_response(&request, &submitted_attempt());

    assert_eq!(evidence.state, SubmitResponseRedactionState::Rejected);
    assert_eq!(evidence.code, SubmitResponseRedactionCode::Rejected);
    assert_eq!(evidence.reject_code.as_deref(), Some("MIN_NOTIONAL"));
    assert_eq!(evidence.reject_reason_code.as_deref(), Some("min_notional"));
    assert_eq!(evidence.client_order_id.as_deref(), Some("O-V200-007"));
    assert!(evidence.readback_correlation_ready);
    assert!(!evidence.readback_success_inferred);
}

#[test]
fn redacts_unknown_response_without_inferred_success() {
    let mut request = accepted_response();
    request.response_id = "response-unknown-v200-007".to_string();
    request.response_kind = SubmitResponseKind::Unknown;
    request.http_status = Some(202);
    request.venue_status = Some("PENDING_UNKNOWN".to_string());
    request.order_id = None;

    let evidence = redact_production_submit_response(&request, &submitted_attempt());

    assert_eq!(evidence.state, SubmitResponseRedactionState::Unknown);
    assert_eq!(evidence.code, SubmitResponseRedactionCode::Unknown);
    assert!(evidence.manual_review_required);
    assert!(evidence.readback_correlation_ready);
    assert!(!evidence.readback_success_inferred);
}

#[test]
fn redacts_malformed_response_as_diagnostic_only() {
    let mut request = accepted_response();
    request.response_id = "response-malformed-v200-007".to_string();
    request.response_kind = SubmitResponseKind::Malformed;
    request.http_status = None;
    request.venue_status = None;
    request.order_id = None;
    request.client_order_id = None;
    request.venue_timestamp_unix_ms = None;
    request.malformed_reason_code = Some("json_parse_error".to_string());

    let evidence = redact_production_submit_response(&request, &submitted_attempt());

    assert_eq!(evidence.state, SubmitResponseRedactionState::Malformed);
    assert_eq!(evidence.code, SubmitResponseRedactionCode::Malformed);
    assert_eq!(
        evidence.malformed_reason_code.as_deref(),
        Some("json_parse_error")
    );
    assert!(evidence.redacted_evidence_ready);
    assert!(evidence.manual_review_required);
    assert!(!evidence.readback_correlation_ready);
    assert!(!evidence.readback_success_inferred);
    assert!(!evidence.raw_exchange_response_recorded);
}

#[test]
fn blocks_request_digest_mismatch() {
    let mut request = accepted_response();
    request.request_digest = Some("unexpected-request-digest".to_string());

    let evidence = redact_production_submit_response(&request, &submitted_attempt());

    assert_eq!(evidence.state, SubmitResponseRedactionState::Blocked);
    assert_eq!(
        evidence.code,
        SubmitResponseRedactionCode::RequestDigestMismatch
    );
    assert!(!evidence.response_redacted);
    assert!(!evidence.redacted_evidence_ready);
}

#[test]
fn blocks_sensitive_material_without_leaking_marker() {
    let mut request = accepted_response();
    request.response_id = "response-sensitive-v200-007".to_string();
    request.venue_status = Some("prod-secret-should-never-appear".to_string());
    request.credential_material_present = true;
    request.signature_material_present = true;
    request.sensitive_marker_count = 2;

    let evidence = redact_production_submit_response(&request, &submitted_attempt());

    assert_eq!(evidence.state, SubmitResponseRedactionState::Blocked);
    assert_eq!(
        evidence.code,
        SubmitResponseRedactionCode::SensitiveMaterialObserved
    );
    assert_eq!(evidence.sensitive_marker_count, 2);
    assert!(!evidence.response_redacted);
    assert!(!evidence.credential_material_recorded);
    assert!(!evidence.signature_material_recorded);
    assert!(!evidence.raw_exchange_response_recorded);

    let serialized = serde_json::to_string(&evidence).expect("evidence serializes");
    assert!(!serialized.contains("prod-secret-should-never-appear"));
}

#[test]
fn blocks_without_submitted_attempt_evidence() {
    let mut attempt = submitted_attempt();
    attempt.state = GuardedSubmitCandidateState::Preview;
    attempt.production_submit_attempted = false;

    let evidence = redact_production_submit_response(&accepted_response(), &attempt);

    assert_eq!(evidence.state, SubmitResponseRedactionState::Blocked);
    assert_eq!(
        evidence.code,
        SubmitResponseRedactionCode::MissingSubmitAttempt
    );
}

fn accepted_response() -> SubmitResponseRedactionRequest {
    SubmitResponseRedactionRequest {
        response_id: "response-accepted-v200-007".to_string(),
        lifecycle_id: "lc-v200-007".to_string(),
        attempt_id: "attempt-v200-007".to_string(),
        request_digest: Some(REQUEST_DIGEST.to_string()),
        response_kind: SubmitResponseKind::Accepted,
        venue: "BINANCE".to_string(),
        http_status: Some(200),
        venue_status: Some("NEW".to_string()),
        order_id: Some("venue-order-007".to_string()),
        client_order_id: Some("O-V200-007".to_string()),
        venue_timestamp_unix_ms: Some(1_780_000_001_000),
        received_at_unix_ns: NOW_NS + 1_000,
        reject_code: None,
        reject_reason_code: None,
        malformed_reason_code: None,
        raw_payload_present: true,
        response_headers_present: true,
        unrestricted_payload_present: false,
        credential_material_present: false,
        signature_material_present: false,
        token_value_present: false,
        signed_query_present: false,
        signed_url_present: false,
        sensitive_marker_count: 0,
    }
}

fn submitted_attempt() -> GuardedSubmitCandidateEvidence {
    GuardedSubmitCandidateEvidence {
        schema_version: V20_GUARDED_SUBMIT_CANDIDATE_SCHEMA_VERSION.to_string(),
        contract_id: V20_ORDER_LIFECYCLE_CONTRACT_ID.to_string(),
        candidate_id: "candidate-v200-007".to_string(),
        attempt_id: "attempt-v200-007".to_string(),
        lifecycle_id: "lc-v200-007".to_string(),
        mode: GuardedSubmitMode::Submit,
        state: GuardedSubmitCandidateState::Submitted,
        code: GuardedSubmitCandidateCode::Submitted,
        reason: "guarded submit attempt evidence recorded".to_string(),
        evaluated_at_unix_ns: NOW_NS,
        request_digest: Some(REQUEST_DIGEST.to_string()),
        risk_gate_id: "risk-gate-v200-007".to_string(),
        approval_id: "approval-v200-007".to_string(),
        signing_gate_id: "signing-gate-v200-007".to_string(),
        release_tag: Some(V20_REQUIRED_RELEASE_TAG.to_string()),
        release_commit: Some(V20_RELEASE_COMMIT.to_string()),
        release_gate: Some(V20_REQUIRED_RELEASE_GATE.to_string()),
        owner_approval_state_before_attempt: OwnerApprovalState::Approved,
        owner_approval_state_after_attempt: OwnerApprovalState::Consumed,
        approval_consumed_at_unix_ns: Some(NOW_NS),
        owner_approval_consumed: true,
        previous_attempt_count: 0,
        attempt_ledger_required: true,
        attempt_ledger_trusted: true,
        attempt_ledger_key: Some("ledger-v200-007".to_string()),
        atomic_approval_consumption_required: true,
        atomic_approval_consumption_recorded: true,
        submit_attempt_evidence_ready: true,
        preview_evidence_ready: false,
        dry_run_evidence_ready: false,
        production_submit_attempted: true,
        adapter_submit_handoff_allowed: true,
        readback_required: true,
        audit_artifact_required: true,
        single_attempt_required: true,
        single_order_required: true,
        single_venue_required: true,
        single_account_required: true,
        manual_online_gate_required: true,
        manual_online_gate_present: true,
        pre_submit_risk_gate_required: true,
        owner_approval_required: true,
        signing_material_gate_required: true,
        request_digest_required: true,
        release_provenance_required: true,
        retry_attempted: false,
        replace_attempted: false,
        amend_attempted: false,
        flatten_attempted: false,
        bulk_submit_attempted: false,
        automatic_remediation_allowed: false,
        dashboard_order_controls_enabled: false,
        raw_secret_persisted: false,
        raw_signed_payload_persisted: false,
        raw_exchange_response_persisted: false,
    }
}
