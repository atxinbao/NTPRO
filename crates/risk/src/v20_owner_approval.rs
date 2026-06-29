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

//! V200 owner approval lifecycle evidence for production submit.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::v20_pre_submit_gate::{
    PreSubmitApproval, PreSubmitReleaseProvenance, V20_ORDER_LIFECYCLE_CONTRACT_ID,
};

/// Stable schema for V200 owner approval lifecycle evidence.
pub const V20_OWNER_APPROVAL_SCHEMA_VERSION: &str = "ntpro.v200_owner_approval_lifecycle_event.v1";

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001b3;

/// Lifecycle state for one owner approval artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerApprovalState {
    Requested,
    Approved,
    Rejected,
    Expired,
    Revoked,
    Consumed,
}

/// Owner decision recorded against an approval request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerApprovalDecision {
    Approved,
    Rejected,
}

/// Stable owner approval evidence codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerApprovalCode {
    #[serde(rename = "v200_owner_approval_allowed")]
    Allowed,
    #[serde(rename = "v200_owner_approval_request_digest_mismatch")]
    RequestDigestMismatch,
    #[serde(rename = "v200_owner_approval_candidate_digest_mismatch")]
    CandidateDigestMismatch,
    #[serde(rename = "v200_owner_approval_scope_mismatch")]
    ScopeMismatch,
    #[serde(rename = "v200_owner_approval_environment_mismatch")]
    EnvironmentMismatch,
    #[serde(rename = "v200_owner_approval_release_provenance_mismatch")]
    ReleaseProvenanceMismatch,
    #[serde(rename = "v200_owner_approval_owner_missing")]
    OwnerMissing,
    #[serde(rename = "v200_owner_approval_nonce_missing")]
    NonceMissing,
    #[serde(rename = "v200_owner_approval_rejected")]
    Rejected,
    #[serde(rename = "v200_owner_approval_expired")]
    Expired,
    #[serde(rename = "v200_owner_approval_revoked")]
    Revoked,
    #[serde(rename = "v200_owner_approval_already_consumed")]
    AlreadyConsumed,
    #[serde(rename = "v200_owner_approval_consumed")]
    Consumed,
    #[serde(rename = "v200_owner_approval_consumption_requires_approved_evidence")]
    ConsumptionRequiresApprovedEvidence,
}

impl OwnerApprovalCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "v200_owner_approval_allowed",
            Self::RequestDigestMismatch => "v200_owner_approval_request_digest_mismatch",
            Self::CandidateDigestMismatch => "v200_owner_approval_candidate_digest_mismatch",
            Self::ScopeMismatch => "v200_owner_approval_scope_mismatch",
            Self::EnvironmentMismatch => "v200_owner_approval_environment_mismatch",
            Self::ReleaseProvenanceMismatch => "v200_owner_approval_release_provenance_mismatch",
            Self::OwnerMissing => "v200_owner_approval_owner_missing",
            Self::NonceMissing => "v200_owner_approval_nonce_missing",
            Self::Rejected => "v200_owner_approval_rejected",
            Self::Expired => "v200_owner_approval_expired",
            Self::Revoked => "v200_owner_approval_revoked",
            Self::AlreadyConsumed => "v200_owner_approval_already_consumed",
            Self::Consumed => "v200_owner_approval_consumed",
            Self::ConsumptionRequiresApprovedEvidence => {
                "v200_owner_approval_consumption_requires_approved_evidence"
            }
        }
    }
}

/// Immutable production order scope bound to one owner approval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerApprovalScope {
    pub account_label: String,
    pub instrument_id: String,
    pub venue: String,
    pub side: String,
    pub quantity: Decimal,
    pub price: Decimal,
    pub notional: Decimal,
    pub order_type: String,
    pub time_in_force: String,
    pub order_intent_hash: String,
}

/// Owner approval request before the owner decision is recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerApprovalRequest {
    pub request_id: String,
    pub lifecycle_id: String,
    pub owner_label: String,
    pub scope: OwnerApprovalScope,
    pub nonce: String,
    pub environment: String,
    pub release_provenance: PreSubmitReleaseProvenance,
    pub approval_digest: String,
    pub expires_at_unix_ns: u64,
}

/// Candidate submit scope attempting to use owner approval evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerApprovalCandidate {
    pub lifecycle_id: String,
    pub scope: OwnerApprovalScope,
    pub environment: String,
    pub release_provenance: PreSubmitReleaseProvenance,
    pub approval_digest: String,
}

/// Recorded owner decision and mutable lifecycle markers for one approval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerApprovalRecord {
    pub approval_id: String,
    pub request: OwnerApprovalRequest,
    pub decision: OwnerApprovalDecision,
    pub decided_at_unix_ns: u64,
    pub revoked_at_unix_ns: Option<u64>,
    pub consumed_at_unix_ns: Option<u64>,
}

/// Auditable owner approval lifecycle evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerApprovalEvidence {
    pub schema_version: String,
    pub contract_id: String,
    pub approval_id: String,
    pub request_id: String,
    pub lifecycle_id: String,
    pub state: OwnerApprovalState,
    pub code: OwnerApprovalCode,
    pub reason: String,
    pub owner_label: String,
    pub approval_digest: String,
    pub expected_approval_digest: String,
    pub candidate_approval_digest: String,
    pub account_label: String,
    pub instrument_id: String,
    pub venue: String,
    pub side: String,
    pub quantity: Decimal,
    pub price: Decimal,
    pub notional: Decimal,
    pub order_type: String,
    pub time_in_force: String,
    pub order_intent_hash: String,
    pub nonce: String,
    pub environment: String,
    pub release_tag: String,
    pub release_commit: String,
    pub release_gate: String,
    pub expires_at_unix_ns: u64,
    pub decided_at_unix_ns: u64,
    pub revoked_at_unix_ns: Option<u64>,
    pub consumed_at_unix_ns: Option<u64>,
    pub owner_approval_required: bool,
    pub owner_decision_recorded: bool,
    pub candidate_scope_match: bool,
    pub digest_match: bool,
    pub release_provenance_match: bool,
    pub single_use: bool,
    pub consumed: bool,
    pub submit_consumption_allowed: bool,
    pub approval_reusable: bool,
    pub approval_execution_authorized_after_attempt: bool,
    pub dashboard_approval_controls_enabled: bool,
    pub dashboard_order_controls_enabled: bool,
    pub retry_attempted: bool,
    pub automatic_remediation_allowed: bool,
}

impl OwnerApprovalEvidence {
    fn from_record(
        record: &OwnerApprovalRecord,
        candidate: &OwnerApprovalCandidate,
        expected_approval_digest: String,
    ) -> Self {
        Self {
            schema_version: V20_OWNER_APPROVAL_SCHEMA_VERSION.to_string(),
            contract_id: V20_ORDER_LIFECYCLE_CONTRACT_ID.to_string(),
            approval_id: record.approval_id.clone(),
            request_id: record.request.request_id.clone(),
            lifecycle_id: record.request.lifecycle_id.clone(),
            state: OwnerApprovalState::Requested,
            code: OwnerApprovalCode::Allowed,
            reason: String::new(),
            owner_label: record.request.owner_label.clone(),
            approval_digest: record.request.approval_digest.clone(),
            expected_approval_digest,
            candidate_approval_digest: candidate.approval_digest.clone(),
            account_label: record.request.scope.account_label.clone(),
            instrument_id: record.request.scope.instrument_id.clone(),
            venue: record.request.scope.venue.clone(),
            side: record.request.scope.side.clone(),
            quantity: record.request.scope.quantity,
            price: record.request.scope.price,
            notional: record.request.scope.notional,
            order_type: record.request.scope.order_type.clone(),
            time_in_force: record.request.scope.time_in_force.clone(),
            order_intent_hash: record.request.scope.order_intent_hash.clone(),
            nonce: record.request.nonce.clone(),
            environment: record.request.environment.clone(),
            release_tag: record.request.release_provenance.release_tag.clone(),
            release_commit: record.request.release_provenance.release_commit.clone(),
            release_gate: record.request.release_provenance.release_gate.clone(),
            expires_at_unix_ns: record.request.expires_at_unix_ns,
            decided_at_unix_ns: record.decided_at_unix_ns,
            revoked_at_unix_ns: record.revoked_at_unix_ns,
            consumed_at_unix_ns: record.consumed_at_unix_ns,
            owner_approval_required: true,
            owner_decision_recorded: true,
            candidate_scope_match: false,
            digest_match: false,
            release_provenance_match: false,
            single_use: true,
            consumed: record.consumed_at_unix_ns.is_some(),
            submit_consumption_allowed: false,
            approval_reusable: false,
            approval_execution_authorized_after_attempt: false,
            dashboard_approval_controls_enabled: false,
            dashboard_order_controls_enabled: false,
            retry_attempted: false,
            automatic_remediation_allowed: false,
        }
    }

    fn finish(
        mut self,
        state: OwnerApprovalState,
        code: OwnerApprovalCode,
        reason: impl Into<String>,
    ) -> Self {
        self.state = state;
        self.code = code;
        self.reason = reason.into();
        if state == OwnerApprovalState::Approved {
            self.candidate_scope_match = true;
            self.digest_match = true;
            self.release_provenance_match = true;
            self.submit_consumption_allowed = true;
        }
        self
    }

    /// Converts active owner approval evidence into the V200 pre-submit gate
    /// approval shape.
    #[must_use]
    pub fn as_pre_submit_approval(&self) -> Option<PreSubmitApproval> {
        if self.state != OwnerApprovalState::Approved || !self.submit_consumption_allowed {
            return None;
        }

        Some(PreSubmitApproval {
            approval_id: self.approval_id.clone(),
            owner_label: self.owner_label.clone(),
            order_intent_hash: self.order_intent_hash.clone(),
            expires_at_unix_ns: self.expires_at_unix_ns,
            single_use: true,
            consumed: false,
        })
    }
}

/// Computes the deterministic approval digest for an approval request.
#[must_use]
pub fn owner_approval_digest(
    scope: &OwnerApprovalScope,
    nonce: &str,
    environment: &str,
    release_provenance: &PreSubmitReleaseProvenance,
) -> String {
    let fields = vec![
        scope.account_label.clone(),
        scope.instrument_id.clone(),
        scope.venue.clone(),
        scope.side.clone(),
        scope.quantity.to_string(),
        scope.price.to_string(),
        scope.notional.to_string(),
        scope.order_type.clone(),
        scope.time_in_force.clone(),
        scope.order_intent_hash.clone(),
        nonce.to_string(),
        environment.to_string(),
        release_provenance.release_tag.clone(),
        release_provenance.release_commit.clone(),
        release_provenance.release_gate.clone(),
        release_provenance.strict_provenance.to_string(),
    ];
    checksum_fields(&fields)
}

/// Evaluates whether an owner approval record can be consumed by one candidate.
#[must_use]
pub fn evaluate_owner_approval(
    record: &OwnerApprovalRecord,
    candidate: &OwnerApprovalCandidate,
    evaluated_at_unix_ns: u64,
) -> OwnerApprovalEvidence {
    let expected_digest = owner_approval_digest(
        &record.request.scope,
        &record.request.nonce,
        &record.request.environment,
        &record.request.release_provenance,
    );
    let evidence = OwnerApprovalEvidence::from_record(record, candidate, expected_digest.clone());

    if missing(&record.request.owner_label) {
        return evidence.finish(
            OwnerApprovalState::Rejected,
            OwnerApprovalCode::OwnerMissing,
            "owner_label is required",
        );
    }
    if missing(&record.request.nonce) {
        return evidence.finish(
            OwnerApprovalState::Rejected,
            OwnerApprovalCode::NonceMissing,
            "nonce is required",
        );
    }
    if record.request.approval_digest != expected_digest {
        return evidence.finish(
            OwnerApprovalState::Rejected,
            OwnerApprovalCode::RequestDigestMismatch,
            "approval request digest does not match request scope",
        );
    }
    if candidate.approval_digest != record.request.approval_digest {
        return evidence.finish(
            OwnerApprovalState::Rejected,
            OwnerApprovalCode::CandidateDigestMismatch,
            "candidate digest does not match approval digest",
        );
    }
    if candidate.lifecycle_id != record.request.lifecycle_id
        || candidate.scope != record.request.scope
    {
        return evidence.finish(
            OwnerApprovalState::Rejected,
            OwnerApprovalCode::ScopeMismatch,
            "candidate scope does not match approval request scope",
        );
    }
    if candidate.environment != record.request.environment {
        return evidence.finish(
            OwnerApprovalState::Rejected,
            OwnerApprovalCode::EnvironmentMismatch,
            "candidate environment does not match approval request environment",
        );
    }
    if candidate.release_provenance != record.request.release_provenance {
        return evidence.finish(
            OwnerApprovalState::Rejected,
            OwnerApprovalCode::ReleaseProvenanceMismatch,
            "candidate release provenance does not match approval request provenance",
        );
    }
    if record.decision == OwnerApprovalDecision::Rejected {
        return evidence.finish(
            OwnerApprovalState::Rejected,
            OwnerApprovalCode::Rejected,
            "owner rejected the approval request",
        );
    }
    if record
        .revoked_at_unix_ns
        .is_some_and(|revoked_at| revoked_at <= evaluated_at_unix_ns)
    {
        return evidence.finish(
            OwnerApprovalState::Revoked,
            OwnerApprovalCode::Revoked,
            "owner approval was revoked before consumption",
        );
    }
    if evaluated_at_unix_ns > record.request.expires_at_unix_ns {
        return evidence.finish(
            OwnerApprovalState::Expired,
            OwnerApprovalCode::Expired,
            "owner approval expired before consumption",
        );
    }
    if record.consumed_at_unix_ns.is_some() {
        return evidence.finish(
            OwnerApprovalState::Consumed,
            OwnerApprovalCode::AlreadyConsumed,
            "owner approval was already consumed",
        );
    }

    evidence.finish(
        OwnerApprovalState::Approved,
        OwnerApprovalCode::Allowed,
        "owner approval is active and may be consumed once",
    )
}

/// Marks active approval evidence consumed before a submit attempt.
#[must_use]
pub fn consume_owner_approval(
    evidence: &OwnerApprovalEvidence,
    consumed_at_unix_ns: u64,
) -> OwnerApprovalEvidence {
    let mut consumed = evidence.clone();
    consumed.consumed_at_unix_ns = Some(consumed_at_unix_ns);
    consumed.consumed = true;
    consumed.submit_consumption_allowed = false;
    consumed.approval_execution_authorized_after_attempt = false;

    if evidence.state == OwnerApprovalState::Approved && evidence.submit_consumption_allowed {
        consumed.state = OwnerApprovalState::Consumed;
        consumed.code = OwnerApprovalCode::Consumed;
        consumed.reason = "owner approval consumed before submit attempt".to_string();
    } else {
        consumed.state = OwnerApprovalState::Rejected;
        consumed.code = OwnerApprovalCode::ConsumptionRequiresApprovedEvidence;
        consumed.reason = "only active approved evidence may be consumed".to_string();
    }

    consumed
}

fn missing(value: &str) -> bool {
    value.trim().is_empty()
}

fn checksum_fields(fields: &[String]) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for field in fields {
        for byte in field.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash ^= u64::from(b'\n');
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{hash:016x}")
}
