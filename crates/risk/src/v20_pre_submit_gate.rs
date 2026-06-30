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

//! V200 pre-submit production risk gate evidence model.

use std::collections::BTreeSet;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Stable schema for V200 pre-submit risk gate decisions.
pub const V20_PRE_SUBMIT_RISK_GATE_SCHEMA_VERSION: &str =
    "ntpro.v200_pre_submit_risk_gate_decision.v1";
/// Contract inherited from V200-001 production order lifecycle safety work.
pub const V20_ORDER_LIFECYCLE_CONTRACT_ID: &str = "ntpro.v200_order_lifecycle_safety_contract.v1";
/// Runtime release tag required for V20 production submit evidence.
pub const V20_REQUIRED_RELEASE_TAG: &str = "ntpro-rust-only-v0.20.0";
/// Runtime release gate required for V20 production submit evidence.
pub const V20_REQUIRED_RELEASE_GATE: &str = "v20-release-gates";

/// Final pre-submit gate decision class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreSubmitRiskDecisionKind {
    Allow,
    Deny,
    Blocked,
}

/// Stable code for every V200 pre-submit gate outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreSubmitRiskCode {
    #[serde(rename = "v200_pre_submit_allowed")]
    Allowed,
    #[serde(rename = "v200_pre_submit_unknown_field")]
    UnknownField,
    #[serde(rename = "v200_pre_submit_gate_id_missing")]
    GateIdMissing,
    #[serde(rename = "v200_pre_submit_lifecycle_id_missing")]
    LifecycleIdMissing,
    #[serde(rename = "v200_pre_submit_account_missing")]
    AccountMissing,
    #[serde(rename = "v200_pre_submit_account_unknown")]
    AccountUnknown,
    #[serde(rename = "v200_pre_submit_instrument_missing")]
    InstrumentMissing,
    #[serde(rename = "v200_pre_submit_instrument_unknown")]
    InstrumentUnknown,
    #[serde(rename = "v200_pre_submit_venue_missing")]
    VenueMissing,
    #[serde(rename = "v200_pre_submit_venue_unknown")]
    VenueUnknown,
    #[serde(rename = "v200_pre_submit_side_missing")]
    SideMissing,
    #[serde(rename = "v200_pre_submit_side_unsupported")]
    SideUnsupported,
    #[serde(rename = "v200_pre_submit_quantity_missing")]
    QuantityMissing,
    #[serde(rename = "v200_pre_submit_quantity_not_positive")]
    QuantityNotPositive,
    #[serde(rename = "v200_pre_submit_quantity_limit_exceeded")]
    QuantityLimitExceeded,
    #[serde(rename = "v200_pre_submit_price_missing")]
    PriceMissing,
    #[serde(rename = "v200_pre_submit_price_not_positive")]
    PriceNotPositive,
    #[serde(rename = "v200_pre_submit_price_limit_exceeded")]
    PriceLimitExceeded,
    #[serde(rename = "v200_pre_submit_notional_missing")]
    NotionalMissing,
    #[serde(rename = "v200_pre_submit_notional_not_positive")]
    NotionalNotPositive,
    #[serde(rename = "v200_pre_submit_notional_mismatch")]
    NotionalMismatch,
    #[serde(rename = "v200_pre_submit_notional_limit_exceeded")]
    NotionalLimitExceeded,
    #[serde(rename = "v200_pre_submit_order_type_missing")]
    OrderTypeMissing,
    #[serde(rename = "v200_pre_submit_order_type_unsupported")]
    OrderTypeUnsupported,
    #[serde(rename = "v200_pre_submit_time_in_force_missing")]
    TimeInForceMissing,
    #[serde(rename = "v200_pre_submit_time_in_force_unsupported")]
    TimeInForceUnsupported,
    #[serde(rename = "v200_pre_submit_environment_missing")]
    EnvironmentMissing,
    #[serde(rename = "v200_pre_submit_environment_mismatch")]
    EnvironmentMismatch,
    #[serde(rename = "v200_pre_submit_intent_hash_missing")]
    IntentHashMissing,
    #[serde(rename = "v200_pre_submit_approval_missing")]
    ApprovalMissing,
    #[serde(rename = "v200_pre_submit_approval_id_missing")]
    ApprovalIdMissing,
    #[serde(rename = "v200_pre_submit_approval_expired")]
    ApprovalExpired,
    #[serde(rename = "v200_pre_submit_approval_intent_mismatch")]
    ApprovalIntentMismatch,
    #[serde(rename = "v200_pre_submit_approval_not_single_use")]
    ApprovalNotSingleUse,
    #[serde(rename = "v200_pre_submit_approval_already_consumed")]
    ApprovalAlreadyConsumed,
    #[serde(rename = "v200_pre_submit_provenance_missing")]
    ProvenanceMissing,
    #[serde(rename = "v200_pre_submit_provenance_tag_missing")]
    ProvenanceTagMissing,
    #[serde(rename = "v200_pre_submit_provenance_tag_mismatch")]
    ProvenanceTagMismatch,
    #[serde(rename = "v200_pre_submit_provenance_commit_missing")]
    ProvenanceCommitMissing,
    #[serde(rename = "v200_pre_submit_provenance_gate_mismatch")]
    ProvenanceGateMismatch,
    #[serde(rename = "v200_pre_submit_provenance_not_strict")]
    ProvenanceNotStrict,
}

impl PreSubmitRiskCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "v200_pre_submit_allowed",
            Self::UnknownField => "v200_pre_submit_unknown_field",
            Self::GateIdMissing => "v200_pre_submit_gate_id_missing",
            Self::LifecycleIdMissing => "v200_pre_submit_lifecycle_id_missing",
            Self::AccountMissing => "v200_pre_submit_account_missing",
            Self::AccountUnknown => "v200_pre_submit_account_unknown",
            Self::InstrumentMissing => "v200_pre_submit_instrument_missing",
            Self::InstrumentUnknown => "v200_pre_submit_instrument_unknown",
            Self::VenueMissing => "v200_pre_submit_venue_missing",
            Self::VenueUnknown => "v200_pre_submit_venue_unknown",
            Self::SideMissing => "v200_pre_submit_side_missing",
            Self::SideUnsupported => "v200_pre_submit_side_unsupported",
            Self::QuantityMissing => "v200_pre_submit_quantity_missing",
            Self::QuantityNotPositive => "v200_pre_submit_quantity_not_positive",
            Self::QuantityLimitExceeded => "v200_pre_submit_quantity_limit_exceeded",
            Self::PriceMissing => "v200_pre_submit_price_missing",
            Self::PriceNotPositive => "v200_pre_submit_price_not_positive",
            Self::PriceLimitExceeded => "v200_pre_submit_price_limit_exceeded",
            Self::NotionalMissing => "v200_pre_submit_notional_missing",
            Self::NotionalNotPositive => "v200_pre_submit_notional_not_positive",
            Self::NotionalMismatch => "v200_pre_submit_notional_mismatch",
            Self::NotionalLimitExceeded => "v200_pre_submit_notional_limit_exceeded",
            Self::OrderTypeMissing => "v200_pre_submit_order_type_missing",
            Self::OrderTypeUnsupported => "v200_pre_submit_order_type_unsupported",
            Self::TimeInForceMissing => "v200_pre_submit_time_in_force_missing",
            Self::TimeInForceUnsupported => "v200_pre_submit_time_in_force_unsupported",
            Self::EnvironmentMissing => "v200_pre_submit_environment_missing",
            Self::EnvironmentMismatch => "v200_pre_submit_environment_mismatch",
            Self::IntentHashMissing => "v200_pre_submit_intent_hash_missing",
            Self::ApprovalMissing => "v200_pre_submit_approval_missing",
            Self::ApprovalIdMissing => "v200_pre_submit_approval_id_missing",
            Self::ApprovalExpired => "v200_pre_submit_approval_expired",
            Self::ApprovalIntentMismatch => "v200_pre_submit_approval_intent_mismatch",
            Self::ApprovalNotSingleUse => "v200_pre_submit_approval_not_single_use",
            Self::ApprovalAlreadyConsumed => "v200_pre_submit_approval_already_consumed",
            Self::ProvenanceMissing => "v200_pre_submit_provenance_missing",
            Self::ProvenanceTagMissing => "v200_pre_submit_provenance_tag_missing",
            Self::ProvenanceTagMismatch => "v200_pre_submit_provenance_tag_mismatch",
            Self::ProvenanceCommitMissing => "v200_pre_submit_provenance_commit_missing",
            Self::ProvenanceGateMismatch => "v200_pre_submit_provenance_gate_mismatch",
            Self::ProvenanceNotStrict => "v200_pre_submit_provenance_not_strict",
        }
    }
}

/// Owner approval fields required before a production submit candidate can be built.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreSubmitApproval {
    pub approval_id: String,
    pub owner_label: String,
    pub order_intent_hash: String,
    pub expires_at_unix_ns: u64,
    pub single_use: bool,
    pub consumed: bool,
}

/// Release provenance fields required before production submit construction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreSubmitReleaseProvenance {
    pub release_tag: String,
    pub release_commit: String,
    pub release_gate: String,
    pub strict_provenance: bool,
}

/// Typed candidate evaluated by the V200 pre-submit risk gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreSubmitRiskRequest {
    pub gate_id: String,
    pub lifecycle_id: String,
    pub account_label: Option<String>,
    pub instrument_id: Option<String>,
    pub venue: Option<String>,
    pub side: Option<String>,
    pub quantity: Option<Decimal>,
    pub price: Option<Decimal>,
    pub notional: Option<Decimal>,
    pub order_type: Option<String>,
    pub time_in_force: Option<String>,
    pub environment: Option<String>,
    pub order_intent_hash: Option<String>,
    pub approval: Option<PreSubmitApproval>,
    pub release_provenance: Option<PreSubmitReleaseProvenance>,
    #[serde(default)]
    pub unrecognized_fields: Vec<String>,
}

/// Allowlist and limit configuration for one V200 production pre-submit gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreSubmitRiskPolicy {
    pub allowed_accounts: BTreeSet<String>,
    pub allowed_instruments: BTreeSet<String>,
    pub allowed_venues: BTreeSet<String>,
    pub allowed_sides: BTreeSet<String>,
    pub allowed_order_types: BTreeSet<String>,
    pub allowed_time_in_force: BTreeSet<String>,
    pub expected_environment: String,
    pub required_release_tag: String,
    pub required_release_gate: String,
    pub max_quantity: Decimal,
    pub max_price: Decimal,
    pub max_notional: Decimal,
}

/// Auditable result of one pre-submit gate evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreSubmitRiskGateEvidence {
    pub schema_version: String,
    pub contract_id: String,
    pub gate_id: String,
    pub lifecycle_id: String,
    pub decision: PreSubmitRiskDecisionKind,
    pub code: PreSubmitRiskCode,
    pub reason: String,
    pub account_label: Option<String>,
    pub instrument_id: Option<String>,
    pub venue: Option<String>,
    pub side: Option<String>,
    pub quantity: Option<Decimal>,
    pub price: Option<Decimal>,
    pub notional: Option<Decimal>,
    pub computed_notional: Option<Decimal>,
    pub order_type: Option<String>,
    pub time_in_force: Option<String>,
    pub environment: Option<String>,
    pub order_intent_hash: Option<String>,
    pub approval_id: Option<String>,
    pub release_tag: Option<String>,
    pub release_commit: Option<String>,
    pub release_gate: Option<String>,
    pub unrecognized_fields: Vec<String>,
    pub owner_approval_required: bool,
    pub owner_approval_valid: bool,
    pub release_provenance_required: bool,
    pub release_provenance_valid: bool,
    pub pre_submit_risk_gate_required: bool,
    pub notional_consistency_required: bool,
    pub notional_consistent: bool,
    pub single_order_required: bool,
    pub single_venue_required: bool,
    pub single_account_required: bool,
    pub production_order_submission_allowed: bool,
    pub submit_builder_entry_allowed: bool,
    pub retry_attempted: bool,
    pub replace_attempted: bool,
    pub amend_attempted: bool,
    pub flatten_attempted: bool,
    pub automatic_remediation_allowed: bool,
    pub dashboard_order_controls_enabled: bool,
}

impl PreSubmitRiskGateEvidence {
    fn from_request(request: &PreSubmitRiskRequest) -> Self {
        Self {
            schema_version: V20_PRE_SUBMIT_RISK_GATE_SCHEMA_VERSION.to_string(),
            contract_id: V20_ORDER_LIFECYCLE_CONTRACT_ID.to_string(),
            gate_id: request.gate_id.clone(),
            lifecycle_id: request.lifecycle_id.clone(),
            decision: PreSubmitRiskDecisionKind::Blocked,
            code: PreSubmitRiskCode::GateIdMissing,
            reason: String::new(),
            account_label: request.account_label.clone(),
            instrument_id: request.instrument_id.clone(),
            venue: request.venue.clone(),
            side: request.side.clone(),
            quantity: request.quantity,
            price: request.price,
            notional: request.notional,
            computed_notional: request
                .quantity
                .zip(request.price)
                .and_then(|(quantity, price)| compute_exact_notional(quantity, price)),
            order_type: request.order_type.clone(),
            time_in_force: request.time_in_force.clone(),
            environment: request.environment.clone(),
            order_intent_hash: request.order_intent_hash.clone(),
            approval_id: request
                .approval
                .as_ref()
                .map(|approval| approval.approval_id.clone()),
            release_tag: request
                .release_provenance
                .as_ref()
                .map(|provenance| provenance.release_tag.clone()),
            release_commit: request
                .release_provenance
                .as_ref()
                .map(|provenance| provenance.release_commit.clone()),
            release_gate: request
                .release_provenance
                .as_ref()
                .map(|provenance| provenance.release_gate.clone()),
            unrecognized_fields: request.unrecognized_fields.clone(),
            owner_approval_required: true,
            owner_approval_valid: false,
            release_provenance_required: true,
            release_provenance_valid: false,
            pre_submit_risk_gate_required: true,
            notional_consistency_required: true,
            notional_consistent: false,
            single_order_required: true,
            single_venue_required: true,
            single_account_required: true,
            production_order_submission_allowed: false,
            submit_builder_entry_allowed: false,
            retry_attempted: false,
            replace_attempted: false,
            amend_attempted: false,
            flatten_attempted: false,
            automatic_remediation_allowed: false,
            dashboard_order_controls_enabled: false,
        }
    }

    fn finish(
        mut self,
        decision: PreSubmitRiskDecisionKind,
        code: PreSubmitRiskCode,
        reason: impl Into<String>,
    ) -> Self {
        self.decision = decision;
        self.code = code;
        self.reason = reason.into();
        if decision == PreSubmitRiskDecisionKind::Allow {
            self.owner_approval_valid = true;
            self.release_provenance_valid = true;
            self.production_order_submission_allowed = true;
            self.submit_builder_entry_allowed = true;
        }
        self
    }
}

/// Evaluates a production order candidate before it may enter the submit builder.
///
/// The gate fails closed: missing fields, unknown allowlist values, expired or
/// reused approval, missing provenance, and exceeded limits all return auditable
/// denial or blocked evidence with stable codes.
#[must_use]
pub fn evaluate_pre_submit_risk_gate(
    request: &PreSubmitRiskRequest,
    policy: &PreSubmitRiskPolicy,
    evaluated_at_unix_ns: u64,
) -> PreSubmitRiskGateEvidence {
    let evidence = PreSubmitRiskGateEvidence::from_request(request);

    if !request.unrecognized_fields.is_empty() {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::UnknownField,
            format!(
                "unrecognized fields are not accepted: {}",
                request.unrecognized_fields.join(",")
            ),
        );
    }

    if missing(&request.gate_id) {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Blocked,
            PreSubmitRiskCode::GateIdMissing,
            "gate_id is required",
        );
    }
    if missing(&request.lifecycle_id) {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::LifecycleIdMissing,
            "lifecycle_id is required",
        );
    }

    let Some(account_label) = present(&request.account_label) else {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::AccountMissing,
            "account_label is required",
        );
    };
    if !contains(&policy.allowed_accounts, account_label) {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::AccountUnknown,
            format!("account_label {account_label} is not allowlisted"),
        );
    }

    let Some(instrument_id) = present(&request.instrument_id) else {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::InstrumentMissing,
            "instrument_id is required",
        );
    };
    if !contains(&policy.allowed_instruments, instrument_id) {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::InstrumentUnknown,
            format!("instrument_id {instrument_id} is not allowlisted"),
        );
    }

    let Some(venue) = present(&request.venue) else {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::VenueMissing,
            "venue is required",
        );
    };
    if !contains(&policy.allowed_venues, venue) {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::VenueUnknown,
            format!("venue {venue} is not allowlisted"),
        );
    }

    let Some(side) = present(&request.side) else {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::SideMissing,
            "side is required",
        );
    };
    if !contains(&policy.allowed_sides, side) {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::SideUnsupported,
            format!("side {side} is not supported"),
        );
    }

    let Some(quantity) = request.quantity else {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::QuantityMissing,
            "quantity is required",
        );
    };
    if quantity <= Decimal::ZERO {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::QuantityNotPositive,
            "quantity must be positive",
        );
    }
    if quantity > policy.max_quantity {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::QuantityLimitExceeded,
            format!("quantity {quantity} exceeds max {}", policy.max_quantity),
        );
    }

    let Some(price) = request.price else {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::PriceMissing,
            "price is required",
        );
    };
    if price <= Decimal::ZERO {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::PriceNotPositive,
            "price must be positive",
        );
    }
    if price > policy.max_price {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::PriceLimitExceeded,
            format!("price {price} exceeds max {}", policy.max_price),
        );
    }

    let Some(notional) = request.notional else {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::NotionalMissing,
            "notional is required",
        );
    };
    if notional <= Decimal::ZERO {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::NotionalNotPositive,
            "notional must be positive",
        );
    }
    let Some(computed_notional) = compute_exact_notional(quantity, price) else {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::NotionalMismatch,
            "quantity * price cannot be represented as an exact Decimal notional",
        );
    };
    if computed_notional != notional {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::NotionalMismatch,
            format!(
                "request notional {notional} does not match exact quantity * price {computed_notional}"
            ),
        );
    }
    let mut evidence = evidence;
    evidence.notional_consistent = true;
    if computed_notional > policy.max_notional {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::NotionalLimitExceeded,
            format!(
                "computed notional {computed_notional} exceeds max {}",
                policy.max_notional
            ),
        );
    }

    let Some(order_type) = present(&request.order_type) else {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::OrderTypeMissing,
            "order_type is required",
        );
    };
    if !contains(&policy.allowed_order_types, order_type) {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::OrderTypeUnsupported,
            format!("order_type {order_type} is not supported"),
        );
    }

    let Some(time_in_force) = present(&request.time_in_force) else {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::TimeInForceMissing,
            "time_in_force is required",
        );
    };
    if !contains(&policy.allowed_time_in_force, time_in_force) {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::TimeInForceUnsupported,
            format!("time_in_force {time_in_force} is not supported"),
        );
    }

    let Some(environment) = present(&request.environment) else {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Blocked,
            PreSubmitRiskCode::EnvironmentMissing,
            "environment is required",
        );
    };
    if environment != policy.expected_environment {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Blocked,
            PreSubmitRiskCode::EnvironmentMismatch,
            format!(
                "environment {environment} does not match expected {}",
                policy.expected_environment
            ),
        );
    }

    let Some(order_intent_hash) = present(&request.order_intent_hash) else {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::IntentHashMissing,
            "order_intent_hash is required",
        );
    };

    let Some(approval) = request.approval.as_ref() else {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::ApprovalMissing,
            "owner approval is required",
        );
    };
    if missing(&approval.approval_id) {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::ApprovalIdMissing,
            "approval_id is required",
        );
    }
    if evaluated_at_unix_ns > approval.expires_at_unix_ns {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::ApprovalExpired,
            "owner approval expired before submit builder entry",
        );
    }
    if approval.order_intent_hash != order_intent_hash {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::ApprovalIntentMismatch,
            "owner approval does not match order_intent_hash",
        );
    }
    if !approval.single_use {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::ApprovalNotSingleUse,
            "owner approval must be single-use",
        );
    }
    if approval.consumed {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Deny,
            PreSubmitRiskCode::ApprovalAlreadyConsumed,
            "owner approval has already been consumed",
        );
    }

    let Some(provenance) = request.release_provenance.as_ref() else {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Blocked,
            PreSubmitRiskCode::ProvenanceMissing,
            "release provenance is required",
        );
    };
    if missing(&provenance.release_tag) {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Blocked,
            PreSubmitRiskCode::ProvenanceTagMissing,
            "release_tag is required",
        );
    }
    if provenance.release_tag != policy.required_release_tag {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Blocked,
            PreSubmitRiskCode::ProvenanceTagMismatch,
            format!(
                "release_tag {} does not match required {}",
                provenance.release_tag, policy.required_release_tag
            ),
        );
    }
    if missing(&provenance.release_commit) {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Blocked,
            PreSubmitRiskCode::ProvenanceCommitMissing,
            "release_commit is required",
        );
    }
    if provenance.release_gate != policy.required_release_gate {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Blocked,
            PreSubmitRiskCode::ProvenanceGateMismatch,
            format!(
                "release_gate {} does not match required {}",
                provenance.release_gate, policy.required_release_gate
            ),
        );
    }
    if !provenance.strict_provenance {
        return evidence.finish(
            PreSubmitRiskDecisionKind::Blocked,
            PreSubmitRiskCode::ProvenanceNotStrict,
            "strict release provenance is required",
        );
    }

    evidence.finish(
        PreSubmitRiskDecisionKind::Allow,
        PreSubmitRiskCode::Allowed,
        "pre-submit risk gate allowed submit builder entry",
    )
}

fn missing(value: &str) -> bool {
    value.trim().is_empty()
}

fn present(value: &Option<String>) -> Option<&str> {
    value.as_deref().filter(|value| !missing(value))
}

fn contains(values: &BTreeSet<String>, value: &str) -> bool {
    values.iter().any(|item| item == value)
}

/// Computes the exact LIMIT/GTC notional used by V20 submit gates.
#[must_use]
pub fn compute_exact_notional(quantity: Decimal, price: Decimal) -> Option<Decimal> {
    quantity.checked_mul(price)
}
