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
    v20_failure_no_retry::{
        FailureNextAllowedAction, FailureNoRetryCategory, FailureNoRetryCode,
        FailureNoRetryRequest, FailureSourceEvidence, FailureSourceEvidenceKind,
        FailureTerminalAction, V20_FAILURE_NO_RETRY_SCHEMA_VERSION,
        build_failure_no_retry_evidence,
    },
    v20_pre_submit_gate::V20_ORDER_LIFECYCLE_CONTRACT_ID,
};

const NOW_NS: u64 = 1_780_000_000_000_000_000;

#[test]
fn emits_stable_failure_evidence_for_all_categories() {
    let cases = [
        (
            FailureNoRetryCategory::Blocked,
            FailureNoRetryCode::Blocked,
            "v200_failure_blocked",
            FailureNextAllowedAction::NoActionUntilEvidenceReady,
        ),
        (
            FailureNoRetryCategory::ValidationFailed,
            FailureNoRetryCode::ValidationFailed,
            "v200_failure_validation_failed",
            FailureNextAllowedAction::FixInputAndRebuild,
        ),
        (
            FailureNoRetryCategory::ApprovalFailed,
            FailureNoRetryCode::ApprovalFailed,
            "v200_failure_approval_failed",
            FailureNextAllowedAction::RequestOwnerApproval,
        ),
        (
            FailureNoRetryCategory::CredentialUnavailable,
            FailureNoRetryCode::CredentialUnavailable,
            "v200_failure_credential_unavailable",
            FailureNextAllowedAction::ProvideSigningMaterial,
        ),
        (
            FailureNoRetryCategory::SubmitFailed,
            FailureNoRetryCode::SubmitFailed,
            "v200_failure_submit_failed",
            FailureNextAllowedAction::WriteSubmitFailureEvidence,
        ),
        (
            FailureNoRetryCategory::VenueRejected,
            FailureNoRetryCode::VenueRejected,
            "v200_failure_venue_rejected",
            FailureNextAllowedAction::AuditVenueRejection,
        ),
        (
            FailureNoRetryCategory::ResponseUnknown,
            FailureNoRetryCode::ResponseUnknown,
            "v200_failure_response_unknown",
            FailureNextAllowedAction::ManualReviewUnknownResponse,
        ),
        (
            FailureNoRetryCategory::ReadbackMissing,
            FailureNoRetryCode::ReadbackMissing,
            "v200_failure_readback_missing",
            FailureNextAllowedAction::PrepareCancelOrAudit,
        ),
        (
            FailureNoRetryCategory::ReadbackMismatch,
            FailureNoRetryCode::ReadbackMismatch,
            "v200_failure_readback_mismatch",
            FailureNextAllowedAction::PrepareCancelOrAudit,
        ),
        (
            FailureNoRetryCategory::CancelRequired,
            FailureNoRetryCode::CancelRequired,
            "v200_failure_cancel_required",
            FailureNextAllowedAction::PrepareOwnerApprovedCancel,
        ),
        (
            FailureNoRetryCategory::AuditIncomplete,
            FailureNoRetryCode::AuditIncomplete,
            "v200_failure_audit_incomplete",
            FailureNextAllowedAction::CompleteAudit,
        ),
    ];

    for (category, code, code_text, next_action) in cases {
        let request = request_for(category);
        let evidence = build_failure_no_retry_evidence(&request);

        assert_eq!(evidence.schema_version, V20_FAILURE_NO_RETRY_SCHEMA_VERSION);
        assert_eq!(evidence.contract_id, V20_ORDER_LIFECYCLE_CONTRACT_ID);
        assert_eq!(evidence.category, category);
        assert_eq!(evidence.code, code);
        assert_eq!(evidence.code.as_str(), code_text);
        assert_eq!(evidence.reason, request.reason);
        assert_eq!(evidence.source_evidence, request.source_evidence);
        assert_eq!(evidence.next_allowed_action, next_action);
        assert_eq!(
            evidence.terminal_action,
            FailureTerminalAction::WriteEvidenceAndStop
        );
        assert!(evidence.evidence_written);
        assert!(evidence.stop_after_evidence);
        assert!(evidence.dashboard_audit_consumable);
        assert!(evidence.release_gate_consumable);
        assert!(evidence.owner_visible);
        assert!(evidence.operator_visible);
        assert!(evidence.no_implicit_retry);
        assert_no_retry_or_remediation(&evidence);
    }
}

#[test]
fn keeps_unknown_state_visible_for_response_unknown_and_audit_incomplete() {
    let response_unknown =
        build_failure_no_retry_evidence(&request_for(FailureNoRetryCategory::ResponseUnknown));
    let audit_incomplete =
        build_failure_no_retry_evidence(&request_for(FailureNoRetryCategory::AuditIncomplete));
    let validation_failed =
        build_failure_no_retry_evidence(&request_for(FailureNoRetryCategory::ValidationFailed));

    assert!(response_unknown.unknown_state_visible);
    assert!(audit_incomplete.unknown_state_visible);
    assert!(!validation_failed.unknown_state_visible);
}

#[test]
fn blocks_without_source_evidence() {
    let mut request = request_for(FailureNoRetryCategory::SubmitFailed);
    request.source_evidence_ready = false;
    request.source_evidence.source_id.clear();

    let evidence = build_failure_no_retry_evidence(&request);

    assert_eq!(evidence.category, FailureNoRetryCategory::Blocked);
    assert_eq!(evidence.code, FailureNoRetryCode::SourceEvidenceMissing);
    assert_eq!(
        evidence.code.as_str(),
        "v200_failure_source_evidence_missing"
    );
    assert!(!evidence.evidence_written);
    assert!(!evidence.dashboard_audit_consumable);
    assert!(!evidence.release_gate_consumable);
    assert!(evidence.unknown_state_visible);
    assert_no_retry_or_remediation(&evidence);
}

#[test]
fn blocks_source_lineage_mismatch_before_dashboard_or_release_consumption() {
    let mut request = request_for(FailureNoRetryCategory::ReadbackMismatch);
    request.source_evidence.lifecycle_id = "unexpected-lifecycle".to_string();

    let evidence = build_failure_no_retry_evidence(&request);

    assert_eq!(evidence.category, FailureNoRetryCategory::Blocked);
    assert_eq!(evidence.code, FailureNoRetryCode::SourceLineageMismatch);
    assert_eq!(
        evidence.code.as_str(),
        "v200_failure_source_lineage_mismatch"
    );
    assert_eq!(
        evidence.next_allowed_action,
        FailureNextAllowedAction::NoActionUntilEvidenceReady
    );
    assert!(!evidence.evidence_written);
    assert!(!evidence.dashboard_audit_consumable);
    assert!(!evidence.release_gate_consumable);
    assert_no_retry_or_remediation(&evidence);
}

#[test]
fn blocks_without_human_readable_reason() {
    let mut request = request_for(FailureNoRetryCategory::VenueRejected);
    request.reason.clear();

    let evidence = build_failure_no_retry_evidence(&request);

    assert_eq!(evidence.category, FailureNoRetryCategory::Blocked);
    assert_eq!(evidence.code, FailureNoRetryCode::ReasonMissing);
    assert_eq!(evidence.code.as_str(), "v200_failure_reason_missing");
    assert!(!evidence.evidence_written);
    assert!(!evidence.dashboard_audit_consumable);
    assert!(!evidence.release_gate_consumable);
    assert_no_retry_or_remediation(&evidence);
}

fn request_for(category: FailureNoRetryCategory) -> FailureNoRetryRequest {
    FailureNoRetryRequest {
        failure_id: format!("failure-v200-009-{}", category_name(category)),
        lifecycle_id: "lc-v200-009".to_string(),
        attempt_id: Some("attempt-v200-009".to_string()),
        category,
        reason: format!("{} failure evidence recorded", category_name(category)),
        source_evidence: source_for(category),
        source_evidence_ready: true,
        occurred_at_unix_ns: NOW_NS,
    }
}

fn source_for(category: FailureNoRetryCategory) -> FailureSourceEvidence {
    let (kind, schema_version, state, code) = match category {
        FailureNoRetryCategory::ValidationFailed => (
            FailureSourceEvidenceKind::PreSubmitRiskGate,
            "ntpro.v200_pre_submit_risk_gate_decision.v1",
            "deny",
            "v200_pre_submit_notional_limit_exceeded",
        ),
        FailureNoRetryCategory::ApprovalFailed => (
            FailureSourceEvidenceKind::OwnerApproval,
            "ntpro.v200_owner_approval_lifecycle_event.v1",
            "expired",
            "v200_owner_approval_expired",
        ),
        FailureNoRetryCategory::CredentialUnavailable => (
            FailureSourceEvidenceKind::SigningMaterialGate,
            "ntpro.v200_signing_material_gate.v1",
            "blocked",
            "v200_signing_material_missing",
        ),
        FailureNoRetryCategory::SubmitFailed => (
            FailureSourceEvidenceKind::GuardedSubmitCandidate,
            "ntpro.v200_guarded_single_shot_submit_candidate.v1",
            "blocked",
            "v200_guarded_submit_manual_gate_missing",
        ),
        FailureNoRetryCategory::VenueRejected => (
            FailureSourceEvidenceKind::SubmitResponseRedaction,
            "ntpro.v200_submit_response_redaction.v1",
            "rejected",
            "v200_submit_response_rejected",
        ),
        FailureNoRetryCategory::ResponseUnknown => (
            FailureSourceEvidenceKind::SubmitResponseRedaction,
            "ntpro.v200_submit_response_redaction.v1",
            "unknown",
            "v200_submit_response_unknown",
        ),
        FailureNoRetryCategory::ReadbackMissing => (
            FailureSourceEvidenceKind::SubmitReadbackReconciliation,
            "ntpro.v200_submit_readback_reconciliation.v1",
            "missing",
            "v200_submit_readback_missing",
        ),
        FailureNoRetryCategory::ReadbackMismatch => (
            FailureSourceEvidenceKind::SubmitReadbackReconciliation,
            "ntpro.v200_submit_readback_reconciliation.v1",
            "mismatched",
            "v200_submit_readback_mismatched",
        ),
        FailureNoRetryCategory::CancelRequired => (
            FailureSourceEvidenceKind::SubmitReadbackReconciliation,
            "ntpro.v200_submit_readback_reconciliation.v1",
            "mismatched",
            "v200_submit_readback_mismatched",
        ),
        FailureNoRetryCategory::AuditIncomplete => (
            FailureSourceEvidenceKind::AuditTrail,
            "ntpro.v200_order_lifecycle_audit.v1",
            "incomplete",
            "v200_audit_incomplete",
        ),
        FailureNoRetryCategory::Blocked => (
            FailureSourceEvidenceKind::SubmitRequestBuilder,
            "ntpro.v200_single_shot_submit_request_builder.v1",
            "blocked",
            "v200_submit_request_missing_risk_allow",
        ),
    };

    FailureSourceEvidence {
        kind,
        source_id: format!("source-v200-009-{}", category_name(category)),
        schema_version: schema_version.to_string(),
        lifecycle_id: "lc-v200-009".to_string(),
        attempt_id: Some("attempt-v200-009".to_string()),
        state: Some(state.to_string()),
        code: Some(code.to_string()),
    }
}

fn assert_no_retry_or_remediation(
    evidence: &nautilus_risk::v20_failure_no_retry::FailureNoRetryEvidence,
) {
    assert!(!evidence.retry_allowed);
    assert!(!evidence.retry_attempted);
    assert_eq!(evidence.retry_attempts, 0);
    assert_eq!(evidence.max_retry_attempts, 0);
    assert!(!evidence.replace_attempted);
    assert!(!evidence.amend_attempted);
    assert!(!evidence.flatten_attempted);
    assert!(!evidence.automatic_cancel_attempted);
    assert!(!evidence.automatic_remediation_allowed);
    assert!(!evidence.strategy_continuation_allowed);
    assert!(!evidence.dashboard_order_controls_enabled);
}

const fn category_name(category: FailureNoRetryCategory) -> &'static str {
    match category {
        FailureNoRetryCategory::Blocked => "blocked",
        FailureNoRetryCategory::ValidationFailed => "validation_failed",
        FailureNoRetryCategory::ApprovalFailed => "approval_failed",
        FailureNoRetryCategory::CredentialUnavailable => "credential_unavailable",
        FailureNoRetryCategory::SubmitFailed => "submit_failed",
        FailureNoRetryCategory::VenueRejected => "venue_rejected",
        FailureNoRetryCategory::ResponseUnknown => "response_unknown",
        FailureNoRetryCategory::ReadbackMissing => "readback_missing",
        FailureNoRetryCategory::ReadbackMismatch => "readback_mismatch",
        FailureNoRetryCategory::CancelRequired => "cancel_required",
        FailureNoRetryCategory::AuditIncomplete => "audit_incomplete",
    }
}
