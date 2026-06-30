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
    v20_owner_approval::{
        OwnerApprovalCandidate, OwnerApprovalCode, OwnerApprovalDecision, OwnerApprovalRecord,
        OwnerApprovalRequest, OwnerApprovalScope, OwnerApprovalState,
        V20_OWNER_APPROVAL_SCHEMA_VERSION, consume_owner_approval, evaluate_owner_approval,
        owner_approval_digest,
    },
    v20_pre_submit_gate::{
        PreSubmitReleaseProvenance, V20_ORDER_LIFECYCLE_CONTRACT_ID, V20_REQUIRED_RELEASE_GATE,
        V20_REQUIRED_RELEASE_TAG, evaluate_pre_submit_risk_gate,
    },
};
use rust_decimal_macros::dec;

const NOW_NS: u64 = 1_780_000_000_000_000_000;
const V20_RELEASE_COMMIT: &str = "d29a764a2fb6b3f9c187d2af17337b08b40d794b";

#[test]
fn approves_matching_owner_decision_and_exports_pre_submit_shape() {
    let record = record();
    let candidate = candidate();

    let evidence = evaluate_owner_approval(&record, &candidate, NOW_NS);

    assert_eq!(evidence.schema_version, V20_OWNER_APPROVAL_SCHEMA_VERSION);
    assert_eq!(evidence.contract_id, V20_ORDER_LIFECYCLE_CONTRACT_ID);
    assert_eq!(evidence.state, OwnerApprovalState::Approved);
    assert_eq!(evidence.code, OwnerApprovalCode::Allowed);
    assert!(evidence.candidate_scope_match);
    assert!(evidence.digest_match);
    assert!(evidence.release_provenance_match);
    assert!(evidence.submit_consumption_allowed);
    assert!(!evidence.dashboard_approval_controls_enabled);
    assert!(!evidence.dashboard_order_controls_enabled);
    assert!(!evidence.retry_attempted);
    assert!(!evidence.automatic_remediation_allowed);

    let pre_submit = evidence
        .as_pre_submit_approval()
        .expect("approved evidence exports pre-submit approval");
    assert_eq!(pre_submit.approval_id, "approval-v200-003");
    assert_eq!(pre_submit.owner_label, "owner-001");
    assert_eq!(pre_submit.order_intent_hash, "intent-v200-003");
    assert!(pre_submit.single_use);
    assert!(!pre_submit.consumed);
}

#[test]
fn consumes_valid_approval_once_before_submit_attempt() {
    let evidence = evaluate_owner_approval(&record(), &candidate(), NOW_NS);

    let consumed = consume_owner_approval(&evidence, NOW_NS + 1);
    let second_consume = consume_owner_approval(&consumed, NOW_NS + 2);

    assert_eq!(consumed.state, OwnerApprovalState::Consumed);
    assert_eq!(consumed.code, OwnerApprovalCode::Consumed);
    assert!(consumed.consumed);
    assert!(!consumed.submit_consumption_allowed);
    assert!(!consumed.approval_execution_authorized_after_attempt);
    assert!(consumed.as_pre_submit_approval().is_none());

    assert_eq!(second_consume.state, OwnerApprovalState::Rejected);
    assert_eq!(
        second_consume.code,
        OwnerApprovalCode::ConsumptionRequiresApprovedEvidence
    );
}

#[test]
fn rejects_expired_approval_with_evidence() {
    let mut record = record();
    record.request.expires_at_unix_ns = NOW_NS - 1;
    record.request.approval_digest = owner_approval_digest(
        &record.request.scope,
        &record.request.nonce,
        &record.request.environment,
        &record.request.release_provenance,
    );
    let candidate = OwnerApprovalCandidate {
        approval_digest: record.request.approval_digest.clone(),
        ..candidate()
    };

    let evidence = evaluate_owner_approval(&record, &candidate, NOW_NS);

    assert_eq!(evidence.state, OwnerApprovalState::Expired);
    assert_eq!(evidence.code, OwnerApprovalCode::Expired);
    assert_eq!(evidence.code.as_str(), "v200_owner_approval_expired");
    assert!(!evidence.submit_consumption_allowed);
}

#[test]
fn rejects_revoked_approval_with_evidence() {
    let mut record = record();
    record.revoked_at_unix_ns = Some(NOW_NS);

    let evidence = evaluate_owner_approval(&record, &candidate(), NOW_NS + 1);

    assert_eq!(evidence.state, OwnerApprovalState::Revoked);
    assert_eq!(evidence.code, OwnerApprovalCode::Revoked);
    assert_eq!(evidence.code.as_str(), "v200_owner_approval_revoked");
}

#[test]
fn rejects_owner_rejected_decision() {
    let mut record = record();
    record.decision = OwnerApprovalDecision::Rejected;

    let evidence = evaluate_owner_approval(&record, &candidate(), NOW_NS);

    assert_eq!(evidence.state, OwnerApprovalState::Rejected);
    assert_eq!(evidence.code, OwnerApprovalCode::Rejected);
    assert_eq!(evidence.code.as_str(), "v200_owner_approval_rejected");
}

#[test]
fn rejects_already_consumed_record() {
    let mut record = record();
    record.consumed_at_unix_ns = Some(NOW_NS - 1);

    let evidence = evaluate_owner_approval(&record, &candidate(), NOW_NS);

    assert_eq!(evidence.state, OwnerApprovalState::Consumed);
    assert_eq!(evidence.code, OwnerApprovalCode::AlreadyConsumed);
    assert_eq!(
        evidence.code.as_str(),
        "v200_owner_approval_already_consumed"
    );
    assert!(evidence.consumed);
}

#[test]
fn rejects_digest_mismatch() {
    let mut record = record();
    record.request.approval_digest = "bad-digest".to_string();

    let evidence = evaluate_owner_approval(&record, &candidate(), NOW_NS);

    assert_eq!(evidence.state, OwnerApprovalState::Rejected);
    assert_eq!(evidence.code, OwnerApprovalCode::RequestDigestMismatch);
    assert_eq!(
        evidence.code.as_str(),
        "v200_owner_approval_request_digest_mismatch"
    );
}

#[test]
fn rejects_cross_account_scope_reuse() {
    let mut candidate = candidate();
    candidate.scope.account_label = "acct-prod-002".to_string();
    candidate.approval_digest = owner_approval_digest(
        &candidate.scope,
        "nonce-v200-003",
        "production",
        &provenance(),
    );

    let evidence = evaluate_owner_approval(&record(), &candidate, NOW_NS);

    assert_eq!(evidence.state, OwnerApprovalState::Rejected);
    assert_eq!(evidence.code, OwnerApprovalCode::CandidateDigestMismatch);
    assert!(!evidence.submit_consumption_allowed);
}

#[test]
fn rejects_cross_environment_reuse() {
    let mut candidate = candidate();
    candidate.environment = "sandbox".to_string();

    let evidence = evaluate_owner_approval(&record(), &candidate, NOW_NS);

    assert_eq!(evidence.state, OwnerApprovalState::Rejected);
    assert_eq!(evidence.code, OwnerApprovalCode::EnvironmentMismatch);
    assert_eq!(
        evidence.code.as_str(),
        "v200_owner_approval_environment_mismatch"
    );
}

#[test]
fn approved_evidence_can_feed_v200_pre_submit_gate() {
    let owner_evidence = evaluate_owner_approval(&record(), &candidate(), NOW_NS);
    let approval = owner_evidence
        .as_pre_submit_approval()
        .expect("approved evidence exports pre-submit approval");
    let mut request = super_like_pre_submit_request();
    request.approval = Some(approval);

    let risk_evidence = evaluate_pre_submit_risk_gate(&request, &super_like_policy(), NOW_NS);

    assert_eq!(
        risk_evidence.decision,
        nautilus_risk::v20_pre_submit_gate::PreSubmitRiskDecisionKind::Allow
    );
}

fn record() -> OwnerApprovalRecord {
    OwnerApprovalRecord {
        approval_id: "approval-v200-003".to_string(),
        request: request(),
        decision: OwnerApprovalDecision::Approved,
        decided_at_unix_ns: NOW_NS - 10,
        revoked_at_unix_ns: None,
        consumed_at_unix_ns: None,
    }
}

fn request() -> OwnerApprovalRequest {
    let scope = scope();
    let release_provenance = provenance();
    let nonce = "nonce-v200-003";
    let environment = "production";
    let approval_digest = owner_approval_digest(&scope, nonce, environment, &release_provenance);

    OwnerApprovalRequest {
        request_id: "request-v200-003".to_string(),
        lifecycle_id: "lc-v200-003".to_string(),
        owner_label: "owner-001".to_string(),
        scope,
        nonce: nonce.to_string(),
        environment: environment.to_string(),
        release_provenance,
        approval_digest,
        expires_at_unix_ns: NOW_NS + 1_000,
    }
}

fn candidate() -> OwnerApprovalCandidate {
    let scope = scope();
    let release_provenance = provenance();
    let approval_digest =
        owner_approval_digest(&scope, "nonce-v200-003", "production", &release_provenance);

    OwnerApprovalCandidate {
        lifecycle_id: "lc-v200-003".to_string(),
        scope,
        environment: "production".to_string(),
        release_provenance,
        approval_digest,
    }
}

fn scope() -> OwnerApprovalScope {
    OwnerApprovalScope {
        account_label: "acct-prod-001".to_string(),
        instrument_id: "BTCUSDT.BINANCE".to_string(),
        venue: "BINANCE".to_string(),
        side: "buy".to_string(),
        quantity: dec!(0.10),
        price: dec!(50000),
        notional: dec!(5000),
        order_type: "limit".to_string(),
        time_in_force: "gtc".to_string(),
        order_intent_hash: "intent-v200-003".to_string(),
    }
}

fn provenance() -> PreSubmitReleaseProvenance {
    PreSubmitReleaseProvenance {
        release_tag: V20_REQUIRED_RELEASE_TAG.to_string(),
        release_commit: V20_RELEASE_COMMIT.to_string(),
        release_gate: V20_REQUIRED_RELEASE_GATE.to_string(),
        strict_provenance: true,
    }
}

fn super_like_pre_submit_request() -> nautilus_risk::v20_pre_submit_gate::PreSubmitRiskRequest {
    nautilus_risk::v20_pre_submit_gate::PreSubmitRiskRequest {
        gate_id: "gate-v200-003".to_string(),
        lifecycle_id: "lc-v200-003".to_string(),
        account_label: Some("acct-prod-001".to_string()),
        instrument_id: Some("BTCUSDT.BINANCE".to_string()),
        venue: Some("BINANCE".to_string()),
        side: Some("buy".to_string()),
        quantity: Some(dec!(0.10)),
        price: Some(dec!(50000)),
        notional: Some(dec!(5000)),
        order_type: Some("limit".to_string()),
        time_in_force: Some("gtc".to_string()),
        environment: Some("production".to_string()),
        order_intent_hash: Some("intent-v200-003".to_string()),
        approval: None,
        release_provenance: Some(provenance()),
        unrecognized_fields: Vec::new(),
    }
}

fn super_like_policy() -> nautilus_risk::v20_pre_submit_gate::PreSubmitRiskPolicy {
    nautilus_risk::v20_pre_submit_gate::PreSubmitRiskPolicy {
        allowed_accounts: ["acct-prod-001"].into_iter().map(str::to_string).collect(),
        allowed_instruments: ["BTCUSDT.BINANCE"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        allowed_venues: ["BINANCE"].into_iter().map(str::to_string).collect(),
        allowed_sides: ["buy", "sell"].into_iter().map(str::to_string).collect(),
        allowed_order_types: ["limit"].into_iter().map(str::to_string).collect(),
        allowed_time_in_force: ["gtc"].into_iter().map(str::to_string).collect(),
        expected_environment: "production".to_string(),
        required_release_tag: V20_REQUIRED_RELEASE_TAG.to_string(),
        required_release_gate: V20_REQUIRED_RELEASE_GATE.to_string(),
        max_quantity: dec!(0.25),
        max_price: dec!(100000),
        max_notional: dec!(10000),
    }
}
