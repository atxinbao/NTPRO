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

//! V200 guarded single-shot production submit candidate evidence.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    v20_owner_approval::{OwnerApprovalEvidence, OwnerApprovalState, consume_owner_approval},
    v20_pre_submit_gate::{
        PreSubmitRiskDecisionKind, PreSubmitRiskGateEvidence, V20_ORDER_LIFECYCLE_CONTRACT_ID,
    },
    v20_signing_material_gate::{SigningMaterialDecision, SigningMaterialGateEvidence},
    v20_submit_request_builder::{SubmitRequestBuildDecision, SubmitRequestBuilderEvidence},
};

/// Stable schema for V200 guarded submit candidate evidence.
pub const V20_GUARDED_SUBMIT_CANDIDATE_SCHEMA_VERSION: &str =
    "ntpro.v200_guarded_single_shot_submit_candidate.v1";

/// Operator-selected candidate mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardedSubmitMode {
    Preview,
    DryRun,
    Submit,
}

/// Final guarded submit candidate state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardedSubmitCandidateState {
    Blocked,
    Preview,
    DryRun,
    Submitted,
}

/// Stable guarded submit candidate evidence codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuardedSubmitCandidateCode {
    #[serde(rename = "v200_guarded_submit_preview_ready")]
    PreviewReady,
    #[serde(rename = "v200_guarded_submit_dry_run_ready")]
    DryRunReady,
    #[serde(rename = "v200_guarded_submit_submitted")]
    Submitted,
    #[serde(rename = "v200_guarded_submit_candidate_id_missing")]
    CandidateIdMissing,
    #[serde(rename = "v200_guarded_submit_attempt_id_missing")]
    AttemptIdMissing,
    #[serde(rename = "v200_guarded_submit_lifecycle_mismatch")]
    LifecycleMismatch,
    #[serde(rename = "v200_guarded_submit_missing_risk_allow")]
    MissingRiskAllow,
    #[serde(rename = "v200_guarded_submit_missing_owner_approval")]
    MissingOwnerApproval,
    #[serde(rename = "v200_guarded_submit_missing_signing_readiness")]
    MissingSigningReadiness,
    #[serde(rename = "v200_guarded_submit_missing_request_build")]
    MissingRequestBuild,
    #[serde(rename = "v200_guarded_submit_evidence_mismatch")]
    EvidenceMismatch,
    #[serde(rename = "v200_guarded_submit_missing_release_provenance")]
    MissingReleaseProvenance,
    #[serde(rename = "v200_guarded_submit_request_digest_missing")]
    RequestDigestMissing,
    #[serde(rename = "v200_guarded_submit_request_digest_mismatch")]
    RequestDigestMismatch,
    #[serde(rename = "v200_guarded_submit_manual_gate_missing")]
    ManualGateMissing,
    #[serde(rename = "v200_guarded_submit_duplicate_rejected")]
    DuplicateSubmitRejected,
}

impl GuardedSubmitCandidateCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreviewReady => "v200_guarded_submit_preview_ready",
            Self::DryRunReady => "v200_guarded_submit_dry_run_ready",
            Self::Submitted => "v200_guarded_submit_submitted",
            Self::CandidateIdMissing => "v200_guarded_submit_candidate_id_missing",
            Self::AttemptIdMissing => "v200_guarded_submit_attempt_id_missing",
            Self::LifecycleMismatch => "v200_guarded_submit_lifecycle_mismatch",
            Self::MissingRiskAllow => "v200_guarded_submit_missing_risk_allow",
            Self::MissingOwnerApproval => "v200_guarded_submit_missing_owner_approval",
            Self::MissingSigningReadiness => "v200_guarded_submit_missing_signing_readiness",
            Self::MissingRequestBuild => "v200_guarded_submit_missing_request_build",
            Self::EvidenceMismatch => "v200_guarded_submit_evidence_mismatch",
            Self::MissingReleaseProvenance => "v200_guarded_submit_missing_release_provenance",
            Self::RequestDigestMissing => "v200_guarded_submit_request_digest_missing",
            Self::RequestDigestMismatch => "v200_guarded_submit_request_digest_mismatch",
            Self::ManualGateMissing => "v200_guarded_submit_manual_gate_missing",
            Self::DuplicateSubmitRejected => "v200_guarded_submit_duplicate_rejected",
        }
    }
}

/// Request to evaluate one guarded single-shot submit candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuardedSubmitCandidateRequest {
    pub candidate_id: String,
    pub attempt_id: String,
    pub lifecycle_id: String,
    pub mode: GuardedSubmitMode,
    pub manual_online_gate: bool,
    pub expected_request_digest: Option<String>,
    pub prior_attempt_digests: BTreeSet<String>,
}

/// Auditable evidence for one guarded single-shot submit candidate decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuardedSubmitCandidateEvidence {
    pub schema_version: String,
    pub contract_id: String,
    pub candidate_id: String,
    pub attempt_id: String,
    pub lifecycle_id: String,
    pub mode: GuardedSubmitMode,
    pub state: GuardedSubmitCandidateState,
    pub code: GuardedSubmitCandidateCode,
    pub reason: String,
    pub evaluated_at_unix_ns: u64,
    pub request_digest: Option<String>,
    pub risk_gate_id: String,
    pub approval_id: String,
    pub signing_gate_id: String,
    pub release_tag: Option<String>,
    pub release_commit: Option<String>,
    pub release_gate: Option<String>,
    pub owner_approval_state_before_attempt: OwnerApprovalState,
    pub owner_approval_state_after_attempt: OwnerApprovalState,
    pub approval_consumed_at_unix_ns: Option<u64>,
    pub owner_approval_consumed: bool,
    pub previous_attempt_count: usize,
    pub submit_attempt_evidence_ready: bool,
    pub preview_evidence_ready: bool,
    pub dry_run_evidence_ready: bool,
    pub production_submit_attempted: bool,
    pub adapter_submit_handoff_allowed: bool,
    pub readback_required: bool,
    pub audit_artifact_required: bool,
    pub single_attempt_required: bool,
    pub single_order_required: bool,
    pub single_venue_required: bool,
    pub single_account_required: bool,
    pub manual_online_gate_required: bool,
    pub manual_online_gate_present: bool,
    pub pre_submit_risk_gate_required: bool,
    pub owner_approval_required: bool,
    pub signing_material_gate_required: bool,
    pub request_digest_required: bool,
    pub release_provenance_required: bool,
    pub retry_attempted: bool,
    pub replace_attempted: bool,
    pub amend_attempted: bool,
    pub flatten_attempted: bool,
    pub bulk_submit_attempted: bool,
    pub automatic_remediation_allowed: bool,
    pub dashboard_order_controls_enabled: bool,
    pub raw_secret_persisted: bool,
    pub raw_signed_payload_persisted: bool,
    pub raw_exchange_response_persisted: bool,
}

impl GuardedSubmitCandidateEvidence {
    fn from_request(
        request: &GuardedSubmitCandidateRequest,
        risk: &PreSubmitRiskGateEvidence,
        approval: &OwnerApprovalEvidence,
        signing: &SigningMaterialGateEvidence,
        evaluated_at_unix_ns: u64,
    ) -> Self {
        Self {
            schema_version: V20_GUARDED_SUBMIT_CANDIDATE_SCHEMA_VERSION.to_string(),
            contract_id: V20_ORDER_LIFECYCLE_CONTRACT_ID.to_string(),
            candidate_id: request.candidate_id.clone(),
            attempt_id: request.attempt_id.clone(),
            lifecycle_id: request.lifecycle_id.clone(),
            mode: request.mode,
            state: GuardedSubmitCandidateState::Blocked,
            code: GuardedSubmitCandidateCode::CandidateIdMissing,
            reason: String::new(),
            evaluated_at_unix_ns,
            request_digest: None,
            risk_gate_id: risk.gate_id.clone(),
            approval_id: approval.approval_id.clone(),
            signing_gate_id: signing.gate_id.clone(),
            release_tag: risk.release_tag.clone(),
            release_commit: risk.release_commit.clone(),
            release_gate: risk.release_gate.clone(),
            owner_approval_state_before_attempt: approval.state,
            owner_approval_state_after_attempt: approval.state,
            approval_consumed_at_unix_ns: approval.consumed_at_unix_ns,
            owner_approval_consumed: approval.consumed,
            previous_attempt_count: request.prior_attempt_digests.len(),
            submit_attempt_evidence_ready: false,
            preview_evidence_ready: false,
            dry_run_evidence_ready: false,
            production_submit_attempted: false,
            adapter_submit_handoff_allowed: false,
            readback_required: false,
            audit_artifact_required: true,
            single_attempt_required: true,
            single_order_required: true,
            single_venue_required: true,
            single_account_required: true,
            manual_online_gate_required: true,
            manual_online_gate_present: request.manual_online_gate,
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

    fn finish(
        mut self,
        state: GuardedSubmitCandidateState,
        code: GuardedSubmitCandidateCode,
        reason: impl Into<String>,
    ) -> Self {
        self.state = state;
        self.code = code;
        self.reason = reason.into();

        match state {
            GuardedSubmitCandidateState::Blocked => {}
            GuardedSubmitCandidateState::Preview => {
                self.submit_attempt_evidence_ready = true;
                self.preview_evidence_ready = true;
            }
            GuardedSubmitCandidateState::DryRun => {
                self.submit_attempt_evidence_ready = true;
                self.dry_run_evidence_ready = true;
            }
            GuardedSubmitCandidateState::Submitted => {
                self.submit_attempt_evidence_ready = true;
                self.production_submit_attempted = true;
                self.adapter_submit_handoff_allowed = true;
                self.readback_required = true;
            }
        }

        self
    }

    fn with_request_digest(mut self, request_digest: &str) -> Self {
        self.request_digest = Some(request_digest.to_string());
        self
    }

    fn with_consumed_approval(mut self, consumed_approval: &OwnerApprovalEvidence) -> Self {
        self.owner_approval_state_after_attempt = consumed_approval.state;
        self.approval_consumed_at_unix_ns = consumed_approval.consumed_at_unix_ns;
        self.owner_approval_consumed = consumed_approval.consumed;
        self
    }
}

/// Evaluates one guarded single-shot production submit candidate.
#[must_use]
pub fn evaluate_guarded_single_shot_submit_candidate(
    request: &GuardedSubmitCandidateRequest,
    risk: &PreSubmitRiskGateEvidence,
    approval: &OwnerApprovalEvidence,
    signing: &SigningMaterialGateEvidence,
    builder: &SubmitRequestBuilderEvidence,
    evaluated_at_unix_ns: u64,
) -> GuardedSubmitCandidateEvidence {
    let evidence = GuardedSubmitCandidateEvidence::from_request(
        request,
        risk,
        approval,
        signing,
        evaluated_at_unix_ns,
    );

    if is_blank(&request.candidate_id) {
        return evidence.finish(
            GuardedSubmitCandidateState::Blocked,
            GuardedSubmitCandidateCode::CandidateIdMissing,
            "candidate_id is required",
        );
    }
    if is_blank(&request.attempt_id) {
        return evidence.finish(
            GuardedSubmitCandidateState::Blocked,
            GuardedSubmitCandidateCode::AttemptIdMissing,
            "attempt_id is required",
        );
    }
    if request.lifecycle_id != risk.lifecycle_id
        || request.lifecycle_id != approval.lifecycle_id
        || request.lifecycle_id != signing.lifecycle_id
        || request.lifecycle_id != builder.lifecycle_id
    {
        return evidence.finish(
            GuardedSubmitCandidateState::Blocked,
            GuardedSubmitCandidateCode::LifecycleMismatch,
            "candidate lifecycle does not match prerequisite evidence",
        );
    }
    if risk.decision != PreSubmitRiskDecisionKind::Allow
        || !risk.production_order_submission_allowed
        || !risk.submit_builder_entry_allowed
    {
        return evidence.finish(
            GuardedSubmitCandidateState::Blocked,
            GuardedSubmitCandidateCode::MissingRiskAllow,
            "risk allow evidence is required",
        );
    }
    if approval.state != OwnerApprovalState::Approved || !approval.submit_consumption_allowed {
        return evidence.finish(
            GuardedSubmitCandidateState::Blocked,
            GuardedSubmitCandidateCode::MissingOwnerApproval,
            "active owner approval evidence is required",
        );
    }
    if signing.decision != SigningMaterialDecision::Ready
        || !signing.submit_builder_credential_ready
    {
        return evidence.finish(
            GuardedSubmitCandidateState::Blocked,
            GuardedSubmitCandidateCode::MissingSigningReadiness,
            "signing material readiness evidence is required",
        );
    }
    if builder.decision != SubmitRequestBuildDecision::Built
        || !builder.submit_request_built
        || builder.production_order_submitted
        || builder.network_attempted
    {
        return evidence.finish(
            GuardedSubmitCandidateState::Blocked,
            GuardedSubmitCandidateCode::MissingRequestBuild,
            "built redacted submit request evidence is required",
        );
    }
    if builder.risk_gate_id != risk.gate_id
        || builder.approval_id != approval.approval_id
        || builder.signing_gate_id != signing.gate_id
    {
        return evidence.finish(
            GuardedSubmitCandidateState::Blocked,
            GuardedSubmitCandidateCode::EvidenceMismatch,
            "builder evidence does not reference the supplied prerequisite evidence",
        );
    }
    if !risk.release_provenance_valid
        || risk.release_tag.as_deref().is_none_or(is_blank)
        || risk.release_commit.as_deref().is_none_or(is_blank)
        || risk.release_gate.as_deref().is_none_or(is_blank)
    {
        return evidence.finish(
            GuardedSubmitCandidateState::Blocked,
            GuardedSubmitCandidateCode::MissingReleaseProvenance,
            "strict release provenance evidence is required",
        );
    }

    let Some(actual_request_digest) = builder.request_digest.as_deref() else {
        return evidence.finish(
            GuardedSubmitCandidateState::Blocked,
            GuardedSubmitCandidateCode::RequestDigestMissing,
            "built request digest is required",
        );
    };
    let Some(expected_request_digest) = request.expected_request_digest.as_deref() else {
        return evidence.finish(
            GuardedSubmitCandidateState::Blocked,
            GuardedSubmitCandidateCode::RequestDigestMissing,
            "expected request digest is required",
        );
    };
    if expected_request_digest != actual_request_digest {
        return evidence.with_request_digest(actual_request_digest).finish(
            GuardedSubmitCandidateState::Blocked,
            GuardedSubmitCandidateCode::RequestDigestMismatch,
            "expected request digest does not match built request evidence",
        );
    }
    if request.mode == GuardedSubmitMode::Submit
        && request
            .prior_attempt_digests
            .contains(actual_request_digest)
    {
        return evidence.with_request_digest(actual_request_digest).finish(
            GuardedSubmitCandidateState::Blocked,
            GuardedSubmitCandidateCode::DuplicateSubmitRejected,
            "single-shot request digest was already submitted",
        );
    }

    match request.mode {
        GuardedSubmitMode::Preview => evidence.with_request_digest(actual_request_digest).finish(
            GuardedSubmitCandidateState::Preview,
            GuardedSubmitCandidateCode::PreviewReady,
            "guarded submit candidate preview evidence is ready",
        ),
        GuardedSubmitMode::DryRun => evidence.with_request_digest(actual_request_digest).finish(
            GuardedSubmitCandidateState::DryRun,
            GuardedSubmitCandidateCode::DryRunReady,
            "guarded submit candidate dry-run evidence is ready",
        ),
        GuardedSubmitMode::Submit => {
            if !request.manual_online_gate {
                return evidence.with_request_digest(actual_request_digest).finish(
                    GuardedSubmitCandidateState::Blocked,
                    GuardedSubmitCandidateCode::ManualGateMissing,
                    "manual online gate is required before submit",
                );
            }

            let consumed_approval = consume_owner_approval(approval, evaluated_at_unix_ns);
            evidence
                .with_request_digest(actual_request_digest)
                .with_consumed_approval(&consumed_approval)
                .finish(
                    GuardedSubmitCandidateState::Submitted,
                    GuardedSubmitCandidateCode::Submitted,
                    "guarded single-shot submit attempt evidence recorded",
                )
        }
    }
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}
