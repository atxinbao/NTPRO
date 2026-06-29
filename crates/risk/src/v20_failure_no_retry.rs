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

//! V200 failure and no-retry evidence model.

use serde::{Deserialize, Serialize};

use crate::v20_pre_submit_gate::V20_ORDER_LIFECYCLE_CONTRACT_ID;

/// Stable schema for V200 failure and no-retry evidence.
pub const V20_FAILURE_NO_RETRY_SCHEMA_VERSION: &str = "ntpro.v200_failure_no_retry_evidence.v1";

/// Failure category consumed by Dashboard audit and release gates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureNoRetryCategory {
    Blocked,
    ValidationFailed,
    ApprovalFailed,
    CredentialUnavailable,
    SubmitFailed,
    VenueRejected,
    ResponseUnknown,
    ReadbackMissing,
    ReadbackMismatch,
    CancelRequired,
    AuditIncomplete,
}

/// Stable code for every V200 failure/no-retry outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureNoRetryCode {
    #[serde(rename = "v200_failure_blocked")]
    Blocked,
    #[serde(rename = "v200_failure_validation_failed")]
    ValidationFailed,
    #[serde(rename = "v200_failure_approval_failed")]
    ApprovalFailed,
    #[serde(rename = "v200_failure_credential_unavailable")]
    CredentialUnavailable,
    #[serde(rename = "v200_failure_submit_failed")]
    SubmitFailed,
    #[serde(rename = "v200_failure_venue_rejected")]
    VenueRejected,
    #[serde(rename = "v200_failure_response_unknown")]
    ResponseUnknown,
    #[serde(rename = "v200_failure_readback_missing")]
    ReadbackMissing,
    #[serde(rename = "v200_failure_readback_mismatch")]
    ReadbackMismatch,
    #[serde(rename = "v200_failure_cancel_required")]
    CancelRequired,
    #[serde(rename = "v200_failure_audit_incomplete")]
    AuditIncomplete,
    #[serde(rename = "v200_failure_id_missing")]
    FailureIdMissing,
    #[serde(rename = "v200_failure_lifecycle_id_missing")]
    LifecycleIdMissing,
    #[serde(rename = "v200_failure_reason_missing")]
    ReasonMissing,
    #[serde(rename = "v200_failure_source_evidence_missing")]
    SourceEvidenceMissing,
    #[serde(rename = "v200_failure_source_lineage_mismatch")]
    SourceLineageMismatch,
}

impl FailureNoRetryCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Blocked => "v200_failure_blocked",
            Self::ValidationFailed => "v200_failure_validation_failed",
            Self::ApprovalFailed => "v200_failure_approval_failed",
            Self::CredentialUnavailable => "v200_failure_credential_unavailable",
            Self::SubmitFailed => "v200_failure_submit_failed",
            Self::VenueRejected => "v200_failure_venue_rejected",
            Self::ResponseUnknown => "v200_failure_response_unknown",
            Self::ReadbackMissing => "v200_failure_readback_missing",
            Self::ReadbackMismatch => "v200_failure_readback_mismatch",
            Self::CancelRequired => "v200_failure_cancel_required",
            Self::AuditIncomplete => "v200_failure_audit_incomplete",
            Self::FailureIdMissing => "v200_failure_id_missing",
            Self::LifecycleIdMissing => "v200_failure_lifecycle_id_missing",
            Self::ReasonMissing => "v200_failure_reason_missing",
            Self::SourceEvidenceMissing => "v200_failure_source_evidence_missing",
            Self::SourceLineageMismatch => "v200_failure_source_lineage_mismatch",
        }
    }
}

/// Evidence family that caused the failure/no-retry record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureSourceEvidenceKind {
    PreSubmitRiskGate,
    OwnerApproval,
    SigningMaterialGate,
    SubmitRequestBuilder,
    GuardedSubmitCandidate,
    SubmitResponseRedaction,
    SubmitReadbackReconciliation,
    CancelFollowup,
    AuditTrail,
}

/// Next action that remains allowed after writing failure evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureNextAllowedAction {
    FixInputAndRebuild,
    RequestOwnerApproval,
    ProvideSigningMaterial,
    WriteSubmitFailureEvidence,
    AuditVenueRejection,
    ManualReviewUnknownResponse,
    PrepareCancelOrAudit,
    AuditReadbackFailure,
    PrepareOwnerApprovedCancel,
    CompleteAudit,
    NoActionUntilEvidenceReady,
}

/// Terminal behavior required for every failure/no-retry category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureTerminalAction {
    WriteEvidenceAndStop,
}

/// Source evidence pointer retained without embedding raw payloads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureSourceEvidence {
    pub kind: FailureSourceEvidenceKind,
    pub source_id: String,
    pub schema_version: String,
    pub lifecycle_id: String,
    pub attempt_id: Option<String>,
    pub state: Option<String>,
    pub code: Option<String>,
}

/// Request to build one terminal failure/no-retry evidence record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureNoRetryRequest {
    pub failure_id: String,
    pub lifecycle_id: String,
    pub attempt_id: Option<String>,
    pub category: FailureNoRetryCategory,
    pub reason: String,
    pub source_evidence: FailureSourceEvidence,
    pub source_evidence_ready: bool,
    pub occurred_at_unix_ns: u64,
}

/// Auditable V200 failure record consumed by Dashboard audit and release gates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureNoRetryEvidence {
    pub schema_version: String,
    pub contract_id: String,
    pub failure_id: String,
    pub lifecycle_id: String,
    pub attempt_id: Option<String>,
    pub category: FailureNoRetryCategory,
    pub code: FailureNoRetryCode,
    pub reason: String,
    pub source_evidence: FailureSourceEvidence,
    pub source_evidence_ready: bool,
    pub occurred_at_unix_ns: u64,
    pub next_allowed_action: FailureNextAllowedAction,
    pub terminal_action: FailureTerminalAction,
    pub evidence_written: bool,
    pub stop_after_evidence: bool,
    pub dashboard_audit_consumable: bool,
    pub release_gate_consumable: bool,
    pub owner_visible: bool,
    pub operator_visible: bool,
    pub unknown_state_visible: bool,
    pub no_implicit_retry: bool,
    pub retry_allowed: bool,
    pub retry_attempted: bool,
    pub retry_attempts: u32,
    pub max_retry_attempts: u32,
    pub replace_attempted: bool,
    pub amend_attempted: bool,
    pub flatten_attempted: bool,
    pub automatic_cancel_attempted: bool,
    pub automatic_remediation_allowed: bool,
    pub strategy_continuation_allowed: bool,
    pub dashboard_order_controls_enabled: bool,
}

impl FailureNoRetryEvidence {
    fn from_request(request: &FailureNoRetryRequest) -> Self {
        Self {
            schema_version: V20_FAILURE_NO_RETRY_SCHEMA_VERSION.to_string(),
            contract_id: V20_ORDER_LIFECYCLE_CONTRACT_ID.to_string(),
            failure_id: request.failure_id.clone(),
            lifecycle_id: request.lifecycle_id.clone(),
            attempt_id: request.attempt_id.clone(),
            category: FailureNoRetryCategory::Blocked,
            code: FailureNoRetryCode::FailureIdMissing,
            reason: String::new(),
            source_evidence: request.source_evidence.clone(),
            source_evidence_ready: request.source_evidence_ready,
            occurred_at_unix_ns: request.occurred_at_unix_ns,
            next_allowed_action: FailureNextAllowedAction::NoActionUntilEvidenceReady,
            terminal_action: FailureTerminalAction::WriteEvidenceAndStop,
            evidence_written: false,
            stop_after_evidence: false,
            dashboard_audit_consumable: false,
            release_gate_consumable: false,
            owner_visible: false,
            operator_visible: false,
            unknown_state_visible: false,
            no_implicit_retry: true,
            retry_allowed: false,
            retry_attempted: false,
            retry_attempts: 0,
            max_retry_attempts: 0,
            replace_attempted: false,
            amend_attempted: false,
            flatten_attempted: false,
            automatic_cancel_attempted: false,
            automatic_remediation_allowed: false,
            strategy_continuation_allowed: false,
            dashboard_order_controls_enabled: false,
        }
    }

    fn finish_blocked(mut self, code: FailureNoRetryCode, reason: impl Into<String>) -> Self {
        self.category = FailureNoRetryCategory::Blocked;
        self.code = code;
        self.reason = reason.into();
        self.unknown_state_visible = true;
        self
    }

    fn finish_ready(mut self, category: FailureNoRetryCategory, reason: impl Into<String>) -> Self {
        self.category = category;
        self.code = code_for_category(category);
        self.reason = reason.into();
        self.next_allowed_action = next_action_for_category(category);
        self.evidence_written = true;
        self.stop_after_evidence = true;
        self.dashboard_audit_consumable = true;
        self.release_gate_consumable = true;
        self.owner_visible = true;
        self.operator_visible = true;
        self.unknown_state_visible = matches!(
            category,
            FailureNoRetryCategory::Blocked
                | FailureNoRetryCategory::ResponseUnknown
                | FailureNoRetryCategory::AuditIncomplete
        );
        self
    }
}

/// Builds one terminal failure/no-retry evidence record.
#[must_use]
pub fn build_failure_no_retry_evidence(request: &FailureNoRetryRequest) -> FailureNoRetryEvidence {
    let evidence = FailureNoRetryEvidence::from_request(request);

    if is_blank(&request.failure_id) {
        return evidence.finish_blocked(
            FailureNoRetryCode::FailureIdMissing,
            "failure_id is required",
        );
    }
    if is_blank(&request.lifecycle_id) {
        return evidence.finish_blocked(
            FailureNoRetryCode::LifecycleIdMissing,
            "lifecycle_id is required",
        );
    }
    if !request.source_evidence_ready
        || is_blank(&request.source_evidence.source_id)
        || is_blank(&request.source_evidence.schema_version)
        || is_blank(&request.source_evidence.lifecycle_id)
    {
        return evidence.finish_blocked(
            FailureNoRetryCode::SourceEvidenceMissing,
            "source evidence is required before failure/no-retry evidence can be written",
        );
    }
    if request.source_evidence.lifecycle_id != request.lifecycle_id
        || request.source_evidence.attempt_id != request.attempt_id
    {
        return evidence.finish_blocked(
            FailureNoRetryCode::SourceLineageMismatch,
            "source evidence lineage does not match the failure request",
        );
    }
    if is_blank(&request.reason) {
        return evidence.finish_blocked(
            FailureNoRetryCode::ReasonMissing,
            "human-readable failure reason is required",
        );
    }

    evidence.finish_ready(request.category, request.reason.clone())
}

const fn code_for_category(category: FailureNoRetryCategory) -> FailureNoRetryCode {
    match category {
        FailureNoRetryCategory::Blocked => FailureNoRetryCode::Blocked,
        FailureNoRetryCategory::ValidationFailed => FailureNoRetryCode::ValidationFailed,
        FailureNoRetryCategory::ApprovalFailed => FailureNoRetryCode::ApprovalFailed,
        FailureNoRetryCategory::CredentialUnavailable => FailureNoRetryCode::CredentialUnavailable,
        FailureNoRetryCategory::SubmitFailed => FailureNoRetryCode::SubmitFailed,
        FailureNoRetryCategory::VenueRejected => FailureNoRetryCode::VenueRejected,
        FailureNoRetryCategory::ResponseUnknown => FailureNoRetryCode::ResponseUnknown,
        FailureNoRetryCategory::ReadbackMissing => FailureNoRetryCode::ReadbackMissing,
        FailureNoRetryCategory::ReadbackMismatch => FailureNoRetryCode::ReadbackMismatch,
        FailureNoRetryCategory::CancelRequired => FailureNoRetryCode::CancelRequired,
        FailureNoRetryCategory::AuditIncomplete => FailureNoRetryCode::AuditIncomplete,
    }
}

const fn next_action_for_category(category: FailureNoRetryCategory) -> FailureNextAllowedAction {
    match category {
        FailureNoRetryCategory::Blocked => FailureNextAllowedAction::NoActionUntilEvidenceReady,
        FailureNoRetryCategory::ValidationFailed => FailureNextAllowedAction::FixInputAndRebuild,
        FailureNoRetryCategory::ApprovalFailed => FailureNextAllowedAction::RequestOwnerApproval,
        FailureNoRetryCategory::CredentialUnavailable => {
            FailureNextAllowedAction::ProvideSigningMaterial
        }
        FailureNoRetryCategory::SubmitFailed => {
            FailureNextAllowedAction::WriteSubmitFailureEvidence
        }
        FailureNoRetryCategory::VenueRejected => FailureNextAllowedAction::AuditVenueRejection,
        FailureNoRetryCategory::ResponseUnknown => {
            FailureNextAllowedAction::ManualReviewUnknownResponse
        }
        FailureNoRetryCategory::ReadbackMissing | FailureNoRetryCategory::ReadbackMismatch => {
            FailureNextAllowedAction::PrepareCancelOrAudit
        }
        FailureNoRetryCategory::CancelRequired => {
            FailureNextAllowedAction::PrepareOwnerApprovedCancel
        }
        FailureNoRetryCategory::AuditIncomplete => FailureNextAllowedAction::CompleteAudit,
    }
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}
