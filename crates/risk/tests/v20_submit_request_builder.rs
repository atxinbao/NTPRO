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

use std::collections::{BTreeMap, BTreeSet};

use nautilus_risk::{
    v20_owner_approval::{
        OwnerApprovalCandidate, OwnerApprovalDecision, OwnerApprovalRecord, OwnerApprovalRequest,
        OwnerApprovalScope, OwnerApprovalState, evaluate_owner_approval, owner_approval_digest,
    },
    v20_pre_submit_gate::{
        PreSubmitApproval, PreSubmitReleaseProvenance, PreSubmitRiskCode,
        PreSubmitRiskDecisionKind, PreSubmitRiskPolicy, PreSubmitRiskRequest,
        V20_REQUIRED_RELEASE_GATE, V20_REQUIRED_RELEASE_TAG, evaluate_pre_submit_risk_gate,
    },
    v20_signing_material_gate::{
        SigningMaterialCode, SigningMaterialDecision, SigningMaterialEnvSnapshot,
        SigningMaterialPolicy, SigningMaterialRequirement, SigningMaterialSource,
        evaluate_signing_material_env_gate,
    },
    v20_submit_request_builder::{
        SingleShotSubmitCandidate, SubmitRequestBuildCode, SubmitRequestBuildDecision,
        V20_SUBMIT_REQUEST_BUILDER_SCHEMA_VERSION, build_single_shot_submit_request,
        submit_request_digest,
    },
};
use rust_decimal_macros::dec;

const NOW_NS: u64 = 1_780_000_000_000_000_000;
const V20_RELEASE_COMMIT: &str = "d29a764a2fb6b3f9c187d2af17337b08b40d794b";

#[test]
fn builds_deterministic_redacted_single_shot_request() {
    let candidate = candidate();
    let risk = risk_allow();
    let approval = owner_approval();
    let signing = signing_ready();

    let first = build_single_shot_submit_request(&candidate, &risk, &approval, &signing);
    let second = build_single_shot_submit_request(&candidate, &risk, &approval, &signing);

    assert_eq!(
        first.schema_version,
        V20_SUBMIT_REQUEST_BUILDER_SCHEMA_VERSION
    );
    assert_eq!(first.decision, SubmitRequestBuildDecision::Built);
    assert_eq!(first.code, SubmitRequestBuildCode::Built);
    assert!(first.submit_request_built);
    assert!(!first.network_attempted);
    assert!(!first.production_order_submitted);
    assert!(!first.retry_attempted);
    assert!(!first.automatic_remediation_allowed);
    assert!(!first.raw_secret_persisted);
    assert!(!first.raw_signed_payload_persisted);
    assert!(!first.dashboard_order_controls_enabled);
    assert_eq!(first.request_digest, second.request_digest);
    assert_eq!(
        first.request_digest.as_deref(),
        Some(submit_request_digest(&candidate, &risk, &approval, &signing).as_str())
    );

    let json = serde_json::to_string(&first).expect("builder evidence serializes");
    assert!(json.contains("redacted_env_fingerprint_only"));
    assert!(!json.contains("prod-secret-should-never-appear"));
    assert!(!json.contains("prod-key-should-never-appear"));
}

#[test]
fn rejects_missing_risk_allow() {
    let mut risk = risk_allow();
    risk.decision = PreSubmitRiskDecisionKind::Deny;
    risk.code = PreSubmitRiskCode::AccountUnknown;
    risk.submit_builder_entry_allowed = false;

    let evidence =
        build_single_shot_submit_request(&candidate(), &risk, &owner_approval(), &signing_ready());

    assert_eq!(evidence.decision, SubmitRequestBuildDecision::Rejected);
    assert_eq!(evidence.code, SubmitRequestBuildCode::MissingRiskAllow);
    assert_eq!(
        evidence.code.as_str(),
        "v200_submit_request_missing_risk_allow"
    );
    assert!(evidence.redacted_preview.is_none());
}

#[test]
fn rejects_v19_release_provenance_even_when_risk_flag_is_valid() {
    let mut risk = risk_allow();
    risk.release_tag = Some("ntpro-rust-only-v0.19.1".to_string());
    risk.release_gate = Some("v19-release-gates".to_string());
    risk.release_provenance_valid = true;

    let evidence =
        build_single_shot_submit_request(&candidate(), &risk, &owner_approval(), &signing_ready());

    assert_eq!(evidence.decision, SubmitRequestBuildDecision::Rejected);
    assert_eq!(
        evidence.code,
        SubmitRequestBuildCode::MissingReleaseProvenance
    );
    assert_eq!(
        evidence.code.as_str(),
        "v200_submit_request_missing_release_provenance"
    );
    assert!(!evidence.submit_request_built);
    assert!(evidence.redacted_preview.is_none());
}

#[test]
fn rejects_missing_owner_approval() {
    let mut approval = owner_approval();
    approval.state = OwnerApprovalState::Consumed;
    approval.submit_consumption_allowed = false;

    let evidence =
        build_single_shot_submit_request(&candidate(), &risk_allow(), &approval, &signing_ready());

    assert_eq!(evidence.decision, SubmitRequestBuildDecision::Rejected);
    assert_eq!(evidence.code, SubmitRequestBuildCode::MissingOwnerApproval);
}

#[test]
fn rejects_missing_signing_readiness() {
    let mut signing = signing_ready();
    signing.decision = SigningMaterialDecision::Blocked;
    signing.code = SigningMaterialCode::Missing;
    signing.submit_builder_credential_ready = false;

    let evidence =
        build_single_shot_submit_request(&candidate(), &risk_allow(), &owner_approval(), &signing);

    assert_eq!(evidence.decision, SubmitRequestBuildDecision::Rejected);
    assert_eq!(
        evidence.code,
        SubmitRequestBuildCode::MissingSigningReadiness
    );
}

#[test]
fn rejects_candidate_mismatch() {
    let mut candidate = candidate();
    candidate.account_label = "acct-other".to_string();

    let evidence = build_single_shot_submit_request(
        &candidate,
        &risk_allow(),
        &owner_approval(),
        &signing_ready(),
    );

    assert_eq!(evidence.decision, SubmitRequestBuildDecision::Rejected);
    assert_eq!(evidence.code, SubmitRequestBuildCode::CandidateMismatch);
}

#[test]
fn rejects_unsupported_order_shape() {
    let mut candidate = candidate();
    let mut risk = risk_allow();
    candidate.order_type = "market".to_string();
    risk.order_type = Some("market".to_string());

    let evidence =
        build_single_shot_submit_request(&candidate, &risk, &owner_approval(), &signing_ready());

    assert_eq!(evidence.decision, SubmitRequestBuildDecision::Rejected);
    assert_eq!(evidence.code, SubmitRequestBuildCode::UnsupportedOrderShape);
}

fn candidate() -> SingleShotSubmitCandidate {
    SingleShotSubmitCandidate {
        lifecycle_id: "lc-v200-005".to_string(),
        client_order_id: "O-V200-005".to_string(),
        account_label: "acct-prod-001".to_string(),
        instrument_id: "BTCUSDT.BINANCE".to_string(),
        venue: "BINANCE".to_string(),
        side: "buy".to_string(),
        quantity: dec!(0.10),
        price: dec!(50000),
        notional: dec!(5000),
        order_type: "limit".to_string(),
        time_in_force: "gtc".to_string(),
        order_intent_hash: "intent-v200-005".to_string(),
    }
}

fn risk_allow() -> nautilus_risk::v20_pre_submit_gate::PreSubmitRiskGateEvidence {
    evaluate_pre_submit_risk_gate(&risk_request(), &risk_policy(), NOW_NS)
}

fn risk_request() -> PreSubmitRiskRequest {
    PreSubmitRiskRequest {
        gate_id: "risk-gate-v200-005".to_string(),
        lifecycle_id: "lc-v200-005".to_string(),
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
        order_intent_hash: Some("intent-v200-005".to_string()),
        approval: Some(PreSubmitApproval {
            approval_id: "approval-v200-005".to_string(),
            owner_label: "owner-001".to_string(),
            order_intent_hash: "intent-v200-005".to_string(),
            expires_at_unix_ns: NOW_NS + 1_000,
            single_use: true,
            consumed: false,
        }),
        release_provenance: Some(provenance()),
        unrecognized_fields: Vec::new(),
    }
}

fn risk_policy() -> PreSubmitRiskPolicy {
    PreSubmitRiskPolicy {
        allowed_accounts: set(["acct-prod-001"]),
        allowed_instruments: set(["BTCUSDT.BINANCE"]),
        allowed_venues: set(["BINANCE"]),
        allowed_sides: set(["buy", "sell"]),
        allowed_order_types: set(["limit", "market"]),
        allowed_time_in_force: set(["gtc"]),
        expected_environment: "production".to_string(),
        required_release_tag: V20_REQUIRED_RELEASE_TAG.to_string(),
        required_release_gate: V20_REQUIRED_RELEASE_GATE.to_string(),
        max_quantity: dec!(0.25),
        max_price: dec!(100000),
        max_notional: dec!(10000),
    }
}

fn owner_approval() -> nautilus_risk::v20_owner_approval::OwnerApprovalEvidence {
    evaluate_owner_approval(&owner_record(), &owner_candidate(), NOW_NS)
}

fn owner_record() -> OwnerApprovalRecord {
    OwnerApprovalRecord {
        approval_id: "approval-v200-005".to_string(),
        request: owner_request(),
        decision: OwnerApprovalDecision::Approved,
        decided_at_unix_ns: NOW_NS - 10,
        revoked_at_unix_ns: None,
        consumed_at_unix_ns: None,
    }
}

fn owner_request() -> OwnerApprovalRequest {
    let scope = owner_scope();
    let release_provenance = provenance();
    let nonce = "nonce-v200-005";
    let environment = "production";
    let approval_digest = owner_approval_digest(&scope, nonce, environment, &release_provenance);
    OwnerApprovalRequest {
        request_id: "request-v200-005".to_string(),
        lifecycle_id: "lc-v200-005".to_string(),
        owner_label: "owner-001".to_string(),
        scope,
        nonce: nonce.to_string(),
        environment: environment.to_string(),
        release_provenance,
        approval_digest,
        expires_at_unix_ns: NOW_NS + 1_000,
    }
}

fn owner_candidate() -> OwnerApprovalCandidate {
    let scope = owner_scope();
    let release_provenance = provenance();
    let approval_digest =
        owner_approval_digest(&scope, "nonce-v200-005", "production", &release_provenance);
    OwnerApprovalCandidate {
        lifecycle_id: "lc-v200-005".to_string(),
        scope,
        environment: "production".to_string(),
        release_provenance,
        approval_digest,
    }
}

fn owner_scope() -> OwnerApprovalScope {
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
        order_intent_hash: "intent-v200-005".to_string(),
    }
}

fn signing_ready() -> nautilus_risk::v20_signing_material_gate::SigningMaterialGateEvidence {
    evaluate_signing_material_env_gate(&signing_policy(), &signing_snapshot())
}

fn signing_policy() -> SigningMaterialPolicy {
    SigningMaterialPolicy {
        gate_id: "signing-gate-v200-005".to_string(),
        lifecycle_id: "lc-v200-005".to_string(),
        expected_environment: "production".to_string(),
        requirements: vec![
            SigningMaterialRequirement {
                env_var: "NTPRO_BINANCE_API_KEY".to_string(),
                material_kind: "api_key".to_string(),
            },
            SigningMaterialRequirement {
                env_var: "NTPRO_BINANCE_API_SECRET".to_string(),
                material_kind: "api_secret".to_string(),
            },
        ],
    }
}

fn signing_snapshot() -> SigningMaterialEnvSnapshot {
    SigningMaterialEnvSnapshot {
        environment: "production".to_string(),
        values: BTreeMap::from([
            (
                "NTPRO_BINANCE_API_KEY".to_string(),
                "prod-key-should-never-appear".to_string(),
            ),
            (
                "NTPRO_BINANCE_API_SECRET".to_string(),
                "prod-secret-should-never-appear".to_string(),
            ),
        ]),
        sources: BTreeMap::from([
            (
                "NTPRO_BINANCE_API_KEY".to_string(),
                SigningMaterialSource::Env,
            ),
            (
                "NTPRO_BINANCE_API_SECRET".to_string(),
                SigningMaterialSource::Env,
            ),
        ]),
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

fn set<const N: usize>(values: [&str; N]) -> BTreeSet<String> {
    values.into_iter().map(str::to_string).collect()
}
