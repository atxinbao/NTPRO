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

use std::collections::BTreeSet;

use nautilus_risk::v20_pre_submit_gate::{
    PreSubmitApproval, PreSubmitReleaseProvenance, PreSubmitRiskCode, PreSubmitRiskDecisionKind,
    PreSubmitRiskPolicy, PreSubmitRiskRequest, V20_ORDER_LIFECYCLE_CONTRACT_ID,
    V20_PRE_SUBMIT_RISK_GATE_SCHEMA_VERSION, V20_REQUIRED_RELEASE_GATE, V20_REQUIRED_RELEASE_TAG,
    evaluate_pre_submit_risk_gate,
};
use rust_decimal_macros::dec;

const NOW_NS: u64 = 1_780_000_000_000_000_000;
const V20_RELEASE_COMMIT: &str = "d29a764a2fb6b3f9c187d2af17337b08b40d794b";

#[test]
fn allows_valid_owner_approved_production_limit_order() {
    let evidence = evaluate_pre_submit_risk_gate(&valid_request(), &policy(), NOW_NS);

    assert_eq!(evidence.decision, PreSubmitRiskDecisionKind::Allow);
    assert_eq!(evidence.code, PreSubmitRiskCode::Allowed);
    assert_eq!(
        evidence.schema_version,
        V20_PRE_SUBMIT_RISK_GATE_SCHEMA_VERSION
    );
    assert_eq!(evidence.contract_id, V20_ORDER_LIFECYCLE_CONTRACT_ID);
    assert!(evidence.owner_approval_valid);
    assert!(evidence.release_provenance_valid);
    assert!(evidence.production_order_submission_allowed);
    assert!(evidence.submit_builder_entry_allowed);
    assert!(!evidence.retry_attempted);
    assert!(!evidence.replace_attempted);
    assert!(!evidence.amend_attempted);
    assert!(!evidence.flatten_attempted);
    assert!(!evidence.automatic_remediation_allowed);
    assert!(!evidence.dashboard_order_controls_enabled);
}

#[test]
fn denies_unknown_account_by_default() {
    let mut request = valid_request();
    request.account_label = Some("acct-unknown".to_string());

    let evidence = evaluate_pre_submit_risk_gate(&request, &policy(), NOW_NS);

    assert_eq!(evidence.decision, PreSubmitRiskDecisionKind::Deny);
    assert_eq!(evidence.code, PreSubmitRiskCode::AccountUnknown);
    assert_eq!(evidence.code.as_str(), "v200_pre_submit_account_unknown");
    assert!(!evidence.production_order_submission_allowed);
    assert!(!evidence.submit_builder_entry_allowed);
}

#[test]
fn denies_missing_required_field_with_stable_code() {
    let mut request = valid_request();
    request.price = None;

    let evidence = evaluate_pre_submit_risk_gate(&request, &policy(), NOW_NS);

    assert_eq!(evidence.decision, PreSubmitRiskDecisionKind::Deny);
    assert_eq!(evidence.code, PreSubmitRiskCode::PriceMissing);
    assert_eq!(evidence.code.as_str(), "v200_pre_submit_price_missing");
    assert!(evidence.price.is_none());
}

#[test]
fn denies_notional_limit_exceeded() {
    let mut request = valid_request();
    request.notional = Some(dec!(10001));

    let evidence = evaluate_pre_submit_risk_gate(&request, &policy(), NOW_NS);

    assert_eq!(evidence.decision, PreSubmitRiskDecisionKind::Deny);
    assert_eq!(evidence.code, PreSubmitRiskCode::NotionalLimitExceeded);
    assert_eq!(
        evidence.code.as_str(),
        "v200_pre_submit_notional_limit_exceeded"
    );
}

#[test]
fn denies_expired_approval() {
    let mut request = valid_request();
    request.approval = Some(PreSubmitApproval {
        expires_at_unix_ns: NOW_NS - 1,
        ..approval()
    });

    let evidence = evaluate_pre_submit_risk_gate(&request, &policy(), NOW_NS);

    assert_eq!(evidence.decision, PreSubmitRiskDecisionKind::Deny);
    assert_eq!(evidence.code, PreSubmitRiskCode::ApprovalExpired);
    assert_eq!(evidence.code.as_str(), "v200_pre_submit_approval_expired");
}

#[test]
fn denies_missing_approval_before_submit_builder_entry() {
    let mut request = valid_request();
    request.approval = None;

    let evidence = evaluate_pre_submit_risk_gate(&request, &policy(), NOW_NS);

    assert_eq!(evidence.decision, PreSubmitRiskDecisionKind::Deny);
    assert_eq!(evidence.code, PreSubmitRiskCode::ApprovalMissing);
    assert_eq!(evidence.code.as_str(), "v200_pre_submit_approval_missing");
    assert!(!evidence.owner_approval_valid);
}

#[test]
fn blocks_environment_mismatch() {
    let mut request = valid_request();
    request.environment = Some("sandbox".to_string());

    let evidence = evaluate_pre_submit_risk_gate(&request, &policy(), NOW_NS);

    assert_eq!(evidence.decision, PreSubmitRiskDecisionKind::Blocked);
    assert_eq!(evidence.code, PreSubmitRiskCode::EnvironmentMismatch);
    assert_eq!(
        evidence.code.as_str(),
        "v200_pre_submit_environment_mismatch"
    );
    assert!(!evidence.production_order_submission_allowed);
}

#[test]
fn blocks_missing_release_provenance() {
    let mut request = valid_request();
    request.release_provenance = None;

    let evidence = evaluate_pre_submit_risk_gate(&request, &policy(), NOW_NS);

    assert_eq!(evidence.decision, PreSubmitRiskDecisionKind::Blocked);
    assert_eq!(evidence.code, PreSubmitRiskCode::ProvenanceMissing);
    assert_eq!(evidence.code.as_str(), "v200_pre_submit_provenance_missing");
    assert!(!evidence.release_provenance_valid);
}

#[test]
fn blocks_v19_release_tag_even_with_v20_gate() {
    let mut request = valid_request();
    let mut release_provenance = provenance();
    release_provenance.release_tag = "ntpro-rust-only-v0.19.1".to_string();
    request.release_provenance = Some(release_provenance);

    let evidence = evaluate_pre_submit_risk_gate(&request, &policy(), NOW_NS);

    assert_eq!(evidence.decision, PreSubmitRiskDecisionKind::Blocked);
    assert_eq!(evidence.code, PreSubmitRiskCode::ProvenanceTagMismatch);
    assert_eq!(
        evidence.code.as_str(),
        "v200_pre_submit_provenance_tag_mismatch"
    );
    assert_eq!(
        evidence.release_tag.as_deref(),
        Some("ntpro-rust-only-v0.19.1")
    );
    assert!(!evidence.release_provenance_valid);
    assert!(!evidence.production_order_submission_allowed);
}

#[test]
fn blocks_v19_release_gate_even_with_v20_tag() {
    let mut request = valid_request();
    let mut release_provenance = provenance();
    release_provenance.release_gate = "v19-release-gates".to_string();
    request.release_provenance = Some(release_provenance);

    let evidence = evaluate_pre_submit_risk_gate(&request, &policy(), NOW_NS);

    assert_eq!(evidence.decision, PreSubmitRiskDecisionKind::Blocked);
    assert_eq!(evidence.code, PreSubmitRiskCode::ProvenanceGateMismatch);
    assert_eq!(
        evidence.code.as_str(),
        "v200_pre_submit_provenance_gate_mismatch"
    );
    assert_eq!(evidence.release_gate.as_deref(), Some("v19-release-gates"));
    assert!(!evidence.release_provenance_valid);
    assert!(!evidence.submit_builder_entry_allowed);
}

#[test]
fn denies_unrecognized_fields_and_serializes_evidence() {
    let mut request = valid_request();
    request
        .unrecognized_fields
        .push("strategy_generated_retry_hint".to_string());

    let evidence = evaluate_pre_submit_risk_gate(&request, &policy(), NOW_NS);
    let json = serde_json::to_string(&evidence).expect("evidence serializes");

    assert_eq!(evidence.decision, PreSubmitRiskDecisionKind::Deny);
    assert_eq!(evidence.code, PreSubmitRiskCode::UnknownField);
    assert_eq!(evidence.code.as_str(), "v200_pre_submit_unknown_field");
    assert!(json.contains("v200_pre_submit_unknown_field"));
    assert!(json.contains("strategy_generated_retry_hint"));
}

#[test]
fn rejects_unknown_json_shape_before_evaluation() {
    let json = r#"{
        "gate_id": "gate-v200-002",
        "lifecycle_id": "lc-v200-002",
        "account_label": "acct-prod-001",
        "instrument_id": "BTCUSDT.BINANCE",
        "venue": "BINANCE",
        "side": "buy",
        "quantity": "0.10",
        "price": "50000",
        "notional": "5000",
        "order_type": "limit",
        "time_in_force": "gtc",
        "environment": "production",
        "order_intent_hash": "intent-v200-002",
        "approval": {
            "approval_id": "approval-v200-002",
            "owner_label": "owner-001",
            "order_intent_hash": "intent-v200-002",
            "expires_at_unix_ns": 1780000000000000100,
            "single_use": true,
            "consumed": false
        },
        "release_provenance": {
            "release_tag": "ntpro-rust-only-v0.20.0",
            "release_commit": "d29a764a2fb6b3f9c187d2af17337b08b40d794b",
            "release_gate": "v20-release-gates",
            "strict_provenance": true
        },
        "unexpected": true
    }"#;

    let parsed = serde_json::from_str::<PreSubmitRiskRequest>(json);

    assert!(parsed.is_err());
}

fn valid_request() -> PreSubmitRiskRequest {
    PreSubmitRiskRequest {
        gate_id: "gate-v200-002".to_string(),
        lifecycle_id: "lc-v200-002".to_string(),
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
        order_intent_hash: Some("intent-v200-002".to_string()),
        approval: Some(approval()),
        release_provenance: Some(provenance()),
        unrecognized_fields: Vec::new(),
    }
}

fn approval() -> PreSubmitApproval {
    PreSubmitApproval {
        approval_id: "approval-v200-002".to_string(),
        owner_label: "owner-001".to_string(),
        order_intent_hash: "intent-v200-002".to_string(),
        expires_at_unix_ns: NOW_NS + 100,
        single_use: true,
        consumed: false,
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

fn policy() -> PreSubmitRiskPolicy {
    PreSubmitRiskPolicy {
        allowed_accounts: set(["acct-prod-001"]),
        allowed_instruments: set(["BTCUSDT.BINANCE"]),
        allowed_venues: set(["BINANCE"]),
        allowed_sides: set(["buy", "sell"]),
        allowed_order_types: set(["limit"]),
        allowed_time_in_force: set(["gtc"]),
        expected_environment: "production".to_string(),
        required_release_tag: V20_REQUIRED_RELEASE_TAG.to_string(),
        required_release_gate: V20_REQUIRED_RELEASE_GATE.to_string(),
        max_quantity: dec!(0.25),
        max_price: dec!(100000),
        max_notional: dec!(10000),
    }
}

fn set<const N: usize>(values: [&str; N]) -> BTreeSet<String> {
    values.into_iter().map(str::to_string).collect()
}
