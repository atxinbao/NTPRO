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
    v20_submit_candidate::{
        GuardedSubmitCandidateCode, GuardedSubmitCandidateRequest, GuardedSubmitCandidateState,
        GuardedSubmitMode, SubmitAttemptLedgerEntry, SubmitAttemptLedgerSnapshot,
        V20_GUARDED_SUBMIT_CANDIDATE_SCHEMA_VERSION, V20_SUBMIT_ATTEMPT_LEDGER_SCHEMA_VERSION,
        evaluate_guarded_single_shot_submit_candidate,
    },
    v20_submit_request_builder::{
        SingleShotSubmitCandidate, SubmitRequestBuilderEvidence, build_single_shot_submit_request,
        submit_request_digest,
    },
};
use rust_decimal_macros::dec;

const NOW_NS: u64 = 1_780_000_000_000_000_000;
const V20_RELEASE_COMMIT: &str = "d29a764a2fb6b3f9c187d2af17337b08b40d794b";

#[test]
fn preview_records_ready_evidence_without_consuming_approval() {
    let risk = risk_allow();
    let approval = owner_approval();
    let signing = signing_ready();
    let builder = builder_evidence(&risk, &approval, &signing);
    let request = guarded_request(GuardedSubmitMode::Preview, false, Some(empty_ledger()));

    let evidence = evaluate_guarded_single_shot_submit_candidate(
        &request, &risk, &approval, &signing, &builder, NOW_NS,
    );

    assert_eq!(
        evidence.schema_version,
        V20_GUARDED_SUBMIT_CANDIDATE_SCHEMA_VERSION
    );
    assert_eq!(evidence.state, GuardedSubmitCandidateState::Preview);
    assert_eq!(evidence.code, GuardedSubmitCandidateCode::PreviewReady);
    assert!(evidence.submit_attempt_evidence_ready);
    assert!(evidence.preview_evidence_ready);
    assert!(!evidence.production_submit_attempted);
    assert!(!evidence.adapter_submit_handoff_allowed);
    assert_eq!(
        evidence.owner_approval_state_after_attempt,
        OwnerApprovalState::Approved
    );
    assert!(!evidence.owner_approval_consumed);
    assert!(!evidence.retry_attempted);
    assert!(!evidence.replace_attempted);
    assert!(!evidence.amend_attempted);
    assert!(!evidence.flatten_attempted);
    assert!(!evidence.bulk_submit_attempted);
    assert!(!evidence.dashboard_order_controls_enabled);
}

#[test]
fn dry_run_records_candidate_without_submit_side_effects() {
    let risk = risk_allow();
    let approval = owner_approval();
    let signing = signing_ready();
    let builder = builder_evidence(&risk, &approval, &signing);
    let request = guarded_request(GuardedSubmitMode::DryRun, false, Some(empty_ledger()));

    let evidence = evaluate_guarded_single_shot_submit_candidate(
        &request, &risk, &approval, &signing, &builder, NOW_NS,
    );

    assert_eq!(evidence.state, GuardedSubmitCandidateState::DryRun);
    assert_eq!(evidence.code, GuardedSubmitCandidateCode::DryRunReady);
    assert!(evidence.dry_run_evidence_ready);
    assert!(!evidence.production_submit_attempted);
    assert!(!evidence.owner_approval_consumed);
    assert!(!evidence.raw_secret_persisted);
    assert!(!evidence.raw_signed_payload_persisted);
    assert!(!evidence.raw_exchange_response_persisted);
}

#[test]
fn submit_consumes_approval_and_records_single_attempt() {
    let risk = risk_allow();
    let approval = owner_approval();
    let signing = signing_ready();
    let builder = builder_evidence(&risk, &approval, &signing);
    let request = guarded_request(GuardedSubmitMode::Submit, true, Some(empty_ledger()));

    let evidence = evaluate_guarded_single_shot_submit_candidate(
        &request, &risk, &approval, &signing, &builder, NOW_NS,
    );

    assert_eq!(evidence.state, GuardedSubmitCandidateState::Submitted);
    assert_eq!(evidence.code, GuardedSubmitCandidateCode::Submitted);
    assert_eq!(evidence.code.as_str(), "v200_guarded_submit_submitted");
    assert!(evidence.submit_attempt_evidence_ready);
    assert!(evidence.production_submit_attempted);
    assert!(evidence.adapter_submit_handoff_allowed);
    assert!(evidence.readback_required);
    assert!(evidence.audit_artifact_required);
    assert_eq!(
        evidence.owner_approval_state_before_attempt,
        OwnerApprovalState::Approved
    );
    assert_eq!(
        evidence.owner_approval_state_after_attempt,
        OwnerApprovalState::Consumed
    );
    assert_eq!(evidence.approval_consumed_at_unix_ns, Some(NOW_NS));
    assert!(evidence.owner_approval_consumed);
    assert_eq!(evidence.previous_attempt_count, 0);
    assert!(evidence.attempt_ledger_required);
    assert!(evidence.attempt_ledger_trusted);
    assert_eq!(
        evidence.attempt_ledger_key.as_deref(),
        Some("ledger-v200-006")
    );
    assert!(evidence.atomic_approval_consumption_required);
    assert!(evidence.atomic_approval_consumption_recorded);
    assert!(evidence.single_attempt_required);
    assert!(evidence.manual_online_gate_present);
    assert!(!evidence.retry_attempted);
    assert!(!evidence.automatic_remediation_allowed);
}

#[test]
fn blocks_missing_manual_gate_for_real_submit() {
    let risk = risk_allow();
    let approval = owner_approval();
    let signing = signing_ready();
    let builder = builder_evidence(&risk, &approval, &signing);
    let request = guarded_request(GuardedSubmitMode::Submit, false, Some(empty_ledger()));

    let evidence = evaluate_guarded_single_shot_submit_candidate(
        &request, &risk, &approval, &signing, &builder, NOW_NS,
    );

    assert_eq!(evidence.state, GuardedSubmitCandidateState::Blocked);
    assert_eq!(evidence.code, GuardedSubmitCandidateCode::ManualGateMissing);
    assert!(!evidence.production_submit_attempted);
    assert!(!evidence.owner_approval_consumed);
}

#[test]
fn blocks_missing_risk_allow() {
    let mut risk = risk_allow();
    risk.decision = PreSubmitRiskDecisionKind::Deny;
    risk.code = PreSubmitRiskCode::AccountUnknown;
    risk.production_order_submission_allowed = false;
    risk.submit_builder_entry_allowed = false;
    let approval = owner_approval();
    let signing = signing_ready();
    let builder = builder_evidence(&risk_allow(), &approval, &signing);
    let request = guarded_request(GuardedSubmitMode::Submit, true, Some(empty_ledger()));

    let evidence = evaluate_guarded_single_shot_submit_candidate(
        &request, &risk, &approval, &signing, &builder, NOW_NS,
    );

    assert_eq!(evidence.state, GuardedSubmitCandidateState::Blocked);
    assert_eq!(evidence.code, GuardedSubmitCandidateCode::MissingRiskAllow);
}

#[test]
fn blocks_missing_owner_approval_after_consumption() {
    let risk = risk_allow();
    let mut approval = owner_approval();
    approval.state = OwnerApprovalState::Consumed;
    approval.consumed = true;
    approval.submit_consumption_allowed = false;
    let signing = signing_ready();
    let builder = builder_evidence(&risk, &owner_approval(), &signing);
    let request = guarded_request(GuardedSubmitMode::Submit, true, Some(empty_ledger()));

    let evidence = evaluate_guarded_single_shot_submit_candidate(
        &request, &risk, &approval, &signing, &builder, NOW_NS,
    );

    assert_eq!(evidence.state, GuardedSubmitCandidateState::Blocked);
    assert_eq!(
        evidence.code,
        GuardedSubmitCandidateCode::MissingOwnerApproval
    );
    assert!(!evidence.production_submit_attempted);
}

#[test]
fn blocks_duplicate_submit_digest() {
    let risk = risk_allow();
    let approval = owner_approval();
    let signing = signing_ready();
    let builder = builder_evidence(&risk, &approval, &signing);
    let digest = builder
        .request_digest
        .as_ref()
        .expect("builder should emit digest")
        .clone();
    let request = guarded_request(
        GuardedSubmitMode::Submit,
        true,
        Some(ledger_with_entry(
            "attempt-v200-005",
            &digest,
            "approval-v200-005",
            true,
        )),
    );

    let evidence = evaluate_guarded_single_shot_submit_candidate(
        &request, &risk, &approval, &signing, &builder, NOW_NS,
    );

    assert_eq!(evidence.state, GuardedSubmitCandidateState::Blocked);
    assert_eq!(
        evidence.code,
        GuardedSubmitCandidateCode::DuplicateSubmitRejected
    );
    assert_eq!(evidence.previous_attempt_count, 1);
    assert!(!evidence.owner_approval_consumed);
}

#[test]
fn blocks_request_digest_mismatch() {
    let risk = risk_allow();
    let approval = owner_approval();
    let signing = signing_ready();
    let builder = builder_evidence(&risk, &approval, &signing);
    let mut request = guarded_request(GuardedSubmitMode::Submit, true, Some(empty_ledger()));
    request.expected_request_digest = Some("unexpected-digest".to_string());

    let evidence = evaluate_guarded_single_shot_submit_candidate(
        &request, &risk, &approval, &signing, &builder, NOW_NS,
    );

    assert_eq!(evidence.state, GuardedSubmitCandidateState::Blocked);
    assert_eq!(
        evidence.code,
        GuardedSubmitCandidateCode::RequestDigestMismatch
    );
    assert!(!evidence.production_submit_attempted);
}

#[test]
fn blocks_missing_signing_readiness() {
    let risk = risk_allow();
    let approval = owner_approval();
    let mut signing = signing_ready();
    signing.decision = SigningMaterialDecision::Blocked;
    signing.code = SigningMaterialCode::Missing;
    signing.submit_builder_credential_ready = false;
    let builder = builder_evidence(&risk, &approval, &signing_ready());
    let request = guarded_request(GuardedSubmitMode::Submit, true, Some(empty_ledger()));

    let evidence = evaluate_guarded_single_shot_submit_candidate(
        &request, &risk, &approval, &signing, &builder, NOW_NS,
    );

    assert_eq!(evidence.state, GuardedSubmitCandidateState::Blocked);
    assert_eq!(
        evidence.code,
        GuardedSubmitCandidateCode::MissingSigningReadiness
    );
}

#[test]
fn blocks_v19_release_provenance_even_when_risk_flag_is_valid() {
    let mut risk = risk_allow();
    risk.release_tag = Some("ntpro-rust-only-v0.19.1".to_string());
    risk.release_gate = Some("v19-release-gates".to_string());
    risk.release_provenance_valid = true;
    let approval = owner_approval();
    let signing = signing_ready();
    let builder = builder_evidence(&risk_allow(), &approval, &signing);
    let request = guarded_request(GuardedSubmitMode::Submit, true, Some(empty_ledger()));

    let evidence = evaluate_guarded_single_shot_submit_candidate(
        &request, &risk, &approval, &signing, &builder, NOW_NS,
    );

    assert_eq!(evidence.state, GuardedSubmitCandidateState::Blocked);
    assert_eq!(
        evidence.code,
        GuardedSubmitCandidateCode::MissingReleaseProvenance
    );
    assert_eq!(
        evidence.release_tag.as_deref(),
        Some("ntpro-rust-only-v0.19.1")
    );
    assert_eq!(evidence.release_gate.as_deref(), Some("v19-release-gates"));
    assert!(!evidence.production_submit_attempted);
    assert!(!evidence.adapter_submit_handoff_allowed);
}

#[test]
fn blocks_missing_durable_attempt_ledger() {
    let risk = risk_allow();
    let approval = owner_approval();
    let signing = signing_ready();
    let builder = builder_evidence(&risk, &approval, &signing);
    let request = guarded_request(GuardedSubmitMode::Submit, true, None);

    let evidence = evaluate_guarded_single_shot_submit_candidate(
        &request, &risk, &approval, &signing, &builder, NOW_NS,
    );

    assert_eq!(evidence.state, GuardedSubmitCandidateState::Blocked);
    assert_eq!(
        evidence.code,
        GuardedSubmitCandidateCode::AttemptLedgerMissing
    );
    assert!(!evidence.attempt_ledger_trusted);
    assert!(!evidence.atomic_approval_consumption_recorded);
}

#[test]
fn blocks_stale_attempt_ledger() {
    let risk = risk_allow();
    let approval = owner_approval();
    let signing = signing_ready();
    let builder = builder_evidence(&risk, &approval, &signing);
    let mut ledger = empty_ledger();
    ledger.stale = true;
    let request = guarded_request(GuardedSubmitMode::Submit, true, Some(ledger));

    let evidence = evaluate_guarded_single_shot_submit_candidate(
        &request, &risk, &approval, &signing, &builder, NOW_NS,
    );

    assert_eq!(evidence.state, GuardedSubmitCandidateState::Blocked);
    assert_eq!(
        evidence.code,
        GuardedSubmitCandidateCode::AttemptLedgerUntrusted
    );
    assert!(!evidence.production_submit_attempted);
}

#[test]
fn blocks_attempt_ledger_lineage_mismatch() {
    let risk = risk_allow();
    let approval = owner_approval();
    let signing = signing_ready();
    let builder = builder_evidence(&risk, &approval, &signing);
    let mut ledger = empty_ledger();
    ledger.lifecycle_id = "lc-v200-other".to_string();
    let request = guarded_request(GuardedSubmitMode::Submit, true, Some(ledger));

    let evidence = evaluate_guarded_single_shot_submit_candidate(
        &request, &risk, &approval, &signing, &builder, NOW_NS,
    );

    assert_eq!(evidence.state, GuardedSubmitCandidateState::Blocked);
    assert_eq!(
        evidence.code,
        GuardedSubmitCandidateCode::AttemptLedgerLineageMismatch
    );
}

#[test]
fn blocks_attempt_ledger_provenance_mismatch() {
    let risk = risk_allow();
    let approval = owner_approval();
    let signing = signing_ready();
    let builder = builder_evidence(&risk, &approval, &signing);
    let mut ledger = empty_ledger();
    ledger.release_gate = "v19-release-gates".to_string();
    let request = guarded_request(GuardedSubmitMode::Submit, true, Some(ledger));

    let evidence = evaluate_guarded_single_shot_submit_candidate(
        &request, &risk, &approval, &signing, &builder, NOW_NS,
    );

    assert_eq!(evidence.state, GuardedSubmitCandidateState::Blocked);
    assert_eq!(
        evidence.code,
        GuardedSubmitCandidateCode::AttemptLedgerProvenanceMismatch
    );
}

#[test]
fn blocks_approval_already_consumed_by_attempt_ledger() {
    let risk = risk_allow();
    let approval = owner_approval();
    let signing = signing_ready();
    let builder = builder_evidence(&risk, &approval, &signing);
    let request = guarded_request(
        GuardedSubmitMode::Submit,
        true,
        Some(ledger_with_entry(
            "attempt-v200-005",
            "request-digest-v200-other",
            "approval-v200-006",
            true,
        )),
    );

    let evidence = evaluate_guarded_single_shot_submit_candidate(
        &request, &risk, &approval, &signing, &builder, NOW_NS,
    );

    assert_eq!(evidence.state, GuardedSubmitCandidateState::Blocked);
    assert_eq!(
        evidence.code,
        GuardedSubmitCandidateCode::ApprovalAlreadyConsumed
    );
    assert!(!evidence.owner_approval_consumed);
}

fn guarded_request(
    mode: GuardedSubmitMode,
    manual_online_gate: bool,
    attempt_ledger: Option<SubmitAttemptLedgerSnapshot>,
) -> GuardedSubmitCandidateRequest {
    GuardedSubmitCandidateRequest {
        candidate_id: "candidate-v200-006".to_string(),
        attempt_id: "attempt-v200-006".to_string(),
        lifecycle_id: "lc-v200-006".to_string(),
        mode,
        manual_online_gate,
        expected_request_digest: Some(expected_request_digest()),
        attempt_ledger,
    }
}

fn empty_ledger() -> SubmitAttemptLedgerSnapshot {
    SubmitAttemptLedgerSnapshot {
        schema_version: V20_SUBMIT_ATTEMPT_LEDGER_SCHEMA_VERSION.to_string(),
        ledger_key: "ledger-v200-006".to_string(),
        lifecycle_id: "lc-v200-006".to_string(),
        release_tag: V20_REQUIRED_RELEASE_TAG.to_string(),
        release_gate: V20_REQUIRED_RELEASE_GATE.to_string(),
        trusted: true,
        stale: false,
        entries: Vec::new(),
    }
}

fn ledger_with_entry(
    attempt_id: &str,
    request_digest: &str,
    approval_id: &str,
    approval_consumed: bool,
) -> SubmitAttemptLedgerSnapshot {
    let mut ledger = empty_ledger();
    ledger.entries.push(SubmitAttemptLedgerEntry {
        attempt_id: attempt_id.to_string(),
        lifecycle_id: "lc-v200-006".to_string(),
        request_digest: request_digest.to_string(),
        approval_id: approval_id.to_string(),
        approval_consumed,
        consumed_at_unix_ns: approval_consumed.then_some(NOW_NS - 1),
    });
    ledger
}

fn expected_request_digest() -> String {
    let candidate = candidate();
    let risk = risk_allow();
    let approval = owner_approval();
    let signing = signing_ready();
    submit_request_digest(&candidate, &risk, &approval, &signing)
}

fn builder_evidence(
    risk: &nautilus_risk::v20_pre_submit_gate::PreSubmitRiskGateEvidence,
    approval: &nautilus_risk::v20_owner_approval::OwnerApprovalEvidence,
    signing: &nautilus_risk::v20_signing_material_gate::SigningMaterialGateEvidence,
) -> SubmitRequestBuilderEvidence {
    build_single_shot_submit_request(&candidate(), risk, approval, signing)
}

fn candidate() -> SingleShotSubmitCandidate {
    SingleShotSubmitCandidate {
        lifecycle_id: "lc-v200-006".to_string(),
        client_order_id: "O-V200-006".to_string(),
        account_label: "acct-prod-001".to_string(),
        instrument_id: "BTCUSDT.BINANCE".to_string(),
        venue: "BINANCE".to_string(),
        side: "buy".to_string(),
        quantity: dec!(0.10),
        price: dec!(50000),
        notional: dec!(5000),
        order_type: "limit".to_string(),
        time_in_force: "gtc".to_string(),
        order_intent_hash: "intent-v200-006".to_string(),
    }
}

fn risk_allow() -> nautilus_risk::v20_pre_submit_gate::PreSubmitRiskGateEvidence {
    evaluate_pre_submit_risk_gate(&risk_request(), &risk_policy(), NOW_NS)
}

fn risk_request() -> PreSubmitRiskRequest {
    PreSubmitRiskRequest {
        gate_id: "risk-gate-v200-006".to_string(),
        lifecycle_id: "lc-v200-006".to_string(),
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
        order_intent_hash: Some("intent-v200-006".to_string()),
        approval: Some(PreSubmitApproval {
            approval_id: "approval-v200-006".to_string(),
            owner_label: "owner-001".to_string(),
            order_intent_hash: "intent-v200-006".to_string(),
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
        approval_id: "approval-v200-006".to_string(),
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
    let nonce = "nonce-v200-006";
    let environment = "production";
    let approval_digest = owner_approval_digest(&scope, nonce, environment, &release_provenance);
    OwnerApprovalRequest {
        request_id: "request-v200-006".to_string(),
        lifecycle_id: "lc-v200-006".to_string(),
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
        owner_approval_digest(&scope, "nonce-v200-006", "production", &release_provenance);
    OwnerApprovalCandidate {
        lifecycle_id: "lc-v200-006".to_string(),
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
        order_intent_hash: "intent-v200-006".to_string(),
    }
}

fn signing_ready() -> nautilus_risk::v20_signing_material_gate::SigningMaterialGateEvidence {
    evaluate_signing_material_env_gate(&signing_policy(), &signing_snapshot())
}

fn signing_policy() -> SigningMaterialPolicy {
    SigningMaterialPolicy {
        gate_id: "signing-gate-v200-006".to_string(),
        lifecycle_id: "lc-v200-006".to_string(),
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
