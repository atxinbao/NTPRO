use std::{
    fs::{self, FileTimes, OpenOptions},
    path::{Path, PathBuf},
    time::{Duration, UNIX_EPOCH},
};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{HeaderMap, Method, Request, StatusCode, header::ALLOW},
};
use nautilus_live::status::SnapshotValue;
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::{
    mvp_contract::{
        MVP_IDENTITY_CONTRACT_PATH, MVP_STATUS_CONTRACT_PATH, MvpIdentityBoundaries,
        MvpIdentityProvenance, MvpIdentitySet, MvpStatusContract,
    },
    supervisor::{
        NodeMetricArtifacts, NodeMetricCounts, NodeMetrics, RegisterNodeRequest,
        SupervisorRegistryStore,
    },
};

use super::*;
use crate::dashboard::server::dashboard_router;

struct Fixture {
    root: PathBuf,
    registry_path: PathBuf,
    identity_path: PathBuf,
    status_contract_path: PathBuf,
    config_path: PathBuf,
    identity: MvpIdentityContract,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let now = unix_time_ms();
        let root = std::env::temp_dir().join(format!(
            "ntpro-s0-api-001a-{name}-{}-{now}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("supervisor"))
            .expect("fixture supervisor directory should be created");
        let registry_path = root.join("supervisor/registry.json");
        let config_path = root.join("node.toml");
        fs::write(&config_path, valid_config()).expect("product config fixture should be written");
        let store = SupervisorRegistryStore::new(&registry_path);
        store
            .register_node(RegisterNodeRequest {
                node_id: "mvp-node-001".to_string(),
                config_path: config_path.clone(),
                artifact_root: Some(root.join("nodes/mvp-node-001")),
            })
            .expect("fixture node should be registered");
        let identity_path = root.join(MVP_IDENTITY_CONTRACT_PATH);
        fs::create_dir_all(
            identity_path
                .parent()
                .expect("identity path should have a parent"),
        )
        .expect("identity directory should be created");
        let identity = valid_identity(&config_path, unix_time_ms());
        write_json(&identity_path, &identity);
        let registry = store.load().expect("fixture registry should load");
        let record = registry
            .nodes
            .get("mvp-node-001")
            .expect("fixture node should exist");
        let status_contract_path = root.join(MVP_STATUS_CONTRACT_PATH);
        let status_contract = MvpStatusContract::from_runtime(
            &identity,
            &identity_path,
            &registry_path,
            record,
            None,
            None,
            None,
            None,
            TEST_FRESHNESS_MAX_AGE_MS,
        );
        write_json(&status_contract_path, &status_contract);
        Self {
            root,
            registry_path,
            identity_path,
            status_contract_path,
            config_path,
            identity,
        }
    }

    fn state(&self) -> DashboardServerState {
        DashboardServerState {
            registry_path: self.registry_path.clone(),
            workflow_root: None,
            ntpro_node_bin: PathBuf::from("missing-ntpro-node"),
            lifecycle_action_lock: std::sync::Arc::new(std::sync::Mutex::new(())),
        }
    }

    fn router(&self) -> Router {
        dashboard_router(
            self.registry_path.clone(),
            PathBuf::from("missing-ntpro-node"),
        )
    }

    fn write_identity(&self, identity: &MvpIdentityContract) {
        write_json(&self.identity_path, identity);
    }

    fn read_status_contract(&self) -> MvpStatusContract {
        serde_json::from_slice(
            &fs::read(&self.status_contract_path)
                .expect("status contract fixture should be readable"),
        )
        .expect("status contract fixture should parse")
    }

    fn write_status_contract(&self, status: &MvpStatusContract) {
        write_json(&self.status_contract_path, status);
    }

    fn write_config(&self, value: &str) {
        fs::write(&self.config_path, value).expect("product config fixture should be written");
    }
}

const TEST_FRESHNESS_MAX_AGE_MS: u64 = 30_000;

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn strategy_projection_uses_identity_and_explicit_product_metadata() {
    let fixture = Fixture::new("projection");
    let strategy = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect("valid product source should project");

    assert_eq!(strategy.strategy_id, "ema-cross");
    assert_eq!(strategy.name, "BTC/USDT EMA Cross");
    assert_eq!(strategy.owner, "Systematic Desk");
    assert_eq!(strategy.lifecycle, StrategyLifecycle::Active);
    assert_eq!(strategy.default_version_id, "ema-cross@v1");
    assert_eq!(strategy.source.freshness_status, "fresh");
    assert_eq!(
        strategy.source.source_refs,
        [
            "mvp/identity_contract.json",
            "mvp/status_contract.json",
            "node-config:node.toml"
        ]
    );
}

#[test]
fn list_query_supports_filter_sort_and_stable_cursor() {
    let fixture = Fixture::new("list-query");
    let strategy = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect("valid product source should project");
    let query = parse_strategy_list_query(Some(
        "limit=1&sort=updated_at&order=desc&lifecycle=active&owner=Systematic+Desk",
    ))
    .expect("valid query should parse");
    let response = project_strategy_list(strategy.clone(), &query, product_request_id())
        .expect("valid list should project");
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data.first(), Some(&strategy));
    assert_eq!(response.page.returned_count, 1);
    assert!(!response.page.has_more);

    let cursor = encode_cursor(&strategy.strategy_id);
    let query = parse_strategy_list_query(Some(&format!("cursor={cursor}")))
        .expect("valid cursor should parse");
    let response = project_strategy_list(strategy.clone(), &query, product_request_id())
        .expect("known cursor should project an empty next page");
    assert!(response.data.is_empty());

    let query =
        parse_strategy_list_query(Some("lifecycle=archived")).expect("valid filter should parse");
    let response = project_strategy_list(strategy, &query, product_request_id())
        .expect("non-matching filter should return an empty page");
    assert!(response.data.is_empty());
}

#[test]
fn invalid_queries_fail_closed() {
    for query in [
        "limit=0",
        "limit=101",
        "limit=abc",
        "limit=1&limit=2",
        "cursor=",
        "cursor=forged",
        "sort=created_at",
        "order=random",
        "lifecycle=deleted",
        "owner=%ZZ",
        "owner=%FF",
        "unknown=value",
    ] {
        assert!(
            parse_strategy_list_query(Some(query)).is_err(),
            "{query} must fail closed"
        );
    }
}

#[test]
fn strategy_identifiers_reject_dot_segments_but_allow_embedded_periods() {
    for invalid in [".", ".."] {
        assert!(
            validate_identifier("strategy_id", invalid).is_err(),
            "{invalid} must not be accepted as an addressable resource identifier"
        );
    }
    validate_identifier("strategy_id", "ema.cross.v1")
        .expect("periods inside an ordinary identifier must remain supported");
}

#[test]
fn product_metadata_edit_after_identity_publication_fails_stale() {
    let fixture = Fixture::new("product-metadata-drift");
    let mut identity = fixture.identity.clone();
    identity.provenance.generated_at_unix_ms = unix_time_ms();
    fixture.write_identity(&identity);
    fixture.write_config(
        &valid_config().replace("BTC/USDT EMA Cross", "Changed product display name"),
    );
    let publication_floor =
        UNIX_EPOCH + Duration::from_millis(identity.provenance.generated_at_unix_ms);
    set_modified_time(
        &fixture.identity_path,
        publication_floor + Duration::from_micros(250),
    );
    set_modified_time(
        &fixture.config_path,
        publication_floor + Duration::from_micros(750),
    );

    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("post-publication product metadata drift must fail stale");
    assert_eq!(error.kind, ProductErrorKind::SourceStale);
    assert_eq!(error.field, "strategy_config");
}

#[test]
fn identity_config_and_boundary_drift_fail_closed() {
    let fixture = Fixture::new("negative-sources");

    let mut identity = fixture.identity.clone();
    identity.identities.strategy_id = "other-strategy".to_string();
    identity.contract_id = "mvp-node-001:other-strategy:mvp-strategy-001".to_string();
    fixture.write_identity(&identity);
    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("strategy identity mismatch must fail");
    assert_eq!(error.kind, ProductErrorKind::SourceInvalid);
    assert_eq!(error.field, "config_projection");

    let mut identity = fixture.identity.clone();
    identity.boundaries.order_submission_allowed = true;
    fixture.write_identity(&identity);
    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("enabled order boundary must fail");
    assert_eq!(error.kind, ProductErrorKind::BoundaryViolation);

    let mut identity = fixture.identity.clone();
    identity.provenance.generated_at_unix_ms = 1;
    fixture.write_identity(&identity);
    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("config newer than identity must fail stale");
    assert_eq!(error.kind, ProductErrorKind::SourceStale);
}

#[test]
fn runtime_status_and_metrics_boundary_violations_fail_closed() {
    let fixture = Fixture::new("runtime-registry-boundary");
    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let mut registry = store.load().expect("fixture registry should load");
    registry
        .nodes
        .get_mut("mvp-node-001")
        .expect("fixture node should exist")
        .last_known_status
        .external_venue_connection = true;
    store.save(&registry).expect("fixture registry should save");
    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("cached runtime boundary violation must fail closed");
    assert_eq!(error.kind, ProductErrorKind::BoundaryViolation);
    assert_eq!(error.field, "registry_status");

    let fixture = Fixture::new("runtime-status-boundary");
    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let registry = store.load().expect("fixture registry should load");
    let record = registry
        .nodes
        .get("mvp-node-001")
        .expect("fixture node should exist");
    let mut status = record.last_known_status.clone();
    status.generated_at = fresh_generated_at();
    status.real_orders_submitted = true;
    write_json(&record.status_path, &status);
    let mut registry = registry;
    registry
        .nodes
        .get_mut("mvp-node-001")
        .expect("fixture node should exist")
        .status_artifact = RegistryArtifactState::Available;
    store.save(&registry).expect("fixture registry should save");
    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("status artifact boundary violation must fail closed");
    assert_eq!(error.kind, ProductErrorKind::BoundaryViolation);
    assert_eq!(error.field, "node_status");

    let fixture = Fixture::new("runtime-metrics-boundary");
    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let registry = store.load().expect("fixture registry should load");
    let record = registry
        .nodes
        .get("mvp-node-001")
        .expect("fixture node should exist");
    let mut metrics = NodeMetrics::from_status(
        &record.last_known_status,
        &NodeMetricArtifacts::from_record(record),
        NodeMetricCounts {
            uptime_ms: Some(1),
            starts_total: 1,
            stops_total: 0,
            state_transitions_total: 1,
        },
    );
    metrics.generated_at = fresh_generated_at();
    metrics.external_venue_connection = true;
    write_json(&record.metrics_path, &metrics);
    let mut registry = registry;
    registry
        .nodes
        .get_mut("mvp-node-001")
        .expect("fixture node should exist")
        .metrics_artifact = RegistryArtifactState::Available;
    store.save(&registry).expect("fixture registry should save");
    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("metrics artifact boundary violation must fail closed");
    assert_eq!(error.kind, ProductErrorKind::BoundaryViolation);
    assert_eq!(error.field, "node_metrics");
}

#[test]
fn runtime_artifact_freshness_and_registry_identity_fail_closed() {
    let fixture = Fixture::new("registry-node-id-drift");
    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let mut registry = store.load().expect("fixture registry should load");
    registry
        .nodes
        .get_mut("mvp-node-001")
        .expect("fixture node should exist")
        .node_id = "other-node".to_string();
    store.save(&registry).expect("fixture registry should save");
    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("registry key and embedded node identity must agree");
    assert_eq!(error.kind, ProductErrorKind::SourceInvalid);
    assert_eq!(error.field, "registry_node_id");

    let fixture = Fixture::new("runtime-status-old");
    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let mut registry = store.load().expect("fixture registry should load");
    let record = registry
        .nodes
        .get_mut("mvp-node-001")
        .expect("fixture node should exist");
    let now = unix_time_ms();
    let mut status = record.last_known_status.clone();
    status.generated_at = SnapshotValue::available(
        now.saturating_sub(TEST_FRESHNESS_MAX_AGE_MS + 1)
            .to_string(),
    );
    write_json(&record.status_path, &status);
    record.status_artifact = RegistryArtifactState::Available;
    store.save(&registry).expect("fixture registry should save");
    let error = load_product_strategy(&fixture.state(), now)
        .expect_err("old status evidence must fail stale");
    assert_eq!(error.kind, ProductErrorKind::SourceStale);
    assert_eq!(error.field, "node_status");

    let fixture = Fixture::new("runtime-metrics-old");
    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let mut registry = store.load().expect("fixture registry should load");
    let record = registry
        .nodes
        .get_mut("mvp-node-001")
        .expect("fixture node should exist");
    let now = unix_time_ms();
    let mut metrics = NodeMetrics::from_status(
        &record.last_known_status,
        &NodeMetricArtifacts::from_record(record),
        NodeMetricCounts {
            uptime_ms: Some(1),
            starts_total: 1,
            stops_total: 0,
            state_transitions_total: 1,
        },
    );
    metrics.generated_at = SnapshotValue::available(
        now.saturating_sub(TEST_FRESHNESS_MAX_AGE_MS + 1)
            .to_string(),
    );
    write_json(&record.metrics_path, &metrics);
    record.metrics_artifact = RegistryArtifactState::Available;
    store.save(&registry).expect("fixture registry should save");
    let error = load_product_strategy(&fixture.state(), now)
        .expect_err("old metrics evidence must fail stale");
    assert_eq!(error.kind, ProductErrorKind::SourceStale);
    assert_eq!(error.field, "node_metrics");
}

#[test]
fn runtime_artifact_paths_must_remain_inside_the_registered_node_root() {
    let fixture = Fixture::new("artifact-root-escape");
    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let mut registry = store.load().expect("fixture registry should load");
    registry
        .nodes
        .get_mut("mvp-node-001")
        .expect("fixture node should exist")
        .artifact_root = fixture.root.join("external-node-root");
    fs::create_dir_all(fixture.root.join("external-node-root"))
        .expect("external artifact root should be created");
    store.save(&registry).expect("fixture registry should save");
    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("artifact root outside the canonical node root must fail closed");
    assert_eq!(error.kind, ProductErrorKind::SourceInvalid);
    assert_eq!(error.field, "artifact_root_containment");

    for (name, path_field, expected_field) in [
        (
            "status-path-escape",
            "status",
            "node_status_path_containment",
        ),
        (
            "metrics-path-escape",
            "metrics",
            "node_metrics_path_containment",
        ),
    ] {
        let fixture = Fixture::new(name);
        let store = SupervisorRegistryStore::new(&fixture.registry_path);
        let mut registry = store.load().expect("fixture registry should load");
        let record = registry
            .nodes
            .get_mut("mvp-node-001")
            .expect("fixture node should exist");
        let escaped_path = fixture.root.join(format!("external-{path_field}.json"));
        if path_field == "status" {
            record.status_path = escaped_path.clone();
        } else {
            record.metrics_path = escaped_path.clone();
        }
        store.save(&registry).expect("fixture registry should save");

        let error = load_product_strategy(&fixture.state(), unix_time_ms())
            .expect_err("runtime artifact path outside the node root must fail closed");
        assert_eq!(error.kind, ProductErrorKind::SourceInvalid, "{name}");
        assert_eq!(error.field, expected_field, "{name}");
    }
}

#[test]
fn nested_metrics_boundary_violations_fail_closed() {
    type MetricsMutation = fn(&mut NodeMetrics);
    let cases: [(&str, MetricsMutation); 6] = [
        ("submission-missing", |metrics| {
            metrics
                .kill_switch_dry_run
                .production_order_submission_allowed = SnapshotValue::unknown();
        }),
        ("submission-allowed", |metrics| {
            metrics
                .kill_switch_dry_run
                .production_order_submission_allowed = SnapshotValue::available(true);
        }),
        ("mutation-allowed", |metrics| {
            metrics
                .kill_switch_dry_run
                .production_order_mutation_allowed = SnapshotValue::available(true);
        }),
        ("dashboard-controls", |metrics| {
            metrics.kill_switch_dry_run.dashboard_order_controls_enabled =
                SnapshotValue::available(true);
        }),
        ("nested-real-orders", |metrics| {
            metrics.kill_switch_dry_run.real_orders_submitted = SnapshotValue::available(true);
        }),
        ("submitted-count", |metrics| {
            metrics.kill_switch_dry_run.production_orders_submitted = SnapshotValue::available(1);
        }),
    ];

    for (name, mutate) in cases {
        let fixture = Fixture::new(name);
        let store = SupervisorRegistryStore::new(&fixture.registry_path);
        let mut registry = store.load().expect("fixture registry should load");
        let record = registry
            .nodes
            .get_mut("mvp-node-001")
            .expect("fixture node should exist");
        let mut metrics = NodeMetrics::from_status(
            &record.last_known_status,
            &NodeMetricArtifacts::from_record(record),
            NodeMetricCounts {
                uptime_ms: Some(1),
                starts_total: 1,
                stops_total: 0,
                state_transitions_total: 1,
            },
        );
        metrics.generated_at = fresh_generated_at();
        mutate(&mut metrics);
        write_json(&record.metrics_path, &metrics);
        record.metrics_artifact = RegistryArtifactState::Available;
        store.save(&registry).expect("fixture registry should save");
        let error = load_product_strategy(&fixture.state(), unix_time_ms())
            .expect_err("nested runtime boundary violation must fail closed");
        assert_eq!(error.kind, ProductErrorKind::BoundaryViolation, "{name}");
        assert_eq!(error.field, "node_metrics", "{name}");
    }
}

#[test]
fn runtime_freshness_uses_mvp_configured_threshold() {
    let fixture = Fixture::new("configured-freshness");
    let now = unix_time_ms();
    let configured_max_age_ms = 30_000;
    let artifact_age_ms = 10_000;
    let mut status_contract = fixture.read_status_contract();
    status_contract.provenance.freshness_max_age_ms = configured_max_age_ms;
    status_contract.provenance.generated_at_unix_ms = now;
    fixture.write_status_contract(&status_contract);

    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let mut registry = store.load().expect("fixture registry should load");
    let record = registry
        .nodes
        .get_mut("mvp-node-001")
        .expect("fixture node should exist");
    let mut status = record.last_known_status.clone();
    status.generated_at = SnapshotValue::available(now.saturating_sub(artifact_age_ms).to_string());
    write_json(&record.status_path, &status);
    record.status_artifact = RegistryArtifactState::Available;
    store.save(&registry).expect("fixture registry should save");

    load_product_strategy(&fixture.state(), now)
        .expect("artifact inside the configured heartbeat window should remain fresh");
}

#[test]
fn runtime_artifact_registry_state_drift_fails_closed() {
    let fixture = Fixture::new("running-status-missing");
    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let mut registry = store.load().expect("fixture registry should load");
    registry
        .nodes
        .get_mut("mvp-node-001")
        .expect("fixture node should exist")
        .process
        .state = SupervisorProcessState::Running;
    store.save(&registry).expect("fixture registry should save");
    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("a running node cannot omit its status artifact");
    assert_eq!(error.kind, ProductErrorKind::SourceUnavailable);
    assert_eq!(error.field, "node_status");

    let fixture = Fixture::new("running-metrics-missing");
    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let mut registry = store.load().expect("fixture registry should load");
    let record = registry
        .nodes
        .get_mut("mvp-node-001")
        .expect("fixture node should exist");
    record.process.state = SupervisorProcessState::Running;
    let mut status = record.last_known_status.clone();
    status.generated_at = fresh_generated_at();
    write_json(&record.status_path, &status);
    record.status_artifact = RegistryArtifactState::Available;
    store.save(&registry).expect("fixture registry should save");
    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("a running node cannot omit its metrics artifact");
    assert_eq!(error.kind, ProductErrorKind::SourceUnavailable);
    assert_eq!(error.field, "node_metrics");

    let fixture = Fixture::new("runtime-status-disappeared");
    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let mut registry = store.load().expect("fixture registry should load");
    registry
        .nodes
        .get_mut("mvp-node-001")
        .expect("fixture node should exist")
        .status_artifact = RegistryArtifactState::Available;
    store.save(&registry).expect("fixture registry should save");
    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("an available status artifact cannot disappear");
    assert_eq!(error.kind, ProductErrorKind::SourceUnavailable);
    assert_eq!(error.field, "node_status");

    let fixture = Fixture::new("runtime-status-stale");
    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let mut registry = store.load().expect("fixture registry should load");
    registry
        .nodes
        .get_mut("mvp-node-001")
        .expect("fixture node should exist")
        .status_artifact = RegistryArtifactState::Stale;
    store.save(&registry).expect("fixture registry should save");
    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("a stale status artifact must fail closed");
    assert_eq!(error.kind, ProductErrorKind::SourceStale);
    assert_eq!(error.field, "node_status");

    let fixture = Fixture::new("runtime-status-unexpected-file");
    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let registry = store.load().expect("fixture registry should load");
    let record = registry
        .nodes
        .get("mvp-node-001")
        .expect("fixture node should exist");
    write_json(&record.status_path, &record.last_known_status);
    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("a missing status state cannot hide an existing artifact");
    assert_eq!(error.kind, ProductErrorKind::SourceInvalid);
    assert_eq!(error.field, "node_status");

    let fixture = Fixture::new("runtime-metrics-disappeared");
    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let mut registry = store.load().expect("fixture registry should load");
    registry
        .nodes
        .get_mut("mvp-node-001")
        .expect("fixture node should exist")
        .metrics_artifact = RegistryArtifactState::Available;
    store.save(&registry).expect("fixture registry should save");
    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("an available metrics artifact cannot disappear");
    assert_eq!(error.kind, ProductErrorKind::SourceUnavailable);
    assert_eq!(error.field, "node_metrics");
}

#[test]
fn stale_source_summary_does_not_misdiagnose_runtime_evidence_as_config_drift() {
    let (_, Json(body)) = product_error_response(
        &product_error(ProductErrorKind::SourceStale, "node_metrics"),
        "req-stale",
    );
    assert_eq!(body["error"]["code"], "product_source_stale");
    assert_eq!(body["error"]["retryable"], true);
    let summary = body["error"]["summary"]
        .as_str()
        .expect("stale error summary must be a string");
    assert!(summary.contains("数据源已过期"));
    assert!(!summary.contains("配置已变化"));
}

#[test]
fn missing_product_metadata_and_invalid_identity_ownership_fail_closed() {
    let fixture = Fixture::new("missing-product-metadata");
    fixture.write_config(&valid_config().replace("display_name = \"BTC/USDT EMA Cross\"\n", ""));
    let mut identity = fixture.identity.clone();
    identity.provenance.generated_at_unix_ms = unix_time_ms();
    fixture.write_identity(&identity);
    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("missing product metadata must fail");
    assert_eq!(error.kind, ProductErrorKind::SourceInvalid);
    assert_eq!(error.field, "strategy_config");

    fixture.write_config(valid_config());
    let mut identity = fixture.identity.clone();
    identity.identities.strategy_instance_id = identity.identities.node_id.clone();
    identity.contract_id = format!(
        "{}:{}:{}",
        identity.identities.node_id,
        identity.identities.strategy_id,
        identity.identities.strategy_instance_id
    );
    identity.provenance.generated_at_unix_ms = unix_time_ms();
    fixture.write_identity(&identity);
    let error = load_product_strategy(&fixture.state(), unix_time_ms())
        .expect_err("node and strategy instance ownership must remain distinct");
    assert_eq!(error.kind, ProductErrorKind::SourceInvalid);
    assert_eq!(error.field, "identity_ownership");
}

#[test]
fn complete_identity_drift_from_registered_config_fails_closed() {
    type IdentityMutation = fn(&mut MvpIdentityContract);

    let fixture = Fixture::new("complete-identity-drift");
    let cases: [(&str, IdentityMutation); 5] = [
        ("environment", |identity: &mut MvpIdentityContract| {
            identity.identities.environment = "production".to_string();
        }),
        ("venue_id", |identity: &mut MvpIdentityContract| {
            identity.identities.venue_id = "OTHER".to_string();
        }),
        ("account_id", |identity: &mut MvpIdentityContract| {
            identity.identities.account_id = "other-account".to_string();
        }),
        ("backtest_run_id", |identity: &mut MvpIdentityContract| {
            identity.identities.backtest_run_id = "other-run".to_string();
        }),
        (
            "backtest_result_ref",
            |identity: &mut MvpIdentityContract| {
                identity.identities.backtest_result_ref =
                    "artifact://other/result.json".to_string();
            },
        ),
    ];
    for (field, mutate) in cases {
        let mut identity = fixture.identity.clone();
        mutate(&mut identity);
        fixture.write_identity(&identity);
        let error = load_product_strategy(&fixture.state(), unix_time_ms())
            .expect_err("complete identity drift must fail");
        assert_eq!(error.kind, ProductErrorKind::SourceInvalid, "{field}");
        assert_eq!(error.field, "config_projection", "{field}");
    }
}

#[tokio::test]
async fn strategy_routes_are_read_only_and_return_stable_envelopes() {
    let fixture = Fixture::new("routes");
    let router = fixture.router();

    let (status, list) = router_json(&router, Method::GET, "/api/product/v1/strategies").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        list["schema_version"],
        "ntpro.product_api.strategy_list.response.v1"
    );
    assert_eq!(list["data"][0]["strategy_id"], "ema-cross");
    assert_read_only_boundaries(&list);
    validate_openapi_instance("StrategyListResponse", &list);

    let (status, detail) =
        router_json(&router, Method::GET, "/api/product/v1/strategies/ema-cross").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["data"]["default_version_id"], "ema-cross@v1");
    assert_read_only_boundaries(&detail);
    validate_openapi_instance("StrategyDetailResponse", &detail);

    let (status, missing) =
        router_json(&router, Method::GET, "/api/product/v1/strategies/missing").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(missing["error"]["code"], "strategy_not_found");
    assert_eq!(missing["error"]["retryable"], false);
    validate_openapi_instance("ProductErrorResponse", &missing);

    let (status, malformed_path) =
        router_json(&router, Method::GET, "/api/product/v1/strategies/%FF").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(malformed_path["error"]["code"], "product_query_invalid");
    assert_eq!(malformed_path["error"]["field"], "strategy_id");
    validate_openapi_instance("ProductErrorResponse", &malformed_path);

    let (status, invalid) = router_json(
        &router,
        Method::GET,
        "/api/product/v1/strategies?limit=1&limit=2",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(invalid["error"]["code"], "product_query_invalid");
    validate_openapi_instance("ProductErrorResponse", &invalid);

    let unavailable_router = dashboard_router(
        fixture.root.join("missing/supervisor/registry.json"),
        PathBuf::from("missing-ntpro-node"),
    );
    let (status, invalid_cursor) = router_json(
        &unavailable_router,
        Method::GET,
        "/api/product/v1/strategies?cursor=forged",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(invalid_cursor["error"]["code"], "product_query_invalid");
    validate_openapi_instance("ProductErrorResponse", &invalid_cursor);

    for path in [
        "/api/product/v1/strategies",
        "/api/product/v1/strategies/ema-cross",
    ] {
        for method in [
            Method::HEAD,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
            Method::CONNECT,
            Method::TRACE,
        ] {
            let (status, headers, body) =
                router_json_with_headers(&router, method.clone(), path).await;
            assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED, "{method} {path}");
            assert_eq!(
                headers.get(ALLOW).and_then(|value| value.to_str().ok()),
                Some("GET")
            );
            if method == Method::HEAD {
                assert_eq!(body, json!({}), "HEAD responses must not include a body");
                continue;
            }
            assert_eq!(body["error"]["code"], "product_method_not_allowed");
            assert_eq!(body["error"]["retryable"], false);
            assert!(
                body["request_id"]
                    .as_str()
                    .is_some_and(|value| !value.is_empty())
            );
            validate_openapi_instance("ProductErrorResponse", &body);
        }
    }
}

#[tokio::test]
async fn missing_registry_is_retryable_source_unavailable() {
    let fixture = Fixture::new("missing-registry");
    let router = fixture.router();
    fs::remove_file(&fixture.registry_path).expect("fixture registry should be removed");

    let (status, body) = router_json(&router, Method::GET, "/api/product/v1/strategies").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"]["code"], "product_source_unavailable");
    assert_eq!(body["error"]["field"], "registry");
    assert_eq!(body["error"]["retryable"], true);
    validate_openapi_instance("ProductErrorResponse", &body);
}

#[tokio::test]
async fn malformed_registry_is_nonretryable_source_invalid() {
    let fixture = Fixture::new("malformed-registry");
    let router = fixture.router();
    fs::write(&fixture.registry_path, b"{not-json")
        .expect("malformed registry fixture should be written");

    let (status, body) = router_json(&router, Method::GET, "/api/product/v1/strategies").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"]["code"], "product_source_invalid");
    assert_eq!(body["error"]["field"], "registry");
    assert_eq!(body["error"]["retryable"], false);
    validate_openapi_instance("ProductErrorResponse", &body);
}

#[test]
fn openapi_is_authoritative_and_declares_exact_strategy_routes() {
    let openapi: Value =
        serde_json::from_str(PRODUCT_OPENAPI_SOURCE).expect("OpenAPI source must be valid JSON");
    assert_eq!(openapi["openapi"], "3.1.0");
    assert_eq!(openapi["x-ntpro-authoritative-contract"], true);
    assert_eq!(
        openapi["components"]["securitySchemes"]
            .as_object()
            .expect("OpenAPI security schemes must be an object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["InstitutionCookie", "OperatorCookie"]
    );
    let paths = openapi["paths"]
        .as_object()
        .expect("OpenAPI paths must be an object");
    assert_eq!(
        paths.keys().map(String::as_str).collect::<Vec<_>>(),
        ["/strategies", "/strategies/{strategy_id}"]
    );
    for path in paths.values() {
        let methods = path.as_object().expect("path item must be an object");
        assert_eq!(
            methods.keys().map(String::as_str).collect::<Vec<_>>(),
            ["get"]
        );
        let operation = &path["get"];
        assert_eq!(
            operation["security"],
            json!([{"InstitutionCookie": []}, {"OperatorCookie": []}])
        );
        assert_eq!(
            operation["responses"]["403"]["$ref"],
            "#/components/responses/ProductError"
        );
        assert_eq!(
            operation["responses"]["405"]["$ref"],
            "#/components/responses/ProductMethodNotAllowed"
        );
    }
}

fn valid_config() -> &'static str {
    r#"[node]
node_id = "mvp-strategy-001"

[strategy]
strategy_id = "ema-cross"
display_name = "BTC/USDT EMA Cross"
description = "BTC/USDT EMA 交叉策略"
owner = "Systematic Desk"
lifecycle = "active"
created_at_unix_ms = 1767225600000
updated_at_unix_ms = 1785542400000

[market]
venue = "BINANCE"

[execution]
venue = "BINANCE"

[mvp]
strategy_version = "v1"
backtest_run_id = "backtest-001"
backtest_result_ref = "artifact://backtests/backtest-001.json"
account_id = "acct-sandbox-001"
environment = "sandbox"
"#
}

fn valid_identity(config_path: &Path, generated_at_unix_ms: u64) -> MvpIdentityContract {
    MvpIdentityContract {
        schema_version: MVP_IDENTITY_CONTRACT_SCHEMA_VERSION.to_string(),
        contract_id: "mvp-node-001:ema-cross:mvp-strategy-001".to_string(),
        identities: MvpIdentitySet {
            strategy_id: "ema-cross".to_string(),
            strategy_version: "v1".to_string(),
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
            generated_at_unix_ms,
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
    }
}

fn write_json(path: &Path, value: &impl serde::Serialize) {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("fixture must serialize"),
    )
    .expect("fixture JSON should be written");
}

fn set_modified_time(path: &Path, modified: std::time::SystemTime) {
    OpenOptions::new()
        .write(true)
        .open(path)
        .expect("fixture file should open for timestamp update")
        .set_times(FileTimes::new().set_modified(modified))
        .expect("fixture timestamp should update");
}

fn fresh_generated_at() -> SnapshotValue<String> {
    SnapshotValue::available(unix_time_ms().to_string())
}

async fn router_json(router: &Router, method: Method, path: &str) -> (StatusCode, Value) {
    let (status, _, value) = router_json_with_headers(router, method, path).await;
    (status, value)
}

async fn router_json_with_headers(
    router: &Router,
    method: Method,
    path: &str,
) -> (StatusCode, HeaderMap, Value) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("router request should complete");
    let status = response.status();
    let headers = response.headers().clone();
    let body = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .expect("response body should be readable");
    let value = if body.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&body).expect("response should be valid JSON")
    };
    (status, headers, value)
}

fn assert_read_only_boundaries(value: &Value) {
    assert_eq!(value["boundaries"]["read_only"], true);
    for field in [
        "strategy_mutation_allowed",
        "run_mutation_allowed",
        "external_venue_connection",
        "order_submission_allowed",
        "order_mutation_allowed",
        "automatic_retry_allowed",
        "automatic_remediation_allowed",
        "real_orders_submitted",
        "trading_controls_enabled",
    ] {
        assert_eq!(value["boundaries"][field], false, "{field}");
    }
}

fn validate_openapi_instance(schema_name: &str, instance: &Value) {
    let openapi: Value =
        serde_json::from_str(PRODUCT_OPENAPI_SOURCE).expect("OpenAPI source must be valid JSON");
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$ref": format!("#/components/schemas/{schema_name}"),
        "components": openapi["components"].clone(),
    });
    jsonschema::draft202012::meta::validate(&schema)
        .expect("extracted OpenAPI schema must be valid Draft 2020-12");
    let validator =
        jsonschema::draft202012::new(&schema).expect("OpenAPI response validator should build");
    if let Err(error) = validator.validate(instance) {
        panic!("{schema_name} response must match OpenAPI: {error}");
    }
}
