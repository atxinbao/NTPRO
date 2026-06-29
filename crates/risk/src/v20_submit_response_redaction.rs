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

//! V200 production submit response redaction evidence.

use serde::{Deserialize, Serialize};

use crate::{
    v20_pre_submit_gate::V20_ORDER_LIFECYCLE_CONTRACT_ID,
    v20_submit_candidate::{GuardedSubmitCandidateEvidence, GuardedSubmitCandidateState},
};

/// Stable schema for V200 submit response redaction evidence.
pub const V20_SUBMIT_RESPONSE_REDACTION_SCHEMA_VERSION: &str =
    "ntpro.v200_submit_response_redaction.v1";

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001b3;

/// Classified venue response state after whitelist redaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmitResponseRedactionState {
    Accepted,
    Rejected,
    Unknown,
    Malformed,
    Blocked,
}

/// Operator-visible response kind supplied to the redactor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmitResponseKind {
    Accepted,
    Rejected,
    Unknown,
    Malformed,
}

/// Stable submit response redaction evidence codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubmitResponseRedactionCode {
    #[serde(rename = "v200_submit_response_accepted")]
    Accepted,
    #[serde(rename = "v200_submit_response_rejected")]
    Rejected,
    #[serde(rename = "v200_submit_response_unknown")]
    Unknown,
    #[serde(rename = "v200_submit_response_malformed")]
    Malformed,
    #[serde(rename = "v200_submit_response_missing_submit_attempt")]
    MissingSubmitAttempt,
    #[serde(rename = "v200_submit_response_lifecycle_mismatch")]
    LifecycleMismatch,
    #[serde(rename = "v200_submit_response_request_digest_missing")]
    RequestDigestMissing,
    #[serde(rename = "v200_submit_response_request_digest_mismatch")]
    RequestDigestMismatch,
    #[serde(rename = "v200_submit_response_id_missing")]
    ResponseIdMissing,
    #[serde(rename = "v200_submit_response_sensitive_material_observed")]
    SensitiveMaterialObserved,
}

impl SubmitResponseRedactionCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "v200_submit_response_accepted",
            Self::Rejected => "v200_submit_response_rejected",
            Self::Unknown => "v200_submit_response_unknown",
            Self::Malformed => "v200_submit_response_malformed",
            Self::MissingSubmitAttempt => "v200_submit_response_missing_submit_attempt",
            Self::LifecycleMismatch => "v200_submit_response_lifecycle_mismatch",
            Self::RequestDigestMissing => "v200_submit_response_request_digest_missing",
            Self::RequestDigestMismatch => "v200_submit_response_request_digest_mismatch",
            Self::ResponseIdMissing => "v200_submit_response_id_missing",
            Self::SensitiveMaterialObserved => "v200_submit_response_sensitive_material_observed",
        }
    }
}

/// Structured venue response summary supplied to the redactor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitResponseRedactionRequest {
    pub response_id: String,
    pub lifecycle_id: String,
    pub attempt_id: String,
    pub request_digest: Option<String>,
    pub response_kind: SubmitResponseKind,
    pub venue: String,
    pub http_status: Option<u16>,
    pub venue_status: Option<String>,
    pub order_id: Option<String>,
    pub client_order_id: Option<String>,
    pub venue_timestamp_unix_ms: Option<u64>,
    pub received_at_unix_ns: u64,
    pub reject_code: Option<String>,
    pub reject_reason_code: Option<String>,
    pub malformed_reason_code: Option<String>,
    pub raw_payload_present: bool,
    pub response_headers_present: bool,
    pub unrestricted_payload_present: bool,
    pub credential_material_present: bool,
    pub signature_material_present: bool,
    pub token_value_present: bool,
    pub signed_query_present: bool,
    pub signed_url_present: bool,
    pub sensitive_marker_count: u32,
}

/// Redacted production submit response evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmitResponseRedactionEvidence {
    pub schema_version: String,
    pub contract_id: String,
    pub response_id: String,
    pub lifecycle_id: String,
    pub attempt_id: String,
    pub state: SubmitResponseRedactionState,
    pub code: SubmitResponseRedactionCode,
    pub reason: String,
    pub venue: String,
    pub http_status: Option<u16>,
    pub venue_status: Option<String>,
    pub order_id: Option<String>,
    pub client_order_id: Option<String>,
    pub venue_timestamp_unix_ms: Option<u64>,
    pub received_at_unix_ns: u64,
    pub request_digest: Option<String>,
    pub response_digest: Option<String>,
    pub reject_code: Option<String>,
    pub reject_reason_code: Option<String>,
    pub malformed_reason_code: Option<String>,
    pub submit_attempt_state: GuardedSubmitCandidateState,
    pub submit_attempt_evidence_ready: bool,
    pub response_redacted: bool,
    pub redacted_evidence_ready: bool,
    pub readback_correlation_ready: bool,
    pub readback_success_inferred: bool,
    pub manual_review_required: bool,
    pub raw_exchange_response_recorded: bool,
    pub response_headers_recorded: bool,
    pub unrestricted_payload_recorded: bool,
    pub credential_material_recorded: bool,
    pub signature_material_recorded: bool,
    pub token_value_recorded: bool,
    pub signed_query_recorded: bool,
    pub signed_url_recorded: bool,
    pub dashboard_raw_response_enabled: bool,
    pub dashboard_order_controls_enabled: bool,
    pub raw_payload_present: bool,
    pub response_headers_present: bool,
    pub sensitive_marker_count: u32,
}

impl SubmitResponseRedactionEvidence {
    fn from_request(
        request: &SubmitResponseRedactionRequest,
        submit_attempt: &GuardedSubmitCandidateEvidence,
    ) -> Self {
        Self {
            schema_version: V20_SUBMIT_RESPONSE_REDACTION_SCHEMA_VERSION.to_string(),
            contract_id: V20_ORDER_LIFECYCLE_CONTRACT_ID.to_string(),
            response_id: request.response_id.clone(),
            lifecycle_id: request.lifecycle_id.clone(),
            attempt_id: request.attempt_id.clone(),
            state: SubmitResponseRedactionState::Blocked,
            code: SubmitResponseRedactionCode::ResponseIdMissing,
            reason: String::new(),
            venue: request.venue.clone(),
            http_status: request.http_status,
            venue_status: None,
            order_id: None,
            client_order_id: None,
            venue_timestamp_unix_ms: request.venue_timestamp_unix_ms,
            received_at_unix_ns: request.received_at_unix_ns,
            request_digest: request.request_digest.clone(),
            response_digest: None,
            reject_code: None,
            reject_reason_code: None,
            malformed_reason_code: None,
            submit_attempt_state: submit_attempt.state,
            submit_attempt_evidence_ready: submit_attempt.submit_attempt_evidence_ready,
            response_redacted: false,
            redacted_evidence_ready: false,
            readback_correlation_ready: false,
            readback_success_inferred: false,
            manual_review_required: false,
            raw_exchange_response_recorded: false,
            response_headers_recorded: false,
            unrestricted_payload_recorded: false,
            credential_material_recorded: false,
            signature_material_recorded: false,
            token_value_recorded: false,
            signed_query_recorded: false,
            signed_url_recorded: false,
            dashboard_raw_response_enabled: false,
            dashboard_order_controls_enabled: false,
            raw_payload_present: request.raw_payload_present,
            response_headers_present: request.response_headers_present,
            sensitive_marker_count: request.sensitive_marker_count,
        }
    }

    fn finish(
        mut self,
        state: SubmitResponseRedactionState,
        code: SubmitResponseRedactionCode,
        reason: impl Into<String>,
    ) -> Self {
        self.state = state;
        self.code = code;
        self.reason = reason.into();
        self.manual_review_required = matches!(
            state,
            SubmitResponseRedactionState::Unknown | SubmitResponseRedactionState::Malformed
        );
        self.redacted_evidence_ready = state != SubmitResponseRedactionState::Blocked;
        self.response_redacted = self.redacted_evidence_ready;
        self.readback_correlation_ready = self.request_digest.is_some()
            && (self.client_order_id.is_some() || self.order_id.is_some())
            && state != SubmitResponseRedactionState::Malformed;
        self
    }

    fn with_allowed_fields(mut self, request: &SubmitResponseRedactionRequest) -> Self {
        self.venue_status = request.venue_status.clone();
        self.order_id = request.order_id.clone();
        self.client_order_id = request.client_order_id.clone();
        self.reject_code = request.reject_code.clone();
        self.reject_reason_code = request.reject_reason_code.clone();
        self.malformed_reason_code = request.malformed_reason_code.clone();
        self.response_digest = Some(response_digest(request));
        self
    }
}

/// Redacts one production submit response into stable audit evidence.
#[must_use]
pub fn redact_production_submit_response(
    request: &SubmitResponseRedactionRequest,
    submit_attempt: &GuardedSubmitCandidateEvidence,
) -> SubmitResponseRedactionEvidence {
    let evidence = SubmitResponseRedactionEvidence::from_request(request, submit_attempt);

    if is_blank(&request.response_id) {
        return evidence.finish(
            SubmitResponseRedactionState::Blocked,
            SubmitResponseRedactionCode::ResponseIdMissing,
            "response_id is required",
        );
    }
    if submit_attempt.state != GuardedSubmitCandidateState::Submitted
        || !submit_attempt.production_submit_attempted
        || !submit_attempt.submit_attempt_evidence_ready
    {
        return evidence.finish(
            SubmitResponseRedactionState::Blocked,
            SubmitResponseRedactionCode::MissingSubmitAttempt,
            "submitted V200-006 attempt evidence is required",
        );
    }
    if request.lifecycle_id != submit_attempt.lifecycle_id
        || request.attempt_id != submit_attempt.attempt_id
    {
        return evidence.finish(
            SubmitResponseRedactionState::Blocked,
            SubmitResponseRedactionCode::LifecycleMismatch,
            "response lineage does not match submit attempt evidence",
        );
    }

    let Some(expected_request_digest) = submit_attempt.request_digest.as_deref() else {
        return evidence.finish(
            SubmitResponseRedactionState::Blocked,
            SubmitResponseRedactionCode::RequestDigestMissing,
            "submit attempt request digest is required",
        );
    };
    let Some(observed_request_digest) = request.request_digest.as_deref() else {
        return evidence.finish(
            SubmitResponseRedactionState::Blocked,
            SubmitResponseRedactionCode::RequestDigestMissing,
            "response request digest is required",
        );
    };
    if observed_request_digest != expected_request_digest {
        return evidence.finish(
            SubmitResponseRedactionState::Blocked,
            SubmitResponseRedactionCode::RequestDigestMismatch,
            "response request digest does not match submit attempt",
        );
    }
    if request.credential_material_present
        || request.signature_material_present
        || request.token_value_present
        || request.signed_query_present
        || request.signed_url_present
        || request.unrestricted_payload_present
        || request.sensitive_marker_count > 0
    {
        return evidence.finish(
            SubmitResponseRedactionState::Blocked,
            SubmitResponseRedactionCode::SensitiveMaterialObserved,
            "response contains material that must not enter redacted evidence",
        );
    }

    let evidence = evidence.with_allowed_fields(request);
    match request.response_kind {
        SubmitResponseKind::Accepted => evidence.finish(
            SubmitResponseRedactionState::Accepted,
            SubmitResponseRedactionCode::Accepted,
            "submit response accepted by venue and redacted",
        ),
        SubmitResponseKind::Rejected => evidence.finish(
            SubmitResponseRedactionState::Rejected,
            SubmitResponseRedactionCode::Rejected,
            "submit response rejected by venue and redacted",
        ),
        SubmitResponseKind::Unknown => evidence.finish(
            SubmitResponseRedactionState::Unknown,
            SubmitResponseRedactionCode::Unknown,
            "submit response state is unknown after redaction",
        ),
        SubmitResponseKind::Malformed => evidence.finish(
            SubmitResponseRedactionState::Malformed,
            SubmitResponseRedactionCode::Malformed,
            "submit response was malformed and only diagnostic code was retained",
        ),
    }
}

fn response_digest(request: &SubmitResponseRedactionRequest) -> String {
    let fields = vec![
        request.response_id.clone(),
        request.lifecycle_id.clone(),
        request.attempt_id.clone(),
        request.request_digest.clone().unwrap_or_default(),
        format!("{:?}", request.response_kind),
        request.venue.clone(),
        request
            .http_status
            .map(|value| value.to_string())
            .unwrap_or_default(),
        request.venue_status.clone().unwrap_or_default(),
        request.order_id.clone().unwrap_or_default(),
        request.client_order_id.clone().unwrap_or_default(),
        request
            .venue_timestamp_unix_ms
            .map(|value| value.to_string())
            .unwrap_or_default(),
        request.reject_code.clone().unwrap_or_default(),
        request.reject_reason_code.clone().unwrap_or_default(),
        request.malformed_reason_code.clone().unwrap_or_default(),
    ];
    checksum_fields(&fields)
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

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}
