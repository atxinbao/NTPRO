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

//! 单节点 MVP 的身份与追溯合同。

use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, ensure};
use nautilus_live::status::{LifecycleStatus, SnapshotAvailability, SnapshotValue};
use serde::{Deserialize, Serialize};

use crate::supervisor::{
    NodeMetrics, RegistryArtifactState, SupervisorNodeRecord, SupervisorProcessState,
};

pub(crate) const MVP_IDENTITY_CONTRACT_SCHEMA_VERSION: &str = "ntpro.mvp_identity_contract.v1";
pub(crate) const MVP_IDENTITY_CONTRACT_PATH: &str = "mvp/identity_contract.json";
pub(crate) const MVP_STATUS_CONTRACT_SCHEMA_VERSION: &str = "ntpro.mvp_status_contract.v1";
pub(crate) const MVP_STATUS_CONTRACT_PATH: &str = "mvp/status_contract.json";
const UNIFIED_READ_MODEL_RELATIVE_PATH: &str = "v0_21/unified_read_model_snapshot.json";
const SANDBOX_ENVIRONMENT: &str = "sandbox";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MvpIdentityContract {
    pub schema_version: String,
    pub contract_id: String,
    pub identities: MvpIdentitySet,
    pub provenance: MvpIdentityProvenance,
    pub boundaries: MvpIdentityBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MvpIdentitySet {
    pub strategy_id: String,
    pub strategy_version: String,
    pub backtest_run_id: String,
    pub backtest_result_ref: String,
    pub node_id: String,
    pub strategy_instance_id: String,
    pub account_id: String,
    pub venue_id: String,
    pub environment: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MvpIdentityProvenance {
    pub config_path: String,
    pub generated_at_unix_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MvpIdentityBoundaries {
    pub read_only_product_contract: bool,
    pub external_venue_connection: bool,
    pub order_submission_allowed: bool,
    pub order_mutation_allowed: bool,
    pub automatic_retry_allowed: bool,
    pub automatic_remediation_allowed: bool,
    pub real_orders_submitted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MvpStatusContract {
    pub schema_version: String,
    pub identity_contract_id: String,
    pub research: MvpStatusAxis<MvpResearchStatus>,
    pub runtime: MvpStatusAxis<MvpRuntimeStatus>,
    pub technical_health: MvpStatusAxis<MvpTechnicalHealth>,
    pub trading_readiness: MvpStatusAxis<MvpTradingReadiness>,
    pub provenance: MvpStatusProvenance,
    pub boundaries: MvpStatusBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MvpStatusAxis<T> {
    pub status: T,
    pub availability: MvpStatusAvailability,
    pub freshness: MvpStatusFreshness,
    pub source_refs: Vec<String>,
    pub observed_at_unix_ms: u64,
    pub reasons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MvpStatusAvailability {
    Available,
    Missing,
    Error,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MvpStatusFreshness {
    Fresh,
    Stale,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MvpResearchStatus {
    ReferenceBound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MvpRuntimeStatus {
    Running,
    Stopped,
    Transitioning,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MvpTechnicalHealth {
    Healthy,
    Degraded,
    Unhealthy,
    NotRunning,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MvpTradingReadiness {
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MvpStatusProvenance {
    pub identity_contract_path: String,
    pub identity_contract_available: bool,
    pub supervisor_registry_path: String,
    pub node_status_path: String,
    pub node_metrics_path: String,
    pub unified_read_model_path: String,
    pub freshness_max_age_ms: u64,
    pub generated_at_unix_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MvpStatusBoundaries {
    pub read_only_product_contract: bool,
    pub http_success_implies_technical_health: bool,
    pub process_alive_implies_technical_health: bool,
    pub backtest_reference_implies_research_accepted: bool,
    pub backtest_complete_implies_trading_readiness: bool,
    pub external_venue_connection: bool,
    pub order_submission_allowed: bool,
    pub order_mutation_allowed: bool,
    pub automatic_retry_allowed: bool,
    pub automatic_remediation_allowed: bool,
    pub real_orders_submitted: bool,
}

#[derive(Debug, Deserialize)]
struct IdentityConfigProjection {
    node: IdentityNodeSection,
    strategy: IdentityStrategySection,
    market: IdentityVenueSection,
    execution: IdentityVenueSection,
    mvp: IdentityMvpSection,
}

#[derive(Debug, Deserialize)]
struct IdentityNodeSection {
    node_id: String,
}

#[derive(Debug, Deserialize)]
struct IdentityStrategySection {
    strategy_id: String,
}

#[derive(Debug, Deserialize)]
struct IdentityVenueSection {
    venue: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentityMvpSection {
    strategy_version: String,
    backtest_run_id: String,
    backtest_result_ref: String,
    account_id: String,
    environment: String,
}

impl MvpIdentityContract {
    pub(crate) fn load(config_path: &Path, supervisor_node_id: &str) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(config_path)
            .with_context(|| format!("读取 MVP 身份配置 '{}' 失败", config_path.display()))?;
        let config: IdentityConfigProjection = toml::from_str(&raw).with_context(|| {
            format!(
                "解析 MVP 身份配置 '{}' 失败；mvp serve 要求显式 [mvp] 身份段",
                config_path.display()
            )
        })?;

        let node_id = required("node_id", supervisor_node_id)?;
        let strategy_instance_id = required("node.node_id", &config.node.node_id)?;
        ensure!(
            node_id != strategy_instance_id,
            "Supervisor node_id 与 strategy_instance_id 必须使用不同身份"
        );

        let market_venue = required("market.venue", &config.market.venue)?;
        let execution_venue = required("execution.venue", &config.execution.venue)?;
        ensure!(
            market_venue == execution_venue,
            "market.venue '{market_venue}' 与 execution.venue '{execution_venue}' 不一致"
        );

        let environment = required("mvp.environment", &config.mvp.environment)?;
        ensure!(
            environment == SANDBOX_ENVIRONMENT,
            "mvp.environment 必须为 sandbox，实际为 '{environment}'"
        );

        let identities = MvpIdentitySet {
            strategy_id: required("strategy.strategy_id", &config.strategy.strategy_id)?,
            strategy_version: required("mvp.strategy_version", &config.mvp.strategy_version)?,
            backtest_run_id: required("mvp.backtest_run_id", &config.mvp.backtest_run_id)?,
            backtest_result_ref: required(
                "mvp.backtest_result_ref",
                &config.mvp.backtest_result_ref,
            )?,
            node_id,
            strategy_instance_id,
            account_id: required("mvp.account_id", &config.mvp.account_id)?,
            venue_id: market_venue,
            environment,
        };
        let contract_id = format!(
            "{}:{}:{}",
            identities.node_id, identities.strategy_id, identities.strategy_instance_id
        );

        Ok(Self {
            schema_version: MVP_IDENTITY_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id,
            identities,
            provenance: MvpIdentityProvenance {
                config_path: config_path.display().to_string(),
                generated_at_unix_ms: unix_time_ms(),
            },
            boundaries: MvpIdentityBoundaries {
                read_only_product_contract: true,
                external_venue_connection: false,
                order_submission_allowed: false,
                order_mutation_allowed: false,
                automatic_retry_allowed: false,
                automatic_remediation_allowed: false,
                real_orders_submitted: false,
            },
        })
    }
}

impl MvpStatusContract {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_runtime(
        identity: &MvpIdentityContract,
        identity_contract_path: &Path,
        registry_path: &Path,
        record: &SupervisorNodeRecord,
        metrics: Option<&NodeMetrics>,
        status_error: Option<&str>,
        metrics_error: Option<&str>,
        identity_error: Option<&str>,
        freshness_max_age_ms: u64,
    ) -> Self {
        let observed_at_unix_ms = unix_time_ms();
        let unified_read_model_path = record.artifact_root.join(UNIFIED_READ_MODEL_RELATIVE_PATH);
        let status_freshness = artifact_timestamp_assessment(
            "node_status",
            &record.last_known_status.generated_at,
            observed_at_unix_ms,
            freshness_max_age_ms,
        );
        let metrics_freshness = metrics.map_or_else(
            || ArtifactTimestampAssessment::unknown("node_metrics_timestamp_missing"),
            |value| {
                artifact_timestamp_assessment(
                    "node_metrics",
                    &value.generated_at,
                    observed_at_unix_ms,
                    freshness_max_age_ms,
                )
            },
        );
        let mut combined_freshness =
            combine_timestamp_assessments(&status_freshness, &metrics_freshness);
        if record.process.state == SupervisorProcessState::Stale
            || record.status_artifact == RegistryArtifactState::Stale
            || record.metrics_artifact == RegistryArtifactState::Stale
        {
            combined_freshness.freshness = MvpStatusFreshness::Stale;
            combined_freshness
                .reasons
                .push("supervisor_evidence_marked_stale".to_string());
        }
        let runtime = runtime_axis(record, status_error, &status_freshness, observed_at_unix_ms);
        let technical_health = technical_health_axis(
            record,
            metrics,
            status_error,
            metrics_error,
            identity_error,
            &combined_freshness,
            observed_at_unix_ms,
        );
        let trading_readiness =
            trading_readiness_axis(&unified_read_model_path, observed_at_unix_ms);

        Self {
            schema_version: MVP_STATUS_CONTRACT_SCHEMA_VERSION.to_string(),
            identity_contract_id: identity.contract_id.clone(),
            research: MvpStatusAxis {
                status: MvpResearchStatus::ReferenceBound,
                availability: if identity_error.is_some() {
                    MvpStatusAvailability::Error
                } else {
                    MvpStatusAvailability::Available
                },
                freshness: MvpStatusFreshness::Unknown,
                source_refs: vec![identity.identities.backtest_result_ref.clone()],
                observed_at_unix_ms,
                reasons: {
                    let mut reasons = vec![
                        "backtest_reference_bound".to_string(),
                        "backtest_result_not_verified_by_runtime".to_string(),
                        "research_acceptance_not_claimed".to_string(),
                    ];
                    if identity_error.is_some() {
                        reasons.push("identity_contract_unavailable".to_string());
                    }
                    reasons
                },
                error: identity_error.map(ToString::to_string),
            },
            runtime,
            technical_health,
            trading_readiness,
            provenance: MvpStatusProvenance {
                identity_contract_path: identity_contract_path.display().to_string(),
                identity_contract_available: identity_error.is_none(),
                supervisor_registry_path: registry_path.display().to_string(),
                node_status_path: record.status_path.display().to_string(),
                node_metrics_path: record.metrics_path.display().to_string(),
                unified_read_model_path: unified_read_model_path.display().to_string(),
                freshness_max_age_ms,
                generated_at_unix_ms: observed_at_unix_ms,
            },
            boundaries: MvpStatusBoundaries {
                read_only_product_contract: true,
                http_success_implies_technical_health: false,
                process_alive_implies_technical_health: false,
                backtest_reference_implies_research_accepted: false,
                backtest_complete_implies_trading_readiness: false,
                external_venue_connection: false,
                order_submission_allowed: false,
                order_mutation_allowed: false,
                automatic_retry_allowed: false,
                automatic_remediation_allowed: false,
                real_orders_submitted: false,
            },
        }
    }
}

fn runtime_axis(
    record: &SupervisorNodeRecord,
    status_error: Option<&str>,
    status_freshness: &ArtifactTimestampAssessment,
    observed_at_unix_ms: u64,
) -> MvpStatusAxis<MvpRuntimeStatus> {
    let source_refs = vec![
        record.pid_path.display().to_string(),
        record.status_path.display().to_string(),
    ];
    let timestamp_error = status_freshness.error.as_deref();
    let lifecycle_error = (record.last_known_status.lifecycle_state == LifecycleStatus::Error)
        .then(|| {
            record
                .last_known_status
                .last_error
                .clone()
                .unwrap_or_else(|| "node lifecycle state is error".to_string())
        });
    let invalid_artifact_error =
        (record.status_artifact == RegistryArtifactState::Invalid).then(|| {
            record
                .last_known_status
                .last_error
                .clone()
                .unwrap_or_else(|| "invalid node status artifact".to_string())
        });
    let errors = [
        status_error.map(ToString::to_string),
        timestamp_error.map(ToString::to_string),
        lifecycle_error,
        invalid_artifact_error,
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if !errors.is_empty() {
        let mut reasons = status_freshness.reasons.clone();
        reasons.push("node_runtime_evidence_error".to_string());
        return MvpStatusAxis {
            status: MvpRuntimeStatus::Unknown,
            availability: MvpStatusAvailability::Error,
            freshness: status_freshness.freshness,
            source_refs,
            observed_at_unix_ms,
            reasons,
            error: Some(errors.join("; ")),
        };
    }

    let (status, availability, freshness, mut reasons) = match record.process.state {
        SupervisorProcessState::Running
            if record.status_artifact == RegistryArtifactState::Available
                && record.last_known_status.lifecycle_state == LifecycleStatus::Running =>
        {
            (
                MvpRuntimeStatus::Running,
                MvpStatusAvailability::Available,
                status_freshness.freshness,
                vec!["supervisor_process_and_node_lifecycle_running".to_string()],
            )
        }
        SupervisorProcessState::Running
            if matches!(
                record.last_known_status.lifecycle_state,
                LifecycleStatus::Starting
                    | LifecycleStatus::Pausing
                    | LifecycleStatus::Paused
                    | LifecycleStatus::Resuming
                    | LifecycleStatus::Stopping
            ) =>
        {
            (
                MvpRuntimeStatus::Transitioning,
                MvpStatusAvailability::Available,
                status_freshness.freshness,
                vec!["node_lifecycle_transition_in_progress".to_string()],
            )
        }
        SupervisorProcessState::Running => (
            MvpRuntimeStatus::Unknown,
            artifact_availability(record.status_artifact),
            status_freshness.freshness,
            vec!["process_alive_without_confirmed_running_lifecycle".to_string()],
        ),
        SupervisorProcessState::Stopped | SupervisorProcessState::NotStarted => (
            MvpRuntimeStatus::Stopped,
            MvpStatusAvailability::Available,
            status_freshness.freshness,
            vec!["supervisor_process_not_running".to_string()],
        ),
        SupervisorProcessState::Stale => {
            let availability = if record.status_artifact == RegistryArtifactState::Invalid {
                MvpStatusAvailability::Error
            } else {
                MvpStatusAvailability::Unknown
            };
            (
                MvpRuntimeStatus::Unknown,
                availability,
                MvpStatusFreshness::Stale,
                vec!["supervisor_process_state_stale".to_string()],
            )
        }
        SupervisorProcessState::Unknown => (
            MvpRuntimeStatus::Unknown,
            MvpStatusAvailability::Unknown,
            MvpStatusFreshness::Unknown,
            vec!["supervisor_process_state_unknown".to_string()],
        ),
    };
    reasons.extend(status_freshness.reasons.iter().cloned());
    MvpStatusAxis {
        status,
        availability,
        freshness,
        source_refs,
        observed_at_unix_ms,
        reasons,
        error: None,
    }
}

fn technical_health_axis(
    record: &SupervisorNodeRecord,
    metrics: Option<&NodeMetrics>,
    status_error: Option<&str>,
    metrics_error: Option<&str>,
    identity_error: Option<&str>,
    combined_freshness: &ArtifactTimestampAssessment,
    observed_at_unix_ms: u64,
) -> MvpStatusAxis<MvpTechnicalHealth> {
    let mut reasons = Vec::new();
    let effective_metrics_error = (record.metrics_artifact != RegistryArtifactState::Missing)
        .then_some(metrics_error)
        .flatten();
    let mut errors = [status_error, effective_metrics_error, identity_error]
        .into_iter()
        .flatten()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if let Some(error) = record.last_known_status.last_error.as_ref() {
        errors.push(error.clone());
    }
    if let Some(error) = record.last_known_status.execution.last_error.as_ref() {
        errors.push(error.clone());
    }
    if let Some(error) = record.last_known_status.risk.last_error.as_ref() {
        errors.push(error.clone());
    }
    if let Some(error) = metrics.and_then(|value| value.last_error_summary.as_ref()) {
        errors.push(error.clone());
    }
    if let Some(error) = combined_freshness.error.as_ref() {
        errors.push(error.clone());
    }

    let boundary_violation = record.last_known_status.external_venue_connection
        || record.last_known_status.real_orders_submitted
        || metrics
            .is_some_and(|value| value.external_venue_connection || value.real_orders_submitted);
    if boundary_violation {
        reasons.push("mvp_trading_boundary_violation".to_string());
    }

    let availability = combined_artifact_availability(
        record.status_artifact,
        record.metrics_artifact,
        !errors.is_empty(),
    );
    let freshness = combined_freshness.freshness;
    reasons.extend(combined_freshness.reasons.iter().cloned());
    let metrics_lifecycle = metrics.map(|value| value.lifecycle_state);
    let status = if boundary_violation
        || !errors.is_empty()
        || record.last_known_status.lifecycle_state == LifecycleStatus::Error
        || metrics_lifecycle == Some(LifecycleStatus::Error)
        || matches!(record.status_artifact, RegistryArtifactState::Invalid)
        || matches!(record.metrics_artifact, RegistryArtifactState::Invalid)
    {
        reasons.push("runtime_evidence_reports_error".to_string());
        MvpTechnicalHealth::Unhealthy
    } else if matches!(
        record.process.state,
        SupervisorProcessState::Stopped | SupervisorProcessState::NotStarted
    ) {
        reasons.push("cleanly_stopped_runtime_is_not_running".to_string());
        MvpTechnicalHealth::NotRunning
    } else if record.process.state == SupervisorProcessState::Running
        && record.status_artifact == RegistryArtifactState::Available
        && record.metrics_artifact == RegistryArtifactState::Available
        && record.last_known_status.lifecycle_state == LifecycleStatus::Running
        && metrics_lifecycle == Some(LifecycleStatus::Running)
        && freshness == MvpStatusFreshness::Fresh
    {
        reasons.push("status_and_metrics_confirm_runtime_health".to_string());
        MvpTechnicalHealth::Healthy
    } else if record.process.state == SupervisorProcessState::Unknown
        && record.status_artifact == RegistryArtifactState::Unknown
        && record.metrics_artifact == RegistryArtifactState::Unknown
    {
        reasons.push("technical_health_evidence_unknown".to_string());
        MvpTechnicalHealth::Unknown
    } else {
        reasons.push("technical_health_evidence_incomplete_or_stale".to_string());
        MvpTechnicalHealth::Degraded
    };

    if record.process.state == SupervisorProcessState::Running {
        reasons.push("process_alive_not_sufficient_for_technical_health".to_string());
    }

    MvpStatusAxis {
        status,
        availability,
        freshness,
        source_refs: vec![
            record.status_path.display().to_string(),
            record.metrics_path.display().to_string(),
        ],
        observed_at_unix_ms,
        reasons,
        error: (!errors.is_empty()).then(|| errors.join("; ")),
    }
}

fn trading_readiness_axis(
    unified_read_model_path: &Path,
    observed_at_unix_ms: u64,
) -> MvpStatusAxis<MvpTradingReadiness> {
    let (availability, reasons, error) = match fs::read_to_string(unified_read_model_path) {
        Ok(raw) => match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(value) if value.is_object() => (
                MvpStatusAvailability::Available,
                vec![
                    "unified_read_model_present_but_not_validated_by_mvp_status_contract"
                        .to_string(),
                    "read_only_mvp_never_implies_trading_permission".to_string(),
                ],
                None,
            ),
            Ok(_) => (
                MvpStatusAvailability::Error,
                vec![
                    "unified_read_model_not_an_object".to_string(),
                    "trading_readiness_fail_closed".to_string(),
                ],
                Some("Unified Read Model JSON root must be an object".to_string()),
            ),
            Err(parse_error) => (
                MvpStatusAvailability::Error,
                vec![
                    "invalid_unified_read_model_json".to_string(),
                    "trading_readiness_fail_closed".to_string(),
                ],
                Some(format!("invalid Unified Read Model JSON: {parse_error}")),
            ),
        },
        Err(read_error) if read_error.kind() == std::io::ErrorKind::NotFound => (
            MvpStatusAvailability::Missing,
            vec![
                "missing_unified_read_model".to_string(),
                "trading_readiness_fail_closed".to_string(),
            ],
            None,
        ),
        Err(read_error) => (
            MvpStatusAvailability::Error,
            vec![
                "unreadable_unified_read_model".to_string(),
                "trading_readiness_fail_closed".to_string(),
            ],
            Some(format!("cannot read Unified Read Model: {read_error}")),
        ),
    };
    MvpStatusAxis {
        status: MvpTradingReadiness::Blocked,
        availability,
        freshness: MvpStatusFreshness::Unknown,
        source_refs: vec![unified_read_model_path.display().to_string()],
        observed_at_unix_ms,
        reasons,
        error,
    }
}

fn artifact_availability(state: RegistryArtifactState) -> MvpStatusAvailability {
    match state {
        RegistryArtifactState::Available | RegistryArtifactState::Stale => {
            MvpStatusAvailability::Available
        }
        RegistryArtifactState::Missing => MvpStatusAvailability::Missing,
        RegistryArtifactState::Invalid => MvpStatusAvailability::Error,
        RegistryArtifactState::Unknown => MvpStatusAvailability::Unknown,
    }
}

fn combined_artifact_availability(
    status: RegistryArtifactState,
    metrics: RegistryArtifactState,
    has_error: bool,
) -> MvpStatusAvailability {
    if has_error
        || matches!(status, RegistryArtifactState::Invalid)
        || matches!(metrics, RegistryArtifactState::Invalid)
    {
        MvpStatusAvailability::Error
    } else if matches!(status, RegistryArtifactState::Missing)
        || matches!(metrics, RegistryArtifactState::Missing)
    {
        MvpStatusAvailability::Missing
    } else if matches!(status, RegistryArtifactState::Unknown)
        || matches!(metrics, RegistryArtifactState::Unknown)
    {
        MvpStatusAvailability::Unknown
    } else {
        MvpStatusAvailability::Available
    }
}

#[derive(Clone, Debug)]
struct ArtifactTimestampAssessment {
    freshness: MvpStatusFreshness,
    generated_at_unix_ms: Option<u64>,
    reasons: Vec<String>,
    error: Option<String>,
}

impl ArtifactTimestampAssessment {
    fn unknown(reason: &str) -> Self {
        Self {
            freshness: MvpStatusFreshness::Unknown,
            generated_at_unix_ms: None,
            reasons: vec![reason.to_string()],
            error: None,
        }
    }
}

fn artifact_timestamp_assessment(
    label: &str,
    generated_at: &SnapshotValue<String>,
    observed_at_unix_ms: u64,
    freshness_max_age_ms: u64,
) -> ArtifactTimestampAssessment {
    match generated_at.availability {
        SnapshotAvailability::Available => {
            let Some(raw) = generated_at.value.as_deref() else {
                return ArtifactTimestampAssessment {
                    freshness: MvpStatusFreshness::Unknown,
                    generated_at_unix_ms: None,
                    reasons: vec![format!("{label}_timestamp_value_missing")],
                    error: Some(format!("{label} generated_at is available without a value")),
                };
            };
            let Ok(timestamp) = raw.parse::<u64>() else {
                return ArtifactTimestampAssessment {
                    freshness: MvpStatusFreshness::Unknown,
                    generated_at_unix_ms: None,
                    reasons: vec![format!("{label}_timestamp_invalid")],
                    error: Some(format!(
                        "{label} generated_at '{raw}' is not Unix milliseconds"
                    )),
                };
            };
            if timestamp > observed_at_unix_ms.saturating_add(freshness_max_age_ms) {
                return ArtifactTimestampAssessment {
                    freshness: MvpStatusFreshness::Unknown,
                    generated_at_unix_ms: Some(timestamp),
                    reasons: vec![format!("{label}_timestamp_in_future")],
                    error: Some(format!(
                        "{label} generated_at {timestamp} exceeds allowed clock skew"
                    )),
                };
            }
            if observed_at_unix_ms.saturating_sub(timestamp) > freshness_max_age_ms {
                ArtifactTimestampAssessment {
                    freshness: MvpStatusFreshness::Stale,
                    generated_at_unix_ms: Some(timestamp),
                    reasons: vec![format!("{label}_timestamp_stale")],
                    error: None,
                }
            } else {
                ArtifactTimestampAssessment {
                    freshness: MvpStatusFreshness::Fresh,
                    generated_at_unix_ms: Some(timestamp),
                    reasons: vec![format!("{label}_timestamp_fresh")],
                    error: None,
                }
            }
        }
        SnapshotAvailability::Stale => ArtifactTimestampAssessment {
            freshness: MvpStatusFreshness::Stale,
            generated_at_unix_ms: None,
            reasons: vec![format!("{label}_timestamp_marked_stale")],
            error: None,
        },
        SnapshotAvailability::NotConfigured
        | SnapshotAvailability::NotSupported
        | SnapshotAvailability::Unknown => {
            ArtifactTimestampAssessment::unknown(&format!("{label}_timestamp_unknown"))
        }
    }
}

fn combine_timestamp_assessments(
    status: &ArtifactTimestampAssessment,
    metrics: &ArtifactTimestampAssessment,
) -> ArtifactTimestampAssessment {
    let mut reasons = status.reasons.clone();
    reasons.extend(metrics.reasons.iter().cloned());
    let errors = [status.error.as_ref(), metrics.error.as_ref()]
        .into_iter()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        return ArtifactTimestampAssessment {
            freshness: MvpStatusFreshness::Unknown,
            generated_at_unix_ms: None,
            reasons,
            error: Some(errors.join("; ")),
        };
    }
    if let (Some(status_timestamp), Some(metrics_timestamp)) =
        (status.generated_at_unix_ms, metrics.generated_at_unix_ms)
        && status_timestamp != metrics_timestamp
    {
        reasons.push("status_metrics_generation_mismatch".to_string());
        return ArtifactTimestampAssessment {
            freshness: MvpStatusFreshness::Stale,
            generated_at_unix_ms: None,
            reasons,
            error: None,
        };
    }
    let freshness = if status.freshness == MvpStatusFreshness::Stale
        || metrics.freshness == MvpStatusFreshness::Stale
    {
        MvpStatusFreshness::Stale
    } else if status.freshness == MvpStatusFreshness::Fresh
        && metrics.freshness == MvpStatusFreshness::Fresh
    {
        MvpStatusFreshness::Fresh
    } else {
        MvpStatusFreshness::Unknown
    };
    ArtifactTimestampAssessment {
        freshness,
        generated_at_unix_ms: status.generated_at_unix_ms,
        reasons,
        error: None,
    }
}

fn required(field: &str, value: &str) -> anyhow::Result<String> {
    let value = value.trim();
    ensure!(!value.is_empty(), "{field} 不能为空");
    Ok(value.to_string())
}

fn unix_time_ms() -> u64 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    u64::try_from(millis).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;

    fn temp_config(name: &str, content: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("ntpro-mvp-contract-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("MVP contract test root should be created");
        let path = root.join("node.toml");
        fs::write(&path, content).expect("MVP contract test config should be written");
        path
    }

    fn valid_config() -> &'static str {
        r#"[node]
node_id = "strategy-instance-alpha"

[strategy]
strategy_id = "strategy-alpha"

[market]
venue = "SANDBOX"

[execution]
venue = "SANDBOX"

[mvp]
strategy_version = "v1"
backtest_run_id = "backtest-alpha-001"
backtest_result_ref = "artifact://backtests/backtest-alpha-001/summary.json"
account_id = "SANDBOX-001"
environment = "sandbox"
"#
    }

    #[test]
    fn mvp_contract_loads_eight_stable_identities_and_closed_boundaries() {
        let path = temp_config("valid", valid_config());
        let contract = MvpIdentityContract::load(&path, "mvp-node-001")
            .expect("valid MVP identity contract should load");

        assert_eq!(
            contract.schema_version,
            MVP_IDENTITY_CONTRACT_SCHEMA_VERSION
        );
        assert_eq!(contract.identities.strategy_id, "strategy-alpha");
        assert_eq!(contract.identities.strategy_version, "v1");
        assert_eq!(contract.identities.backtest_run_id, "backtest-alpha-001");
        assert_eq!(contract.identities.node_id, "mvp-node-001");
        assert_eq!(
            contract.identities.strategy_instance_id,
            "strategy-instance-alpha"
        );
        assert_eq!(contract.identities.account_id, "SANDBOX-001");
        assert_eq!(contract.identities.venue_id, "SANDBOX");
        assert_eq!(contract.identities.environment, "sandbox");
        assert!(contract.boundaries.read_only_product_contract);
        assert!(!contract.boundaries.external_venue_connection);
        assert!(!contract.boundaries.order_submission_allowed);
        assert!(!contract.boundaries.order_mutation_allowed);
        assert!(!contract.boundaries.automatic_retry_allowed);
        assert!(!contract.boundaries.automatic_remediation_allowed);
        assert!(!contract.boundaries.real_orders_submitted);
    }

    #[test]
    fn mvp_contract_rejects_missing_mvp_identity_section() {
        let config = valid_config()
            .split("[mvp]")
            .next()
            .expect("fixture prefix should exist");
        let path = temp_config("missing-mvp", config);
        let error = MvpIdentityContract::load(&path, "mvp-node-001")
            .expect_err("missing MVP identity section must fail closed");
        assert!(format!("{error:#}").contains("missing field `mvp`"));
    }

    #[test]
    fn mvp_contract_rejects_empty_identity() {
        let config =
            valid_config().replace("strategy_version = \"v1\"", "strategy_version = \" \"");
        let path = temp_config("empty", &config);
        let error = MvpIdentityContract::load(&path, "mvp-node-001")
            .expect_err("empty identity must fail closed");
        assert!(format!("{error:#}").contains("mvp.strategy_version 不能为空"));
    }

    #[test]
    fn mvp_contract_rejects_venue_mismatch() {
        let config = valid_config().replace(
            "[execution]\nvenue = \"SANDBOX\"",
            "[execution]\nvenue = \"OTHER\"",
        );
        let path = temp_config("venue-mismatch", &config);
        let error = MvpIdentityContract::load(&path, "mvp-node-001")
            .expect_err("venue mismatch must fail closed");
        assert!(
            format!("{error:#}")
                .contains("market.venue 'SANDBOX' 与 execution.venue 'OTHER' 不一致")
        );
    }

    #[test]
    fn mvp_contract_rejects_non_sandbox_environment() {
        let config =
            valid_config().replace("environment = \"sandbox\"", "environment = \"production\"");
        let path = temp_config("production", &config);
        let error = MvpIdentityContract::load(&path, "mvp-node-001")
            .expect_err("non-sandbox environment must fail closed");
        assert!(format!("{error:#}").contains("mvp.environment 必须为 sandbox"));
    }

    #[test]
    fn mvp_contract_rejects_node_and_strategy_instance_identity_collision() {
        let path = temp_config("identity-collision", valid_config());
        let error = MvpIdentityContract::load(&path, "strategy-instance-alpha")
            .expect_err("identity collision must fail closed");
        assert!(format!("{error:#}").contains("必须使用不同身份"));
    }
}
