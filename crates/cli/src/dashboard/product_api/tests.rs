use std::{
    fs::{self, FileTimes, OpenOptions},
    path::{Path, PathBuf},
    time::{Duration, UNIX_EPOCH},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{HeaderMap, Method, Request, StatusCode, header::ALLOW},
};
use nautilus_live::status::{LifecycleStatus, SnapshotValue};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::{
    mvp_contract::{
        MVP_IDENTITY_CONTRACT_PATH, MVP_STATUS_CONTRACT_PATH, MvpIdentityBoundaries,
        MvpIdentityProvenance, MvpIdentitySet, MvpStatusContract,
    },
    supervisor::{
        NodeMetricArtifacts, NodeMetricCounts, NodeMetrics, RegisterNodeRequest, StartNodeRequest,
        StopNodeRequest, SupervisorProcessState, SupervisorRegistryStore, SupervisorRunOwnership,
        SupervisorRunTerminalAnchor,
    },
};

type AnalysisMutation = fn(&mut Value);

use super::run::*;
use super::strategy_version::*;
use super::*;
use crate::dashboard::server::{dashboard_router, dashboard_router_with_access};

#[cfg(unix)]
fn create_file_symlink(original: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}

#[cfg(windows)]
fn create_file_symlink(original: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(original, link)
}

#[cfg(unix)]
fn create_directory_symlink(original: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}

#[cfg(windows)]
fn create_directory_symlink(original: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(original, link)
}

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
        let result_sha256 = write_valid_backtest_artifact(&root, &config_path);
        let details_sha256 = write_valid_backtest_details_artifact(&root, &config_path);
        let analysis_sha256 = write_valid_backtest_analysis_artifact(
            &root,
            &config_path,
            &result_sha256,
            &details_sha256,
        );
        fs::write(
            &config_path,
            with_backtest_analysis_sha256(
                &with_backtest_details_sha256(
                    &with_backtest_result_sha256(&valid_config(), &result_sha256),
                    &details_sha256,
                ),
                &analysis_sha256,
            ),
        )
        .expect("trusted result hash should be written to product config");
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
            backtest_creation_gate: std::sync::Arc::new(tokio::sync::Semaphore::new(1)),
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

    fn refresh_identity_and_status_provenance(&self) {
        let mut identity = self.read_identity();
        identity.provenance.generated_at_unix_ms = unix_time_ms().saturating_add(1_000);
        self.write_identity(&identity);
        let store = SupervisorRegistryStore::new(&self.registry_path);
        let registry = store.load().expect("fixture registry should load");
        let record = registry
            .nodes
            .get("mvp-node-001")
            .expect("fixture node should exist");
        self.write_status_contract(&MvpStatusContract::from_runtime(
            &identity,
            &self.identity_path,
            &self.registry_path,
            record,
            None,
            None,
            None,
            None,
            TEST_FRESHNESS_MAX_AGE_MS,
        ));
    }

    fn activate_strategy_version(&self, version: &str) {
        let current = fs::read_to_string(&self.config_path)
            .expect("product config fixture should be readable");
        let current_version = self.read_identity().identities.strategy_version;
        let current_id = format!("ema-cross@{current_version}");
        let next_id = format!("ema-cross@{version}");
        let candidate = current
            .replace(&current_id, &next_id)
            .replace(
                &format!("version = \"{current_version}\""),
                &format!("version = \"{version}\""),
            )
            .replace(
                &format!("strategy_version = \"{current_version}\""),
                &format!("strategy_version = \"{version}\""),
            );
        let config = config_with_computed_version_hash(&candidate);
        self.write_config(&config);
        let mut identity = self.read_identity();
        identity.identities.strategy_version = version.to_string();
        identity.identities.strategy_version_content_hash =
            strategy_version_content_hash(&self.config_path);
        identity.provenance.generated_at_unix_ms = unix_time_ms();
        self.write_identity(&identity);
        let store = SupervisorRegistryStore::new(&self.registry_path);
        let registry = store.load().expect("fixture registry should load");
        let record = registry
            .nodes
            .get("mvp-node-001")
            .expect("fixture node should exist");
        let status = MvpStatusContract::from_runtime(
            &identity,
            &self.identity_path,
            &self.registry_path,
            record,
            None,
            None,
            None,
            None,
            TEST_FRESHNESS_MAX_AGE_MS,
        );
        self.write_status_contract(&status);
    }

    fn read_identity(&self) -> MvpIdentityContract {
        serde_json::from_slice(
            &fs::read(&self.identity_path).expect("identity fixture should be readable"),
        )
        .expect("identity fixture should parse")
    }

    fn trust_current_backtest_result(&self) {
        let result_path = self
            .root
            .join("artifacts/backtests/backtest-001/summary.json");
        let result_sha256 = sha256_bytes_ref(
            &fs::read(result_path).expect("backtest result fixture should be readable"),
        );
        let config = fs::read_to_string(&self.config_path)
            .expect("product config fixture should be readable");
        self.write_config(&with_backtest_result_sha256(&config, &result_sha256));
        let mut identity = self.identity.clone();
        identity.provenance.generated_at_unix_ms = unix_time_ms().saturating_add(1_000);
        self.write_identity(&identity);
    }

    fn trust_current_backtest_details(&self) {
        let details_path = self
            .root
            .join("artifacts/backtests/backtest-001/details.json");
        let details_sha256 = sha256_bytes_ref(
            &fs::read(details_path).expect("backtest details fixture should be readable"),
        );
        let config = fs::read_to_string(&self.config_path)
            .expect("product config fixture should be readable");
        self.write_config(&with_backtest_details_sha256(&config, &details_sha256));
        let mut identity = self.identity.clone();
        identity.provenance.generated_at_unix_ms = unix_time_ms().saturating_add(1_000);
        self.write_identity(&identity);
    }

    fn trust_current_backtest_analysis(&self) {
        let analysis_path = self
            .root
            .join("artifacts/backtests/backtest-001/analysis.json");
        let analysis_sha256 = sha256_bytes_ref(
            &fs::read(analysis_path).expect("backtest analysis fixture should be readable"),
        );
        let config = fs::read_to_string(&self.config_path)
            .expect("product config fixture should be readable");
        self.write_config(&with_backtest_analysis_sha256(&config, &analysis_sha256));
        let mut identity = self.identity.clone();
        identity.provenance.generated_at_unix_ms = unix_time_ms().saturating_add(1_000);
        self.write_identity(&identity);
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
fn strategy_version_projection_exposes_immutable_product_contract() {
    let fixture = Fixture::new("version-projection");
    let source = load_product_source(&fixture.state(), unix_time_ms())
        .expect("valid product source should load");
    let version = load_product_strategy_version(&source, unix_time_ms())
        .expect("valid strategy version should project");
    let value = serde_json::to_value(version).expect("version should serialize");

    assert_eq!(value["strategy_version_id"], "ema-cross@v1");
    assert_eq!(value["strategy_id"], "ema-cross");
    assert_eq!(value["version"], "v1");
    assert_eq!(
        value["code_ref"],
        "git://NTPRO@e24de1825b66f9e7b9bfb2fc4662c928e56d6c18/crates/cli/src/strategy_session.rs#ema_cross_demo"
    );
    assert!(
        value["content_hash"]
            .as_str()
            .is_some_and(|hash| hash.starts_with("sha256:") && hash.len() == 71)
    );
    assert_eq!(value["parameter_schema"]["additionalProperties"], false);
    assert_eq!(
        value["data_requirements"]["deterministic_replay_required"],
        true
    );
    assert_eq!(value["risk_config"]["kill_switch_required"], true);
    assert_eq!(value["risk_config"]["order_submission_default"], false);
    assert_eq!(value["status"], "registered");
    assert_eq!(value["source"]["source_type"], "strategy_version_manifest");
}

#[test]
fn run_projection_exposes_static_backtest_and_blocked_live_with_closed_capabilities() {
    let fixture = Fixture::new("run-projection");
    let runs = load_product_runs(&fixture.state(), unix_time_ms())
        .expect("valid run manifest should project");
    let value = serde_json::to_value(runs).expect("runs should serialize");

    assert_eq!(value.as_array().map(Vec::len), Some(2));
    assert_eq!(value[0]["environment"], "backtest");
    assert_eq!(value[0]["lifecycle"], "completed");
    assert_eq!(value[0]["result"]["status"], "available");
    assert_eq!(value[1]["environment"], "live");
    assert_eq!(value[1]["lifecycle"], "created");
    assert_eq!(value[1]["risk"]["status"], "blocked");
    assert_eq!(value[1]["adapter_ref"], "adapter://live/disabled");
    for run in value.as_array().expect("runs should be an array") {
        assert_eq!(run["strategy_id"], "ema-cross");
        assert_eq!(run["strategy_version_id"], "ema-cross@v1");
        assert_eq!(run["source"]["freshness_status"], "fresh");
        for field in [
            "external_venue_connection",
            "order_submission_allowed",
            "order_mutation_allowed",
            "automatic_retry_allowed",
            "automatic_remediation_allowed",
            "real_orders_submitted",
            "trading_controls_enabled",
        ] {
            assert_eq!(run["capabilities"][field], false, "{field}");
        }
    }
}

#[test]
fn run_list_supports_identity_environment_lifecycle_and_stable_cursor() {
    let fixture = Fixture::new("run-list");
    let runs = load_product_runs(&fixture.state(), unix_time_ms())
        .expect("valid run manifest should project");
    let query = parse_run_list_query(Some(
        "limit=1&sort=created_at&order=asc&strategy_id=ema-cross&strategy_version_id=ema-cross%40v1&environment=backtest&lifecycle=completed",
    ))
    .expect("valid run query should parse");
    let response = project_run_list(runs.clone(), &query, product_request_id())
        .expect("filtered run list should project");
    let value = serde_json::to_value(response).expect("run list should serialize");
    assert_eq!(value["page"]["returned_count"], 1);
    assert_eq!(value["data"][0]["run_id"], "backtest-001");

    let first_page = parse_run_list_query(Some("limit=1"))
        .and_then(|query| project_run_list(runs.clone(), &query, product_request_id()))
        .expect("first run page should project");
    let first_value = serde_json::to_value(first_page).expect("first page should serialize");
    assert_eq!(first_value["page"]["has_more"], true);
    let cursor = first_value["page"]["next_cursor"]
        .as_str()
        .expect("next cursor should exist")
        .to_string();
    assert_eq!(cursor, encode_run_cursor("backtest-001"));
    let second_page = parse_run_list_query(Some(&format!("limit=1&cursor={cursor}")))
        .and_then(|query| project_run_list(runs, &query, product_request_id()))
        .expect("second run page should project");
    let second_value = serde_json::to_value(second_page).expect("second page should serialize");
    assert_eq!(second_value["data"][0]["run_id"], "ema-cross-live-001");
}

#[test]
fn invalid_run_queries_and_manifest_drift_fail_closed() {
    for query in [
        "limit=0",
        "limit=101",
        "limit=1&limit=2",
        "cursor=forged",
        "sort=environment",
        "order=random",
        "strategy_id=..",
        "strategy_version_id=ema-cross",
        "environment=production",
        "lifecycle=unknown",
        "unknown=value",
    ] {
        assert!(
            parse_run_list_query(Some(query)).is_err(),
            "{query} must fail closed"
        );
    }

    for (name, mutate, expected_kind, expected_field) in [
        (
            "ownership",
            (|raw: String| {
                raw.replace(
                    "run_id = \"backtest-001\"\nstrategy_id = \"ema-cross\"\nstrategy_version_id = \"ema-cross@v1\"",
                    "run_id = \"backtest-001\"\nstrategy_id = \"ema-cross\"\nstrategy_version_id = \"other@v1\"",
                )
            }) as fn(String) -> String,
            ProductErrorKind::SourceInvalid,
            "run_ownership",
        ),
        (
            "environment",
            (|raw: String| {
                raw.replacen(
                    "environment = \"backtest\"",
                    "environment = \"production\"",
                    1,
                )
            }) as fn(String) -> String,
            ProductErrorKind::SourceInvalid,
            "run_manifest",
        ),
        (
            "partial-backtest-expectation",
            (|raw: String| raw.replace("backtest_slow_period = 5\n", "")) as fn(String) -> String,
            ProductErrorKind::SourceInvalid,
            "run_expectation",
        ),
        (
            "invalid-backtest-hash",
            (|raw: String| {
                raw.replace(
                    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                )
            }) as fn(String) -> String,
            ProductErrorKind::SourceInvalid,
            "run_expectation",
        ),
        (
            "invalid-backtest-result-hash",
            (|raw: String| {
                raw.replace(
                    "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
                    "sha256:CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC",
                )
            }) as fn(String) -> String,
            ProductErrorKind::SourceInvalid,
            "run_expectation",
        ),
        (
            "live-reference",
            (|raw: String| raw.replace("adapter://live/disabled", "adapter://live/binance"))
                as fn(String) -> String,
            ProductErrorKind::BoundaryViolation,
            "run_environment_references",
        ),
        (
            "live-data-reference",
            (|raw: String| {
                raw.replace("market://live/disabled", "market://live/disabled/connected")
            }) as fn(String) -> String,
            ProductErrorKind::BoundaryViolation,
            "run_environment_references",
        ),
        (
            "backtest-data-reference",
            (|raw: String| {
                raw.replace("dataset://fixtures/ema-cross", "dataset://fixtures/unknown")
            }) as fn(String) -> String,
            ProductErrorKind::BoundaryViolation,
            "run_environment_references",
        ),
        (
            "backtest-adapter-reference",
            (|raw: String| {
                raw.replace("adapter://backtest/simulated", "adapter://backtest/unknown")
            }) as fn(String) -> String,
            ProductErrorKind::BoundaryViolation,
            "run_environment_references",
        ),
        (
            "backtest-account-reference",
            (|raw: String| {
                raw.replace(
                    "account://simulated/backtest-001",
                    "account://simulated/unknown",
                )
            }) as fn(String) -> String,
            ProductErrorKind::BoundaryViolation,
            "run_environment_references",
        ),
        (
            "backtest-venue-reference",
            (|raw: String| raw.replace("venue://simulated/BINANCE", "venue://simulated/UNKNOWN"))
                as fn(String) -> String,
            ProductErrorKind::BoundaryViolation,
            "run_environment_references",
        ),
        (
            "capability",
            (|raw: String| {
                raw.replacen(
                    "order_submission_allowed = false",
                    "order_submission_allowed = true",
                    1,
                )
            }) as fn(String) -> String,
            ProductErrorKind::BoundaryViolation,
            "run_capabilities",
        ),
        (
            "timestamp",
            (|raw: String| {
                raw.replacen(
                    "completed_at_unix_ms = 1767225660000",
                    "completed_at_unix_ms = 1767225599999",
                    1,
                )
            }) as fn(String) -> String,
            ProductErrorKind::SourceInvalid,
            "run_timestamps",
        ),
        (
            "run-before-strategy-version",
            (|raw: String| {
                raw.replacen(
                    "created_at_unix_ms = 1767225600000\nstarted_at_unix_ms = 1767225600000",
                    "created_at_unix_ms = 1767225599999\nstarted_at_unix_ms = 1767225600000",
                    1,
                )
            }) as fn(String) -> String,
            ProductErrorKind::SourceInvalid,
            "run_timestamps",
        ),
    ] {
        let fixture = Fixture::new(&format!("run-drift-{name}"));
        fixture.write_config(&mutate(valid_config()));
        let mut identity = fixture.identity.clone();
        identity.provenance.generated_at_unix_ms = unix_time_ms().saturating_add(1_000);
        fixture.write_identity(&identity);
        let error = load_product_runs(&fixture.state(), unix_time_ms())
            .expect_err("invalid run manifest must fail closed");
        assert_eq!(error.kind, expected_kind, "{name}");
        assert_eq!(error.field, expected_field, "{name}");
    }
}

#[test]
fn tracked_strategy_version_and_run_manifest_match_authoritative_identity() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../configs/nodes/btc-ema-shadow.toml");
    let raw = fs::read_to_string(path).expect("tracked node config should be readable");
    let config: toml::Value = toml::from_str(&raw).expect("tracked node config should parse");
    let declared = config["strategy_version"]["content_hash"]
        .as_str()
        .expect("tracked version hash should be a string");
    assert_eq!(declared, computed_strategy_version_hash(&raw));

    let backtest_run_id = config["mvp"]["backtest_run_id"]
        .as_str()
        .expect("tracked backtest identity should exist");
    let backtest_run = config["product_runs"]
        .as_array()
        .expect("tracked Run manifest should be an array")
        .iter()
        .find(|run| run["environment"].as_str() == Some("backtest"))
        .expect("tracked Backtest Run should exist");
    assert_eq!(backtest_run["run_id"].as_str(), Some(backtest_run_id));
    let expected_account_ref = format!("account://simulated/{backtest_run_id}");
    assert_eq!(
        backtest_run["account_ref"].as_str(),
        Some(expected_account_ref.as_str())
    );
    let backtest_config = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/backtests/ema-cross-btcusdt-product.toml");
    let expected_config_hash = sha256_bytes_ref(
        &fs::read(backtest_config).expect("tracked Backtest config should be readable"),
    );
    assert_eq!(
        backtest_run["backtest_config_sha256"].as_str(),
        Some(expected_config_hash.as_str())
    );
    assert_eq!(
        backtest_run["backtest_data_sha256"].as_str(),
        Some("sha256:18ed30b352b17a11c33294df39387976f15a587b859f729ffbe5e59bc9c75d1e")
    );
    assert_eq!(
        backtest_run["backtest_result_sha256"].as_str(),
        Some("sha256:4b9bc548f226e55b136eb4c08f2ef5e0274bed104b8626d5431b39fb0a3b8760")
    );
    assert_eq!(
        backtest_run["backtest_trade_size"].as_str(),
        Some("0.001000")
    );
    assert_eq!(backtest_run["backtest_quotes"].as_integer(), Some(120));
    assert_eq!(backtest_run["backtest_fast_period"].as_integer(), Some(3));
    assert_eq!(backtest_run["backtest_slow_period"].as_integer(), Some(5));
}

fn sha256_bytes_ref(bytes: &[u8]) -> String {
    use aws_lc_rs::digest::{SHA256, digest};
    use std::fmt::Write as _;

    let mut value = String::from("sha256:");
    for byte in digest(&SHA256, bytes).as_ref() {
        write!(&mut value, "{byte:02x}").expect("writing to a String cannot fail");
    }
    value
}

#[test]
fn strategy_version_list_supports_filter_sort_and_stable_cursor() {
    let fixture = Fixture::new("version-list");
    let source = load_product_source(&fixture.state(), unix_time_ms())
        .expect("valid product source should load");
    let version = load_product_strategy_version(&source, unix_time_ms())
        .expect("valid strategy version should project");
    let query = parse_strategy_version_list_query(Some(
        "limit=1&sort=created_at&order=desc&status=registered",
    ))
    .expect("valid version query should parse");
    let response = project_strategy_version_list(version.clone(), &query, product_request_id())
        .expect("version list should project");
    let value = serde_json::to_value(response).expect("response should serialize");
    assert_eq!(value["page"]["returned_count"], 1);
    assert_eq!(value["data"][0]["strategy_version_id"], "ema-cross@v1");

    let cursor = encode_version_cursor("ema-cross@v1");
    let query = parse_strategy_version_list_query(Some(&format!("cursor={cursor}")))
        .expect("known version cursor should parse");
    let response = project_strategy_version_list(version, &query, product_request_id())
        .expect("cursor should address the next page");
    assert!(
        serde_json::to_value(response).expect("response should serialize")["data"]
            .as_array()
            .is_some_and(Vec::is_empty)
    );
}

#[test]
fn invalid_strategy_version_queries_and_identifiers_fail_closed() {
    for query in [
        "limit=0",
        "limit=101",
        "limit=1&limit=2",
        "cursor=forged",
        "sort=updated_at",
        "order=random",
        "status=deprecated",
        "unknown=value",
    ] {
        assert!(
            parse_strategy_version_list_query(Some(query)).is_err(),
            "{query} must fail closed"
        );
    }
    for version_id in ["", ".", "..", "ema-cross", "@v1", "ema-cross@", "a@b@c"] {
        assert!(
            validate_requested_version_id("version_id", version_id).is_err(),
            "{version_id} must fail closed"
        );
    }
    assert!(
        validate_requested_version_id("version_id", &format!("ema-cross@{}", "v".repeat(129)))
            .is_err()
    );
    assert!(
        validate_requested_version_id("version_id", &format!("{}@v1", "s".repeat(129))).is_err()
    );
}

#[test]
fn strategy_version_hash_code_schema_and_identity_drift_fail_closed() {
    for (name, mutate, expected_field) in [
        (
            "code-ref",
            (|raw: String| raw.replace("#ema_cross_demo", "#ema_cross_changed"))
                as fn(String) -> String,
            "strategy_version_code_ref",
        ),
        (
            "parameter-schema",
            (|raw: String| raw.replace("const = 3", "const = 4")) as fn(String) -> String,
            "strategy_version_content_hash",
        ),
        (
            "risk-boundary",
            (|raw: String| {
                raw.replace(
                    "order_submission_default = false",
                    "order_submission_default = true",
                )
            }) as fn(String) -> String,
            "strategy_version_risk_config",
        ),
    ] {
        let fixture = Fixture::new(&format!("version-drift-{name}"));
        fixture.write_config(&mutate(valid_config()));
        let mut identity = fixture.identity.clone();
        identity.provenance.generated_at_unix_ms = unix_time_ms().saturating_add(1_000);
        fixture.write_identity(&identity);
        let source = load_product_source(&fixture.state(), unix_time_ms())
            .expect("declared identity hash should still anchor the source snapshot");
        let error = load_product_strategy_version(&source, unix_time_ms())
            .expect_err("immutable version content drift must fail");
        assert_eq!(error.field, expected_field, "{name}");
    }

    let fixture = Fixture::new("version-self-rehash");
    let changed = valid_config().replace("#ema_cross_demo", "#ema_cross_changed");
    fixture.write_config(&config_with_computed_version_hash(&changed));
    let mut identity = fixture.identity.clone();
    identity.provenance.generated_at_unix_ms = unix_time_ms().saturating_add(1_000);
    fixture.write_identity(&identity);
    let error = load_product_source(&fixture.state(), unix_time_ms())
        .expect_err("re-hashing a registered version must not bypass its identity anchor");
    assert_eq!(error.kind, ProductErrorKind::SourceInvalid);
    assert_eq!(error.field, "config_projection");
}

#[test]
fn strategy_version_parameter_schema_rejects_invalid_draft_keyword_value() {
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "additionalProperties": false,
        "required": ["fast_period"],
        "properties": {
            "fast_period": {"type": "interger"}
        }
    });
    let error = validate_parameter_schema(&schema)
        .expect_err("unknown JSON Schema type must fail Draft 2020-12 meta validation");
    assert_eq!(error.kind, ProductErrorKind::SourceInvalid);
    assert_eq!(error.field, "strategy_version_parameter_schema");
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
fn terminal_demo_snapshot_remains_readable_without_weakening_boundaries() {
    let fixture = Fixture::new("terminal-snapshot-readback");
    let now = unix_time_ms().saturating_add(TEST_FRESHNESS_MAX_AGE_MS + 2_000);
    let terminal_at = now.saturating_sub(TEST_FRESHNESS_MAX_AGE_MS + 1_000);
    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let mut registry = store.load().expect("fixture registry should load");
    let record = registry
        .nodes
        .get_mut("mvp-node-001")
        .expect("fixture node should exist");
    record.process.state = SupervisorProcessState::Stopped;
    record.last_known_status.lifecycle_state = LifecycleStatus::Stopped;
    let mut status = record.last_known_status.clone();
    status.generated_at = SnapshotValue::available(terminal_at.to_string());
    write_json(&record.status_path, &status);
    record.status_artifact = RegistryArtifactState::Available;

    let mut metrics = NodeMetrics::from_status(
        &status,
        &NodeMetricArtifacts::from_record(record),
        NodeMetricCounts {
            uptime_ms: Some(1),
            starts_total: 1,
            stops_total: 1,
            state_transitions_total: 2,
        },
    );
    metrics.generated_at = SnapshotValue::available(terminal_at.to_string());
    metrics
        .kill_switch_dry_run
        .production_order_submission_allowed = SnapshotValue::available(false);
    metrics
        .kill_switch_dry_run
        .production_order_mutation_allowed = SnapshotValue::available(false);
    metrics.kill_switch_dry_run.dashboard_order_controls_enabled = SnapshotValue::available(false);
    metrics.kill_switch_dry_run.real_orders_submitted = SnapshotValue::available(false);
    metrics.kill_switch_dry_run.production_orders_submitted = SnapshotValue::available(0);
    write_json(&record.metrics_path, &metrics);
    let metrics_path = record.metrics_path.clone();
    record.metrics_artifact = RegistryArtifactState::Available;
    record.run_ownership.insert(
        "demo-terminal-001".to_string(),
        SupervisorRunOwnership {
            run_id: "demo-terminal-001".to_string(),
            manifest_sha256: format!("sha256:{}", "a".repeat(64)),
            claimed_at_unix_ms: terminal_at.saturating_sub(1),
            terminal: Some(SupervisorRunTerminalAnchor {
                lifecycle: "stopped".to_string(),
                terminal_state_sha256: format!("sha256:{}", "b".repeat(64)),
                completed_at_unix_ms: terminal_at,
            }),
        },
    );
    store.save(&registry).expect("fixture registry should save");
    let mut status_contract = fixture.read_status_contract();
    status_contract.provenance.generated_at_unix_ms = terminal_at;
    fixture.write_status_contract(&status_contract);

    load_product_strategy(&fixture.state(), now)
        .expect("an anchored stopped Demo snapshot should remain readable");

    metrics
        .kill_switch_dry_run
        .production_order_submission_allowed = SnapshotValue::available(true);
    write_json(&metrics_path, &metrics);
    let error = load_product_strategy(&fixture.state(), now)
        .expect_err("terminal freshness must not bypass runtime boundary validation");
    assert_eq!(error.kind, ProductErrorKind::BoundaryViolation);
    assert_eq!(error.field, "node_metrics");
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

    fixture.write_config(&valid_config());
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
async fn strategy_version_routes_are_read_only_and_schema_compatible() {
    let fixture = Fixture::new("version-routes");
    let router = fixture.router();
    let list_path = "/api/product/v1/strategies/ema-cross/versions";
    let detail_path = "/api/product/v1/strategies/ema-cross/versions/ema-cross@v1";

    let (status, list) = router_json(&router, Method::GET, list_path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        list["schema_version"],
        "ntpro.product_api.strategy_version_list.response.v1"
    );
    assert_eq!(list["data"][0]["strategy_version_id"], "ema-cross@v1");
    assert_read_only_boundaries(&list);
    validate_openapi_instance("StrategyVersionListResponse", &list);

    let (status, detail) = router_json(&router, Method::GET, detail_path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        detail["schema_version"],
        "ntpro.product_api.strategy_version_detail.response.v1"
    );
    assert_eq!(detail["data"]["strategy_id"], "ema-cross");
    assert_read_only_boundaries(&detail);
    validate_openapi_instance("StrategyVersionDetailResponse", &detail);

    let (status, missing_strategy) = router_json(
        &router,
        Method::GET,
        "/api/product/v1/strategies/missing/versions",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(missing_strategy["error"]["code"], "strategy_not_found");

    let (status, missing_version) = router_json(
        &router,
        Method::GET,
        "/api/product/v1/strategies/ema-cross/versions/ema-cross@v2",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        missing_version["error"]["code"],
        "strategy_version_not_found"
    );
    validate_openapi_instance("ProductErrorResponse", &missing_version);

    let (status, malformed_version) = router_json(
        &router,
        Method::GET,
        "/api/product/v1/strategies/ema-cross/versions/%FF",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(malformed_version["error"]["code"], "product_query_invalid");
    assert_eq!(malformed_version["error"]["field"], "strategy_version_path");
    validate_openapi_instance("ProductErrorResponse", &malformed_version);

    for path in [list_path, detail_path] {
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
            if method != Method::HEAD {
                assert_eq!(body["error"]["code"], "product_method_not_allowed");
            }
        }
    }
}

#[tokio::test]
async fn institution_can_create_a_real_immutable_backtest_run() {
    let fixture = Fixture::new("create-backtest-run");
    let router = fixture.router();
    let request = json!({
        "strategy_id": "ema-cross",
        "strategy_version_id": "ema-cross@v1",
        "environment": "backtest",
        "data_ref": "dataset://fixtures/ema-cross",
        "venue_ref": "venue://simulated/BINANCE",
        "starting_balance": "1000000 USDT",
        "quotes": 120,
        "trade_size": "0.001000",
        "fast_period": 3,
        "slow_period": 5
    });

    let (status, created) =
        router_json_body(&router, Method::POST, "/api/product/v1/runs", &request).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(
        created["schema_version"],
        "ntpro.product_api.run_create.response.v1"
    );
    assert_eq!(created["data"]["environment"], "backtest");
    assert_eq!(created["data"]["lifecycle"], "completed");
    assert_eq!(created["boundaries"]["backtest_run_creation_allowed"], true);
    for field in [
        "sandbox_run_creation_allowed",
        "live_run_creation_allowed",
        "external_venue_connection",
        "order_submission_allowed",
        "order_mutation_allowed",
        "automatic_retry_allowed",
        "automatic_remediation_allowed",
        "real_orders_submitted",
        "trading_controls_enabled",
    ] {
        assert_eq!(created["boundaries"][field], false, "{field}");
    }
    validate_openapi_instance("RunCreateResponse", &created);

    let run_id = created["data"]["run_id"]
        .as_str()
        .expect("created Run ID should be present");
    let run_root = fixture.root.join("artifacts/backtests").join(run_id);
    assert!(run_root.join("request.toml").is_file());
    assert!(run_root.join("summary.json").is_file());
    assert!(run_root.join("details.json").is_file());
    assert!(run_root.join("analysis.json").is_file());
    assert!(run_root.join("run-manifest.json").is_file());

    let (status, detail) = router_json(
        &router,
        Method::GET,
        &format!("/api/product/v1/runs/{run_id}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["data"]["run_id"], run_id);
    assert_eq!(
        detail["data"]["result"]["report_ref"],
        format!("artifact://backtests/{run_id}/details.json")
    );
    assert_eq!(
        detail["data"]["result"]["analysis_ref"],
        format!("artifact://backtests/{run_id}/analysis.json")
    );
    let (status, metrics) = router_json(
        &router,
        Method::GET,
        &format!("/api/product/v1/runs/{run_id}/metrics"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(metrics["data"]["metrics"]["quotes"], 120);
    assert_eq!(metrics["data"]["parameters"]["fast_period"], 3);
    assert_eq!(metrics["data"]["parameters"]["slow_period"], 5);
    let (status, report) = router_json(
        &router,
        Method::GET,
        &format!("/api/product/v1/runs/{run_id}/report"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        report["schema_version"],
        "ntpro.product_api.run_report.response.v1"
    );
    assert_eq!(report["data"]["run_id"], run_id);
    assert!(!report["data"]["trades"].as_array().unwrap().is_empty());
    assert_eq!(
        report["data"]["positions"].as_array().map(Vec::len),
        metrics["data"]["metrics"]["total_positions"]
            .as_u64()
            .map(|value| value as usize)
    );
    assert!(
        !report["data"]["equity_curve"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_read_only_boundaries(&report);
    validate_openapi_instance("RunReportResponse", &report);
    let (status, analysis) = router_json(
        &router,
        Method::GET,
        &format!("/api/product/v1/runs/{run_id}/analysis"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        analysis["schema_version"],
        "ntpro.product_api.run_analysis.response.v1"
    );
    assert_eq!(analysis["data"]["run_id"], run_id);
    assert_eq!(
        analysis["data"]["provenance"]["summary_ref"],
        format!("artifact://backtests/{run_id}/summary.json")
    );
    assert!(
        !analysis["data"]["drawdown_curve"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(!analysis["data"]["timeline"].as_array().unwrap().is_empty());
    assert_read_only_boundaries(&analysis);
    validate_openapi_instance("RunAnalysisResponse", &analysis);
}

#[tokio::test]
async fn institution_creates_one_immutable_demo_run_bound_to_supervisor() {
    let fixture = Fixture::new("create-demo-run");
    let router = fixture.router();
    let request = valid_demo_request();

    let (status, created) =
        router_json_body(&router, Method::POST, "/api/product/v1/demo-runs", &request).await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    assert_eq!(
        created["schema_version"],
        "ntpro.product_api.demo_run_create.response.v1"
    );
    assert_eq!(created["data"]["environment"], "sandbox");
    assert_eq!(created["data"]["lifecycle"], "created");
    assert_eq!(
        created["data"]["runtime"]["supervisor_node_id"],
        "mvp-node-001"
    );
    assert_eq!(created["data"]["runtime"]["process_state"], "not_started");
    assert_eq!(created["data"]["runtime"]["lifecycle_state"], "stopped");
    assert_eq!(created["boundaries"]["demo_run_creation_allowed"], true);
    assert_eq!(created["boundaries"]["demo_start_allowed"], true);
    assert_eq!(created["boundaries"]["demo_stop_allowed"], true);
    for field in [
        "live_run_creation_allowed",
        "external_venue_connection",
        "order_submission_allowed",
        "order_mutation_allowed",
        "automatic_retry_allowed",
        "automatic_remediation_allowed",
        "real_orders_submitted",
        "trading_controls_enabled",
    ] {
        assert_eq!(created["boundaries"][field], false, "{field}");
    }
    validate_openapi_instance("DemoRunCreateResponse", &created);

    let run_id = created["data"]["run_id"]
        .as_str()
        .expect("created Demo Run ID should be a string");
    let (status, snapshot) = router_json(
        &router,
        Method::GET,
        &format!("/api/product/v1/runs/{run_id}/demo-snapshot"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{snapshot}");
    assert_eq!(
        snapshot["schema_version"],
        "ntpro.product_api.demo_run_snapshot.response.v1"
    );
    assert_eq!(snapshot["data"]["run_id"], run_id);
    assert_eq!(snapshot["data"]["lifecycle"], "created");
    assert_eq!(snapshot["data"]["snapshot_status"], "not_started");
    assert_eq!(snapshot["data"]["market"], Value::Null);
    assert_eq!(snapshot["data"]["session"], Value::Null);
    assert_eq!(snapshot["data"]["technical_health"]["status"], "blocked");
    assert_eq!(snapshot["boundaries"]["read_only"], true);
    assert_eq!(snapshot["boundaries"]["sandbox_only"], true);
    assert_eq!(snapshot["boundaries"]["order_submission_allowed"], false);
    validate_openapi_instance("DemoRunSnapshotResponse", &snapshot);
    let directory = fixture.root.join("artifacts/demo-runs").join(run_id);
    let request_raw =
        fs::read(directory.join("request.json")).expect("Demo request artifact should be readable");
    let manifest_raw =
        fs::read(directory.join("run-manifest.json")).expect("Demo manifest should be readable");
    let manifest: Value =
        serde_json::from_slice(&manifest_raw).expect("Demo manifest should be valid JSON");
    assert_eq!(manifest["request_sha256"], sha256_bytes_ref(&request_raw));
    assert!(directory.join("strategy-version.json").is_file());

    let (status, list) = router_json(&router, Method::GET, "/api/product/v1/runs").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["data"].as_array().map(Vec::len), Some(3));
    let demo = list["data"]
        .as_array()
        .and_then(|runs| runs.iter().find(|run| run["run_id"] == run_id))
        .expect("created Demo Run should be listed");
    assert_eq!(demo["runtime"]["strategy_instance_id"], "mvp-strategy-001");
    validate_openapi_instance("RunListResponse", &list);

    let (status, conflict) =
        router_json_body(&router, Method::POST, "/api/product/v1/demo-runs", &request).await;
    assert_eq!(status, StatusCode::CONFLICT, "{conflict}");
    assert_eq!(conflict["error"]["field"], "active_demo_run");
    validate_openapi_instance("ProductErrorResponse", &conflict);
}

#[tokio::test]
async fn static_sandbox_run_blocks_dynamic_demo_creation() {
    let fixture = Fixture::new("static-sandbox-blocks-demo");
    let static_sandbox = r#"[[product_runs]]
run_id = "mvp-strategy-001"
strategy_id = "ema-cross"
strategy_version_id = "ema-cross@v1"
environment = "sandbox"
data_ref = "market://sandbox/BTCUSDT.BINANCE"
config_ref = "node-config:node.toml#product_runs"
adapter_ref = "adapter://sandbox/fixture-stream"
account_ref = "account://sandbox/acct-sandbox-001"
venue_ref = "venue://sandbox/BINANCE"
lifecycle = "running"
result_status = "pending"
risk_status = "active"
risk_ref = "node-config:node.toml#risk"
created_at_unix_ms = 1785542400000
started_at_unix_ms = 1785542400000
updated_at_unix_ms = 1785542400000
external_venue_connection = false
order_submission_allowed = false
order_mutation_allowed = false
automatic_retry_allowed = false
automatic_remediation_allowed = false
real_orders_submitted = false
trading_controls_enabled = false

"#;
    fixture.write_config(&valid_config().replace(
        "[[product_runs]]\nrun_id = \"ema-cross-live-001\"",
        &format!("{static_sandbox}[[product_runs]]\nrun_id = \"ema-cross-live-001\""),
    ));
    fixture.refresh_identity_and_status_provenance();

    let (status, error) = router_json_body(
        &fixture.router(),
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{error}");
    assert_eq!(error["error"]["field"], "active_demo_run");
    assert!(!fixture.root.join("artifacts/demo-runs").exists());
}

#[tokio::test]
async fn demo_creation_and_artifacts_fail_closed() {
    for (name, mutate, field) in [
        (
            "live-environment",
            (|request: &mut Value| request["environment"] = json!("live")) as fn(&mut Value),
            "demo_confirmation",
        ),
        (
            "missing-confirmation",
            (|request: &mut Value| request["user_confirmed"] = json!(false)) as fn(&mut Value),
            "demo_confirmation",
        ),
        (
            "wrong-node",
            (|request: &mut Value| request["supervisor_node_id"] = json!("other-node"))
                as fn(&mut Value),
            "demo_identity",
        ),
        (
            "wrong-account",
            (|request: &mut Value| request["account_ref"] = json!("account://sandbox/other"))
                as fn(&mut Value),
            "demo_identity",
        ),
    ] {
        let fixture = Fixture::new(&format!("demo-invalid-{name}"));
        let mut request = valid_demo_request();
        mutate(&mut request);
        let (status, error) = router_json_body(
            &fixture.router(),
            Method::POST,
            "/api/product/v1/demo-runs",
            &request,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{name}");
        assert_eq!(error["error"]["field"], field, "{name}");
    }

    let fixture = Fixture::new("demo-unknown-field");
    let mut request = valid_demo_request();
    request["unexpected"] = json!(true);
    let (status, error) = router_json_body(
        &fixture.router(),
        Method::POST,
        "/api/product/v1/demo-runs",
        &request,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error["error"]["field"], "request_body");

    let fixture = Fixture::new("demo-interrupted-publication");
    let partial = fixture.root.join("artifacts/demo-runs/demo-partial");
    fs::create_dir_all(&partial).expect("partial Demo directory should be created");
    fs::write(partial.join("request.json"), b"{}").expect("partial Demo request should be written");
    let (status, list) = router_json(&fixture.router(), Method::GET, "/api/product/v1/runs").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["data"].as_array().map(Vec::len), Some(2));

    let fixture = Fixture::new("demo-artifact-tamper");
    let (status, created) = router_json_body(
        &fixture.router(),
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let run_id = created["data"]["run_id"]
        .as_str()
        .expect("created Demo Run ID should exist");
    fs::write(
        fixture
            .root
            .join("artifacts/demo-runs")
            .join(run_id)
            .join("request.json"),
        b"{}",
    )
    .expect("Demo request should be tampered");
    let (status, error) = router_json(&fixture.router(), Method::GET, "/api/product/v1/runs").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(error["error"]["code"], "product_source_invalid");
    assert_eq!(error["error"]["field"], "demo_manifest");
}

#[tokio::test]
async fn demo_resigned_artifacts_and_forbidden_capabilities_fail_closed() {
    let fixture = Fixture::new("demo-resigned-request");
    let router = fixture.router();
    let (status, created) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let run_id = created["data"]["run_id"].as_str().unwrap();
    let directory = fixture.root.join("artifacts/demo-runs").join(run_id);
    let request_path = directory.join("request.json");
    let manifest_path = directory.join("run-manifest.json");
    let mut request: Value = serde_json::from_slice(&fs::read(&request_path).unwrap()).unwrap();
    request["account_ref"] = json!("account://sandbox/attacker");
    let request_raw = serde_json::to_vec_pretty(&request).unwrap();
    fs::write(&request_path, &request_raw).unwrap();
    let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["request_sha256"] = json!(sha256_bytes_ref(&request_raw));
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let (status, error) = router_json(&router, Method::GET, "/api/product/v1/runs").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(error["error"]["code"], "product_source_invalid");
    assert_eq!(error["error"]["field"], "demo_run_ownership");

    let fixture = Fixture::new("demo-resigned-version");
    let router = fixture.router();
    let (status, created) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let run_id = created["data"]["run_id"].as_str().unwrap();
    let directory = fixture.root.join("artifacts/demo-runs").join(run_id);
    let version_path = directory.join("strategy-version.json");
    let manifest_path = directory.join("run-manifest.json");
    let mut version: Value = serde_json::from_slice(&fs::read(&version_path).unwrap()).unwrap();
    version["version"] = json!("attacker");
    let version_raw = serde_json::to_vec_pretty(&version).unwrap();
    fs::write(&version_path, &version_raw).unwrap();
    let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    let version_sha = sha256_bytes_ref(&version_raw);
    manifest["strategy_version_snapshot_sha256"] = json!(version_sha.clone());
    manifest["config"]["strategy_version_snapshot_sha256"] = json!(version_sha);
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let (status, error) = router_json(&router, Method::GET, "/api/product/v1/runs").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(error["error"]["code"], "product_source_invalid");

    let fixture = Fixture::new("demo-capability-true");
    let router = fixture.router();
    let (status, created) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let run_id = created["data"]["run_id"].as_str().unwrap();
    let manifest_path = fixture
        .root
        .join("artifacts/demo-runs")
        .join(run_id)
        .join("run-manifest.json");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["config"]["order_submission_allowed"] = json!(true);
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let (status, error) = router_json(&router, Method::GET, "/api/product/v1/runs").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(error["error"]["code"], "product_boundary_violation");
    assert_eq!(error["error"]["field"], "run_capabilities");
}

#[cfg(any(unix, windows))]
#[tokio::test]
async fn demo_artifact_file_and_directory_symlinks_fail_closed() {
    let fixture = Fixture::new("demo-file-symlink");
    let router = fixture.router();
    let (status, created) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let run_id = created["data"]["run_id"].as_str().unwrap();
    let directory = fixture.root.join("artifacts/demo-runs").join(run_id);
    let request_path = directory.join("request.json");
    let escaped = fixture.root.join("escaped-demo-request.json");
    fs::rename(&request_path, &escaped).unwrap();
    create_file_symlink(&escaped, &request_path).unwrap();
    let (status, error) = router_json(&router, Method::GET, "/api/product/v1/runs").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(error["error"]["code"], "product_source_invalid");

    let fixture = Fixture::new("demo-directory-symlink");
    let router = fixture.router();
    let (status, created) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let run_id = created["data"]["run_id"].as_str().unwrap();
    let directory = fixture.root.join("artifacts/demo-runs").join(run_id);
    let escaped = fixture.root.join("escaped-demo-run");
    fs::rename(&directory, &escaped).unwrap();
    create_directory_symlink(&escaped, &directory).unwrap();
    let (status, error) = router_json(&router, Method::GET, "/api/product/v1/runs").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(error["error"]["code"], "product_source_invalid");
    assert_eq!(error["error"]["field"], "demo_root_containment");
}

#[tokio::test]
async fn demo_mutations_require_the_institution_role() {
    let fixture = Fixture::new("demo-role-matrix");
    let router = dashboard_router_with_access(
        fixture.registry_path.clone(),
        PathBuf::from("missing-ntpro-node"),
        "institution-token",
        "operator-token",
    );

    let (status, denied) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(denied["error"]["code"], "product_access_denied");

    let (status, denied) = router_json_body_with_cookie(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
        "ntpro_mvp_operator_access=operator-token",
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(denied["error"]["code"], "product_access_denied");

    let (status, created) = router_json_body_with_cookie(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
        "ntpro_mvp_institution_access=institution-token",
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    validate_openapi_instance("DemoRunCreateResponse", &created);
    let run_id = created["data"]["run_id"]
        .as_str()
        .expect("created Demo Run ID should exist");
    let action_path = format!("/api/product/v1/demo-runs/{run_id}/actions");
    for action in ["start", "stop"] {
        let body = json!({
            "run_id": run_id,
            "action": action,
            "user_confirmed": true
        });
        let (status, denied) = router_json_body(&router, Method::POST, &action_path, &body).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "unauthenticated {action}");
        assert_eq!(denied["error"]["code"], "product_access_denied");
        let (status, denied) = router_json_body_with_cookie(
            &router,
            Method::POST,
            &action_path,
            &body,
            "ntpro_mvp_operator_access=operator-token",
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "operator {action}");
        assert_eq!(denied["error"]["code"], "product_access_denied");
    }

    let (status, list) = router_json_with_cookie(
        &router,
        Method::GET,
        "/api/product/v1/runs",
        "ntpro_mvp_operator_access=operator-token",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["data"].as_array().map(Vec::len), Some(3));
}

#[cfg(unix)]
#[tokio::test]
async fn demo_actions_drive_the_real_supervisor_process_lifecycle() {
    let fixture = Fixture::new("demo-supervisor-lifecycle");
    let node = write_demo_fixture_node(&fixture.root);
    let router = dashboard_router(fixture.registry_path.clone(), node.clone());
    let (status, created) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let run_id = created["data"]["run_id"]
        .as_str()
        .expect("created Demo Run ID should exist");
    let action_path = format!("/api/product/v1/demo-runs/{run_id}/actions");
    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let bypass_start = store
        .start_node_process(&StartNodeRequest {
            node_id: "mvp-node-001".to_string(),
            ntpro_node_bin: node,
            startup_timeout: Duration::from_secs(3),
            node_max_runtime: Duration::from_secs(60),
            node_heartbeat_interval: Duration::from_millis(100),
            node_parent_pid: Some(std::process::id()),
            node_shutdown_timeout: Duration::from_secs(3),
        })
        .expect_err("generic Supervisor start must not bypass active Demo ownership");
    assert!(bypass_start.to_string().contains("owned by active Run"));

    let start = json!({
        "run_id": run_id,
        "action": "start",
        "user_confirmed": true
    });
    let (status, started) = router_json_body(&router, Method::POST, &action_path, &start).await;
    assert_eq!(status, StatusCode::OK, "{started}");
    assert_eq!(started["data"]["previous_lifecycle"], "created");
    assert_eq!(started["data"]["current_run"]["lifecycle"], "running");
    assert_eq!(
        started["data"]["current_run"]["runtime"]["process_state"],
        "running"
    );
    validate_openapi_instance("DemoRunActionResponse", &started);

    let snapshot_path = format!("/api/product/v1/runs/{run_id}/demo-snapshot");
    let (status, running_snapshot) = router_json(&router, Method::GET, &snapshot_path).await;
    assert_eq!(status, StatusCode::OK, "{running_snapshot}");
    assert_eq!(running_snapshot["data"]["run_id"], run_id);
    assert_eq!(running_snapshot["data"]["lifecycle"], "running");
    assert_eq!(running_snapshot["data"]["snapshot_status"], "running");
    assert_eq!(running_snapshot["data"]["market"]["event_count"], 1);
    assert_eq!(running_snapshot["data"]["session"]["signal_count"], 1);
    assert_eq!(running_snapshot["data"]["session"]["intent_count"], 1);
    assert_eq!(
        running_snapshot["data"]["session"]["actual_submission_count"],
        0
    );
    assert_eq!(
        running_snapshot["data"]["latest_order_intent"]["submission_allowed"],
        false
    );
    assert_eq!(
        running_snapshot["data"]["latest_risk_decision"]["actual_submission"],
        false
    );
    assert_eq!(
        running_snapshot["data"]["provenance"]["result_sha256"],
        Value::Null
    );
    validate_openapi_instance("DemoRunSnapshotResponse", &running_snapshot);

    let mut stale_contract = fixture.read_status_contract();
    stale_contract.provenance.generated_at_unix_ms = 1;
    fixture.write_status_contract(&stale_contract);
    let (status, stale_running) = router_json(
        &router,
        Method::GET,
        &format!("/api/product/v1/runs/{run_id}"),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{stale_running}");
    assert_eq!(stale_running["error"]["code"], "product_source_stale");
    assert_eq!(stale_running["error"]["field"], "status_contract_timestamp");
    refresh_product_status_contract(&fixture.state(), "mvp-node-001")
        .expect("running Demo action should be able to refresh the status contract");

    let bypass_stop = store
        .stop_node_process(&StopNodeRequest {
            node_id: "mvp-node-001".to_string(),
            stop_timeout: Duration::from_secs(3),
        })
        .expect_err("generic Supervisor stop must not bypass active Demo ownership");
    assert!(bypass_stop.to_string().contains("owned by active Run"));

    let (status, duplicate) = router_json_body(&router, Method::POST, &action_path, &start).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(duplicate["error"]["field"], "demo_lifecycle");

    let stop = json!({
        "run_id": run_id,
        "action": "stop",
        "user_confirmed": true
    });
    let (status, stopped) = router_json_body(&router, Method::POST, &action_path, &stop).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(stopped["data"]["previous_lifecycle"], "running");
    assert_eq!(stopped["data"]["current_run"]["lifecycle"], "stopped");
    assert_eq!(
        stopped["data"]["current_run"]["runtime"]["process_state"],
        "stopped"
    );
    validate_openapi_instance("DemoRunActionResponse", &stopped);

    let (status, frozen_snapshot) = router_json(&router, Method::GET, &snapshot_path).await;
    assert_eq!(status, StatusCode::OK, "{frozen_snapshot}");
    assert_eq!(frozen_snapshot["data"]["run_id"], run_id);
    assert_eq!(frozen_snapshot["data"]["lifecycle"], "stopped");
    assert_eq!(frozen_snapshot["data"]["snapshot_status"], "frozen");
    assert_eq!(frozen_snapshot["data"]["session"]["state"], "stopped");
    assert_eq!(
        frozen_snapshot["data"]["provenance"]["result_ref"],
        format!("artifact://demo-runs/{run_id}/demo-result.json")
    );
    let result_sha256 = frozen_snapshot["data"]["provenance"]["result_sha256"]
        .as_str()
        .expect("frozen Demo result must expose its digest");
    assert!(result_sha256.starts_with("sha256:"));
    let run_directory = fixture.root.join("artifacts/demo-runs").join(run_id);
    let terminal: Value = serde_json::from_slice(
        &fs::read(run_directory.join("terminal-state.json"))
            .expect("terminal state should be readable"),
    )
    .expect("terminal state should be valid JSON");
    assert_eq!(terminal["demo_result_sha256"], result_sha256);
    assert_eq!(
        sha256_bytes_ref(
            &fs::read(run_directory.join("demo-result.json"))
                .expect("frozen Demo result should be readable")
        ),
        result_sha256
    );
    validate_openapi_instance("DemoRunSnapshotResponse", &frozen_snapshot);

    let (status, terminal_restart) =
        router_json_body(&router, Method::POST, &action_path, &start).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(terminal_restart["error"]["code"], "demo_run_conflict");
    assert_eq!(terminal_restart["error"]["field"], "demo_lifecycle");

    let (status, second) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{second}");
    let second_run_id = second["data"]["run_id"]
        .as_str()
        .expect("second Demo Run ID should exist");
    assert_ne!(second_run_id, run_id);
    let second_action_path = format!("/api/product/v1/demo-runs/{second_run_id}/actions");
    let second_start = json!({
        "run_id": second_run_id,
        "action": "start",
        "user_confirmed": true
    });
    let (status, second_started) =
        router_json_body(&router, Method::POST, &second_action_path, &second_start).await;
    assert_eq!(status, StatusCode::OK, "{second_started}");
    let (status, old_detail) = router_json(
        &router,
        Method::GET,
        &format!("/api/product/v1/runs/{run_id}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{old_detail}");
    assert_eq!(old_detail["data"]["lifecycle"], "stopped");
    assert_eq!(old_detail["data"]["runtime"]["process_state"], "stopped");
    let second_stop = json!({
        "run_id": second_run_id,
        "action": "stop",
        "user_confirmed": true
    });
    let (status, second_stopped) =
        router_json_body(&router, Method::POST, &second_action_path, &second_stop).await;
    assert_eq!(status, StatusCode::OK, "{second_stopped}");
    let terminal_path = fixture
        .root
        .join("artifacts/demo-runs")
        .join(second_run_id)
        .join("terminal-state.json");
    let mut terminal: Value = serde_json::from_slice(&fs::read(&terminal_path).unwrap()).unwrap();
    let resigned_completed = terminal["completed_at_unix_ms"].as_u64().unwrap() + 1;
    terminal["completed_at_unix_ms"] = json!(resigned_completed);
    terminal["updated_at_unix_ms"] = json!(resigned_completed);
    fs::write(
        &terminal_path,
        serde_json::to_vec_pretty(&terminal).unwrap(),
    )
    .unwrap();
    let (status, terminal_error) = router_json(&router, Method::GET, "/api/product/v1/runs").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(terminal_error["error"]["code"], "product_source_invalid");
    assert_eq!(terminal_error["error"]["field"], "demo_terminal_anchor");
}

#[cfg(unix)]
#[tokio::test]
async fn frozen_demo_result_tampering_fails_closed() {
    let fixture = Fixture::new("demo-result-tampering");
    let node = write_demo_fixture_node(&fixture.root);
    let router = dashboard_router(fixture.registry_path.clone(), node);
    let (status, created) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let run_id = created["data"]["run_id"]
        .as_str()
        .expect("created Demo Run ID should exist");
    let action_path = format!("/api/product/v1/demo-runs/{run_id}/actions");
    let (status, started) = router_json_body(
        &router,
        Method::POST,
        &action_path,
        &json!({"run_id": run_id, "action": "start", "user_confirmed": true}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{started}");
    let (status, stopped) = router_json_body(
        &router,
        Method::POST,
        &action_path,
        &json!({"run_id": run_id, "action": "stop", "user_confirmed": true}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{stopped}");

    let result_path = fixture
        .root
        .join("artifacts/demo-runs")
        .join(run_id)
        .join("demo-result.json");
    let mut result = fs::read(&result_path).expect("frozen Demo result should be readable");
    result.push(b' ');
    fs::write(&result_path, result).expect("tampered Demo result should be written");

    let (status, error) = router_json(
        &router,
        Method::GET,
        &format!("/api/product/v1/runs/{run_id}/demo-snapshot"),
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR, "{error}");
    assert_eq!(error["error"]["code"], "product_source_invalid");
    assert_eq!(error["error"]["field"], "demo_terminal_state");
    validate_openapi_instance("ProductErrorResponse", &error);
}

#[cfg(unix)]
#[tokio::test]
async fn demo_terminal_publication_is_idempotent_for_get_and_stop_races() {
    let fixture = Fixture::new("demo-terminal-races");
    let node = write_demo_fixture_node(&fixture.root);
    let router = dashboard_router(fixture.registry_path.clone(), node);
    let (status, created) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let run_id = created["data"]["run_id"].as_str().unwrap().to_string();
    let action_path = format!("/api/product/v1/demo-runs/{run_id}/actions");
    let start = json!({
        "run_id": run_id,
        "action": "start",
        "user_confirmed": true
    });
    let (status, started) = router_json_body(&router, Method::POST, &action_path, &start).await;
    assert_eq!(status, StatusCode::OK, "{started}");

    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let ownership = store.load().unwrap().nodes["mvp-node-001"].run_ownership[&run_id].clone();
    store
        .stop_node_process_for_run(
            &StopNodeRequest {
                node_id: "mvp-node-001".to_string(),
                stop_timeout: Duration::from_secs(3),
            },
            &run_id,
            &ownership.manifest_sha256,
        )
        .expect("owner-aware stop should leave terminal publication to concurrent readers");
    let detail_path = format!("/api/product/v1/runs/{run_id}");
    let (first, second) = tokio::join!(
        router_json(&router, Method::GET, &detail_path),
        router_json(&router, Method::GET, &detail_path)
    );
    for (status, detail) in [first, second] {
        assert_eq!(status, StatusCode::OK, "{detail}");
        assert_eq!(detail["data"]["lifecycle"], "stopped");
    }

    for attempt in 0..8 {
        let (status, created) = router_json_body(
            &router,
            Method::POST,
            "/api/product/v1/demo-runs",
            &valid_demo_request(),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "attempt={attempt} {created}");
        let run_id = created["data"]["run_id"].as_str().unwrap().to_string();
        let action_path = format!("/api/product/v1/demo-runs/{run_id}/actions");
        let start = json!({
            "run_id": run_id,
            "action": "start",
            "user_confirmed": true
        });
        let (status, started) = router_json_body(&router, Method::POST, &action_path, &start).await;
        assert_eq!(status, StatusCode::OK, "attempt={attempt} {started}");
        let stop = json!({
            "run_id": run_id,
            "action": "stop",
            "user_confirmed": true
        });
        let detail_path = format!("/api/product/v1/runs/{run_id}");
        let (stopped, concurrent_read) = tokio::join!(
            router_json_body(&router, Method::POST, &action_path, &stop),
            router_json(&router, Method::GET, &detail_path)
        );
        assert_eq!(stopped.0, StatusCode::OK, "attempt={attempt} {}", stopped.1);
        assert_eq!(
            concurrent_read.0,
            StatusCode::OK,
            "attempt={attempt} {}",
            concurrent_read.1
        );
        let (status, final_detail) = router_json(&router, Method::GET, &detail_path).await;
        assert_eq!(status, StatusCode::OK, "attempt={attempt} {final_detail}");
        assert_eq!(final_detail["data"]["lifecycle"], "stopped");
    }
}

#[cfg(unix)]
#[tokio::test]
async fn demo_creation_finalizes_a_stopped_owner_before_freshness_validation() {
    let fixture = Fixture::new("demo-create-after-external-stop");
    let node = write_demo_fixture_node(&fixture.root);
    let router = dashboard_router(fixture.registry_path.clone(), node);
    let (status, created) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let run_id = created["data"]["run_id"].as_str().unwrap().to_string();
    let action_path = format!("/api/product/v1/demo-runs/{run_id}/actions");
    let start = json!({
        "run_id": run_id,
        "action": "start",
        "user_confirmed": true
    });
    let (status, started) = router_json_body(&router, Method::POST, &action_path, &start).await;
    assert_eq!(status, StatusCode::OK, "{started}");

    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let ownership = store.load().unwrap().nodes["mvp-node-001"].run_ownership[&run_id].clone();
    let stopped = store
        .stop_node_process_for_run(
            &StopNodeRequest {
                node_id: "mvp-node-001".to_string(),
                stop_timeout: Duration::from_secs(3),
            },
            &run_id,
            &ownership.manifest_sha256,
        )
        .expect("owner-aware stop should leave terminal publication to the next product request");
    assert_eq!(stopped.process.state, SupervisorProcessState::Stopped);
    assert!(stopped.run_ownership[&run_id].terminal.is_none());

    for path in [&stopped.status_path, &stopped.metrics_path] {
        let mut artifact: Value =
            serde_json::from_slice(&fs::read(path).expect("runtime artifact should be readable"))
                .expect("runtime artifact should be valid JSON");
        artifact["generated_at"]["value"] = json!("1");
        write_json(path, &artifact);
    }
    let mut stale_contract = fixture.read_status_contract();
    stale_contract.provenance.generated_at_unix_ms = 1;
    fixture.write_status_contract(&stale_contract);

    let (status, replacement) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{replacement}");
    assert_ne!(
        replacement["data"]["run_id"].as_str(),
        Some(run_id.as_str())
    );
    let registry = store.load().expect("registry should remain readable");
    assert_eq!(
        registry.nodes["mvp-node-001"].run_ownership[&run_id]
            .terminal
            .as_ref()
            .map(|terminal| terminal.lifecycle.as_str()),
        Some("stopped")
    );
}

#[cfg(unix)]
#[tokio::test]
async fn demo_new_ownership_clears_previous_session_times_and_preserves_terminal_history() {
    let fixture = Fixture::new("demo-new-ownership-time-baseline");
    let node = write_demo_fixture_node(&fixture.root);
    let router = dashboard_router(fixture.registry_path.clone(), node);
    let (status, created) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let old_run_id = created["data"]["run_id"].as_str().unwrap().to_string();
    let old_action_path = format!("/api/product/v1/demo-runs/{old_run_id}/actions");
    let start = json!({
        "run_id": old_run_id,
        "action": "start",
        "user_confirmed": true
    });
    let (status, started) = router_json_body(&router, Method::POST, &old_action_path, &start).await;
    assert_eq!(status, StatusCode::OK, "{started}");
    let stop = json!({
        "run_id": old_run_id,
        "action": "stop",
        "user_confirmed": true
    });
    let (status, stopped) = router_json_body(&router, Method::POST, &old_action_path, &stop).await;
    assert_eq!(status, StatusCode::OK, "{stopped}");

    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let old_registry = store.load().expect("stopped registry should be readable");
    let old_record = &old_registry.nodes["mvp-node-001"];
    assert!(old_record.last_known_status.started_at.value.is_some());
    assert!(old_record.last_known_status.stopped_at.value.is_some());
    let old_terminal = old_record.run_ownership[&old_run_id]
        .terminal
        .clone()
        .expect("old Run should have a terminal ownership anchor");

    let (status, replacement) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{replacement}");
    let new_run_id = replacement["data"]["run_id"].as_str().unwrap();
    assert_ne!(new_run_id, old_run_id);

    let registry = store
        .load()
        .expect("replacement registry should be readable");
    let record = &registry.nodes["mvp-node-001"];
    assert!(record.last_known_status.started_at.value.is_none());
    assert!(record.last_known_status.stopped_at.value.is_none());
    assert_eq!(
        record.run_ownership[&old_run_id].terminal.as_ref(),
        Some(&old_terminal)
    );
    assert!(record.run_ownership[new_run_id].terminal.is_none());
}

#[cfg(unix)]
#[tokio::test]
async fn demo_same_millisecond_start_and_stop_projects_a_terminal_run() {
    let fixture = Fixture::new("demo-same-millisecond-terminal");
    let node = write_demo_fixture_node(&fixture.root);
    let router = dashboard_router(fixture.registry_path.clone(), node);
    let (status, created) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let run_id = created["data"]["run_id"].as_str().unwrap().to_string();
    let created_at = created["data"]["created_at_unix_ms"]
        .as_u64()
        .expect("Demo created_at should be an integer");
    let action_path = format!("/api/product/v1/demo-runs/{run_id}/actions");
    let start = json!({
        "run_id": run_id,
        "action": "start",
        "user_confirmed": true
    });
    let (status, started) = router_json_body(&router, Method::POST, &action_path, &start).await;
    assert_eq!(status, StatusCode::OK, "{started}");

    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let ownership = store.load().unwrap().nodes["mvp-node-001"].run_ownership[&run_id].clone();
    store
        .stop_node_process_for_run(
            &StopNodeRequest {
                node_id: "mvp-node-001".to_string(),
                stop_timeout: Duration::from_secs(3),
            },
            &run_id,
            &ownership.manifest_sha256,
        )
        .expect("owner-aware stop should succeed");
    let mut registry = store.load().expect("stopped registry should be readable");
    let record = registry.nodes.get_mut("mvp-node-001").unwrap();
    record.last_known_status.started_at = SnapshotValue::available(created_at.to_string());
    record.last_known_status.stopped_at = SnapshotValue::available(created_at.to_string());
    store
        .save(&registry)
        .expect("same-millisecond fixture should be saved");

    let (status, detail) = router_json(
        &router,
        Method::GET,
        &format!("/api/product/v1/runs/{run_id}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{detail}");
    assert_eq!(detail["data"]["lifecycle"], "stopped");
    assert_eq!(detail["data"]["started_at_unix_ms"], created_at);
    assert_eq!(detail["data"]["completed_at_unix_ms"], created_at);
    let registry = store.load().expect("terminal registry should be readable");
    assert_eq!(
        registry.nodes["mvp-node-001"].run_ownership[&run_id]
            .terminal
            .as_ref()
            .map(|terminal| terminal.lifecycle.as_str()),
        Some("stopped")
    );
}

#[tokio::test]
async fn demo_runtime_exit_is_anchored_failed_and_missing_registry_time_fails_closed() {
    let fixture = Fixture::new("demo-runtime-exit");
    let router = fixture.router();
    let (status, created) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let run_id = created["data"]["run_id"].as_str().unwrap().to_string();
    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let mut registry = store.load().unwrap();
    let record = registry.nodes.get_mut("mvp-node-001").unwrap();
    let failed_at = unix_time_ms().saturating_add(1_000);
    record.process.state = SupervisorProcessState::Stale;
    record.process.pid = SnapshotValue::not_configured();
    record.last_known_status.lifecycle_state = LifecycleStatus::Error;
    record.last_known_status.started_at = SnapshotValue::available(failed_at.to_string());
    record.updated_at = SnapshotValue::available(failed_at.to_string());
    store.save(&registry).unwrap();

    let (status, unavailable) = router_json(
        &router,
        Method::GET,
        &format!("/api/product/v1/runs/{run_id}"),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{unavailable}");
    assert_eq!(unavailable["error"]["code"], "product_source_stale");
    let registry = store.load().unwrap();
    let ownership = registry.nodes["mvp-node-001"].run_ownership[&run_id]
        .terminal
        .as_ref()
        .expect("failed runtime must publish a Supervisor terminal anchor");
    assert_eq!(ownership.lifecycle, "failed");

    let mut registry = store.load().unwrap();
    let record = registry.nodes.get_mut("mvp-node-001").unwrap();
    record.process.state = SupervisorProcessState::Stopped;
    record.last_known_status.lifecycle_state = LifecycleStatus::Stopped;
    record.last_known_status.stopped_at = SnapshotValue::available(failed_at.to_string());
    record.updated_at = SnapshotValue::available(failed_at.saturating_add(1).to_string());
    store.save(&registry).unwrap();
    let (status, failed) = router_json(
        &router,
        Method::GET,
        &format!("/api/product/v1/runs/{run_id}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{failed}");
    assert_eq!(failed["data"]["lifecycle"], "failed");
    assert_eq!(failed["data"]["runtime"]["process_state"], "stale");
    let (status, second) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{second}");
    assert_ne!(second["data"]["run_id"], run_id);

    let mut registry = store.load().unwrap();
    registry.nodes.get_mut("mvp-node-001").unwrap().updated_at = SnapshotValue::unknown();
    store.save(&registry).unwrap();
    let (status, invalid) = router_json(&router, Method::GET, "/api/product/v1/runs").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(invalid["error"]["code"], "product_source_invalid");
    assert_eq!(invalid["error"]["field"], "supervisor_record_updated_at");
}

#[tokio::test]
async fn demo_execution_failures_use_demo_specific_error_contract() {
    let fixture = Fixture::new("demo-error-contract");
    let router = fixture.router();
    let (status, created) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let run_id = created["data"]["run_id"].as_str().unwrap();
    let (status, error) = router_json_body(
        &router,
        Method::POST,
        &format!("/api/product/v1/demo-runs/{run_id}/actions"),
        &json!({
            "run_id": run_id,
            "action": "start",
            "user_confirmed": true
        }),
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(error["error"]["code"], "demo_execution_failed");
    assert_eq!(error["error"]["field"], "demo_start");
    validate_openapi_instance("ProductErrorResponse", &error);
}

#[cfg(unix)]
#[tokio::test]
async fn invalid_runtime_after_demo_start_is_stopped_and_terminalized() {
    let fixture = Fixture::new("demo-start-runtime-invalid");
    let node = write_demo_fixture_node_with_forbidden_metrics(&fixture.root);
    let router = dashboard_router(fixture.registry_path.clone(), node);
    let (status, created) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let run_id = created["data"]["run_id"]
        .as_str()
        .expect("created Demo Run ID should exist");
    let (status, error) = router_json_body(
        &router,
        Method::POST,
        &format!("/api/product/v1/demo-runs/{run_id}/actions"),
        &json!({
            "run_id": run_id,
            "action": "start",
            "user_confirmed": true
        }),
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR, "{error}");
    assert_eq!(error["error"]["code"], "product_boundary_violation");
    assert_eq!(error["error"]["field"], "node_metrics");

    let registry = SupervisorRegistryStore::new(&fixture.registry_path)
        .load()
        .expect("registry should remain readable after failed Demo start");
    let record = registry
        .nodes
        .get("mvp-node-001")
        .expect("Demo node should remain registered");
    assert_eq!(record.process.state, SupervisorProcessState::Stopped);
    assert_eq!(
        record.last_known_status.lifecycle_state,
        nautilus_live::status::LifecycleStatus::Stopped
    );
    let terminal: Value = serde_json::from_slice(
        &fs::read(
            fixture
                .root
                .join("artifacts/demo-runs")
                .join(run_id)
                .join("terminal-state.json"),
        )
        .expect("failed Demo start should publish terminal state"),
    )
    .expect("terminal state should be valid JSON");
    assert_eq!(terminal["lifecycle"], "failed");
    assert_eq!(terminal["runtime"]["process_state"], "stopped");
    assert_eq!(terminal["runtime"]["lifecycle_state"], "stopped");
}

#[cfg(unix)]
#[tokio::test]
async fn mvp_shutdown_stops_owned_demo_before_manifest_validation() {
    let fixture = Fixture::new("demo-shutdown-corrupt-manifest");
    let node = write_demo_fixture_node(&fixture.root);
    let router = dashboard_router(fixture.registry_path.clone(), node);
    let (status, created) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{created}");
    let run_id = created["data"]["run_id"].as_str().unwrap().to_string();
    let action_path = format!("/api/product/v1/demo-runs/{run_id}/actions");
    let (status, started) = router_json_body(
        &router,
        Method::POST,
        &action_path,
        &json!({
            "run_id": run_id,
            "action": "start",
            "user_confirmed": true
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{started}");

    let manifest_path = fixture
        .root
        .join("artifacts/demo-runs")
        .join(&run_id)
        .join("run-manifest.json");
    fs::write(&manifest_path, "{}\n").expect("Demo manifest should be corrupted");
    let error = shutdown_active_demo_run(&fixture.registry_path, Duration::from_secs(3))
        .expect_err("corrupt manifest should prevent terminal publication after shutdown");
    let message = format!("{error:#}");
    assert!(message.contains("active Demo manifest"), "{message}");

    let store = SupervisorRegistryStore::new(&fixture.registry_path);
    let record = store
        .refresh_process_state("mvp-node-001")
        .expect("Supervisor state should remain readable");
    assert_eq!(record.process.state, SupervisorProcessState::Stopped);
    assert_eq!(
        record.last_known_status.lifecycle_state,
        LifecycleStatus::Stopped
    );
    assert!(
        record.run_ownership[&run_id].terminal.is_none(),
        "invalid manifest must not receive a terminal hash anchor"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn concurrent_demo_starts_are_serialized_and_spawn_one_process() {
    let fixture = Fixture::new("demo-concurrent-start");
    let node = write_demo_fixture_node(&fixture.root);
    let router = dashboard_router(fixture.registry_path.clone(), node);
    let (status, created) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/demo-runs",
        &valid_demo_request(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let run_id = created["data"]["run_id"]
        .as_str()
        .expect("created Demo Run ID should exist");
    let action_path = format!("/api/product/v1/demo-runs/{run_id}/actions");
    let start = json!({
        "run_id": run_id,
        "action": "start",
        "user_confirmed": true
    });

    let (first, second) = tokio::join!(
        router_json_body(&router, Method::POST, &action_path, &start),
        router_json_body(&router, Method::POST, &action_path, &start),
    );
    let mut statuses = [first.0, second.0];
    statuses.sort();
    assert_eq!(statuses, [StatusCode::OK, StatusCode::CONFLICT]);
    let conflict = if first.0 == StatusCode::CONFLICT {
        &first.1
    } else {
        &second.1
    };
    assert_eq!(conflict["error"]["field"], "demo_lifecycle");

    let stop = json!({
        "run_id": run_id,
        "action": "stop",
        "user_confirmed": true
    });
    let (status, stopped) = router_json_body(&router, Method::POST, &action_path, &stop).await;
    assert_eq!(status, StatusCode::OK, "{stopped}");
    assert_eq!(stopped["data"]["current_run"]["lifecycle"], "stopped");
}

#[tokio::test]
async fn institution_can_compare_and_explicitly_reproduce_verified_backtests() {
    let fixture = Fixture::new("compare-reproduce-backtests");
    let router = fixture.router();
    let request = json!({
        "strategy_id": "ema-cross",
        "strategy_version_id": "ema-cross@v1",
        "environment": "backtest",
        "data_ref": "dataset://fixtures/ema-cross",
        "venue_ref": "venue://simulated/BINANCE",
        "starting_balance": "1000000 USDT",
        "quotes": 120,
        "trade_size": "0.001000",
        "fast_period": 3,
        "slow_period": 5
    });

    let (status, created) =
        router_json_body(&router, Method::POST, "/api/product/v1/runs", &request).await;
    assert_eq!(status, StatusCode::CREATED);
    let source_run_id = created["data"]["run_id"]
        .as_str()
        .expect("created Run ID should be present");
    assert_eq!(created["data"]["result"]["reproduction_ref"], Value::Null);

    let comparison_path =
        format!("/api/product/v1/run-comparisons?run_ids=backtest-001,{source_run_id}");
    let (status, comparison) = router_json(&router, Method::GET, &comparison_path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        comparison["schema_version"],
        "ntpro.product_api.run_comparison.response.v1"
    );
    assert_eq!(comparison["data"]["baseline_run_id"], "backtest-001");
    assert_eq!(
        comparison["data"]["run_ids"].as_array().map(Vec::len),
        Some(2)
    );
    assert_eq!(
        comparison["data"]["items"].as_array().map(Vec::len),
        Some(2)
    );
    assert_eq!(comparison["data"]["compatibility"]["same_strategy"], true);
    assert_eq!(comparison["data"]["compatibility"]["same_data"], false);
    assert_eq!(comparison["data"]["compatibility"]["same_instrument"], true);
    assert_eq!(
        comparison["data"]["compatibility"]["directly_comparable"],
        false
    );
    assert_read_only_boundaries(&comparison);
    validate_openapi_instance("RunComparisonResponse", &comparison);

    let source_root = fixture.root.join("artifacts/backtests").join(source_run_id);
    let source_manifest_before =
        fs::read(source_root.join("run-manifest.json")).expect("source manifest should exist");
    let source_request_before =
        fs::read(source_root.join("request.toml")).expect("source request should exist");
    let reproduce_path = format!("/api/product/v1/runs/{source_run_id}/reproduction");
    let (status, reproduced) = router_json_body(
        &router,
        Method::POST,
        &reproduce_path,
        &json!({
            "source_run_id": source_run_id,
            "deterministic_replay": true
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{reproduced:#}");
    assert_eq!(
        reproduced["schema_version"],
        "ntpro.product_api.run_reproduction.response.v1"
    );
    assert_eq!(reproduced["data"]["source_run_id"], source_run_id);
    assert_eq!(reproduced["data"]["proof"]["input_equivalent"], true);
    assert_eq!(reproduced["data"]["proof"]["output_equivalent"], true);
    assert_eq!(reproduced["data"]["proof"]["user_initiated"], true);
    assert_eq!(
        reproduced["data"]["proof"]["automatic_retry_allowed"],
        false
    );
    assert_eq!(
        reproduced["data"]["proof"]["automatic_remediation_allowed"],
        false
    );
    assert_eq!(
        reproduced["boundaries"]["backtest_run_creation_allowed"],
        true
    );
    for field in [
        "sandbox_run_creation_allowed",
        "live_run_creation_allowed",
        "external_venue_connection",
        "order_submission_allowed",
        "order_mutation_allowed",
        "automatic_retry_allowed",
        "automatic_remediation_allowed",
        "real_orders_submitted",
        "trading_controls_enabled",
    ] {
        assert_eq!(reproduced["boundaries"][field], false, "{field}");
    }
    validate_openapi_instance("RunReproductionResponse", &reproduced);

    let reproduced_run_id = reproduced["data"]["reproduced_run"]["run_id"]
        .as_str()
        .expect("reproduced Run ID should be present");
    assert_ne!(reproduced_run_id, source_run_id);
    assert_eq!(
        fs::read(source_root.join("run-manifest.json")).unwrap(),
        source_manifest_before,
        "reproduction must not overwrite the source manifest"
    );
    assert_eq!(
        fs::read(source_root.join("request.toml")).unwrap(),
        source_request_before,
        "reproduction must not overwrite the source request"
    );
    let proof_ref = format!("artifact://backtests/{reproduced_run_id}/reproduction.json");
    assert_eq!(
        reproduced["data"]["reproduced_run"]["result"]["reproduction_ref"],
        proof_ref
    );
    assert!(
        fixture
            .root
            .join("artifacts/backtests")
            .join(reproduced_run_id)
            .join("reproduction.json")
            .is_file()
    );

    let proof_path = format!("/api/product/v1/runs/{reproduced_run_id}/reproduction");
    let (status, proof) = router_json(&router, Method::GET, &proof_path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(proof["data"], reproduced["data"]["proof"]);
    assert_read_only_boundaries(&proof);
    validate_openapi_instance("RunReproductionProofResponse", &proof);

    let compare_reproduction_path =
        format!("/api/product/v1/run-comparisons?run_ids={source_run_id},{reproduced_run_id}");
    let (status, comparison) = router_json(&router, Method::GET, &compare_reproduction_path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        comparison["data"]["compatibility"]["directly_comparable"],
        true
    );
    assert_eq!(
        comparison["data"]["items"][1]["reproduction_ref"],
        proof_ref
    );
}

#[tokio::test]
async fn comparison_and_reproduction_use_each_runs_immutable_strategy_version_snapshot() {
    let fixture = Fixture::new("compare-reproduce-cross-version");
    let router = fixture.router();
    let request_for = |version: &str| {
        json!({
            "strategy_id": "ema-cross",
            "strategy_version_id": format!("ema-cross@{version}"),
            "environment": "backtest",
            "data_ref": "dataset://fixtures/ema-cross",
            "venue_ref": "venue://simulated/BINANCE",
            "starting_balance": "1000000 USDT",
            "quotes": 120,
            "trade_size": "0.001000",
            "fast_period": 3,
            "slow_period": 5
        })
    };

    let (status, v1) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/runs",
        &request_for("v1"),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{v1:#}");
    let v1_run_id = v1["data"]["run_id"].as_str().unwrap().to_string();

    fixture.activate_strategy_version("v2");
    let (status, v2) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/runs",
        &request_for("v2"),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{v2:#}");
    let v2_run_id = v2["data"]["run_id"].as_str().unwrap().to_string();
    assert!(
        fixture
            .root
            .join("artifacts/backtests")
            .join(&v2_run_id)
            .join("strategy-version.json")
            .is_file()
    );

    fixture.activate_strategy_version("v1");

    let comparison_path =
        format!("/api/product/v1/run-comparisons?run_ids={v2_run_id},{v1_run_id}");
    let (status, comparison) = router_json(&router, Method::GET, &comparison_path).await;
    assert_eq!(status, StatusCode::OK, "{comparison:#}");
    assert_eq!(comparison["data"]["baseline_run_id"], v2_run_id);
    assert_eq!(comparison["data"]["run_ids"][0], v2_run_id);
    assert_eq!(comparison["data"]["run_ids"][1], v1_run_id);
    assert_eq!(
        comparison["data"]["items"][0]["strategy_version_id"],
        "ema-cross@v2"
    );
    assert_eq!(
        comparison["data"]["items"][1]["strategy_version_id"],
        "ema-cross@v1"
    );
    assert_eq!(comparison["data"]["compatibility"]["same_strategy"], true);
    assert_eq!(
        comparison["data"]["compatibility"]["same_strategy_version"],
        false
    );

    let reproduce_path = format!("/api/product/v1/runs/{v2_run_id}/reproduction");
    let (status, reproduced) = router_json_body(
        &router,
        Method::POST,
        &reproduce_path,
        &json!({
            "source_run_id": v2_run_id,
            "deterministic_replay": true
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{reproduced:#}");
    assert_eq!(
        reproduced["data"]["reproduced_run"]["strategy_version_id"],
        "ema-cross@v2"
    );
    assert_eq!(reproduced["data"]["proof"]["input_equivalent"], true);
    assert_eq!(reproduced["data"]["proof"]["output_equivalent"], true);
}

#[tokio::test]
async fn strategy_version_snapshot_hash_and_semantic_tampering_fail_closed() {
    let fixture = Fixture::new("strategy-version-snapshot-tamper");
    let router = fixture.router();
    let request = json!({
        "strategy_id": "ema-cross",
        "strategy_version_id": "ema-cross@v1",
        "environment": "backtest",
        "data_ref": "dataset://fixtures/ema-cross",
        "venue_ref": "venue://simulated/BINANCE",
        "starting_balance": "1000000 USDT",
        "quotes": 120,
        "trade_size": "0.001000",
        "fast_period": 3,
        "slow_period": 5
    });
    let (status, created) =
        router_json_body(&router, Method::POST, "/api/product/v1/runs", &request).await;
    assert_eq!(status, StatusCode::CREATED, "{created:#}");
    let run_id = created["data"]["run_id"].as_str().unwrap();
    let run_root = fixture.root.join("artifacts/backtests").join(run_id);
    let snapshot_path = run_root.join("strategy-version.json");
    let snapshot_raw = fs::read(&snapshot_path).expect("snapshot should be readable");

    let mut hash_tampered = snapshot_raw.clone();
    hash_tampered.push(b' ');
    fs::write(&snapshot_path, hash_tampered).expect("snapshot should be writable");
    let (status, hash_error) = router_json(&router, Method::GET, "/api/product/v1/runs").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        hash_error["error"]["field"],
        "run_strategy_version_snapshot_sha256"
    );

    let mut snapshot: Value = serde_json::from_slice(&snapshot_raw).expect("snapshot should parse");
    snapshot["strategy_version"]["code_ref"] = json!(
        "git://NTPRO@0000000000000000000000000000000000000000/crates/cli/src/strategy_session.rs#ema_cross_demo"
    );
    let semantic_raw = format!(
        "{}\n",
        serde_json::to_string_pretty(&snapshot).expect("snapshot should serialize")
    )
    .into_bytes();
    fs::write(&snapshot_path, &semantic_raw).expect("snapshot should be writable");
    let manifest_path = run_root.join("run-manifest.json");
    let mut manifest: Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("manifest should be readable"))
            .expect("manifest should parse");
    manifest["config"]["strategy_version_snapshot_sha256"] = json!(sha256_bytes_ref(&semantic_raw));
    write_json(&manifest_path, &manifest);
    let (status, semantic_error) = router_json(&router, Method::GET, "/api/product/v1/runs").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        semantic_error["error"]["field"],
        "strategy_version_content_hash"
    );
}

#[tokio::test]
async fn dynamic_run_strategy_version_snapshot_binding_cannot_be_downgraded() {
    let fixture = Fixture::new("strategy-version-snapshot-downgrade");
    let router = fixture.router();
    let request = json!({
        "strategy_id": "ema-cross",
        "strategy_version_id": "ema-cross@v1",
        "environment": "backtest",
        "data_ref": "dataset://fixtures/ema-cross",
        "venue_ref": "venue://simulated/BINANCE",
        "starting_balance": "1000000 USDT",
        "quotes": 120,
        "trade_size": "0.001000",
        "fast_period": 3,
        "slow_period": 5
    });
    let (status, created) =
        router_json_body(&router, Method::POST, "/api/product/v1/runs", &request).await;
    assert_eq!(status, StatusCode::CREATED, "{created:#}");
    let run_id = created["data"]["run_id"].as_str().unwrap().to_string();
    let run_root = fixture.root.join("artifacts/backtests").join(&run_id);
    fs::remove_file(run_root.join("strategy-version.json"))
        .expect("strategy version snapshot should be removable for the negative test");
    let manifest_path = run_root.join("run-manifest.json");
    let mut manifest: Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("manifest should be readable"))
            .expect("manifest should parse");
    manifest["config"]
        .as_object_mut()
        .expect("manifest config should be an object")
        .remove("strategy_version_snapshot_sha256");
    write_json(&manifest_path, &manifest);

    let paths = [
        "/api/product/v1/runs".to_string(),
        format!("/api/product/v1/run-comparisons?run_ids=backtest-001,{run_id}"),
    ];
    for path in paths {
        let (status, body) = router_json(&router, Method::GET, &path).await;
        assert_eq!(
            status,
            StatusCode::INTERNAL_SERVER_ERROR,
            "{path}: {body:#}"
        );
        assert_eq!(
            body["error"]["field"], "run_strategy_version_snapshot",
            "{path}: {body:#}"
        );
    }

    let reproduction_path = format!("/api/product/v1/runs/{run_id}/reproduction");
    let (status, body) = router_json_body(
        &router,
        Method::POST,
        &reproduction_path,
        &json!({"source_run_id": run_id, "deterministic_replay": true}),
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR, "{body:#}");
    assert_eq!(body["error"]["field"], "run_strategy_version_snapshot");
}

#[tokio::test]
async fn backtest_comparison_and_reproduction_fail_closed() {
    let fixture = Fixture::new("compare-reproduce-negative");
    let router = fixture.router();
    let request = json!({
        "strategy_id": "ema-cross",
        "strategy_version_id": "ema-cross@v1",
        "environment": "backtest",
        "data_ref": "dataset://fixtures/ema-cross",
        "venue_ref": "venue://simulated/BINANCE",
        "starting_balance": "1000000 USDT",
        "quotes": 120,
        "trade_size": "0.001000",
        "fast_period": 3,
        "slow_period": 5
    });
    let (status, created) =
        router_json_body(&router, Method::POST, "/api/product/v1/runs", &request).await;
    assert_eq!(status, StatusCode::CREATED);
    let source_run_id = created["data"]["run_id"].as_str().unwrap();

    for path in [
        "/api/product/v1/run-comparisons",
        "/api/product/v1/run-comparisons?run_ids=backtest-001",
        "/api/product/v1/run-comparisons?run_ids=backtest-001,backtest-001",
        "/api/product/v1/run-comparisons?run_ids=backtest-001,a,b,c,d",
        "/api/product/v1/run-comparisons?run_ids=backtest-001,missing",
        "/api/product/v1/run-comparisons?run_ids=backtest-001,ema-cross-sandbox-001",
        "/api/product/v1/run-comparisons?run_ids=backtest-001,backtest-001&extra=true",
    ] {
        let (status, body) = router_json(&router, Method::GET, path).await;
        assert!(
            matches!(status, StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND),
            "{path}: {status} {body:#}"
        );
        validate_openapi_instance("ProductErrorResponse", &body);
    }

    let reproduce_path = format!("/api/product/v1/runs/{source_run_id}/reproduction");
    for invalid in [
        json!({"source_run_id": source_run_id, "deterministic_replay": false}),
        json!({"source_run_id": "backtest-001", "deterministic_replay": true}),
        json!({"source_run_id": source_run_id, "deterministic_replay": true, "extra": true}),
    ] {
        let (status, body) =
            router_json_body(&router, Method::POST, &reproduce_path, &invalid).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body:#}");
        validate_openapi_instance("ProductErrorResponse", &body);
    }
    let (status, body) = router_json_body(
        &router,
        Method::POST,
        "/api/product/v1/runs/backtest-001/reproduction",
        &json!({"source_run_id": "backtest-001", "deterministic_replay": true}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body:#}");

    let (status, reproduced) = router_json_body(
        &router,
        Method::POST,
        &reproduce_path,
        &json!({"source_run_id": source_run_id, "deterministic_replay": true}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{reproduced:#}");
    let reproduced_run_id = reproduced["data"]["reproduced_run"]["run_id"]
        .as_str()
        .unwrap();
    let proof_path = fixture
        .root
        .join("artifacts/backtests")
        .join(reproduced_run_id)
        .join("reproduction.json");
    let mut proof: Value = serde_json::from_slice(&fs::read(&proof_path).unwrap()).unwrap();
    proof["output_equivalent"] = json!(false);
    fs::write(&proof_path, serde_json::to_vec_pretty(&proof).unwrap()).unwrap();
    let (status, body) = router_json(
        &router,
        Method::GET,
        &format!("/api/product/v1/runs/{reproduced_run_id}/reproduction"),
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR, "{body:#}");
    assert_eq!(body["error"]["retryable"], false);

    for method in [
        Method::PUT,
        Method::PATCH,
        Method::DELETE,
        Method::OPTIONS,
        Method::CONNECT,
        Method::TRACE,
    ] {
        let (status, _, body) =
            router_json_with_headers(&router, method.clone(), &reproduce_path).await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED, "{method}");
        assert_eq!(body["error"]["code"], "product_method_not_allowed");
    }
}

#[tokio::test]
async fn concurrent_backtest_creation_accepts_one_request_and_rejects_one() {
    let fixture = Fixture::new("create-backtest-concurrent");
    let router = fixture.router();
    let request = json!({
        "strategy_id": "ema-cross",
        "strategy_version_id": "ema-cross@v1",
        "environment": "backtest",
        "data_ref": "dataset://fixtures/ema-cross",
        "venue_ref": "venue://simulated/BINANCE",
        "starting_balance": "1000000 USDT",
        "quotes": 10000,
        "trade_size": "0.001000",
        "fast_period": 3,
        "slow_period": 5
    });

    let first = router_json_body(&router, Method::POST, "/api/product/v1/runs", &request);
    let second = router_json_body(&router, Method::POST, "/api/product/v1/runs", &request);
    let ((first_status, first_body), (second_status, second_body)) = tokio::join!(first, second);
    let mut responses = [(first_status, first_body), (second_status, second_body)];
    responses.sort_by_key(|(status, _)| status.as_u16());

    assert_eq!(responses[0].0, StatusCode::CREATED);
    assert_eq!(responses[1].0, StatusCode::CONFLICT);
    assert_eq!(
        responses[1].1["error"]["field"],
        "backtest_creation_in_progress"
    );
    let published_runs = fs::read_dir(fixture.root.join("artifacts/backtests"))
        .expect("artifact root should remain readable")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().join("run-manifest.json").is_file())
        .count();
    assert_eq!(published_runs, 1, "only one dynamic Run may be published");
}

#[test]
fn run_manifest_publication_is_atomic_for_concurrent_readers() {
    let fixture = Fixture::new("manifest-atomic-publication");
    let run_root = fixture.root.join("artifacts/backtests/backtest-atomic");
    fs::create_dir(&run_root).expect("atomic publication directory should be created");
    let canonical_run_root =
        fs::canonicalize(&run_root).expect("run directory should canonicalize");
    let directory = open_absolute_directory_nofollow(&canonical_run_root)
        .expect("run directory should open safely");
    let manifest_path = run_root.join("run-manifest.json");
    let manifest = vec![b'x'; 4 * 1024 * 1024];
    let expected = manifest.clone();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let reader_barrier = barrier.clone();
    let reader = std::thread::spawn(move || {
        reader_barrier.wait();
        for _ in 0..100_000 {
            match fs::read(&manifest_path) {
                Ok(observed) => {
                    assert_eq!(observed, expected, "published manifest must be complete");
                    return;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    std::thread::yield_now();
                }
                Err(error) => panic!("manifest read failed unexpectedly: {error}"),
            }
        }
        panic!("manifest was not published before reader timeout");
    });

    barrier.wait();
    publish_new_run_file(&directory, "run-manifest.json", &manifest)
        .expect("manifest should publish atomically");
    reader.join().expect("concurrent reader should complete");
}

#[tokio::test]
async fn interrupted_manifest_temp_file_does_not_publish_or_poison_run_listing() {
    let fixture = Fixture::new("manifest-interrupted-temp");
    let interrupted_root = fixture
        .root
        .join("artifacts/backtests/backtest-interrupted");
    fs::create_dir(&interrupted_root).expect("interrupted Run directory should be created");
    fs::write(
        interrupted_root.join(".run-manifest.json.tmp.interrupted"),
        b"{partial",
    )
    .expect("interrupted temp file should be written");

    let (status, list) = router_json(&fixture.router(), Method::GET, "/api/product/v1/runs").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list["data"].as_array().map(Vec::len), Some(2));
    assert!(
        list["data"]
            .as_array()
            .expect("Run list should be an array")
            .iter()
            .all(|run| run["run_id"] != "backtest-interrupted"),
        "temporary manifest must not publish a Run"
    );
}

#[tokio::test]
async fn backtest_creation_rejects_unknown_fields_and_non_backtest_environments() {
    let fixture = Fixture::new("create-backtest-negative");
    let router = fixture.router();
    let request = json!({
        "strategy_id": "ema-cross",
        "strategy_version_id": "ema-cross@v1",
        "environment": "live",
        "data_ref": "dataset://fixtures/ema-cross",
        "venue_ref": "venue://simulated/BINANCE",
        "starting_balance": "1000000 USDT",
        "quotes": 120,
        "trade_size": "0.001",
        "fast_period": 3,
        "slow_period": 5
    });
    let (status, invalid) =
        router_json_body(&router, Method::POST, "/api/product/v1/runs", &request).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(invalid["error"]["field"], "environment");

    let mut unknown = request;
    unknown["environment"] = json!("backtest");
    unknown["output_path"] = json!("/tmp/escape");
    let (status, malformed) =
        router_json_body(&router, Method::POST, "/api/product/v1/runs", &unknown).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(malformed["error"]["field"], "request_body");
    assert_eq!(
        fs::read_dir(fixture.root.join("artifacts/backtests"))
            .expect("artifact root should exist")
            .count(),
        1,
        "rejected requests must not create Run directories"
    );
}

#[cfg(any(unix, windows))]
#[tokio::test]
async fn backtest_creation_rejects_symlinked_artifact_root_escape() {
    let fixture = Fixture::new("create-backtest-root-symlink");
    let artifact_root = fixture.root.join("artifacts/backtests");
    let outside_root = fixture.root.join("outside-backtests");
    fs::rename(&artifact_root, &outside_root).expect("artifact root should move");
    create_directory_symlink(&outside_root, &artifact_root)
        .expect("artifact root symlink should be created");
    let request = json!({
        "strategy_id": "ema-cross",
        "strategy_version_id": "ema-cross@v1",
        "environment": "backtest",
        "data_ref": "dataset://fixtures/ema-cross",
        "venue_ref": "venue://simulated/BINANCE",
        "starting_balance": "1000000 USDT",
        "quotes": 120,
        "trade_size": "0.001000",
        "fast_period": 3,
        "slow_period": 5
    });

    let (status, body) = router_json_body(
        &fixture.router(),
        Method::POST,
        "/api/product/v1/runs",
        &request,
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"]["code"], "product_source_invalid");
    assert_eq!(body["error"]["field"], "result_root_containment");
}

#[tokio::test]
async fn run_routes_are_read_only_and_schema_compatible() {
    let fixture = Fixture::new("run-routes");
    let router = fixture.router();
    let list_path = "/api/product/v1/runs";
    let detail_path = "/api/product/v1/runs/ema-cross-live-001";
    let metrics_path = "/api/product/v1/runs/backtest-001/metrics";
    let report_path = "/api/product/v1/runs/backtest-001/report";
    let analysis_path = "/api/product/v1/runs/backtest-001/analysis";

    let (status, list) = router_json(&router, Method::GET, list_path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        list["schema_version"],
        "ntpro.product_api.run_list.response.v1"
    );
    assert_eq!(list["data"].as_array().map(Vec::len), Some(2));
    assert_read_only_boundaries(&list);
    validate_openapi_instance("RunListResponse", &list);

    let (status, detail) = router_json(&router, Method::GET, detail_path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        detail["schema_version"],
        "ntpro.product_api.run_detail.response.v1"
    );
    assert_eq!(detail["data"]["environment"], "live");
    assert_eq!(detail["data"]["risk"]["status"], "blocked");
    assert_eq!(
        detail["data"]["capabilities"]["order_submission_allowed"],
        false
    );
    assert_read_only_boundaries(&detail);
    validate_openapi_instance("RunDetailResponse", &detail);

    let (status, metrics) = router_json(&router, Method::GET, metrics_path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        metrics["schema_version"],
        "ntpro.product_api.run_metrics.response.v1"
    );
    assert_eq!(metrics["data"]["run_id"], "backtest-001");
    assert_eq!(metrics["data"]["metrics"]["quotes"], 120);
    assert_eq!(metrics["data"]["boundaries"]["read_only"], true);
    assert_read_only_boundaries(&metrics);
    validate_openapi_instance("RunMetricsResponse", &metrics);

    let (status, report) = router_json(&router, Method::GET, report_path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        report["schema_version"],
        "ntpro.product_api.run_report.response.v1"
    );
    assert_eq!(report["data"]["run_id"], "backtest-001");
    assert_eq!(report["data"]["trades"].as_array().map(Vec::len), Some(3));
    assert_eq!(
        report["data"]["positions"].as_array().map(Vec::len),
        Some(3)
    );
    assert_read_only_boundaries(&report);
    validate_openapi_instance("RunReportResponse", &report);

    let (status, analysis) = router_json(&router, Method::GET, analysis_path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        analysis["schema_version"],
        "ntpro.product_api.run_analysis.response.v1"
    );
    assert_eq!(analysis["data"]["run_id"], "backtest-001");
    assert_eq!(analysis["data"]["risk"]["closed_positions"], 3);
    assert_eq!(
        analysis["data"]["timeline"].as_array().map(Vec::len),
        Some(13)
    );
    assert_read_only_boundaries(&analysis);
    validate_openapi_instance("RunAnalysisResponse", &analysis);

    let (status, unavailable) = router_json(
        &router,
        Method::GET,
        "/api/product/v1/runs/ema-cross-live-001/metrics",
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(unavailable["error"]["code"], "run_not_found");
    assert_eq!(unavailable["error"]["field"], "run_metrics");

    let (status, filtered) = router_json(
        &router,
        Method::GET,
        "/api/product/v1/runs?environment=sandbox&lifecycle=running",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(filtered["data"].as_array().map(Vec::len), Some(0));
    validate_openapi_instance("RunListResponse", &filtered);

    let (status, missing) = router_json(&router, Method::GET, "/api/product/v1/runs/missing").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(missing["error"]["code"], "run_not_found");
    validate_openapi_instance("ProductErrorResponse", &missing);

    let (status, malformed) = router_json(&router, Method::GET, "/api/product/v1/runs/%FF").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(malformed["error"]["code"], "product_query_invalid");
    assert_eq!(malformed["error"]["field"], "run_id");

    for path in [
        list_path,
        detail_path,
        metrics_path,
        report_path,
        analysis_path,
    ] {
        for method in [
            Method::HEAD,
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
                Some(if path == list_path {
                    "GET, POST"
                } else {
                    "GET"
                })
            );
            if method != Method::HEAD {
                assert_eq!(body["error"]["code"], "product_method_not_allowed");
                validate_openapi_instance("ProductErrorResponse", &body);
            }
        }
    }

    for path in [detail_path, metrics_path, report_path, analysis_path] {
        let (status, headers, body) = router_json_with_headers(&router, Method::POST, path).await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED, "POST {path}");
        assert_eq!(
            headers.get(ALLOW).and_then(|value| value.to_str().ok()),
            Some("GET")
        );
        assert_eq!(body["error"]["code"], "product_method_not_allowed");
    }
}

#[tokio::test]
async fn run_metrics_fail_closed_for_missing_corrupt_and_mismatched_artifacts() {
    type ArtifactMutationCase = (&'static str, fn(&mut Value), &'static str, &'static str);

    let fixture = Fixture::new("run-metrics-negative");
    let result_path = fixture
        .root
        .join("artifacts/backtests/backtest-001/summary.json");
    let route = "/api/product/v1/runs/backtest-001/metrics";

    fs::remove_file(&result_path).expect("result fixture should be removed");
    let (status, missing) = router_json(&fixture.router(), Method::GET, route).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(missing["error"]["code"], "product_source_unavailable");
    assert_eq!(missing["error"]["field"], "result_artifact");

    fs::write(&result_path, b"{not-json").expect("corrupt result should be written");
    let (status, corrupt) = router_json(&fixture.router(), Method::GET, route).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(corrupt["error"]["code"], "product_source_invalid");
    assert_eq!(corrupt["error"]["field"], "result_sha256");

    let cases: [ArtifactMutationCase; 21] = [
        (
            "schema",
            |v| v["schema_version"] = json!("v2"),
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "run",
            |v| v["run_id"] = json!("other"),
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "strategy",
            |v| v["strategy_id"] = json!("other"),
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "version",
            |v| v["strategy_version_id"] = json!("ema-cross@v2"),
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "version-hash",
            |v| {
                v["strategy_version_content_hash"] = json!(
                    "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                );
            },
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "data-ref",
            |v| v["data_ref"] = json!("dataset://fixtures/other"),
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "data-hash",
            |v| {
                v["data_sha256"] = json!(
                    "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                );
            },
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "config-ref",
            |v| v["config_ref"] = json!("node-config:other#product_runs"),
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "config-hash",
            |v| {
                v["config_sha256"] = json!(
                    "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
                );
            },
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "result-ref",
            |v| v["result_ref"] = json!("artifact://backtests/other/summary.json"),
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "instrument",
            |v| v["instrument_id"] = json!("OTHER.BINANCE"),
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "strategy-name",
            |v| v["strategy"] = json!(""),
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "trade-size",
            |v| v["parameters"]["trade_size"] = json!("1.000000"),
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "fast",
            |v| v["parameters"]["fast_period"] = json!(4),
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "slow",
            |v| v["parameters"]["slow_period"] = json!(6),
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "time",
            |v| v["backtest_start"] = json!("1735689800000000000"),
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "quotes",
            |v| v["metrics"]["quotes"] = json!(119),
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "iterations",
            |v| v["metrics"]["iterations"] = json!(0),
            "product_source_invalid",
            "result_artifact",
        ),
        (
            "boundary-read-only",
            |v| v["boundaries"]["read_only"] = json!(false),
            "product_boundary_violation",
            "result_boundaries",
        ),
        (
            "boundary-submit",
            |v| v["boundaries"]["order_submission_allowed"] = json!(true),
            "product_boundary_violation",
            "result_boundaries",
        ),
        (
            "unknown-field",
            |v| v["unexpected"] = json!(true),
            "product_source_invalid",
            "result_artifact",
        ),
    ];
    for (name, mutate, expected_code, expected_field) in cases {
        write_valid_backtest_artifact(&fixture.root, &fixture.config_path);
        let mut mismatched: Value = serde_json::from_slice(
            &fs::read(&result_path).expect("result fixture should be readable"),
        )
        .expect("result fixture should parse");
        mutate(&mut mismatched);
        write_json(&result_path, &mismatched);
        fixture.trust_current_backtest_result();
        let (status, mismatch) = router_json(&fixture.router(), Method::GET, route).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR, "{name}");
        assert_eq!(mismatch["error"]["code"], expected_code, "{name}");
        assert_eq!(mismatch["error"]["field"], expected_field, "{name}");
    }
}

#[tokio::test]
async fn run_metrics_reject_in_place_result_tampering_against_trusted_hash() {
    let fixture = Fixture::new("run-metrics-result-hash");
    let result_path = fixture
        .root
        .join("artifacts/backtests/backtest-001/summary.json");
    let mut artifact: Value =
        serde_json::from_slice(&fs::read(&result_path).expect("result fixture should be readable"))
            .expect("result fixture should parse");
    artifact["metrics"]["total_orders"] = json!(999);
    write_json(&result_path, &artifact);

    let (status, body) = router_json(
        &fixture.router(),
        Method::GET,
        "/api/product/v1/runs/backtest-001/metrics",
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"]["code"], "product_source_invalid");
    assert_eq!(body["error"]["field"], "result_sha256");
}

#[tokio::test]
async fn run_report_fails_closed_for_missing_tampered_and_mismatched_details() {
    type DetailsMutationCase = (&'static str, fn(&mut Value));

    let fixture = Fixture::new("run-report-negative");
    let details_path = fixture
        .root
        .join("artifacts/backtests/backtest-001/details.json");
    let route = "/api/product/v1/runs/backtest-001/report";
    let original = fs::read(&details_path).expect("details fixture should be readable");

    fs::remove_file(&details_path).expect("details fixture should be removed");
    let (status, missing) = router_json(&fixture.router(), Method::GET, route).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(missing["error"]["field"], "result_artifact");

    fs::write(&details_path, b"{not-json").expect("corrupt details should be written");
    let (status, tampered) = router_json(&fixture.router(), Method::GET, route).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(tampered["error"]["field"], "details_sha256");

    let cases: [DetailsMutationCase; 17] = [
        ("run", |value| value["run_id"] = json!("other")),
        ("position-count", |value| {
            value["positions"]
                .as_array_mut()
                .expect("positions should be an array")
                .pop();
        }),
        ("boundary", |value| {
            value["boundaries"]["order_submission_allowed"] = json!(true);
        }),
        ("trade-side", |value| {
            value["trades"][0]["side"] = json!("UNKNOWN");
        }),
        ("commission-currency", |value| {
            value["trades"][0]["commission"] = json!("1.00 USD");
        }),
        ("trade-equity-currency", |value| {
            value["trades"][0]["currency"] = json!("USD");
            value["trades"][0]["commission"] = json!("1.00 USD");
        }),
        ("duplicate-trade-id", |value| {
            value["trades"][1]["trade_id"] = value["trades"][0]["trade_id"].clone();
        }),
        ("duplicate-position-id", |value| {
            value["positions"][1]["position_id"] = value["positions"][0]["position_id"].clone();
        }),
        ("position-trade-count", |value| {
            value["positions"][0]["trade_count"] = json!(2);
        }),
        ("orphan-position", |value| {
            value["trades"][0]["position_id"] = Value::Null;
        }),
        ("position-duration", |value| {
            value["positions"][0]["duration_ns"] = json!("1");
        }),
        ("position-equity-account", |value| {
            value["positions"][0]["account_id"] = json!("OTHER-001");
        }),
        ("position-pnl-currency", |value| {
            value["positions"][0]["realized_pnl"] = json!("1.00 USD");
        }),
        ("equity-account", |value| {
            value["equity_curve"][1]["account_id"] = json!("OTHER-001");
        }),
        ("equity-currency", |value| {
            value["equity_curve"][0]["total"] = json!("100000.00 USD");
        }),
        ("equity-balance", |value| {
            value["equity_curve"][0]["total"] = json!("999999.00 USDT");
        }),
        ("unknown", |value| value["unexpected"] = json!(true)),
    ];
    for (name, mutate) in cases {
        let mut value: Value = serde_json::from_slice(&original).expect("details should parse");
        mutate(&mut value);
        write_json(&details_path, &value);
        fixture.trust_current_backtest_details();
        let (status, body) = router_json(&fixture.router(), Method::GET, route).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR, "{name}");
        assert_eq!(body["error"]["code"], "product_source_invalid", "{name}");
        assert_eq!(body["error"]["field"], "details_artifact", "{name}");
    }
}

#[tokio::test]
async fn run_analysis_fails_closed_for_missing_tampered_and_semantic_drift() {
    let route = "/api/product/v1/runs/backtest-001/analysis";

    let missing_fixture = Fixture::new("run-analysis-missing");
    fs::remove_file(
        missing_fixture
            .root
            .join("artifacts/backtests/backtest-001/analysis.json"),
    )
    .expect("analysis fixture should be removed");
    let (status, missing) = router_json(&missing_fixture.router(), Method::GET, route).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(missing["error"]["field"], "result_artifact");

    let tampered_fixture = Fixture::new("run-analysis-tampered");
    let tampered_path = tampered_fixture
        .root
        .join("artifacts/backtests/backtest-001/analysis.json");
    fs::write(&tampered_path, b"{not-json").expect("tampered analysis should be written");
    let (status, tampered) = router_json(&tampered_fixture.router(), Method::GET, route).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(tampered["error"]["field"], "analysis_sha256");

    let semantic_fixture = Fixture::new("run-analysis-semantic");
    let semantic_path = semantic_fixture
        .root
        .join("artifacts/backtests/backtest-001/analysis.json");
    let original = fs::read(&semantic_path).expect("analysis fixture should be readable");
    let cases: [(&str, AnalysisMutation); 6] = [
        ("risk", |value| {
            value["risk"]["max_drawdown_rate"] = json!("0.500000000000");
        }),
        ("timeline-id", |value| {
            value["timeline"][0]["event_id"] = json!("event-999999");
        }),
        ("timeline-event", |value| {
            value["timeline"][0]["event_type"] = json!("run_completed");
        }),
        ("provenance", |value| {
            value["provenance"]["summary_sha256"] =
                json!("sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc");
        }),
        ("boundary", |value| {
            value["boundaries"]["order_submission_allowed"] = json!(true);
        }),
        ("unknown", |value| value["unexpected"] = json!(true)),
    ];
    for (name, mutate) in cases {
        let mut value: Value =
            serde_json::from_slice(&original).expect("analysis fixture should parse");
        mutate(&mut value);
        write_json(&semantic_path, &value);
        semantic_fixture.trust_current_backtest_analysis();
        let (status, body) = router_json(&semantic_fixture.router(), Method::GET, route).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR, "{name}");
        assert_eq!(body["error"]["code"], "product_source_invalid", "{name}");
        assert_eq!(body["error"]["field"], "analysis_artifact", "{name}");
    }
}

#[tokio::test]
async fn run_analysis_excludes_open_position_realized_pnl_from_closed_outcomes() {
    let fixture = Fixture::new("run-analysis-partial-close");
    let details_path = fixture
        .root
        .join("artifacts/backtests/backtest-001/details.json");
    let mut details: Value = serde_json::from_slice(
        &fs::read(&details_path).expect("details fixture should be readable"),
    )
    .expect("details fixture should parse");
    details["positions"][0]["side"] = json!("LONG");
    details["positions"][0]["sell_quantity"] = json!("0.000500");
    details["positions"][0]["realized_return"] = json!("0.000050000000");
    details["positions"][0]["realized_pnl"] = json!("0.00050000 USDT");
    details["positions"][0]["ts_closed"] = Value::Null;
    details["positions"][0]["duration_ns"] = json!("0");
    write_json(&details_path, &details);
    let details_sha256 = sha256_bytes_ref(
        &fs::read(&details_path).expect("changed details fixture should be readable"),
    );
    fixture.trust_current_backtest_details();

    let analysis_path = fixture
        .root
        .join("artifacts/backtests/backtest-001/analysis.json");
    let mut analysis: Value = serde_json::from_slice(
        &fs::read(&analysis_path).expect("analysis fixture should be readable"),
    )
    .expect("analysis fixture should parse");
    analysis["risk"]["open_positions"] = json!(1);
    analysis["risk"]["closed_positions"] = json!(2);
    analysis["risk"]["profitable_positions"] = json!(0);
    analysis["risk"]["losing_positions"] = json!(2);
    analysis["provenance"]["details_sha256"] = json!(details_sha256);
    let timeline = analysis["timeline"]
        .as_array_mut()
        .expect("analysis timeline should be an array");
    timeline.retain(|event| {
        event["event_type"].as_str() != Some("position_closed")
            || event["entity_ref"].as_str() != Some("position://P-1")
    });
    for (index, event) in timeline.iter_mut().enumerate() {
        event["event_id"] = json!(format!("event-{index:06}"));
    }
    write_json(&analysis_path, &analysis);
    fixture.trust_current_backtest_analysis();

    let (status, body) = router_json(
        &fixture.router(),
        Method::GET,
        "/api/product/v1/runs/backtest-001/analysis",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["risk"]["open_positions"], 1);
    assert_eq!(body["data"]["risk"]["closed_positions"], 2);
    assert_eq!(body["data"]["risk"]["profitable_positions"], 0);
    assert_eq!(body["data"]["risk"]["losing_positions"], 2);
}

#[cfg(any(unix, windows))]
#[test]
fn nofollow_directory_open_rejects_path_replacement() {
    let fixture = Fixture::new("run-metrics-path-replacement");
    let run_root = fixture.root.join("artifacts/backtests/backtest-001");
    let original = fixture.root.join("original-run-root");
    fs::rename(&run_root, &original).expect("original run root should move");
    create_directory_symlink(&original, &run_root).expect("replacement symlink should be created");

    let error = open_absolute_directory_nofollow(&run_root)
        .expect_err("a replaced directory path must not validate");
    assert_eq!(error.kind, ProductErrorKind::SourceInvalid);
    assert_eq!(error.field, "result_root_containment");
}

#[cfg(any(unix, windows))]
#[tokio::test]
async fn run_metrics_reject_symlink_path_escape() {
    let fixture = Fixture::new("run-metrics-symlink");
    let result_path = fixture
        .root
        .join("artifacts/backtests/backtest-001/summary.json");
    let outside = fixture.root.join("outside-result.json");
    fs::rename(&result_path, &outside).expect("result should move outside artifact root");
    create_file_symlink(&outside, &result_path).expect("result symlink should be created");

    let (status, body) = router_json(
        &fixture.router(),
        Method::GET,
        "/api/product/v1/runs/backtest-001/metrics",
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"]["code"], "product_source_invalid");
    assert_eq!(body["error"]["field"], "result_artifact_type");
}

#[cfg(any(unix, windows))]
#[tokio::test]
async fn run_report_rejects_symlink_path_escape() {
    let fixture = Fixture::new("run-report-symlink");
    let details_path = fixture
        .root
        .join("artifacts/backtests/backtest-001/details.json");
    let outside = fixture.root.join("outside-details.json");
    fs::rename(&details_path, &outside).expect("details should move outside artifact root");
    create_file_symlink(&outside, &details_path).expect("details symlink should be created");

    let (status, body) = router_json(
        &fixture.router(),
        Method::GET,
        "/api/product/v1/runs/backtest-001/report",
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"]["code"], "product_source_invalid");
    assert_eq!(body["error"]["field"], "result_artifact_type");
}

#[cfg(any(unix, windows))]
#[tokio::test]
async fn run_analysis_rejects_symlink_path_escape() {
    let fixture = Fixture::new("run-analysis-symlink");
    let analysis_path = fixture
        .root
        .join("artifacts/backtests/backtest-001/analysis.json");
    let outside = fixture.root.join("outside-analysis.json");
    fs::rename(&analysis_path, &outside).expect("analysis should move outside artifact root");
    create_file_symlink(&outside, &analysis_path).expect("analysis symlink should be created");

    let (status, body) = router_json(
        &fixture.router(),
        Method::GET,
        "/api/product/v1/runs/backtest-001/analysis",
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"]["code"], "product_source_invalid");
    assert_eq!(body["error"]["field"], "result_artifact_type");
}

#[cfg(any(unix, windows))]
#[tokio::test]
async fn run_metrics_reject_symlinked_artifact_root_escape() {
    let fixture = Fixture::new("run-metrics-root-symlink");
    let artifact_root = fixture.root.join("artifacts/backtests");
    let outside_root = fixture.root.join("outside-backtests");
    fs::rename(&artifact_root, &outside_root).expect("artifact root should move");
    create_directory_symlink(&outside_root, &artifact_root)
        .expect("artifact root symlink should be created");

    let (status, body) = router_json(
        &fixture.router(),
        Method::GET,
        "/api/product/v1/runs/backtest-001/metrics",
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"]["code"], "product_source_invalid");
    assert_eq!(body["error"]["field"], "result_root_containment");
}

#[tokio::test]
async fn tracked_frontend_fixtures_match_real_rust_routes() {
    let fixture = Fixture::new("frontend-fixtures");
    let router = fixture.router();
    let cases = [
        (
            "strategy-list.json",
            StatusCode::OK,
            "/api/product/v1/strategies",
            "StrategyListResponse",
        ),
        (
            "strategy-detail.json",
            StatusCode::OK,
            "/api/product/v1/strategies/ema-cross",
            "StrategyDetailResponse",
        ),
        (
            "strategy-version-list.json",
            StatusCode::OK,
            "/api/product/v1/strategies/ema-cross/versions",
            "StrategyVersionListResponse",
        ),
        (
            "strategy-version-detail.json",
            StatusCode::OK,
            "/api/product/v1/strategies/ema-cross/versions/ema-cross@v1",
            "StrategyVersionDetailResponse",
        ),
        (
            "run-list.json",
            StatusCode::OK,
            "/api/product/v1/runs",
            "RunListResponse",
        ),
        (
            "run-detail.json",
            StatusCode::OK,
            "/api/product/v1/runs/ema-cross-live-001",
            "RunDetailResponse",
        ),
        (
            "run-metrics.json",
            StatusCode::OK,
            "/api/product/v1/runs/backtest-001/metrics",
            "RunMetricsResponse",
        ),
        (
            "run-report.json",
            StatusCode::OK,
            "/api/product/v1/runs/backtest-001/report",
            "RunReportResponse",
        ),
        (
            "run-analysis.json",
            StatusCode::OK,
            "/api/product/v1/runs/backtest-001/analysis",
            "RunAnalysisResponse",
        ),
        (
            "error.json",
            StatusCode::NOT_FOUND,
            "/api/product/v1/runs/missing",
            "ProductErrorResponse",
        ),
    ];

    for (name, expected_status, path, schema) in cases {
        let (status, mut value) = router_json(&router, Method::GET, path).await;
        assert_eq!(status, expected_status, "{path}");
        value["request_id"] = json!("product-0000000000000000-0000000000000000");
        validate_openapi_instance(schema, &value);
        assert_tracked_frontend_fixture(name, &value);
    }
}

#[tokio::test]
async fn legacy_strategy_source_remains_readable_but_version_routes_fail_closed() {
    let fixture = Fixture::new("legacy-strategy-source");
    fixture.write_config(legacy_config_without_strategy_version());
    let mut identity = fixture.identity.clone();
    identity.identities.strategy_version_content_hash.clear();
    identity.provenance.generated_at_unix_ms = unix_time_ms().saturating_add(1_000);
    fixture.write_identity(&identity);

    let router = fixture.router();
    let (status, strategy) =
        router_json(&router, Method::GET, "/api/product/v1/strategies/ema-cross").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(strategy["data"]["strategy_id"], "ema-cross");

    let (status, version) = router_json(
        &router,
        Method::GET,
        "/api/product/v1/strategies/ema-cross/versions",
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(version["error"]["code"], "product_source_invalid");
    assert_eq!(version["error"]["field"], "strategy_version_content_hash");
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
fn openapi_is_authoritative_and_declares_exact_product_routes() {
    let openapi: Value =
        serde_json::from_str(PRODUCT_OPENAPI_SOURCE).expect("OpenAPI source must be valid JSON");
    assert_eq!(openapi["openapi"], "3.1.0");
    assert_eq!(openapi["x-ntpro-authoritative-contract"], true);
    assert_eq!(
        openapi["components"]["schemas"]["Strategy"]["properties"]["default_version_id"]["$ref"],
        "#/components/schemas/StrategyVersionId"
    );
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
        [
            "/demo-runs",
            "/demo-runs/{run_id}/actions",
            "/run-comparisons",
            "/runs",
            "/runs/{run_id}",
            "/runs/{run_id}/analysis",
            "/runs/{run_id}/demo-snapshot",
            "/runs/{run_id}/metrics",
            "/runs/{run_id}/report",
            "/runs/{run_id}/reproduction",
            "/strategies",
            "/strategies/{strategy_id}",
            "/strategies/{strategy_id}/versions",
            "/strategies/{strategy_id}/versions/{version_id}"
        ]
    );
    for (path_name, path) in paths {
        let methods = path.as_object().expect("path item must be an object");
        let expected_methods =
            if matches!(path_name.as_str(), "/runs" | "/runs/{run_id}/reproduction") {
                vec!["get", "post"]
            } else if matches!(
                path_name.as_str(),
                "/demo-runs" | "/demo-runs/{run_id}/actions"
            ) {
                vec!["post"]
            } else {
                vec!["get"]
            };
        assert_eq!(
            methods.keys().map(String::as_str).collect::<Vec<_>>(),
            expected_methods
        );
        if let Some(operation) = path.get("get") {
            assert_eq!(
                operation["security"],
                json!([{"InstitutionCookie": []}, {"OperatorCookie": []}])
            );
            assert_eq!(
                operation["responses"]["403"]["$ref"],
                "#/components/responses/ProductError"
            );
            let method_response =
                if matches!(path_name.as_str(), "/runs" | "/runs/{run_id}/reproduction") {
                    "#/components/responses/ProductRunMethodNotAllowed"
                } else {
                    "#/components/responses/ProductMethodNotAllowed"
                };
            assert_eq!(operation["responses"]["405"]["$ref"], method_response);
        }
        if path_name == "/runs" {
            assert_eq!(path["post"]["security"], json!([{"InstitutionCookie": []}]));
            assert_eq!(path["post"]["operationId"], "createBacktestRun");
        } else if path_name == "/runs/{run_id}/reproduction" {
            assert_eq!(path["post"]["security"], json!([{"InstitutionCookie": []}]));
            assert_eq!(path["post"]["operationId"], "reproduceBacktestRun");
        } else if path_name == "/demo-runs" {
            assert_eq!(path["post"]["security"], json!([{"InstitutionCookie": []}]));
            assert_eq!(path["post"]["operationId"], "createDemoRun");
        } else if path_name == "/demo-runs/{run_id}/actions" {
            assert_eq!(path["post"]["security"], json!([{"InstitutionCookie": []}]));
            assert_eq!(path["post"]["operationId"], "actOnDemoRun");
        }
    }

    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$ref": "#/components/schemas/StrategyVersionId",
        "components": openapi["components"].clone(),
    });
    let validator = jsonschema::draft202012::new(&schema)
        .expect("StrategyVersionId OpenAPI validator should build");
    assert!(validator.is_valid(&json!("ema-cross@v1")));
    assert!(!validator.is_valid(&json!(format!("ema-cross@{}", "v".repeat(129)))));
    assert!(!validator.is_valid(&json!(format!("{}@v1", "s".repeat(129)))));

    let run_schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$ref": "#/components/schemas/RunId",
        "components": openapi["components"].clone(),
    });
    let run_validator =
        jsonschema::draft202012::new(&run_schema).expect("RunId validator should build");
    assert!(run_validator.is_valid(&json!("ema-cross-live-001")));
    assert!(!run_validator.is_valid(&json!("..")));

    let create_schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$ref": "#/components/schemas/CreateBacktestRunRequest",
        "components": openapi["components"].clone(),
    });
    let create_validator = jsonschema::draft202012::new(&create_schema)
        .expect("CreateBacktestRunRequest validator should build");
    let mut create_request = json!({
        "strategy_id": "ema-cross",
        "strategy_version_id": "ema-cross@v1",
        "environment": "backtest",
        "data_ref": "dataset://fixtures/ema-cross",
        "venue_ref": "venue://simulated/BINANCE",
        "starting_balance": "1000000 USDT",
        "quotes": 120,
        "trade_size": "0.001000",
        "fast_period": 3,
        "slow_period": 5
    });
    assert!(create_validator.is_valid(&create_request));
    create_request["trade_size"] = json!("0.001");
    assert!(
        !create_validator.is_valid(&create_request),
        "public schema must reject precision that the service rejects"
    );
}

fn valid_config() -> String {
    config_with_computed_version_hash(
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

[strategy_version]
strategy_version_id = "ema-cross@v1"
strategy_id = "ema-cross"
version = "v1"
content_hash = "__STRATEGY_VERSION_CONTENT_HASH__"
code_ref = "git://NTPRO@e24de1825b66f9e7b9bfb2fc4662c928e56d6c18/crates/cli/src/strategy_session.rs#ema_cross_demo"
status = "registered"
created_at_unix_ms = 1767225600000

[strategy_version.parameter_schema]
"$schema" = "https://json-schema.org/draft/2020-12/schema"
type = "object"
"additionalProperties" = false
required = ["fast_period", "slow_period"]

[strategy_version.parameter_schema.properties.fast_period]
type = "integer"
const = 3

[strategy_version.parameter_schema.properties.slow_period]
type = "integer"
const = 5

[strategy_version.data_requirements]
venues = ["BINANCE"]
symbols = ["BTCUSDT.BINANCE"]
data_types = ["bar"]
timeframes = ["fixture_sequence"]
deterministic_replay_required = true

[strategy_version.risk_config]
risk_profile_ref = "node-config:#risk"
kill_switch_required = true
external_venue_connection_default = false
order_submission_default = false

[market]
venue = "BINANCE"

[execution]
venue = "BINANCE"

[mvp]
strategy_version = "v1"
backtest_run_id = "backtest-001"
backtest_result_ref = "artifact://backtests/backtest-001/summary.json"
account_id = "acct-sandbox-001"
environment = "sandbox"

[[product_runs]]
run_id = "backtest-001"
strategy_id = "ema-cross"
strategy_version_id = "ema-cross@v1"
environment = "backtest"
data_ref = "dataset://fixtures/ema-cross"
config_ref = "node-config:node.toml#product_runs"
adapter_ref = "adapter://backtest/simulated"
account_ref = "account://simulated/backtest-001"
venue_ref = "venue://simulated/BINANCE"
lifecycle = "completed"
result_status = "available"
result_ref = "artifact://backtests/backtest-001/summary.json"
backtest_config_sha256 = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
backtest_data_sha256 = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
backtest_result_sha256 = "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
backtest_details_sha256 = "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
backtest_analysis_sha256 = "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
backtest_trade_size = "0.001000"
backtest_quotes = 120
backtest_fast_period = 3
backtest_slow_period = 5
risk_status = "passed"
risk_ref = "node-config:node.toml#risk"
created_at_unix_ms = 1767225600000
started_at_unix_ms = 1767225600000
completed_at_unix_ms = 1767225660000
updated_at_unix_ms = 1767225660000
external_venue_connection = false
order_submission_allowed = false
order_mutation_allowed = false
automatic_retry_allowed = false
automatic_remediation_allowed = false
real_orders_submitted = false
trading_controls_enabled = false

[[product_runs]]
run_id = "ema-cross-live-001"
strategy_id = "ema-cross"
strategy_version_id = "ema-cross@v1"
environment = "live"
data_ref = "market://live/disabled"
config_ref = "node-config:node.toml#product_runs"
adapter_ref = "adapter://live/disabled"
account_ref = "account://live/unconfigured"
venue_ref = "venue://live/unconfigured/disabled"
lifecycle = "created"
result_status = "pending"
risk_status = "blocked"
risk_ref = "node-config:node.toml#risk"
created_at_unix_ms = 1785542400000
updated_at_unix_ms = 1785542400000
external_venue_connection = false
order_submission_allowed = false
order_mutation_allowed = false
automatic_retry_allowed = false
automatic_remediation_allowed = false
real_orders_submitted = false
trading_controls_enabled = false
"#,
    )
}

fn valid_demo_request() -> Value {
    json!({
        "strategy_id": "ema-cross",
        "strategy_version_id": "ema-cross@v1",
        "environment": "sandbox",
        "supervisor_node_id": "mvp-node-001",
        "account_ref": "account://sandbox/acct-sandbox-001",
        "venue_ref": "venue://sandbox/BINANCE",
        "user_confirmed": true
    })
}

#[cfg(unix)]
fn write_demo_fixture_node(root: &Path) -> PathBuf {
    let path = root.join("fixture-ntpro-node.sh");
    fs::write(
        &path,
        r#"#!/bin/sh
set -eu
node_id=""
output=""
stop_file=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --run-id) node_id="$2"; shift 2 ;;
    --output) output="$2"; shift 2 ;;
    --stop-file) stop_file="$2"; shift 2 ;;
    --max-runtime-ms|--heartbeat-interval-ms|--parent-pid|--shutdown-timeout-ms) shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$output/logs"
touch "$output/logs/stdout.log" "$output/logs/stderr.log" "$output/logs/events.log"
checksum() {
  if command -v shasum >/dev/null 2>&1; then
    digest="$(shasum -a 256 "$1" | awk '{print $1}')"
  else
    digest="$(sha256sum "$1" | awk '{print $1}')"
  fi
  printf 'sha256:%s' "$digest"
}
byte_len() { wc -c < "$1" | tr -d ' '; }
write_strategy_artifacts() {
  lifecycle="$1"
  now_ms="$2"
  artifact_now_ms=$((now_ms + 1))
  mkdir -p "$output/strategy"
  market_state="exhausted"
  reason="fixture_completed"
  if [ "$lifecycle" = "stopped" ]; then
    market_state="stopped"
    reason="user_stop"
  fi
  cat > "$output/strategy/session_status.json" <<EOF
{"schema_version":"ntpro.v09_strategy_session_status.v1","session_id":"$node_id","strategy_id":"ema-cross","state":"$lifecycle","reason":"$reason","updated_at_unix_ms":$now_ms,"artifacts":{"session_status":"$output/strategy/session_status.json","events":"$output/strategy/events.jsonl","market_status":"$output/strategy/market_status.json","market_events":"$output/strategy/market_events.jsonl","signal":"$output/strategy/signal.jsonl","order_intent":"$output/strategy/order_intent.jsonl","risk_decision":"$output/strategy/risk_decision.jsonl","summary":"$output/strategy/summary.json","manifest":"$output/strategy/manifest.json"}}
EOF
  cat > "$output/strategy/events.jsonl" <<EOF
{"schema_version":"ntpro.v09_strategy_session_event.v1","event_type":"fixture","session_id":"$node_id","strategy_id":"ema-cross","previous_state":null,"state":"$lifecycle","reason":"$reason","occurred_at_unix_ms":$now_ms}
EOF
  cat > "$output/strategy/market_status.json" <<EOF
{"schema_version":"ntpro.v09_market_stream_status.v1","session_id":"$node_id","strategy_id":"ema-cross","connection":"connected","state":"$market_state","source":"fixture_stream","event_count":1,"last_event_at_unix_ms":$now_ms,"updated_at_unix_ms":$artifact_now_ms}
EOF
  cat > "$output/strategy/market_events.jsonl" <<EOF
{"schema_version":"ntpro.v09_market_stream_event.v1","session_id":"$node_id","strategy_id":"ema-cross","event_type":"fixture_bar","source":"fixture_stream","seq":1,"symbol":"BTCUSDT.BINANCE","price":100.5,"event_at_unix_ms":$now_ms,"recorded_at_unix_ms":$now_ms}
EOF
  cat > "$output/strategy/signal.jsonl" <<EOF
{"schema_version":"ntpro.v09_strategy_signal.v1","session_id":"$node_id","strategy_id":"ema-cross","symbol":"BTCUSDT.BINANCE","signal":"sell","confidence":0.72,"market_event_seq":1,"generated_at":"$now_ms","generated_at_unix_ms":$now_ms}
EOF
  cat > "$output/strategy/order_intent.jsonl" <<EOF
{"schema_version":"ntpro.v09_order_intent.v1","session_id":"$node_id","strategy_id":"ema-cross","intent_id":"intent-demo-001","symbol":"BTCUSDT.BINANCE","side":"sell","order_type":"market","quantity":1.0,"source_signal":"sell","confidence":0.72,"market_event_seq":1,"signal_generated_at":"$now_ms","created_at":"$now_ms","created_at_unix_ms":$now_ms,"submission_allowed":false,"submission_status":"blocked"}
EOF
  cat > "$output/strategy/risk_decision.jsonl" <<EOF
{"schema_version":"ntpro.v09_risk_decision.v1","session_id":"$node_id","strategy_id":"ema-cross","decision_id":"decision-demo-001","intent_id":"intent-demo-001","symbol":"BTCUSDT.BINANCE","decision":"rejected","reasons":["order_submission_disabled"],"mode":"sandbox","order_submission":"disabled","kill_switch_enabled":true,"kill_switch_active":false,"account_state":"sandbox","market_state":"fresh","actual_submission":false,"evaluated_at":"$now_ms","evaluated_at_unix_ms":$now_ms}
EOF
  cat > "$output/strategy/summary.json" <<EOF
{"schema_version":"ntpro.v09_strategy_session_summary.v1","session_id":"$node_id","strategy_id":"ema-cross","state":"$lifecycle","event_count":1,"market_event_count":1,"signal_count":1,"intent_count":1,"risk_decision_count":1,"rejection_count":1,"actual_submission_count":0,"updated_at_unix_ms":$artifact_now_ms}
EOF
  cat > "$output/strategy/manifest.json.tmp" <<EOF
{"schema_version":"ntpro.v091_strategy_session_manifest.v1","session_id":"$node_id","strategy_id":"ema-cross","state":"$lifecycle","created_at_unix_ms":$now_ms,"updated_at_unix_ms":$now_ms,"artifacts":[
{"name":"session_status","path":"$output/strategy/session_status.json","format":"json","present":true,"record_count":1,"byte_len":$(byte_len "$output/strategy/session_status.json"),"checksum":"$(checksum "$output/strategy/session_status.json")"},
{"name":"events","path":"$output/strategy/events.jsonl","format":"jsonl","present":true,"record_count":1,"byte_len":$(byte_len "$output/strategy/events.jsonl"),"checksum":"$(checksum "$output/strategy/events.jsonl")"},
{"name":"market_status","path":"$output/strategy/market_status.json","format":"json","present":true,"record_count":1,"byte_len":$(byte_len "$output/strategy/market_status.json"),"checksum":"$(checksum "$output/strategy/market_status.json")"},
{"name":"market_events","path":"$output/strategy/market_events.jsonl","format":"jsonl","present":true,"record_count":1,"byte_len":$(byte_len "$output/strategy/market_events.jsonl"),"checksum":"$(checksum "$output/strategy/market_events.jsonl")"},
{"name":"signal","path":"$output/strategy/signal.jsonl","format":"jsonl","present":true,"record_count":1,"byte_len":$(byte_len "$output/strategy/signal.jsonl"),"checksum":"$(checksum "$output/strategy/signal.jsonl")"},
{"name":"order_intent","path":"$output/strategy/order_intent.jsonl","format":"jsonl","present":true,"record_count":1,"byte_len":$(byte_len "$output/strategy/order_intent.jsonl"),"checksum":"$(checksum "$output/strategy/order_intent.jsonl")"},
{"name":"risk_decision","path":"$output/strategy/risk_decision.jsonl","format":"jsonl","present":true,"record_count":1,"byte_len":$(byte_len "$output/strategy/risk_decision.jsonl"),"checksum":"$(checksum "$output/strategy/risk_decision.jsonl")"},
{"name":"summary","path":"$output/strategy/summary.json","format":"json","present":true,"record_count":1,"byte_len":$(byte_len "$output/strategy/summary.json"),"checksum":"$(checksum "$output/strategy/summary.json")"}]}
EOF
  mv "$output/strategy/manifest.json.tmp" "$output/strategy/manifest.json"
}
write_artifacts() {
  lifecycle="$1"
  previous="$2"
  stops="$3"
  now_ms="$(($(date +%s) * 1000))"
  stopped='{"availability":"unknown"}'
  if [ "$lifecycle" = "stopped" ]; then
    stopped="{\"availability\":\"available\",\"value\":\"$now_ms\"}"
  fi
  write_strategy_artifacts "$lifecycle" "$now_ms"
  cat > "$output/status.json.tmp" <<EOF
{
  "schema_version":"ntpro.node_status.v1",
  "node_id":"$node_id",
  "process_mode":"spawned_process",
  "config_path":{"availability":"available","value":"fixture.toml"},
  "artifact_root":{"availability":"available","value":"$output"},
  "lifecycle_state":"$lifecycle",
  "previous_lifecycle_state":"$previous",
  "data_connection":"not_configured",
  "execution_connection":"disconnected",
  "execution":{
    "gateway_id":{"availability":"available","value":"SANDBOX"},
    "connection":"disconnected",
    "started":{"availability":"available","value":false},
    "account_ref":{"availability":"available","value":"account://sandbox/acct-sandbox-001"},
    "orders_open":{"availability":"available","value":0},
    "orders_inflight":{"availability":"available","value":0},
    "orders_closed":{"availability":"available","value":0},
    "last_report_at":{"availability":"unknown"},
    "last_reconciliation_at":{"availability":"unknown"},
    "last_error":null
  },
  "risk":{
    "trading_state":"unknown",
    "health":"unknown",
    "command_count":{"availability":"available","value":0},
    "event_count":{"availability":"available","value":0},
    "rejections_total":{"availability":"available","value":0},
    "last_rejection":null,
    "last_error":null
  },
  "generated_at":{"availability":"available","value":"$now_ms"},
  "started_at":{"availability":"available","value":"$now_ms"},
  "stopped_at":$stopped,
  "last_transition_at":{"availability":"available","value":"$now_ms"},
  "last_error":null,
  "external_venue_connection":false,
  "real_orders_submitted":false
}
EOF
  mv "$output/status.json.tmp" "$output/status.json"
  cat > "$output/metrics.json.tmp" <<EOF
{
  "schema_version":"ntpro.node_metrics.v1",
  "node_id":"$node_id",
  "lifecycle_state":"$lifecycle",
  "previous_lifecycle_state":"$previous",
  "process_mode":"spawned_process",
  "uptime_ms":{"availability":"available","value":1},
  "starts_total":1,
  "stops_total":$stops,
  "state_transitions_total":2,
  "connection_counts":{
    "data_connected":0,
    "data_disconnected":0,
    "data_not_configured":1,
    "execution_connected":0,
    "execution_disconnected":1,
    "execution_not_configured":0
  },
  "last_error_summary":null,
  "generated_at":{"availability":"available","value":"$now_ms"},
  "started_at":{"availability":"available","value":"$now_ms"},
  "stopped_at":$stopped,
  "status_artifact_path":{"availability":"available","value":"$output/status.json"},
  "stdout_log_path":{"availability":"available","value":"$output/logs/stdout.log"},
  "stderr_log_path":{"availability":"available","value":"$output/logs/stderr.log"},
  "events_log_path":{"availability":"available","value":"$output/logs/events.log"},
  "strategy_signal_count":{"availability":"available","value":0},
  "strategy_rejection_count":{"availability":"available","value":0},
  "kill_switch_dry_run":{
    "artifact_path":{"availability":"available","value":"$output/kill-switch.json"},
    "artifact_status":{"availability":"available","value":"verified"},
    "kill_switch_active":{"availability":"available","value":false},
    "kill_switch_dry_run":{"availability":"available","value":true},
    "manual_approval_recorded":{"availability":"available","value":false},
    "approval_state":{"availability":"available","value":"not_approved"},
    "production_order_submission_allowed":{"availability":"available","value":false},
    "production_order_mutation_allowed":{"availability":"available","value":false},
    "production_order_state_reads_allowed":{"availability":"available","value":false},
    "listen_key_lifecycle_allowed":{"availability":"available","value":false},
    "production_order_submissions_attempted":{"availability":"available","value":0},
    "production_orders_submitted":{"availability":"available","value":0},
    "production_order_mutations_attempted":{"availability":"available","value":0},
    "production_order_state_reads_attempted":{"availability":"available","value":0},
    "dashboard_order_controls_enabled":{"availability":"available","value":false},
    "real_orders_submitted":{"availability":"available","value":false},
    "network_attempted":{"availability":"available","value":false},
    "values_are_exchange_truth":{"availability":"available","value":false}
  },
  "external_venue_connection":false,
  "real_orders_submitted":false
}
EOF
  mv "$output/metrics.json.tmp" "$output/metrics.json"
}
write_artifacts running starting 0
while [ ! -f "$stop_file" ]; do sleep 0.05; done
write_artifacts stopped running 1
"#,
    )
    .expect("Demo fixture node should be written");
    let mut permissions = fs::metadata(&path)
        .expect("Demo fixture node metadata should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("Demo fixture node should become executable");
    path
}

#[cfg(unix)]
fn write_demo_fixture_node_with_forbidden_metrics(root: &Path) -> PathBuf {
    let path = write_demo_fixture_node(root);
    let raw = fs::read_to_string(&path).expect("Demo fixture node should be readable");
    fs::write(
        &path,
        raw.replace(
            "\"production_order_submission_allowed\":{\"availability\":\"available\",\"value\":false}",
            "\"production_order_submission_allowed\":{\"availability\":\"available\",\"value\":true}",
        ),
    )
    .expect("forbidden Demo metrics fixture should be written");
    path
}

fn legacy_config_without_strategy_version() -> &'static str {
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
backtest_result_ref = "artifact://backtests/backtest-001/summary.json"
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
            strategy_version_content_hash: strategy_version_content_hash(config_path),
            backtest_run_id: "backtest-001".to_string(),
            backtest_result_ref: "artifact://backtests/backtest-001/summary.json".to_string(),
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

fn strategy_version_content_hash(config_path: &Path) -> String {
    let raw = fs::read_to_string(config_path).expect("fixture config should be readable");
    let config: toml::Value = toml::from_str(&raw).expect("fixture config should parse");
    config["strategy_version"]["content_hash"]
        .as_str()
        .expect("fixture version hash should be a string")
        .to_string()
}

fn write_json(path: &Path, value: &impl serde::Serialize) {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("fixture must serialize"),
    )
    .expect("fixture JSON should be written");
}

fn write_valid_backtest_artifact(root: &Path, config_path: &Path) -> String {
    let path = root.join("artifacts/backtests/backtest-001/summary.json");
    fs::create_dir_all(path.parent().expect("result path should have a parent"))
        .expect("result directory should be created");
    let raw = serde_json::to_vec_pretty(&json!({
        "schema_version": "ntpro.backtest_result.v1",
        "run_id": "backtest-001",
        "strategy_id": "ema-cross",
        "strategy_version_id": "ema-cross@v1",
        "strategy_version_content_hash": strategy_version_content_hash(config_path),
        "data_ref": "dataset://fixtures/ema-cross",
        "data_sha256": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "config_ref": "node-config:node.toml#product_runs",
        "config_sha256": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "result_ref": "artifact://backtests/backtest-001/summary.json",
        "instrument_id": "BTCUSDT.BINANCE",
        "strategy": "ema-cross",
        "parameters": {
            "trade_size": "0.001000",
            "fast_period": 3,
            "slow_period": 5
        },
        "backtest_start": "1735689600000000000",
        "backtest_end": "1735689719000000000",
        "metrics": {
            "quotes": 120,
            "iterations": 120,
            "total_events": 9,
            "total_orders": 3,
            "total_positions": 3,
            "pnl_stats": {"USDT": {
                "PnL (total)": "-0.004000000000",
                "PnL% (total)": "-0.000000400000",
                "Win Rate": "0.000000000000"
            }},
            "return_stats": {
                "Returns Volatility (252 days)": "NaN",
                "Sharpe Ratio (252 days)": "NaN",
                "Sortino Ratio (252 days)": "NaN"
            },
            "general_stats": {"Long Ratio": "0.330000000000"}
        },
        "boundaries": {
            "read_only": true,
            "external_venue_connection": false,
            "order_submission_allowed": false,
            "order_mutation_allowed": false,
            "automatic_retry_allowed": false,
            "automatic_remediation_allowed": false,
            "real_orders_submitted": false,
            "trading_controls_enabled": false
        }
    }))
    .expect("backtest result fixture must serialize");
    fs::write(&path, &raw).expect("backtest result fixture should be written");
    sha256_bytes_ref(&raw)
}

fn write_valid_backtest_details_artifact(root: &Path, config_path: &Path) -> String {
    let path = root.join("artifacts/backtests/backtest-001/details.json");
    fs::create_dir_all(path.parent().expect("details path should have a parent"))
        .expect("details directory should be created");
    let positions = (1..=3)
        .map(|index| {
            json!({
                "position_id": format!("P-{index}"),
                "account_id": "SIM-001",
                "side": "FLAT",
                "entry_side": if index % 2 == 0 { "SELL" } else { "BUY" },
                "peak_quantity": "0.001000",
                "buy_quantity": "0.001000",
                "sell_quantity": "0.001000",
                "avg_price_open": format!("{}.000000000000", 100 + index),
                "avg_price_close": format!("{}.000000000000", 100 + index),
                "realized_return": "-0.000100000000",
                "realized_pnl": "-0.00100000 USDT",
                "trade_count": 1,
                "ts_opened": format!("17356896{}000000000", index * 10),
                "ts_closed": format!("17356896{}500000000", index * 10),
                "duration_ns": "500000000"
            })
        })
        .collect::<Vec<_>>();
    let trades = (1..=3)
        .map(|index| {
            json!({
                "trade_id": format!("T-{index}"),
                "client_order_id": format!("O-{index}"),
                "venue_order_id": format!("V-{index}"),
                "position_id": format!("P-{index}"),
                "side": if index % 2 == 0 { "SELL" } else { "BUY" },
                "order_type": "MARKET",
                "quantity": "0.001000",
                "price": format!("{}.00", 100 + index),
                "currency": "USDT",
                "liquidity_side": "TAKER",
                "commission": "0.00010000 USDT",
                "ts_event": format!("17356896{}000000000", index * 10)
            })
        })
        .collect::<Vec<_>>();
    let raw = serde_json::to_vec_pretty(&json!({
        "schema_version": "ntpro.backtest_details.v1",
        "run_id": "backtest-001",
        "strategy_id": "ema-cross",
        "strategy_version_id": "ema-cross@v1",
        "strategy_version_content_hash": strategy_version_content_hash(config_path),
        "data_ref": "dataset://fixtures/ema-cross",
        "data_sha256": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "config_ref": "node-config:node.toml#product_runs",
        "config_sha256": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "details_ref": "artifact://backtests/backtest-001/details.json",
        "instrument_id": "BTCUSDT.BINANCE",
        "equity_basis": "account_balance_total",
        "trades": trades,
        "positions": positions,
        "equity_curve": [
            {
                "account_id": "SIM-001",
                "currency": "USDT",
                "total": "1000000.00000000 USDT",
                "free": "1000000.00000000 USDT",
                "locked": "0.00000000 USDT",
                "ts_event": "1735689600000000000"
            },
            {
                "account_id": "SIM-001",
                "currency": "USDT",
                "total": "999999.99700000 USDT",
                "free": "999999.99700000 USDT",
                "locked": "0.00000000 USDT",
                "ts_event": "1735689719000000000"
            }
        ],
        "boundaries": {
            "read_only": true,
            "external_venue_connection": false,
            "order_submission_allowed": false,
            "order_mutation_allowed": false,
            "automatic_retry_allowed": false,
            "automatic_remediation_allowed": false,
            "real_orders_submitted": false,
            "trading_controls_enabled": false
        }
    }))
    .expect("backtest details fixture must serialize");
    fs::write(&path, &raw).expect("backtest details fixture should be written");
    sha256_bytes_ref(&raw)
}

fn write_valid_backtest_analysis_artifact(
    root: &Path,
    config_path: &Path,
    result_sha256: &str,
    details_sha256: &str,
) -> String {
    let path = root.join("artifacts/backtests/backtest-001/analysis.json");
    let timeline = [
        ("run_started", "1735689600000000000", "run://backtest-001"),
        ("equity_updated", "1735689600000000000", "account://SIM-001"),
        ("trade_filled", "1735689610000000000", "trade://T-1"),
        ("position_opened", "1735689610000000000", "position://P-1"),
        ("position_closed", "1735689610500000000", "position://P-1"),
        ("trade_filled", "1735689620000000000", "trade://T-2"),
        ("position_opened", "1735689620000000000", "position://P-2"),
        ("position_closed", "1735689620500000000", "position://P-2"),
        ("trade_filled", "1735689630000000000", "trade://T-3"),
        ("position_opened", "1735689630000000000", "position://P-3"),
        ("position_closed", "1735689630500000000", "position://P-3"),
        ("equity_updated", "1735689719000000000", "account://SIM-001"),
        ("run_completed", "1735689719000000000", "run://backtest-001"),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (event_type, ts_event, entity_ref))| {
        json!({
            "event_id": format!("event-{index:06}"),
            "event_type": event_type,
            "ts_event": ts_event,
            "entity_ref": entity_ref
        })
    })
    .collect::<Vec<_>>();
    let raw = serde_json::to_vec_pretty(&json!({
        "schema_version": "ntpro.backtest_analysis.v1",
        "run_id": "backtest-001",
        "strategy_id": "ema-cross",
        "strategy_version_id": "ema-cross@v1",
        "strategy_version_content_hash": strategy_version_content_hash(config_path),
        "analysis_ref": "artifact://backtests/backtest-001/analysis.json",
        "instrument_id": "BTCUSDT.BINANCE",
        "risk": {
            "currency": "USDT",
            "starting_equity": "1000000.00000000 USDT",
            "ending_equity": "999999.99700000 USDT",
            "peak_equity": "1000000.00000000 USDT",
            "max_drawdown_amount": "0.00300000 USDT",
            "max_drawdown_rate": "0.000000003000",
            "max_drawdown_started_at": "1735689600000000000",
            "max_drawdown_trough_at": "1735689719000000000",
            "current_drawdown_amount": "0.00300000 USDT",
            "current_drawdown_rate": "0.000000003000",
            "open_positions": 0,
            "closed_positions": 3,
            "profitable_positions": 0,
            "losing_positions": 3
        },
        "drawdown_curve": [
            {
                "ts_event": "1735689600000000000",
                "equity": "1000000.00000000 USDT",
                "peak_equity": "1000000.00000000 USDT",
                "drawdown_amount": "0.00000000 USDT",
                "drawdown_rate": "0.000000000000"
            },
            {
                "ts_event": "1735689719000000000",
                "equity": "999999.99700000 USDT",
                "peak_equity": "1000000.00000000 USDT",
                "drawdown_amount": "0.00300000 USDT",
                "drawdown_rate": "0.000000003000"
            }
        ],
        "timeline": timeline,
        "provenance": {
            "generator": "nautilus_backtest::engine::BacktestEngine",
            "engine_mode": "engine-smoke",
            "data_ref": "dataset://fixtures/ema-cross",
            "data_sha256": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "config_ref": "node-config:node.toml#product_runs",
            "config_sha256": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "summary_ref": "artifact://backtests/backtest-001/summary.json",
            "summary_sha256": result_sha256,
            "details_ref": "artifact://backtests/backtest-001/details.json",
            "details_sha256": details_sha256
        },
        "boundaries": {
            "read_only": true,
            "external_venue_connection": false,
            "order_submission_allowed": false,
            "order_mutation_allowed": false,
            "automatic_retry_allowed": false,
            "automatic_remediation_allowed": false,
            "real_orders_submitted": false,
            "trading_controls_enabled": false
        }
    }))
    .expect("backtest analysis fixture must serialize");
    fs::write(&path, &raw).expect("backtest analysis fixture should be written");
    sha256_bytes_ref(&raw)
}

fn with_backtest_result_sha256(config: &str, result_sha256: &str) -> String {
    config
        .lines()
        .map(|line| {
            if line.starts_with("backtest_result_sha256 = ") {
                format!("backtest_result_sha256 = \"{result_sha256}\"")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn with_backtest_details_sha256(config: &str, details_sha256: &str) -> String {
    config
        .lines()
        .map(|line| {
            if line.starts_with("backtest_details_sha256 = ") {
                format!("backtest_details_sha256 = \"{details_sha256}\"")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn with_backtest_analysis_sha256(config: &str, analysis_sha256: &str) -> String {
    config
        .lines()
        .map(|line| {
            if line.starts_with("backtest_analysis_sha256 = ") {
                format!("backtest_analysis_sha256 = \"{analysis_sha256}\"")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn assert_tracked_frontend_fixture(name: &str, value: &Value) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/strategy-workbench/src/test/product-api-fixtures")
        .join(name);
    if std::env::var("NTPRO_UPDATE_PRODUCT_API_FIXTURES").as_deref() == Ok("1") {
        fs::create_dir_all(path.parent().expect("fixture path should have a parent"))
            .expect("frontend fixture directory should be created");
        write_json(&path, value);
    }
    let tracked: Value = serde_json::from_slice(
        &fs::read(&path).unwrap_or_else(|error| {
            panic!(
                "frontend fixture {} must exist; regenerate with NTPRO_UPDATE_PRODUCT_API_FIXTURES=1: {error}",
                path.display()
            )
        }),
    )
    .expect("tracked frontend fixture should be valid JSON");
    assert_eq!(
        tracked,
        *value,
        "frontend fixture {} drifted",
        path.display()
    );
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

async fn router_json_body(
    router: &Router,
    method: Method,
    path: &str,
    body: &Value,
) -> (StatusCode, Value) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(body).expect("request body should serialize"),
                ))
                .expect("request should build"),
        )
        .await
        .expect("router request should complete");
    let status = response.status();
    let raw = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .expect("response body should be readable");
    let value = serde_json::from_slice(&raw).expect("response should be valid JSON");
    (status, value)
}

async fn router_json_body_with_cookie(
    router: &Router,
    method: Method,
    path: &str,
    body: &Value,
    cookie: &str,
) -> (StatusCode, Value) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("content-type", "application/json")
                .header("cookie", cookie)
                .body(Body::from(
                    serde_json::to_vec(body).expect("request body should serialize"),
                ))
                .expect("request should build"),
        )
        .await
        .expect("router request should complete");
    let status = response.status();
    let raw = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .expect("response body should be readable");
    let value = serde_json::from_slice(&raw).expect("response should be valid JSON");
    (status, value)
}

async fn router_json_with_cookie(
    router: &Router,
    method: Method,
    path: &str,
    cookie: &str,
) -> (StatusCode, Value) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("cookie", cookie)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("router request should complete");
    let status = response.status();
    let raw = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .expect("response body should be readable");
    let value = serde_json::from_slice(&raw).expect("response should be valid JSON");
    (status, value)
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
