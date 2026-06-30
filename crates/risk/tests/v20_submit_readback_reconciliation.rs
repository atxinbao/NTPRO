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
    v20_submit_readback_reconciliation::{
        SubmitReadbackExpectation, SubmitReadbackReconciliationCode,
        SubmitReadbackReconciliationState, V20_SUBMIT_READBACK_RECONCILIATION_SCHEMA_VERSION,
        VenueOrderReadback, reconcile_post_submit_readback,
    },
    v20_submit_response_redaction::{
        SubmitEvidenceSource, SubmitResponseKind, SubmitResponseRedactionRequest,
        redact_production_submit_response,
    },
};
use rust_decimal_macros::dec;

const NOW_NS: u64 = 1_780_000_000_000_000_000;
const REQUEST_DIGEST: &str = "request-digest-v200-008";
const V20_RELEASE_COMMIT: &str = "d29a764a2fb6b3f9c187d2af17337b08b40d794b";

#[test]
fn reconciles_matched_readback_for_read_only_audit() {
    let evidence =
        reconcile_post_submit_readback(&expectation(), &response_evidence(), &matched_readback());

    assert_eq!(
        evidence.schema_version,
        V20_SUBMIT_READBACK_RECONCILIATION_SCHEMA_VERSION
    );
    assert_eq!(evidence.state, SubmitReadbackReconciliationState::Matched);
    assert_eq!(evidence.code, SubmitReadbackReconciliationCode::Matched);
    assert_eq!(evidence.code.as_str(), "v200_submit_readback_matched");
    assert_eq!(evidence.mismatch_fields, Vec::<String>::new());
    assert!(evidence.readback_attempted);
    assert!(evidence.readback_response_present);
    assert!(evidence.readback_consistent);
    assert!(!evidence.readback_missing);
    assert!(!evidence.readback_ambiguous);
    assert!(!evidence.readback_failed);
    assert!(!evidence.risk_evidence_required);
    assert!(evidence.cancel_or_audit_input_ready);
    assert!(evidence.dashboard_read_only_consumable);
    assert_eq!(
        evidence.evidence_source,
        SubmitEvidenceSource::ManualStructured
    );
    assert_eq!(
        evidence.source_provenance_id.as_deref(),
        Some("manual-structured-readback-v200-008")
    );
    assert!(evidence.source_provenance_required);
    assert!(evidence.source_provenance_valid);
    assert!(evidence.source_claim_consistent);
    assert!(!evidence.exchange_truth_claimed);
    assert!(!evidence.adapter_runtime_integrated);
    assert!(evidence.foundation_only);
    assert!(!evidence.raw_readback_body_recorded);
    assert!(!evidence.response_headers_recorded);
    assert!(!evidence.automatic_cancel_attempted);
    assert!(!evidence.automatic_remediation_allowed);
    assert!(!evidence.retry_attempted);
    assert!(!evidence.replace_attempted);
    assert!(!evidence.amend_attempted);
    assert!(!evidence.flatten_attempted);
    assert!(!evidence.dashboard_order_controls_enabled);
}

#[test]
fn flags_mismatched_readback_fields_for_risk_evidence() {
    let mut readback = matched_readback();
    readback.quantity = Some(dec!(0.11));
    readback.venue_status = Some("PARTIALLY_FILLED".to_string());

    let evidence = reconcile_post_submit_readback(&expectation(), &response_evidence(), &readback);

    assert_eq!(
        evidence.state,
        SubmitReadbackReconciliationState::Mismatched
    );
    assert_eq!(evidence.code, SubmitReadbackReconciliationCode::Mismatched);
    assert_eq!(evidence.code.as_str(), "v200_submit_readback_mismatched");
    assert_eq!(
        evidence.mismatch_fields,
        vec!["quantity".to_string(), "venue_status".to_string()]
    );
    assert!(!evidence.readback_consistent);
    assert!(evidence.risk_evidence_required);
    assert!(evidence.cancel_or_audit_input_ready);
    assert!(evidence.dashboard_read_only_consumable);
    assert!(!evidence.automatic_cancel_attempted);
}

#[test]
fn marks_missing_order_as_risk_evidence_without_cancel_attempt() {
    let mut readback = matched_readback();
    readback.present = false;
    readback.account_label = None;
    readback.instrument_id = None;
    readback.side = None;
    readback.quantity = None;
    readback.price = None;
    readback.venue_order_id = None;
    readback.venue_status = None;
    readback.venue_timestamp_unix_ms = None;

    let evidence = reconcile_post_submit_readback(&expectation(), &response_evidence(), &readback);

    assert_eq!(evidence.state, SubmitReadbackReconciliationState::Missing);
    assert_eq!(evidence.code, SubmitReadbackReconciliationCode::Missing);
    assert_eq!(evidence.code.as_str(), "v200_submit_readback_missing");
    assert!(evidence.readback_missing);
    assert!(evidence.risk_evidence_required);
    assert!(evidence.cancel_or_audit_input_ready);
    assert!(evidence.dashboard_read_only_consumable);
    assert!(!evidence.automatic_cancel_attempted);
    assert!(!evidence.automatic_remediation_allowed);
}

#[test]
fn marks_ambiguous_readback_as_audit_input() {
    let mut readback = matched_readback();
    readback.ambiguous = true;

    let evidence = reconcile_post_submit_readback(&expectation(), &response_evidence(), &readback);

    assert_eq!(evidence.state, SubmitReadbackReconciliationState::Ambiguous);
    assert_eq!(evidence.code, SubmitReadbackReconciliationCode::Ambiguous);
    assert_eq!(evidence.code.as_str(), "v200_submit_readback_ambiguous");
    assert!(evidence.readback_ambiguous);
    assert!(evidence.risk_evidence_required);
    assert!(evidence.cancel_or_audit_input_ready);
    assert!(evidence.dashboard_read_only_consumable);
    assert!(!evidence.dashboard_order_controls_enabled);
}

#[test]
fn marks_venue_read_failure_without_retry_or_remediation() {
    let mut readback = matched_readback();
    readback.read_failed = true;
    readback.failure_code = Some("venue_timeout".to_string());

    let evidence = reconcile_post_submit_readback(&expectation(), &response_evidence(), &readback);

    assert_eq!(
        evidence.state,
        SubmitReadbackReconciliationState::ReadbackFailed
    );
    assert_eq!(
        evidence.code,
        SubmitReadbackReconciliationCode::ReadbackFailed
    );
    assert_eq!(evidence.code.as_str(), "v200_submit_readback_failed");
    assert!(evidence.readback_failed);
    assert_eq!(evidence.failure_code.as_deref(), Some("venue_timeout"));
    assert!(evidence.risk_evidence_required);
    assert!(evidence.cancel_or_audit_input_ready);
    assert!(evidence.dashboard_read_only_consumable);
    assert!(!evidence.retry_attempted);
    assert!(!evidence.automatic_remediation_allowed);
}

#[test]
fn blocks_when_response_lineage_does_not_match_expectation() {
    let mut expectation = expectation();
    expectation.request_digest = "unexpected-digest".to_string();

    let evidence =
        reconcile_post_submit_readback(&expectation, &response_evidence(), &matched_readback());

    assert_eq!(evidence.state, SubmitReadbackReconciliationState::Blocked);
    assert_eq!(
        evidence.code,
        SubmitReadbackReconciliationCode::LineageMismatch
    );
    assert_eq!(
        evidence.code.as_str(),
        "v200_submit_readback_lineage_mismatch"
    );
    assert!(!evidence.readback_attempted);
    assert!(!evidence.risk_evidence_required);
    assert!(!evidence.cancel_or_audit_input_ready);
    assert!(!evidence.dashboard_read_only_consumable);
}

#[test]
fn blocks_unknown_readback_source() {
    let mut readback = matched_readback();
    readback.evidence_source = SubmitEvidenceSource::Unknown;

    let evidence = reconcile_post_submit_readback(&expectation(), &response_evidence(), &readback);

    assert_eq!(evidence.state, SubmitReadbackReconciliationState::Blocked);
    assert_eq!(
        evidence.code,
        SubmitReadbackReconciliationCode::UnknownSource
    );
    assert_eq!(
        evidence.code.as_str(),
        "v200_submit_readback_unknown_source"
    );
    assert!(!evidence.source_provenance_valid);
    assert!(!evidence.source_claim_consistent);
}

#[test]
fn blocks_manual_readback_claimed_as_exchange_truth() {
    let mut readback = matched_readback();
    readback.exchange_truth_claimed = true;

    let evidence = reconcile_post_submit_readback(&expectation(), &response_evidence(), &readback);

    assert_eq!(evidence.state, SubmitReadbackReconciliationState::Blocked);
    assert_eq!(
        evidence.code,
        SubmitReadbackReconciliationCode::SourceClaimMismatch
    );
    assert_eq!(
        evidence.code.as_str(),
        "v200_submit_readback_source_claim_mismatch"
    );
    assert!(evidence.exchange_truth_claimed);
    assert!(evidence.foundation_only);
}

#[test]
fn blocks_adapter_readback_missing_source_provenance() {
    let mut readback = matched_readback();
    readback.evidence_source = SubmitEvidenceSource::ExchangeReadback;
    readback.adapter_runtime_integrated = true;
    readback.source_provenance_id = None;

    let evidence = reconcile_post_submit_readback(&expectation(), &response_evidence(), &readback);

    assert_eq!(evidence.state, SubmitReadbackReconciliationState::Blocked);
    assert_eq!(
        evidence.code,
        SubmitReadbackReconciliationCode::SourceProvenanceMissing
    );
    assert_eq!(
        evidence.code.as_str(),
        "v200_submit_readback_source_provenance_missing"
    );
    assert_eq!(
        evidence.evidence_source,
        SubmitEvidenceSource::ExchangeReadback
    );
    assert!(!evidence.source_provenance_valid);
}

fn expectation() -> SubmitReadbackExpectation {
    SubmitReadbackExpectation {
        lifecycle_id: "lc-v200-008".to_string(),
        attempt_id: "attempt-v200-008".to_string(),
        request_digest: REQUEST_DIGEST.to_string(),
        account_label: "acct-prod-001".to_string(),
        instrument_id: "BTCUSDT.BINANCE".to_string(),
        venue: "BINANCE".to_string(),
        side: "buy".to_string(),
        quantity: dec!(0.10),
        price: dec!(50000.00),
        client_order_id: "O-V200-008".to_string(),
        venue_order_id: Some("venue-order-008".to_string()),
        expected_venue_status: Some("NEW".to_string()),
        expected_venue_timestamp_unix_ms: Some(1_780_000_001_000),
    }
}

fn matched_readback() -> VenueOrderReadback {
    VenueOrderReadback {
        readback_id: "readback-v200-008".to_string(),
        account_label: Some("acct-prod-001".to_string()),
        instrument_id: Some("BTCUSDT.BINANCE".to_string()),
        venue: "BINANCE".to_string(),
        side: Some("buy".to_string()),
        quantity: Some(dec!(0.10)),
        price: Some(dec!(50000.00)),
        client_order_id: Some("O-V200-008".to_string()),
        venue_order_id: Some("venue-order-008".to_string()),
        venue_status: Some("NEW".to_string()),
        venue_timestamp_unix_ms: Some(1_780_000_001_000),
        present: true,
        ambiguous: false,
        read_failed: false,
        failure_code: None,
        raw_readback_body_present: true,
        response_headers_present: true,
        evidence_source: SubmitEvidenceSource::ManualStructured,
        source_provenance_id: Some("manual-structured-readback-v200-008".to_string()),
        exchange_truth_claimed: false,
        adapter_runtime_integrated: false,
    }
}

fn response_evidence()
-> nautilus_risk::v20_submit_response_redaction::SubmitResponseRedactionEvidence {
    redact_production_submit_response(&accepted_response(), &submitted_attempt())
}

fn accepted_response() -> SubmitResponseRedactionRequest {
    SubmitResponseRedactionRequest {
        response_id: "response-accepted-v200-008".to_string(),
        lifecycle_id: "lc-v200-008".to_string(),
        attempt_id: "attempt-v200-008".to_string(),
        request_digest: Some(REQUEST_DIGEST.to_string()),
        response_kind: SubmitResponseKind::Accepted,
        venue: "BINANCE".to_string(),
        http_status: Some(200),
        venue_status: Some("NEW".to_string()),
        order_id: Some("venue-order-008".to_string()),
        client_order_id: Some("O-V200-008".to_string()),
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
        evidence_source: SubmitEvidenceSource::ManualStructured,
        source_provenance_id: Some("manual-structured-response-v200-008".to_string()),
        exchange_truth_claimed: false,
        adapter_runtime_integrated: false,
    }
}

fn submitted_attempt() -> GuardedSubmitCandidateEvidence {
    GuardedSubmitCandidateEvidence {
        schema_version: V20_GUARDED_SUBMIT_CANDIDATE_SCHEMA_VERSION.to_string(),
        contract_id: V20_ORDER_LIFECYCLE_CONTRACT_ID.to_string(),
        candidate_id: "candidate-v200-008".to_string(),
        attempt_id: "attempt-v200-008".to_string(),
        lifecycle_id: "lc-v200-008".to_string(),
        mode: GuardedSubmitMode::Submit,
        state: GuardedSubmitCandidateState::Submitted,
        code: GuardedSubmitCandidateCode::Submitted,
        reason: "guarded submit attempt evidence recorded".to_string(),
        evaluated_at_unix_ns: NOW_NS,
        request_digest: Some(REQUEST_DIGEST.to_string()),
        risk_gate_id: "risk-gate-v200-008".to_string(),
        approval_id: "approval-v200-008".to_string(),
        signing_gate_id: "signing-gate-v200-008".to_string(),
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
        attempt_ledger_key: Some("ledger-v200-008".to_string()),
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
