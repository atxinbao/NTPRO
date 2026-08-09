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

use super::run::*;
use super::strategy_version::*;
use super::*;
use crate::dashboard::server::dashboard_router;

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
        fs::write(
            &config_path,
            with_backtest_result_sha256(&valid_config(), &result_sha256),
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
fn run_projection_exposes_three_environments_with_closed_capabilities() {
    let fixture = Fixture::new("run-projection");
    let runs = load_product_runs(&fixture.state(), unix_time_ms())
        .expect("valid run manifest should project");
    let value = serde_json::to_value(runs).expect("runs should serialize");

    assert_eq!(value.as_array().map(Vec::len), Some(3));
    assert_eq!(value[0]["environment"], "backtest");
    assert_eq!(value[0]["lifecycle"], "completed");
    assert_eq!(value[0]["result"]["status"], "available");
    assert_eq!(value[1]["environment"], "sandbox");
    assert_eq!(value[1]["lifecycle"], "running");
    assert_eq!(value[2]["environment"], "live");
    assert_eq!(value[2]["lifecycle"], "created");
    assert_eq!(value[2]["risk"]["status"], "blocked");
    assert_eq!(value[2]["adapter_ref"], "adapter://live/disabled");
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
            "duplicate-id",
            (|raw: String| {
                raw.replace(
                    "run_id = \"mvp-strategy-001\"\nstrategy_id = \"ema-cross\"",
                    "run_id = \"backtest-001\"\nstrategy_id = \"ema-cross\"",
                )
            }) as fn(String) -> String,
            ProductErrorKind::SourceInvalid,
            "run_id",
        ),
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
            "sandbox-backtest-expectation",
            (|raw: String| {
                raw.replacen(
                    "result_status = \"pending\"\nrisk_status = \"active\"",
                    "result_status = \"pending\"\nbacktest_quotes = 120\nrisk_status = \"active\"",
                    1,
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
            "sandbox-account-reference",
            (|raw: String| {
                raw.replace(
                    "account://sandbox/acct-sandbox-001",
                    "account://sandbox/unknown-account",
                )
            }) as fn(String) -> String,
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
            "sandbox-data-reference",
            (|raw: String| {
                raw.replace(
                    "market://sandbox/BTCUSDT.BINANCE",
                    "market://sandbox/UNKNOWN.BINANCE",
                )
            }) as fn(String) -> String,
            ProductErrorKind::BoundaryViolation,
            "run_environment_references",
        ),
        (
            "sandbox-adapter-reference",
            (|raw: String| {
                raw.replace(
                    "adapter://sandbox/fixture-stream",
                    "adapter://sandbox/unknown",
                )
            }) as fn(String) -> String,
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
    assert_eq!(status, StatusCode::CREATED);
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
    assert!(run_root.join("run-manifest.json").is_file());

    let (status, detail) = router_json(
        &router,
        Method::GET,
        &format!("/api/product/v1/runs/{run_id}"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["data"]["run_id"], run_id);
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
    assert_eq!(list["data"].as_array().map(Vec::len), Some(3));
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

    let (status, list) = router_json(&router, Method::GET, list_path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        list["schema_version"],
        "ntpro.product_api.run_list.response.v1"
    );
    assert_eq!(list["data"].as_array().map(Vec::len), Some(3));
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

    let (status, unavailable) = router_json(
        &router,
        Method::GET,
        "/api/product/v1/runs/mvp-strategy-001/metrics",
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
    assert_eq!(filtered["data"][0]["run_id"], "mvp-strategy-001");
    validate_openapi_instance("RunListResponse", &filtered);

    let (status, missing) = router_json(&router, Method::GET, "/api/product/v1/runs/missing").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(missing["error"]["code"], "run_not_found");
    validate_openapi_instance("ProductErrorResponse", &missing);

    let (status, malformed) = router_json(&router, Method::GET, "/api/product/v1/runs/%FF").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(malformed["error"]["code"], "product_query_invalid");
    assert_eq!(malformed["error"]["field"], "run_id");

    for path in [list_path, detail_path, metrics_path] {
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

    for path in [detail_path, metrics_path] {
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
            "/runs",
            "/runs/{run_id}",
            "/runs/{run_id}/metrics",
            "/strategies",
            "/strategies/{strategy_id}",
            "/strategies/{strategy_id}/versions",
            "/strategies/{strategy_id}/versions/{version_id}"
        ]
    );
    for (path_name, path) in paths {
        let methods = path.as_object().expect("path item must be an object");
        let expected_methods = if path_name == "/runs" {
            vec!["get", "post"]
        } else {
            vec!["get"]
        };
        assert_eq!(
            methods.keys().map(String::as_str).collect::<Vec<_>>(),
            expected_methods
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
        let method_response = if path_name == "/runs" {
            "#/components/responses/ProductRunMethodNotAllowed"
        } else {
            "#/components/responses/ProductMethodNotAllowed"
        };
        assert_eq!(operation["responses"]["405"]["$ref"], method_response);
        if path_name == "/runs" {
            assert_eq!(path["post"]["security"], json!([{"InstitutionCookie": []}]));
            assert_eq!(path["post"]["operationId"], "createBacktestRun");
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
