use std::{fs, path::PathBuf};

use serde_json::json;

use crate::{
    dashboard::DashboardAvailability,
    mvp_contract::{
        MvpIdentityBoundaries, MvpIdentityProvenance, MvpIdentitySet, MvpResearchStatus,
        MvpRuntimeStatus, MvpStatusAvailability, MvpStatusAxis, MvpStatusBoundaries,
        MvpStatusProvenance, MvpTradingReadiness,
    },
    supervisor::{RegisterNodeRequest, SupervisorRegistryStore},
};

use super::*;

struct Fixture {
    root: PathBuf,
    state: DashboardServerState,
    identity: MvpIdentityContract,
    status: MvpStatusContract,
    read_model_path: PathBuf,
}

impl Fixture {
    fn new(name: &str, now_unix_ms: u64) -> Self {
        let root = std::env::temp_dir().join(format!(
            "ntpro-mvp-005-{name}-{}-{now_unix_ms}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("supervisor"))
            .expect("fixture supervisor directory should be created");
        let registry_path = root.join("supervisor/registry.json");
        let artifact_root = root.join("nodes/mvp-node-001");
        let config_path = root.join("node.toml");
        fs::write(
            &config_path,
            r#"[node]
node_id = "mvp-strategy-001"

[strategy]
strategy_id = "ema-cross"

[market]
venue = "BINANCE"

[execution]
venue = "BINANCE"

[mvp]
strategy_version = "sha256:strategy-v1"
backtest_run_id = "backtest-001"
backtest_result_ref = "artifact://backtests/backtest-001.json"
account_id = "acct-sandbox-001"
environment = "sandbox"
"#,
        )
        .expect("fixture config should be written");
        let store = SupervisorRegistryStore::new(&registry_path);
        let record = store
            .register_node(RegisterNodeRequest {
                node_id: "mvp-node-001".to_string(),
                config_path: config_path.clone(),
                artifact_root: Some(artifact_root.clone()),
            })
            .expect("fixture node should be registered");
        let identity_path = root.join(MVP_IDENTITY_CONTRACT_PATH);
        let status_path = root.join(MVP_STATUS_CONTRACT_PATH);
        fs::create_dir_all(
            identity_path
                .parent()
                .expect("identity parent should exist"),
        )
        .expect("fixture MVP directory should be created");

        let identity = MvpIdentityContract {
            schema_version: MVP_IDENTITY_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id: "mvp-node-001:ema-cross:mvp-strategy-001".to_string(),
            identities: MvpIdentitySet {
                strategy_id: "ema-cross".to_string(),
                strategy_version: "sha256:strategy-v1".to_string(),
                backtest_run_id: "backtest-001".to_string(),
                backtest_result_ref: "artifact://backtests/backtest-001.json".to_string(),
                node_id: "mvp-node-001".to_string(),
                strategy_instance_id: "mvp-strategy-001".to_string(),
                account_id: "acct-sandbox-001".to_string(),
                venue_id: "BINANCE".to_string(),
                environment: "sandbox".to_string(),
            },
            provenance: MvpIdentityProvenance {
                config_path: config_path.display().to_string(),
                generated_at_unix_ms: now_unix_ms,
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
        };
        let read_model_path = artifact_root.join("v0_21/unified_read_model_snapshot.json");
        let status = MvpStatusContract {
            schema_version: MVP_STATUS_CONTRACT_SCHEMA_VERSION.to_string(),
            identity_contract_id: identity.contract_id.clone(),
            research: MvpStatusAxis {
                status: MvpResearchStatus::ReferenceBound,
                availability: MvpStatusAvailability::Available,
                freshness: MvpStatusFreshness::Unknown,
                source_refs: vec![identity.identities.backtest_result_ref.clone()],
                observed_at_unix_ms: now_unix_ms,
                reasons: vec!["research_acceptance_not_claimed".to_string()],
                error: None,
            },
            runtime: MvpStatusAxis {
                status: MvpRuntimeStatus::Running,
                availability: MvpStatusAvailability::Available,
                freshness: MvpStatusFreshness::Fresh,
                source_refs: vec![record.status_path.display().to_string()],
                observed_at_unix_ms: now_unix_ms,
                reasons: vec!["runtime_fixture".to_string()],
                error: None,
            },
            technical_health: MvpStatusAxis {
                status: MvpTechnicalHealth::Healthy,
                availability: MvpStatusAvailability::Available,
                freshness: MvpStatusFreshness::Fresh,
                source_refs: vec![
                    record.status_path.display().to_string(),
                    record.metrics_path.display().to_string(),
                ],
                observed_at_unix_ms: now_unix_ms,
                reasons: vec!["technical_health_fixture".to_string()],
                error: None,
            },
            trading_readiness: MvpStatusAxis {
                status: MvpTradingReadiness::Blocked,
                availability: MvpStatusAvailability::Missing,
                freshness: MvpStatusFreshness::Unknown,
                source_refs: vec![read_model_path.display().to_string()],
                observed_at_unix_ms: now_unix_ms,
                reasons: vec!["missing_unified_read_model".to_string()],
                error: None,
            },
            provenance: MvpStatusProvenance {
                identity_contract_path: identity_path.display().to_string(),
                identity_contract_available: true,
                supervisor_registry_path: registry_path.display().to_string(),
                node_status_path: record.status_path.display().to_string(),
                node_metrics_path: record.metrics_path.display().to_string(),
                unified_read_model_path: read_model_path.display().to_string(),
                freshness_max_age_ms: 2_000,
                generated_at_unix_ms: now_unix_ms,
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
        };
        write_json(&identity_path, &identity);
        write_json(&status_path, &status);

        Self {
            root,
            state: DashboardServerState {
                registry_path,
                workflow_root: None,
                ntpro_node_bin: PathBuf::from("missing-ntpro-node"),
            },
            identity,
            status,
            read_model_path,
        }
    }

    fn write_identity(&self) {
        write_json(&self.root.join(MVP_IDENTITY_CONTRACT_PATH), &self.identity);
    }

    fn write_status(&self) {
        write_json(&self.root.join(MVP_STATUS_CONTRACT_PATH), &self.status);
    }

    fn write_healthy_read_model(&self, now_unix_ms: u64) -> Value {
        let value = healthy_read_model(now_unix_ms);
        fs::create_dir_all(
            self.read_model_path
                .parent()
                .expect("read model parent should exist"),
        )
        .expect("read model parent should be created");
        write_json(&self.read_model_path, &value);
        value
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn shared_status_projects_one_fact_set_for_both_portals() {
    let now = 1_800_000_000_000;
    let fixture = Fixture::new("shared-facts", now);
    let response = project_mvp_shared_status(&fixture.state, now)
        .expect("valid contracts should produce shared status");

    assert_eq!(
        response.schema_version,
        MVP_SHARED_STATUS_API_SCHEMA_VERSION
    );
    assert_eq!(
        response.contract_version,
        MVP_SHARED_STATUS_API_CONTRACT_VERSION
    );
    assert_eq!(
        response.consumers,
        ["institution_workbench", "control_center"]
    );
    assert_eq!(
        response.identity.contract_id,
        response.status.identity_contract_id
    );
    assert_eq!(
        response.business.availability,
        MvpBusinessAvailability::Missing
    );
    assert_eq!(
        response.status.trading_readiness.status,
        MvpTradingReadiness::Blocked
    );
    assert!(response.boundaries.read_only);
    assert!(!response.boundaries.http_success_implies_technical_health);
    assert!(!response.boundaries.order_submission_allowed);
    assert!(!response.boundaries.order_mutation_allowed);
    assert!(!response.boundaries.automatic_retry_allowed);
    assert!(!response.boundaries.automatic_remediation_allowed);
    assert!(!response.boundaries.external_venue_connection);
    assert!(!response.boundaries.real_orders_submitted);

    let serialized = serde_json::to_value(response).expect("response should serialize");
    assert_eq!(
        serialized["business"]["account"]["status"]["availability"],
        "unknown"
    );
    assert_eq!(serialized["boundaries"]["raw_event_store_exposed"], false);
    assert_eq!(serialized["boundaries"]["raw_venue_payload_exposed"], false);
}

#[test]
fn event_correlation_projects_one_minimized_status_event_for_both_portals() {
    let now = 1_800_000_000_000;
    let fixture = Fixture::new("event-correlation", now);
    let response = project_mvp_event_correlation(&fixture.state, now)
        .expect("valid contracts should produce an event correlation");

    assert_eq!(
        response.schema_version,
        MVP_EVENT_CORRELATION_SCHEMA_VERSION
    );
    assert_eq!(
        response.contract_version,
        MVP_EVENT_CORRELATION_CONTRACT_VERSION
    );
    assert_eq!(
        response.event.event_id,
        "mvp-status:mvp-node-001:ema-cross:mvp-strategy-001:technical-health"
    );
    assert_eq!(response.event.event_kind, "technical_health_observation");
    assert_eq!(response.event.event_source, "projected_status_contract");
    assert_eq!(
        response.event.identity_contract_id,
        fixture.identity.contract_id
    );
    assert_eq!(response.event.node_id, "mvp-node-001");
    assert_eq!(response.event.strategy_instance_id, "mvp-strategy-001");
    assert_eq!(
        response.links.institution_workbench_path,
        "/institution-workbench"
    );
    assert_eq!(response.links.control_center_path, "/control-center");
    assert!(response.boundaries.read_only);
    assert!(response.boundaries.projected_status_event);
    assert!(!response.boundaries.raw_event_store_exposed);
    assert!(!response.boundaries.raw_event_payload_exposed);
    assert!(!response.boundaries.raw_errors_exposed);
    assert!(!response.boundaries.supervisor_actions_exposed);
    assert!(!response.boundaries.trading_controls_exposed);

    let serialized = serde_json::to_string(&response).expect("response should serialize");
    for forbidden in [
        "source_refs",
        "config_path",
        "registry_path",
        "node_status_path",
        "node_metrics_path",
        "unified_read_model_path",
        "raw_event",
        "last_error",
        "message",
        "credential",
        "controls",
    ] {
        assert!(
            !serialized.contains(&format!("\"{forbidden}\"")),
            "event correlation exposed forbidden field {forbidden}"
        );
    }
}

#[test]
fn valid_read_model_projects_available_business_summary() {
    let now = 1_800_000_000_000;
    let fixture = Fixture::new("available-business", now);
    fixture.write_healthy_read_model(now);

    let response = project_mvp_shared_status(&fixture.state, now)
        .expect("healthy read model should produce shared status");
    assert_eq!(
        response.business.availability,
        MvpBusinessAvailability::Available
    );
    assert_eq!(response.business.health, HealthStatus::Healthy);
    assert_eq!(
        response.business.readiness_status.value.as_deref(),
        Some("ready_readonly_artifact")
    );
    assert_eq!(
        response.business.account.status.value.as_deref(),
        Some("healthy")
    );
    assert_eq!(
        response.business.positions.status.value.as_deref(),
        Some("healthy")
    );
}

#[test]
fn stale_status_contract_cannot_keep_healthy_fresh_projection() {
    let now = 1_800_000_000_000;
    let mut fixture = Fixture::new("stale-status", now);
    fixture.status.provenance.generated_at_unix_ms = now - 2_001;
    fixture.write_status();

    let response = project_mvp_shared_status(&fixture.state, now)
        .expect("stale status should remain queryable with degraded semantics");
    assert_eq!(response.status.runtime.freshness, MvpStatusFreshness::Stale);
    assert_eq!(
        response.status.technical_health.status,
        MvpTechnicalHealth::Degraded
    );
    assert_eq!(
        response.status.technical_health.freshness,
        MvpStatusFreshness::Stale
    );
    assert!(
        response
            .status
            .technical_health
            .reasons
            .iter()
            .any(|reason| { reason == "shared_api_status_contract_freshness_threshold_exceeded" })
    );
}

#[test]
fn stale_axis_cannot_hide_behind_fresh_top_level_timestamp() {
    let now = 1_800_000_000_000;
    let mut fixture = Fixture::new("stale-axis", now);
    fixture.status.technical_health.observed_at_unix_ms = now - 2_001;
    fixture.write_status();

    let response = project_mvp_shared_status(&fixture.state, now)
        .expect("stale axis should remain queryable with degraded semantics");
    assert_eq!(
        response.status.technical_health.status,
        MvpTechnicalHealth::Degraded
    );
    assert_eq!(
        response.status.technical_health.freshness,
        MvpStatusFreshness::Stale
    );
    assert!(
        response
            .status
            .technical_health
            .reasons
            .iter()
            .any(|reason| reason == "shared_api_axis_freshness_threshold_exceeded")
    );
}

#[test]
fn status_contract_identity_mismatch_fails_closed() {
    let now = 1_800_000_000_000;
    let mut fixture = Fixture::new("status-identity", now);
    fixture.status.identity_contract_id = "different-contract".to_string();
    fixture.write_status();

    let error = project_mvp_shared_status(&fixture.state, now)
        .expect_err("identity mismatch must fail closed");
    assert_eq!(error.kind, SharedStatusErrorKind::IdentityMismatch);
    assert_eq!(error.source, "status_contract");
}

#[test]
fn true_boundary_in_source_contract_fails_closed() {
    let now = 1_800_000_000_000;
    let mut fixture = Fixture::new("boundary", now);
    fixture.identity.boundaries.order_submission_allowed = true;
    fixture.write_identity();

    let error = project_mvp_shared_status(&fixture.state, now)
        .expect_err("source boundary true must fail closed");
    assert_eq!(error.kind, SharedStatusErrorKind::BoundaryViolation);
    let (status, Json(body)) = shared_status_error_response(error);
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["order_submission_allowed"], false);
    assert_eq!(body["automatic_remediation_allowed"], false);
}

#[test]
fn forged_or_incomplete_identity_contract_fails_closed() {
    let now = 1_800_000_000_000;
    let mut fixture = Fixture::new("identity-contract", now);
    fixture.identity.identities.strategy_version.clear();
    fixture.write_identity();

    let error = project_mvp_shared_status(&fixture.state, now)
        .expect_err("incomplete identity must fail closed");
    assert_eq!(error.kind, SharedStatusErrorKind::Invalid);
    assert_eq!(error.field, "identities");

    fixture.identity.identities.strategy_version = "sha256:strategy-v1".to_string();
    fixture.identity.contract_id = "forged-contract-id".to_string();
    fixture.write_identity();
    let error = project_mvp_shared_status(&fixture.state, now)
        .expect_err("forged contract ID must fail closed");
    assert_eq!(error.kind, SharedStatusErrorKind::Invalid);
    assert_eq!(error.field, "contract_id");
}

#[test]
fn inconsistent_axis_error_envelope_fails_closed() {
    let now = 1_800_000_000_000;
    let mut fixture = Fixture::new("axis-error", now);
    fixture.status.runtime.error = Some("runtime source failed".to_string());
    fixture.write_status();

    let error = project_mvp_shared_status(&fixture.state, now)
        .expect_err("error without error availability must fail closed");
    assert_eq!(error.kind, SharedStatusErrorKind::Invalid);
    assert_eq!(error.field, "runtime");
}

#[test]
fn future_status_timestamp_fails_closed() {
    let now = 1_800_000_000_000;
    let mut fixture = Fixture::new("future-status", now);
    fixture.status.provenance.generated_at_unix_ms = now + MAX_CLOCK_SKEW_MS + 1;
    fixture.write_status();

    let error = project_mvp_shared_status(&fixture.state, now)
        .expect_err("future status timestamp must fail closed");
    assert_eq!(error.kind, SharedStatusErrorKind::Invalid);
    assert_eq!(error.field, "generated_at_unix_ms");
}

#[test]
fn zero_status_or_axis_timestamp_fails_closed() {
    let now = 1_800_000_000_000;
    let mut fixture = Fixture::new("zero-status-time", now);
    fixture.status.provenance.generated_at_unix_ms = 0;
    fixture.write_status();
    let error = project_mvp_shared_status(&fixture.state, now)
        .expect_err("zero status timestamp must fail closed");
    assert_eq!(error.kind, SharedStatusErrorKind::Invalid);
    assert_eq!(error.field, "provenance");

    fixture.status.provenance.generated_at_unix_ms = now;
    fixture.status.runtime.observed_at_unix_ms = 0;
    fixture.write_status();
    let error = project_mvp_shared_status(&fixture.state, now)
        .expect_err("zero axis timestamp must fail closed");
    assert_eq!(error.kind, SharedStatusErrorKind::Invalid);
    assert_eq!(error.field, "runtime");
}

#[test]
fn tampered_runtime_provenance_fails_closed() {
    let now = 1_800_000_000_000;
    let mut fixture = Fixture::new("provenance", now);
    fixture.status.provenance.unified_read_model_path =
        "/tmp/unrelated/unified_read_model_snapshot.json".to_string();
    fixture.write_status();

    let error = project_mvp_shared_status(&fixture.state, now)
        .expect_err("tampered runtime provenance must fail closed");
    assert_eq!(error.kind, SharedStatusErrorKind::Invalid);
    assert_eq!(error.field, "runtime_provenance");
}

#[test]
fn read_model_account_or_venue_mismatch_is_not_projected() {
    let now = 1_800_000_000_000;
    let fixture = Fixture::new("read-model-identity", now);
    fs::create_dir_all(
        fixture
            .read_model_path
            .parent()
            .expect("read model parent should exist"),
    )
    .expect("read model parent should be created");
    fs::write(
        &fixture.read_model_path,
        serde_json::to_vec_pretty(&json!({
            "snapshot_identity": {
                "account_id": "different-account",
                "venue": "BINANCE"
            }
        }))
        .expect("read model fixture should serialize"),
    )
    .expect("read model fixture should be written");

    let response = project_mvp_shared_status(&fixture.state, now)
        .expect("identity mismatch should be represented in shared response");
    assert_eq!(
        response.business.availability,
        MvpBusinessAvailability::IdentityMismatch
    );
    assert_eq!(response.business.health, HealthStatus::Error);
    assert_eq!(
        response.business.readiness_status.value.as_deref(),
        Some("fail_closed")
    );
    assert_eq!(
        response.business.account.status.availability,
        DashboardAvailability::Unknown
    );
}

#[test]
fn component_account_mismatch_is_not_projected() {
    let now = 1_800_000_000_000;
    let fixture = Fixture::new("component-account", now);
    let mut read_model = fixture.write_healthy_read_model(now);
    read_model["components"]["account"]["data"]["account_id"] = json!("different-account");
    read_model["components"]["positions"]["data"]["account_id"] = json!("different-account");
    write_json(&fixture.read_model_path, &read_model);

    let response = project_mvp_shared_status(&fixture.state, now)
        .expect("component identity mismatch should be represented");
    assert_eq!(
        response.business.availability,
        MvpBusinessAvailability::IdentityMismatch
    );
    assert_eq!(response.business.health, HealthStatus::Error);
    assert_eq!(
        response.business.account.status.availability,
        DashboardAvailability::Unknown
    );
}

#[test]
fn component_venue_mismatch_is_not_projected() {
    let now = 1_800_000_000_000;
    for component in ["positions", "orders", "fills"] {
        let fixture = Fixture::new(&format!("component-venue-{component}"), now);
        let mut read_model = fixture.write_healthy_read_model(now);
        read_model["components"][component]["data"]["instrument_identity"] = json!({
            "instrument_id": "BTCUSDT.OTHER",
            "symbol": "BTCUSDT",
            "venue": "OTHER"
        });
        write_json(&fixture.read_model_path, &read_model);

        let response = project_mvp_shared_status(&fixture.state, now)
            .expect("component venue mismatch should be represented");
        assert_eq!(
            response.business.availability,
            MvpBusinessAvailability::IdentityMismatch,
            "component {component} must not cross venues"
        );
        assert_eq!(response.business.health, HealthStatus::Error);
    }
}

#[test]
fn stale_component_cannot_keep_available_healthy_projection() {
    let now = 1_800_000_000_000;
    let fixture = Fixture::new("stale-component", now);
    let mut read_model = fixture.write_healthy_read_model(now);
    let stale_ns = u128::from(now - 2_001) * 1_000_000;
    read_model["components"]["account"]["freshness"]["as_of_unix_ns"] = json!(stale_ns.to_string());
    write_json(&fixture.read_model_path, &read_model);

    let response = project_mvp_shared_status(&fixture.state, now)
        .expect("stale component should remain queryable with stale semantics");
    assert_eq!(
        response.business.availability,
        MvpBusinessAvailability::Stale
    );
    assert_eq!(response.business.health, HealthStatus::Stale);
    assert_eq!(
        response.business.account.freshness_status.value.as_deref(),
        Some("stale")
    );
    assert_eq!(
        response
            .business
            .positions
            .freshness_status
            .value
            .as_deref(),
        Some("fresh")
    );
}

#[test]
fn invalid_read_model_and_true_boundary_fail_closed() {
    let now = 1_800_000_000_000;
    let fixture = Fixture::new("invalid-read-model", now);
    fs::create_dir_all(
        fixture
            .read_model_path
            .parent()
            .expect("read model parent should exist"),
    )
    .expect("read model parent should be created");
    fs::write(&fixture.read_model_path, "{").expect("invalid read model should be written");
    let response = project_mvp_shared_status(&fixture.state, now)
        .expect("invalid read model should be represented");
    assert_eq!(
        response.business.availability,
        MvpBusinessAvailability::Error
    );
    assert_eq!(
        response.business.readiness_status.value.as_deref(),
        Some("fail_closed")
    );

    let mut read_model = healthy_read_model(now);
    read_model["capability_boundary"]["production_order_submission_allowed"] = json!(true);
    write_json(&fixture.read_model_path, &read_model);
    let response = project_mvp_shared_status(&fixture.state, now)
        .expect("true read model boundary should be represented");
    assert_eq!(
        response.business.availability,
        MvpBusinessAvailability::Error
    );
    assert_eq!(response.business.health, HealthStatus::Error);
}

#[test]
fn config_projection_drift_fails_closed() {
    let now = 1_800_000_000_000;
    let fixture = Fixture::new("config-drift", now);
    let config_path = fixture.root.join("node.toml");
    let config = fs::read_to_string(&config_path).expect("fixture config should be readable");
    fs::write(
        &config_path,
        config.replace("acct-sandbox-001", "different-account"),
    )
    .expect("drifted config should be written");

    let error = project_mvp_shared_status(&fixture.state, now)
        .expect_err("config identity drift must fail closed");
    assert_eq!(error.kind, SharedStatusErrorKind::IdentityMismatch);
    assert_eq!(error.field, "config_projection");
}

#[test]
fn coordinated_registry_path_escape_fails_closed() {
    let now = 1_800_000_000_000;
    let mut fixture = Fixture::new("registry-escape", now);
    let escaped_root = fixture.root.with_extension("outside");
    fs::create_dir_all(&escaped_root).expect("escaped artifact root should be created");
    let mut registry: Value = serde_json::from_str(
        &fs::read_to_string(&fixture.state.registry_path)
            .expect("fixture registry should be readable"),
    )
    .expect("fixture registry should be valid JSON");
    let record = &mut registry["nodes"]["mvp-node-001"];
    record["artifact_root"] = json!(escaped_root.display().to_string());
    for (field, name) in [
        ("status_path", "status.json"),
        ("metrics_path", "metrics.json"),
    ] {
        record[field] = json!(escaped_root.join(name).display().to_string());
    }
    let escaped_read_model = escaped_root.join("v0_21/unified_read_model_snapshot.json");
    fixture.status.provenance.node_status_path =
        escaped_root.join("status.json").display().to_string();
    fixture.status.provenance.node_metrics_path =
        escaped_root.join("metrics.json").display().to_string();
    fixture.status.provenance.unified_read_model_path = escaped_read_model.display().to_string();
    write_json(&fixture.state.registry_path, &registry);
    fixture.write_status();

    let error = project_mvp_shared_status(&fixture.state, now)
        .expect_err("registry path escape must fail closed");
    assert_eq!(error.kind, SharedStatusErrorKind::Invalid);
    assert_eq!(error.field, "artifact_root_containment");
    fs::remove_dir_all(escaped_root).expect("escaped fixture should be removed");
}

#[test]
fn legal_artifact_root_with_external_status_or_metrics_path_fails_closed() {
    let now = 1_800_000_000_000;
    for (field, provenance_field, file_name) in [
        ("status_path", "node_status_path", "status.json"),
        ("metrics_path", "node_metrics_path", "metrics.json"),
    ] {
        let mut fixture = Fixture::new(&format!("child-path-{field}"), now);
        let escaped_root = fixture.root.with_extension(format!("outside-{field}"));
        fs::create_dir_all(&escaped_root).expect("escaped root should be created");
        let escaped_path = escaped_root.join(file_name);
        fs::write(&escaped_path, "{}").expect("escaped artifact should be written");
        let mut registry: Value = serde_json::from_str(
            &fs::read_to_string(&fixture.state.registry_path)
                .expect("fixture registry should be readable"),
        )
        .expect("fixture registry should be valid JSON");
        registry["nodes"]["mvp-node-001"][field] = json!(escaped_path.display().to_string());
        match provenance_field {
            "node_status_path" => {
                fixture.status.provenance.node_status_path = escaped_path.display().to_string();
            }
            "node_metrics_path" => {
                fixture.status.provenance.node_metrics_path = escaped_path.display().to_string();
            }
            _ => unreachable!("fixture provenance field is fixed"),
        }
        write_json(&fixture.state.registry_path, &registry);
        fixture.write_status();

        let error = project_mvp_shared_status(&fixture.state, now)
            .expect_err("child path outside a legal artifact root must fail closed");
        assert_eq!(error.kind, SharedStatusErrorKind::Invalid);
        assert_eq!(error.field, "runtime_provenance");
        fs::remove_dir_all(escaped_root).expect("escaped fixture should be removed");
    }
}

#[cfg(unix)]
#[test]
fn unified_read_model_symlink_escape_fails_closed() {
    let now = 1_800_000_000_000;
    let fixture = Fixture::new("read-model-symlink", now);
    let escaped_root = fixture.root.with_extension("outside-read-model");
    fs::create_dir_all(&escaped_root).expect("escaped root should be created");
    let escaped_read_model = escaped_root.join("unified_read_model_snapshot.json");
    write_json(&escaped_read_model, &healthy_read_model(now));
    fs::create_dir_all(
        fixture
            .read_model_path
            .parent()
            .expect("read model parent should exist"),
    )
    .expect("read model parent should be created");
    std::os::unix::fs::symlink(&escaped_read_model, &fixture.read_model_path)
        .expect("read model symlink should be created");

    let error = project_mvp_shared_status(&fixture.state, now)
        .expect_err("read model symlink outside artifact root must fail closed");
    assert_eq!(error.kind, SharedStatusErrorKind::Invalid);
    assert_eq!(error.field, "unified_read_model_path_containment");
    fs::remove_dir_all(escaped_root).expect("escaped fixture should be removed");
}

fn healthy_read_model(now_unix_ms: u64) -> Value {
    let mut value = crate::dashboard::tests::healthy_trader_terminal_read_model_artifact();
    let now_unix_ns = (u128::from(now_unix_ms) * 1_000_000).to_string();
    value["snapshot_identity"]["account_id"] = json!("acct-sandbox-001");
    value["snapshot_identity"]["venue"] = json!("BINANCE");
    value["components"]["account"]["data"]["account_id"] = json!("acct-sandbox-001");
    value["components"]["positions"]["data"]["account_id"] = json!("acct-sandbox-001");
    value["components"]["positions"]["data"]["instrument_identity"] = json!({
        "instrument_id": "BTCUSDT.BINANCE",
        "symbol": "BTCUSDT",
        "venue": "BINANCE"
    });
    set_freshness(&mut value["freshness"], &now_unix_ns);
    for component in [
        "account",
        "positions",
        "orders",
        "fills",
        "risk",
        "lifecycle_status",
        "operation_entry",
        "v25_monitoring_observability",
        "v25_alert_taxonomy_routing",
        "v25_incident_lifecycle",
        "v25_runbook_audit",
        "v25_dr_preview_drill",
        "v26_permission_boundary",
        "v26_operation_audit",
        "v26_deployment_provenance",
        "v26_upgrade_rollback",
        "v26_stability_slo",
    ] {
        set_freshness(
            &mut value["components"][component]["freshness"],
            &now_unix_ns,
        );
    }
    value
}

fn set_freshness(freshness: &mut Value, now_unix_ns: &str) {
    freshness["status"] = json!("fresh");
    freshness["observed_age_ms"] = json!(0);
    freshness["max_age_ms"] = json!(2_000);
    freshness["as_of_unix_ns"] = json!(now_unix_ns);
    freshness["checked_at_unix_ns"] = json!(now_unix_ns);
}

fn write_json(path: &Path, value: &impl Serialize) {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("fixture should serialize"),
    )
    .expect("fixture should be written");
}
