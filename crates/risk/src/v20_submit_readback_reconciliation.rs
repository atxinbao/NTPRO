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

//! V200 post-submit readback reconciliation evidence.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    v20_pre_submit_gate::V20_ORDER_LIFECYCLE_CONTRACT_ID,
    v20_submit_response_redaction::{
        SubmitResponseRedactionEvidence, SubmitResponseRedactionState,
    },
};

/// Stable schema for V200 post-submit readback reconciliation evidence.
pub const V20_SUBMIT_READBACK_RECONCILIATION_SCHEMA_VERSION: &str =
    "ntpro.v200_submit_readback_reconciliation.v1";

/// Final readback reconciliation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmitReadbackReconciliationState {
    Matched,
    Mismatched,
    Missing,
    Ambiguous,
    ReadbackFailed,
    Blocked,
}

/// Stable readback reconciliation evidence codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubmitReadbackReconciliationCode {
    #[serde(rename = "v200_submit_readback_matched")]
    Matched,
    #[serde(rename = "v200_submit_readback_mismatched")]
    Mismatched,
    #[serde(rename = "v200_submit_readback_missing")]
    Missing,
    #[serde(rename = "v200_submit_readback_ambiguous")]
    Ambiguous,
    #[serde(rename = "v200_submit_readback_failed")]
    ReadbackFailed,
    #[serde(rename = "v200_submit_readback_missing_response_evidence")]
    MissingResponseEvidence,
    #[serde(rename = "v200_submit_readback_lineage_mismatch")]
    LineageMismatch,
    #[serde(rename = "v200_submit_readback_id_missing")]
    ReadbackIdMissing,
}

impl SubmitReadbackReconciliationCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Matched => "v200_submit_readback_matched",
            Self::Mismatched => "v200_submit_readback_mismatched",
            Self::Missing => "v200_submit_readback_missing",
            Self::Ambiguous => "v200_submit_readback_ambiguous",
            Self::ReadbackFailed => "v200_submit_readback_failed",
            Self::MissingResponseEvidence => "v200_submit_readback_missing_response_evidence",
            Self::LineageMismatch => "v200_submit_readback_lineage_mismatch",
            Self::ReadbackIdMissing => "v200_submit_readback_id_missing",
        }
    }
}

/// Local submit expectation used to compare redacted readback results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitReadbackExpectation {
    pub lifecycle_id: String,
    pub attempt_id: String,
    pub request_digest: String,
    pub account_label: String,
    pub instrument_id: String,
    pub venue: String,
    pub side: String,
    pub quantity: Decimal,
    pub price: Decimal,
    pub client_order_id: String,
    pub venue_order_id: Option<String>,
    pub expected_venue_status: Option<String>,
    pub expected_venue_timestamp_unix_ms: Option<u64>,
}

/// Redacted venue order readback snapshot supplied to reconciliation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VenueOrderReadback {
    pub readback_id: String,
    pub account_label: Option<String>,
    pub instrument_id: Option<String>,
    pub venue: String,
    pub side: Option<String>,
    pub quantity: Option<Decimal>,
    pub price: Option<Decimal>,
    pub client_order_id: Option<String>,
    pub venue_order_id: Option<String>,
    pub venue_status: Option<String>,
    pub venue_timestamp_unix_ms: Option<u64>,
    pub present: bool,
    pub ambiguous: bool,
    pub read_failed: bool,
    pub failure_code: Option<String>,
    pub raw_readback_body_present: bool,
    pub response_headers_present: bool,
}

/// Auditable post-submit readback reconciliation evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitReadbackReconciliationEvidence {
    pub schema_version: String,
    pub contract_id: String,
    pub readback_id: String,
    pub lifecycle_id: String,
    pub attempt_id: String,
    pub state: SubmitReadbackReconciliationState,
    pub code: SubmitReadbackReconciliationCode,
    pub reason: String,
    pub request_digest: String,
    pub response_digest: Option<String>,
    pub response_state: SubmitResponseRedactionState,
    pub expected_account_label: String,
    pub expected_instrument_id: String,
    pub expected_venue: String,
    pub expected_side: String,
    pub expected_quantity: Decimal,
    pub expected_price: Decimal,
    pub expected_client_order_id: String,
    pub expected_venue_order_id: Option<String>,
    pub expected_venue_status: Option<String>,
    pub expected_venue_timestamp_unix_ms: Option<u64>,
    pub observed_account_label: Option<String>,
    pub observed_instrument_id: Option<String>,
    pub observed_venue: String,
    pub observed_side: Option<String>,
    pub observed_quantity: Option<Decimal>,
    pub observed_price: Option<Decimal>,
    pub observed_client_order_id: Option<String>,
    pub observed_venue_order_id: Option<String>,
    pub observed_venue_status: Option<String>,
    pub observed_venue_timestamp_unix_ms: Option<u64>,
    pub mismatch_fields: Vec<String>,
    pub readback_attempted: bool,
    pub readback_response_present: bool,
    pub readback_consistent: bool,
    pub readback_missing: bool,
    pub readback_ambiguous: bool,
    pub readback_failed: bool,
    pub risk_evidence_required: bool,
    pub cancel_or_audit_input_ready: bool,
    pub dashboard_read_only_consumable: bool,
    pub automatic_cancel_attempted: bool,
    pub automatic_remediation_allowed: bool,
    pub retry_attempted: bool,
    pub replace_attempted: bool,
    pub amend_attempted: bool,
    pub flatten_attempted: bool,
    pub dashboard_order_controls_enabled: bool,
    pub raw_readback_body_recorded: bool,
    pub response_headers_recorded: bool,
    pub failure_code: Option<String>,
}

impl SubmitReadbackReconciliationEvidence {
    fn from_inputs(
        expectation: &SubmitReadbackExpectation,
        response: &SubmitResponseRedactionEvidence,
        readback: &VenueOrderReadback,
    ) -> Self {
        Self {
            schema_version: V20_SUBMIT_READBACK_RECONCILIATION_SCHEMA_VERSION.to_string(),
            contract_id: V20_ORDER_LIFECYCLE_CONTRACT_ID.to_string(),
            readback_id: readback.readback_id.clone(),
            lifecycle_id: expectation.lifecycle_id.clone(),
            attempt_id: expectation.attempt_id.clone(),
            state: SubmitReadbackReconciliationState::Blocked,
            code: SubmitReadbackReconciliationCode::ReadbackIdMissing,
            reason: String::new(),
            request_digest: expectation.request_digest.clone(),
            response_digest: response.response_digest.clone(),
            response_state: response.state,
            expected_account_label: expectation.account_label.clone(),
            expected_instrument_id: expectation.instrument_id.clone(),
            expected_venue: expectation.venue.clone(),
            expected_side: expectation.side.clone(),
            expected_quantity: expectation.quantity,
            expected_price: expectation.price,
            expected_client_order_id: expectation.client_order_id.clone(),
            expected_venue_order_id: expectation.venue_order_id.clone(),
            expected_venue_status: expectation.expected_venue_status.clone(),
            expected_venue_timestamp_unix_ms: expectation.expected_venue_timestamp_unix_ms,
            observed_account_label: readback.account_label.clone(),
            observed_instrument_id: readback.instrument_id.clone(),
            observed_venue: readback.venue.clone(),
            observed_side: readback.side.clone(),
            observed_quantity: readback.quantity,
            observed_price: readback.price,
            observed_client_order_id: readback.client_order_id.clone(),
            observed_venue_order_id: readback.venue_order_id.clone(),
            observed_venue_status: readback.venue_status.clone(),
            observed_venue_timestamp_unix_ms: readback.venue_timestamp_unix_ms,
            mismatch_fields: Vec::new(),
            readback_attempted: false,
            readback_response_present: false,
            readback_consistent: false,
            readback_missing: false,
            readback_ambiguous: false,
            readback_failed: false,
            risk_evidence_required: false,
            cancel_or_audit_input_ready: false,
            dashboard_read_only_consumable: false,
            automatic_cancel_attempted: false,
            automatic_remediation_allowed: false,
            retry_attempted: false,
            replace_attempted: false,
            amend_attempted: false,
            flatten_attempted: false,
            dashboard_order_controls_enabled: false,
            raw_readback_body_recorded: false,
            response_headers_recorded: false,
            failure_code: readback.failure_code.clone(),
        }
    }

    fn finish(
        mut self,
        state: SubmitReadbackReconciliationState,
        code: SubmitReadbackReconciliationCode,
        reason: impl Into<String>,
    ) -> Self {
        self.state = state;
        self.code = code;
        self.reason = reason.into();
        self.readback_attempted = state != SubmitReadbackReconciliationState::Blocked;
        self.readback_response_present = matches!(
            state,
            SubmitReadbackReconciliationState::Matched
                | SubmitReadbackReconciliationState::Mismatched
                | SubmitReadbackReconciliationState::Ambiguous
        );
        self.readback_consistent = state == SubmitReadbackReconciliationState::Matched;
        self.readback_missing = state == SubmitReadbackReconciliationState::Missing;
        self.readback_ambiguous = state == SubmitReadbackReconciliationState::Ambiguous;
        self.readback_failed = state == SubmitReadbackReconciliationState::ReadbackFailed;
        self.risk_evidence_required = matches!(
            state,
            SubmitReadbackReconciliationState::Mismatched
                | SubmitReadbackReconciliationState::Missing
                | SubmitReadbackReconciliationState::Ambiguous
                | SubmitReadbackReconciliationState::ReadbackFailed
        );
        self.cancel_or_audit_input_ready = state != SubmitReadbackReconciliationState::Blocked;
        self.dashboard_read_only_consumable = state != SubmitReadbackReconciliationState::Blocked;
        self
    }

    fn with_mismatch_fields(mut self, mismatch_fields: Vec<String>) -> Self {
        self.mismatch_fields = mismatch_fields;
        self
    }
}

/// Reconciles local submit expectation, redacted response evidence, and venue
/// readback snapshot.
#[must_use]
pub fn reconcile_post_submit_readback(
    expectation: &SubmitReadbackExpectation,
    response: &SubmitResponseRedactionEvidence,
    readback: &VenueOrderReadback,
) -> SubmitReadbackReconciliationEvidence {
    let evidence =
        SubmitReadbackReconciliationEvidence::from_inputs(expectation, response, readback);

    if is_blank(&readback.readback_id) {
        return evidence.finish(
            SubmitReadbackReconciliationState::Blocked,
            SubmitReadbackReconciliationCode::ReadbackIdMissing,
            "readback_id is required",
        );
    }
    if !response.redacted_evidence_ready || response.response_digest.is_none() {
        return evidence.finish(
            SubmitReadbackReconciliationState::Blocked,
            SubmitReadbackReconciliationCode::MissingResponseEvidence,
            "redacted submit response evidence is required",
        );
    }
    if expectation.lifecycle_id != response.lifecycle_id
        || expectation.attempt_id != response.attempt_id
        || expectation.request_digest != response.request_digest.clone().unwrap_or_default()
    {
        return evidence.finish(
            SubmitReadbackReconciliationState::Blocked,
            SubmitReadbackReconciliationCode::LineageMismatch,
            "readback expectation does not match redacted response evidence",
        );
    }
    if readback.read_failed {
        return evidence.finish(
            SubmitReadbackReconciliationState::ReadbackFailed,
            SubmitReadbackReconciliationCode::ReadbackFailed,
            "venue readback failed before a comparable order snapshot was available",
        );
    }
    if !readback.present {
        return evidence.finish(
            SubmitReadbackReconciliationState::Missing,
            SubmitReadbackReconciliationCode::Missing,
            "venue readback did not find the submitted order",
        );
    }
    if readback.ambiguous {
        return evidence.finish(
            SubmitReadbackReconciliationState::Ambiguous,
            SubmitReadbackReconciliationCode::Ambiguous,
            "venue readback returned an ambiguous order state",
        );
    }

    let mismatch_fields = mismatch_fields(expectation, readback);
    if !mismatch_fields.is_empty() {
        return evidence.with_mismatch_fields(mismatch_fields).finish(
            SubmitReadbackReconciliationState::Mismatched,
            SubmitReadbackReconciliationCode::Mismatched,
            "venue readback does not match local submit evidence",
        );
    }

    evidence.finish(
        SubmitReadbackReconciliationState::Matched,
        SubmitReadbackReconciliationCode::Matched,
        "venue readback matches local submit evidence",
    )
}

fn mismatch_fields(
    expectation: &SubmitReadbackExpectation,
    readback: &VenueOrderReadback,
) -> Vec<String> {
    let mut fields = Vec::new();

    compare_str(
        &mut fields,
        "account_label",
        Some(expectation.account_label.as_str()),
        readback.account_label.as_deref(),
    );
    compare_str(
        &mut fields,
        "instrument_id",
        Some(expectation.instrument_id.as_str()),
        readback.instrument_id.as_deref(),
    );
    compare_str(
        &mut fields,
        "venue",
        Some(expectation.venue.as_str()),
        Some(readback.venue.as_str()),
    );
    compare_str(
        &mut fields,
        "side",
        Some(expectation.side.as_str()),
        readback.side.as_deref(),
    );
    compare_decimal(
        &mut fields,
        "quantity",
        Some(expectation.quantity),
        readback.quantity,
    );
    compare_decimal(
        &mut fields,
        "price",
        Some(expectation.price),
        readback.price,
    );
    compare_str(
        &mut fields,
        "client_order_id",
        Some(expectation.client_order_id.as_str()),
        readback.client_order_id.as_deref(),
    );
    compare_str(
        &mut fields,
        "venue_order_id",
        expectation.venue_order_id.as_deref(),
        readback.venue_order_id.as_deref(),
    );
    compare_str(
        &mut fields,
        "venue_status",
        expectation.expected_venue_status.as_deref(),
        readback.venue_status.as_deref(),
    );
    if expectation.expected_venue_timestamp_unix_ms != readback.venue_timestamp_unix_ms {
        fields.push("venue_timestamp_unix_ms".to_string());
    }

    fields
}

fn compare_str(
    fields: &mut Vec<String>,
    field: &str,
    expected: Option<&str>,
    observed: Option<&str>,
) {
    if expected != observed {
        fields.push(field.to_string());
    }
}

fn compare_decimal(
    fields: &mut Vec<String>,
    field: &str,
    expected: Option<Decimal>,
    observed: Option<Decimal>,
) {
    if expected != observed {
        fields.push(field.to_string());
    }
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}
