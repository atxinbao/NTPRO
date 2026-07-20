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

//! Top-level `live` command dispatch.
//!
//! This module owns only CLI enum dispatch. Validation, artifact construction,
//! runtime behavior, and production capability boundaries remain in the parent
//! `live` module.

use crate::opt::{LiveCommand, LiveOpt};

use super::*;

pub(crate) async fn run_live_command(opt: LiveOpt) -> anyhow::Result<()> {
    match opt.command {
        LiveCommand::Validate(validate) => run_live_validate(&validate),
        LiveCommand::Run(run) => run_live_run(&run).await,
        LiveCommand::TestnetOrderGate(gate) => run_live_testnet_order_gate(&gate),
        LiveCommand::TestnetOrderPreflight(preflight) => {
            run_live_testnet_order_preflight(&preflight)
        }
        LiveCommand::TestnetOrderRequestPreview(preview) => {
            run_live_testnet_order_request_preview(&preview)
        }
        LiveCommand::TestnetOrderTestPreflight(preflight) => {
            run_live_testnet_order_test_preflight(&preflight)
        }
        LiveCommand::TestnetExecutionArtifactContract(contract) => {
            run_live_testnet_execution_artifact_contract(&contract)
        }
        LiveCommand::TestnetReconciliationFixture(fixture) => {
            run_live_testnet_reconciliation_fixture(&fixture)
        }
        LiveCommand::ProductionPublicReadProbe(probe) => {
            run_live_production_public_read_probe(&probe)
        }
        LiveCommand::ProductionAccountSnapshotContract(contract) => {
            run_live_production_account_snapshot_contract(&contract)
        }
        LiveCommand::ProductionOrderStateReadOnlyProof(proof) => {
            run_live_production_order_state_readonly_proof(&proof)
        }
        LiveCommand::ProductionLiveAlphaDryRunOrderGate(gate) => {
            run_live_production_live_alpha_dry_run_order_gate(&gate)
        }
        LiveCommand::ProductionLiveAlphaOrderRequestPreview(preview) => {
            run_live_production_live_alpha_order_request_preview(&preview)
        }
        LiveCommand::ProductionLiveAlphaManualApprovalLifecycle(approval) => {
            run_live_production_live_alpha_manual_approval_lifecycle(&approval)
        }
        LiveCommand::ProductionLiveAlphaExecutionDryRun(dry_run) => {
            run_live_production_live_alpha_execution_dry_run(&dry_run)
        }
        LiveCommand::ProductionLiveAlphaKillSwitchRuntimeGate(gate) => {
            run_live_production_live_alpha_kill_switch_runtime_gate(&gate)
        }
        LiveCommand::ProductionMutationRuntimeGate(gate) => {
            run_live_production_mutation_runtime_gate(&gate)
        }
        LiveCommand::ProductionMutationSigningApproval(approval) => {
            run_live_production_mutation_signing_approval(&approval)
        }
        LiveCommand::ProductionMutationRequestBuilder(builder) => {
            run_live_production_mutation_request_builder(&builder)
        }
        LiveCommand::ProductionMutationGuardedSend(send) => {
            run_live_production_mutation_guarded_send(&send)
        }
        LiveCommand::ProductionMutationResponseRedaction(redaction) => {
            run_live_production_mutation_response_redaction(&redaction)
        }
        LiveCommand::ProductionMutationOrderStateReadback(readback) => {
            run_live_production_mutation_order_state_readback(&readback)
        }
        LiveCommand::ProductionMutationAuditTrail(audit) => {
            run_live_production_mutation_audit_trail(&audit)
        }
        LiveCommand::ProductionMutationFailureSemantics(failure) => {
            run_live_production_mutation_failure_semantics(&failure)
        }
        LiveCommand::ProductionMutationLocalOrderLedger(ledger) => {
            run_live_production_mutation_local_order_ledger(&ledger)
        }
        LiveCommand::ProductionMutationExchangeReadbackMapper(mapper) => {
            run_live_production_mutation_exchange_readback_mapper(&mapper)
        }
        LiveCommand::ProductionMutationReconciliationClassifier(classifier) => {
            run_live_production_mutation_reconciliation_classifier(&classifier)
        }
        LiveCommand::ProductionMutationOrphanOrderDetector(detector) => {
            run_live_production_mutation_orphan_order_detector(&detector)
        }
        LiveCommand::ProductionMutationCancelRequestPreview(preview) => {
            run_live_production_mutation_cancel_request_preview(&preview)
        }
        LiveCommand::ProductionMutationCancelRiskGate(gate) => {
            run_live_production_mutation_cancel_risk_gate(&gate)
        }
        LiveCommand::ProductionMutationManualOwnerApprovalLifecycle(approval) => {
            run_live_production_mutation_manual_owner_approval_lifecycle(&approval)
        }
        LiveCommand::ProductionMutationActualCancelOwnerApprovalLifecycle(approval) => {
            run_live_production_mutation_actual_cancel_owner_approval_lifecycle(&approval)
        }
        LiveCommand::ProductionMutationActualCancelExecutorAdapterBoundary(boundary) => {
            run_live_production_mutation_actual_cancel_executor_adapter_boundary(&boundary)
        }
        LiveCommand::ProductionMutationActualCancelSingleShot(cancel) => {
            run_live_production_mutation_actual_cancel_single_shot(&cancel)
        }
        LiveCommand::ProductionMutationActualCancelReadbackReconciliation(reconciliation) => {
            run_live_production_mutation_actual_cancel_readback_reconciliation(&reconciliation)
        }
        LiveCommand::ProductionMutationActualCancelFailureEvidence(evidence) => {
            run_live_production_mutation_actual_cancel_failure_evidence(&evidence)
        }
        LiveCommand::ProductionMutationCancelResponseRedaction(redaction) => {
            run_live_production_mutation_cancel_response_redaction(&redaction)
        }
        LiveCommand::ProductionMutationPostCancelReadback(readback) => {
            run_live_production_mutation_post_cancel_readback(&readback)
        }
        LiveCommand::ProductionMutationCancelRecoveryIncidentAuditCloseout(closeout) => {
            run_live_production_mutation_cancel_recovery_incident_audit_closeout(&closeout)
        }
        LiveCommand::ProductionLiveAlphaRiskPreflight(preflight) => {
            run_live_production_live_alpha_risk_preflight(&preflight)
        }
        LiveCommand::ProductionShadowPortfolioRuntime(runtime) => {
            run_live_production_shadow_portfolio_runtime(&runtime)
        }
        LiveCommand::ProductionShadowStrategySession(session) => {
            run_live_production_shadow_strategy_session(&session)
        }
        LiveCommand::ProductionShadowPreflightSession(session) => {
            run_live_production_shadow_preflight_session(&session).await
        }
        LiveCommand::ProductionKillSwitchApprovalArtifact(artifact) => {
            run_live_production_kill_switch_approval_artifact(&artifact)
        }
        LiveCommand::ProductionReadonlyReconciliation(reconciliation) => {
            run_live_production_readonly_reconciliation(&reconciliation)
        }
    }
}
