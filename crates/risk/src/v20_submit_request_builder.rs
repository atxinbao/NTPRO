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

//! V200 deterministic single-shot production submit request builder.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    v20_owner_approval::{OwnerApprovalEvidence, OwnerApprovalState},
    v20_pre_submit_gate::{
        PreSubmitRiskDecisionKind, PreSubmitRiskGateEvidence, V20_ORDER_LIFECYCLE_CONTRACT_ID,
    },
    v20_signing_material_gate::{SigningMaterialDecision, SigningMaterialGateEvidence},
};

/// Stable schema for V200 submit request builder evidence.
pub const V20_SUBMIT_REQUEST_BUILDER_SCHEMA_VERSION: &str =
    "ntpro.v200_single_shot_submit_request_builder.v1";

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001b3;

/// Request builder outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmitRequestBuildDecision {
    Built,
    Rejected,
}

/// Stable request builder code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubmitRequestBuildCode {
    #[serde(rename = "v200_submit_request_built")]
    Built,
    #[serde(rename = "v200_submit_request_missing_risk_allow")]
    MissingRiskAllow,
    #[serde(rename = "v200_submit_request_missing_owner_approval")]
    MissingOwnerApproval,
    #[serde(rename = "v200_submit_request_missing_signing_readiness")]
    MissingSigningReadiness,
    #[serde(rename = "v200_submit_request_candidate_mismatch")]
    CandidateMismatch,
    #[serde(rename = "v200_submit_request_unsupported_order_shape")]
    UnsupportedOrderShape,
}

impl SubmitRequestBuildCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Built => "v200_submit_request_built",
            Self::MissingRiskAllow => "v200_submit_request_missing_risk_allow",
            Self::MissingOwnerApproval => "v200_submit_request_missing_owner_approval",
            Self::MissingSigningReadiness => "v200_submit_request_missing_signing_readiness",
            Self::CandidateMismatch => "v200_submit_request_candidate_mismatch",
            Self::UnsupportedOrderShape => "v200_submit_request_unsupported_order_shape",
        }
    }
}

/// Approved production order candidate for request building.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SingleShotSubmitCandidate {
    pub lifecycle_id: String,
    pub client_order_id: String,
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

/// Redacted deterministic submit request preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedactedSubmitRequestPreview {
    pub venue: String,
    pub instrument_id: String,
    pub side: String,
    pub quantity: Decimal,
    pub price: Decimal,
    pub notional: Decimal,
    pub order_type: String,
    pub time_in_force: String,
    pub client_order_id: String,
    pub credential_material: String,
    pub raw_payload_recorded: bool,
    pub signed_payload_recorded: bool,
}

/// Auditable builder evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitRequestBuilderEvidence {
    pub schema_version: String,
    pub contract_id: String,
    pub lifecycle_id: String,
    pub decision: SubmitRequestBuildDecision,
    pub code: SubmitRequestBuildCode,
    pub reason: String,
    pub request_digest: Option<String>,
    pub redacted_preview: Option<RedactedSubmitRequestPreview>,
    pub risk_gate_id: String,
    pub approval_id: String,
    pub signing_gate_id: String,
    pub submit_request_built: bool,
    pub network_attempted: bool,
    pub production_order_submitted: bool,
    pub retry_attempted: bool,
    pub automatic_remediation_allowed: bool,
    pub raw_secret_persisted: bool,
    pub raw_signed_payload_persisted: bool,
    pub dashboard_order_controls_enabled: bool,
}

impl SubmitRequestBuilderEvidence {
    fn rejected(
        candidate: &SingleShotSubmitCandidate,
        risk: &PreSubmitRiskGateEvidence,
        approval: &OwnerApprovalEvidence,
        signing: &SigningMaterialGateEvidence,
        code: SubmitRequestBuildCode,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: V20_SUBMIT_REQUEST_BUILDER_SCHEMA_VERSION.to_string(),
            contract_id: V20_ORDER_LIFECYCLE_CONTRACT_ID.to_string(),
            lifecycle_id: candidate.lifecycle_id.clone(),
            decision: SubmitRequestBuildDecision::Rejected,
            code,
            reason: reason.into(),
            request_digest: None,
            redacted_preview: None,
            risk_gate_id: risk.gate_id.clone(),
            approval_id: approval.approval_id.clone(),
            signing_gate_id: signing.gate_id.clone(),
            submit_request_built: false,
            network_attempted: false,
            production_order_submitted: false,
            retry_attempted: false,
            automatic_remediation_allowed: false,
            raw_secret_persisted: false,
            raw_signed_payload_persisted: false,
            dashboard_order_controls_enabled: false,
        }
    }
}

/// Builds a deterministic redacted single-shot production submit request.
#[must_use]
pub fn build_single_shot_submit_request(
    candidate: &SingleShotSubmitCandidate,
    risk: &PreSubmitRiskGateEvidence,
    approval: &OwnerApprovalEvidence,
    signing: &SigningMaterialGateEvidence,
) -> SubmitRequestBuilderEvidence {
    if risk.decision != PreSubmitRiskDecisionKind::Allow || !risk.submit_builder_entry_allowed {
        return SubmitRequestBuilderEvidence::rejected(
            candidate,
            risk,
            approval,
            signing,
            SubmitRequestBuildCode::MissingRiskAllow,
            "risk allow evidence is required",
        );
    }
    if approval.state != OwnerApprovalState::Approved || !approval.submit_consumption_allowed {
        return SubmitRequestBuilderEvidence::rejected(
            candidate,
            risk,
            approval,
            signing,
            SubmitRequestBuildCode::MissingOwnerApproval,
            "active owner approval evidence is required",
        );
    }
    if signing.decision != SigningMaterialDecision::Ready
        || !signing.submit_builder_credential_ready
    {
        return SubmitRequestBuilderEvidence::rejected(
            candidate,
            risk,
            approval,
            signing,
            SubmitRequestBuildCode::MissingSigningReadiness,
            "signing material readiness evidence is required",
        );
    }
    if !candidate_matches_risk(candidate, risk)
        || candidate.order_intent_hash != approval.order_intent_hash
    {
        return SubmitRequestBuilderEvidence::rejected(
            candidate,
            risk,
            approval,
            signing,
            SubmitRequestBuildCode::CandidateMismatch,
            "candidate does not match prerequisite evidence",
        );
    }
    if candidate.order_type != "limit"
        || candidate.time_in_force != "gtc"
        || !matches!(candidate.side.as_str(), "buy" | "sell")
        || candidate.quantity <= Decimal::ZERO
        || candidate.price <= Decimal::ZERO
        || candidate.notional <= Decimal::ZERO
    {
        return SubmitRequestBuilderEvidence::rejected(
            candidate,
            risk,
            approval,
            signing,
            SubmitRequestBuildCode::UnsupportedOrderShape,
            "candidate shape is outside V200-005 scope",
        );
    }

    let request_digest = submit_request_digest(candidate, risk, approval, signing);
    let redacted_preview = RedactedSubmitRequestPreview {
        venue: candidate.venue.clone(),
        instrument_id: candidate.instrument_id.clone(),
        side: candidate.side.clone(),
        quantity: candidate.quantity,
        price: candidate.price,
        notional: candidate.notional,
        order_type: candidate.order_type.clone(),
        time_in_force: candidate.time_in_force.clone(),
        client_order_id: candidate.client_order_id.clone(),
        credential_material: "redacted_env_fingerprint_only".to_string(),
        raw_payload_recorded: false,
        signed_payload_recorded: false,
    };

    SubmitRequestBuilderEvidence {
        schema_version: V20_SUBMIT_REQUEST_BUILDER_SCHEMA_VERSION.to_string(),
        contract_id: V20_ORDER_LIFECYCLE_CONTRACT_ID.to_string(),
        lifecycle_id: candidate.lifecycle_id.clone(),
        decision: SubmitRequestBuildDecision::Built,
        code: SubmitRequestBuildCode::Built,
        reason: "single-shot redacted submit request built".to_string(),
        request_digest: Some(request_digest),
        redacted_preview: Some(redacted_preview),
        risk_gate_id: risk.gate_id.clone(),
        approval_id: approval.approval_id.clone(),
        signing_gate_id: signing.gate_id.clone(),
        submit_request_built: true,
        network_attempted: false,
        production_order_submitted: false,
        retry_attempted: false,
        automatic_remediation_allowed: false,
        raw_secret_persisted: false,
        raw_signed_payload_persisted: false,
        dashboard_order_controls_enabled: false,
    }
}

/// Computes the deterministic submit request digest.
#[must_use]
pub fn submit_request_digest(
    candidate: &SingleShotSubmitCandidate,
    risk: &PreSubmitRiskGateEvidence,
    approval: &OwnerApprovalEvidence,
    signing: &SigningMaterialGateEvidence,
) -> String {
    let fields = vec![
        candidate.lifecycle_id.clone(),
        candidate.client_order_id.clone(),
        candidate.account_label.clone(),
        candidate.instrument_id.clone(),
        candidate.venue.clone(),
        candidate.side.clone(),
        candidate.quantity.to_string(),
        candidate.price.to_string(),
        candidate.notional.to_string(),
        candidate.order_type.clone(),
        candidate.time_in_force.clone(),
        candidate.order_intent_hash.clone(),
        risk.gate_id.clone(),
        approval.approval_id.clone(),
        signing.gate_id.clone(),
    ];
    checksum_fields(&fields)
}

fn candidate_matches_risk(
    candidate: &SingleShotSubmitCandidate,
    risk: &PreSubmitRiskGateEvidence,
) -> bool {
    risk.lifecycle_id == candidate.lifecycle_id
        && risk.account_label.as_deref() == Some(candidate.account_label.as_str())
        && risk.instrument_id.as_deref() == Some(candidate.instrument_id.as_str())
        && risk.venue.as_deref() == Some(candidate.venue.as_str())
        && risk.side.as_deref() == Some(candidate.side.as_str())
        && risk.quantity == Some(candidate.quantity)
        && risk.price == Some(candidate.price)
        && risk.notional == Some(candidate.notional)
        && risk.order_type.as_deref() == Some(candidate.order_type.as_str())
        && risk.time_in_force.as_deref() == Some(candidate.time_in_force.as_str())
        && risk.order_intent_hash.as_deref() == Some(candidate.order_intent_hash.as_str())
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
