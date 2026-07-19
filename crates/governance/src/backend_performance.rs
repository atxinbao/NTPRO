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

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail, ensure};
use serde_json::{Map, Value};

const EXPECTED_BASELINE_COMMIT: &str = "6f6896fbd6d5fe79e352f621cb0167debcf0d143";
const EXPECTED_WORKLOADS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "core_stack_str",
        "rust_core_runtime_agent",
        "crates/core/benches/stack_str.rs",
        "StackStr::new (short)",
        "cargo bench --locked --profile bench-lto -p nautilus-core --bench stack_str -- 'StackStr::new' --warm-up-time 1 --measurement-time 2 --sample-size 20",
    ),
    (
        "model_price",
        "rust_model_domain_agent",
        "crates/model/benches/price_criterion.rs",
        "Price::new",
        "cargo bench --locked --profile bench-lto -p nautilus-model --bench price_criterion -- 'Price::new' --warm-up-time 1 --measurement-time 2 --sample-size 20",
    ),
    (
        "data_engine_ingest",
        "rust_data_engine_agent",
        "crates/data/benches/engine.rs",
        "DataEngine ingest/process_data_trade",
        "cargo bench --locked --profile bench-lto -p nautilus-data --bench engine -- 'DataEngine ingest/process_data_trade' --warm-up-time 1 --measurement-time 2 --sample-size 20",
    ),
    (
        "execution_matching_core",
        "rust_execution_engine_agent",
        "crates/execution/benches/matching_core.rs",
        "matching_core/get_order/100",
        "cargo bench --locked --profile bench-lto -p nautilus-execution --bench matching_core -- 'matching_core/get_order/100' --warm-up-time 1 --measurement-time 2 --sample-size 20",
    ),
    (
        "live_runner_dispatch",
        "rust_live_runtime_agent",
        "crates/live/benches/runner.rs",
        "AsyncRunner dispatch/drain_data_events/100",
        "cargo bench --locked --profile bench-lto -p nautilus-live --bench runner -- 'AsyncRunner dispatch/drain_data_events/100' --warm-up-time 1 --measurement-time 2 --sample-size 20",
    ),
    (
        "network_rate_limiter",
        "rust_network_runtime_agent",
        "crates/network/benches/ratelimiter.rs",
        "ratelimiter/check_key_uncontended/single_key",
        "cargo bench --locked --profile bench-lto -p nautilus-network --bench ratelimiter -- 'ratelimiter/check_key_uncontended/single_key' --warm-up-time 1 --measurement-time 2 --sample-size 20",
    ),
];
const FORBIDDEN_CAPABILITIES: &[&str] = &[
    "backend_go_live",
    "production_submit",
    "production_mutation",
    "adapter_send",
    "live_exchange_request",
    "retry_scheduler",
    "automatic_remediation",
    "automatic_recovery",
    "trading_controls",
];

/// Counts emitted after validating the performance baseline contract.
#[derive(Debug)]
pub struct BackendPerformanceCounts {
    pub workloads: usize,
    pub observations: usize,
    pub build_measurements: usize,
    pub binary_measurements: usize,
    pub negative_cases: usize,
}

/// Validates the separately scoped v0.33.0 backend performance baseline.
///
/// # Errors
///
/// Returns an error when workload ownership, reproducibility metadata,
/// measurements, thresholds, source paths, or non-inheritance boundaries drift.
pub fn validate_backend_performance_baseline(
    baseline_path: &Path,
    negative_selftest: bool,
) -> Result<BackendPerformanceCounts> {
    ensure!(
        baseline_path.is_file(),
        "missing performance baseline: {}",
        baseline_path.display()
    );
    let root = repository_root(baseline_path)?;
    let baseline = load_json(baseline_path)?;
    let counts = validate_baseline(&baseline, &root)?;
    let negative_cases = if negative_selftest {
        run_negative_selftest(&baseline, &root)?
    } else {
        0
    };
    Ok(BackendPerformanceCounts {
        negative_cases,
        ..counts
    })
}

fn validate_baseline(value: &Value, root: &Path) -> Result<BackendPerformanceCounts> {
    let top = value
        .as_object()
        .context("performance baseline must be an object")?;
    ensure!(
        string(top, "schema_version")? == "ntpro.backend_performance_baseline.v1",
        "performance baseline schema mismatch"
    );
    ensure!(string(top, "task_id")? == "BPO-001", "task mismatch");
    ensure!(
        string(top, "status")? == "active_local_reference",
        "baseline status must be active_local_reference"
    );
    ensure!(
        string(top, "classification")? == "v33-separately-scoped",
        "backend freeze classification mismatch"
    );
    ensure!(
        string(top, "baseline_commit")? == EXPECTED_BASELINE_COMMIT,
        "baseline commit mismatch"
    );
    ensure_git_object(root, EXPECTED_BASELINE_COMMIT)?;

    validate_environment(object(top, "environment")?)?;
    validate_methodology(object(top, "methodology")?)?;
    validate_thresholds(object(top, "regression_policy")?)?;
    validate_hosted_handoff(object(top, "hosted_handoff")?)?;
    validate_non_inheritance(object(top, "non_inheritance")?)?;

    let workloads = array(top, "workloads")?;
    ensure!(
        workloads.len() == EXPECTED_WORKLOADS.len(),
        "expected exactly {} workloads",
        EXPECTED_WORKLOADS.len()
    );
    let mut ids = BTreeSet::new();
    let mut observations = 0;
    for workload in workloads {
        let workload = workload
            .as_object()
            .context("workload entry must be an object")?;
        let id = string(workload, "id")?;
        ensure!(ids.insert(id), "duplicate workload id: {id}");
        let (_, expected_owner, expected_source, expected_benchmark, expected_command) =
            EXPECTED_WORKLOADS
                .iter()
                .find(|(expected, _, _, _, _)| *expected == id)
                .with_context(|| format!("unexpected workload id: {id}"))?;
        ensure!(
            string(workload, "owner")? == *expected_owner,
            "workload owner mismatch: {id}"
        );
        ensure!(
            string(workload, "source")? == *expected_source,
            "workload source mismatch: {id}"
        );
        ensure!(
            root.join(expected_source).is_file(),
            "workload source missing: {expected_source}"
        );
        ensure!(
            string(workload, "benchmark_id")? == *expected_benchmark,
            "workload benchmark id mismatch: {id}"
        );
        let command = string(workload, "command")?;
        ensure!(
            command == *expected_command,
            "workload command mismatch: {id}"
        );
        for required in [
            "--warm-up-time 1",
            "--measurement-time 2",
            "--sample-size 20",
        ] {
            ensure!(
                command.contains(required),
                "workload command missing '{required}': {id}"
            );
        }
        ensure!(
            string(workload, "metric")? == "latency",
            "workload metric must be latency: {id}"
        );
        ensure!(
            string(workload, "unit")? == "ns",
            "workload unit must be ns: {id}"
        );
        let samples = number_array(workload, "observations")?;
        ensure!(
            samples.len() == 3,
            "workload must contain three session observations: {id}"
        );
        ensure!(
            samples
                .iter()
                .all(|sample| sample.is_finite() && *sample > 0.0),
            "workload observations must be positive finite numbers: {id}"
        );
        let median = number(workload, "median")?;
        let cv = number(workload, "coefficient_of_variation_pct")?;
        ensure!(
            relative_difference(median, calculated_median(&samples)) <= 0.001,
            "workload median does not match observations: {id}"
        );
        ensure!(
            (cv - coefficient_of_variation_pct(&samples)).abs() <= 0.02,
            "workload coefficient of variation does not match observations: {id}"
        );
        observations += samples.len();
    }
    for (id, _, _, _, _) in EXPECTED_WORKLOADS {
        ensure!(ids.contains(id), "missing workload: {id}");
    }

    let resources = object(top, "resource_baseline")?;
    let builds = array(resources, "builds")?;
    ensure!(builds.len() == 2, "expected clean and incremental builds");
    let mut build_kinds = BTreeSet::new();
    for build in builds {
        let build = build
            .as_object()
            .context("build measurement must be an object")?;
        let kind = string(build, "kind")?;
        ensure!(
            matches!(kind, "clean" | "incremental"),
            "unexpected build kind: {kind}"
        );
        ensure!(build_kinds.insert(kind), "duplicate build kind: {kind}");
        ensure_positive(build, "wall_time_seconds", "build wall time")?;
        ensure_positive(build, "user_cpu_seconds", "build user CPU")?;
        ensure_positive(build, "system_cpu_seconds", "build system CPU")?;
        ensure_positive(build, "max_rss_bytes", "build max RSS")?;
    }
    let binaries = array(resources, "binaries")?;
    ensure!(binaries.len() == 2, "expected two release binaries");
    let mut binary_names = BTreeSet::new();
    for binary in binaries {
        let binary = binary
            .as_object()
            .context("binary measurement must be an object")?;
        let name = string(binary, "name")?;
        ensure!(
            matches!(name, "nautilus" | "ntpro-node"),
            "unexpected binary: {name}"
        );
        ensure!(binary_names.insert(name), "duplicate binary: {name}");
        ensure_positive(binary, "size_bytes", "binary size")?;
        ensure!(
            string(binary, "sha256")?.len() == 64,
            "binary sha256 must contain 64 characters: {name}"
        );
    }

    Ok(BackendPerformanceCounts {
        workloads: workloads.len(),
        observations,
        build_measurements: builds.len(),
        binary_measurements: binaries.len(),
        negative_cases: 0,
    })
}

fn validate_environment(environment: &Map<String, Value>) -> Result<()> {
    for key in [
        "captured_at",
        "executor",
        "os",
        "kernel",
        "architecture",
        "cpu",
        "rustc",
        "cargo",
    ] {
        ensure!(
            !string(environment, key)?.is_empty(),
            "missing environment {key}"
        );
    }
    ensure_positive(environment, "logical_cpu_count", "logical CPU count")?;
    ensure_positive(environment, "memory_bytes", "memory bytes")?;
    Ok(())
}

fn validate_methodology(methodology: &Map<String, Value>) -> Result<()> {
    ensure!(
        string(methodology, "profile")? == "bench-lto",
        "published baseline profile must be bench-lto"
    );
    ensure!(
        methodology.get("warmup_seconds").and_then(Value::as_u64) == Some(1),
        "warmup must be one second"
    );
    ensure!(
        methodology
            .get("measurement_seconds")
            .and_then(Value::as_u64)
            == Some(2),
        "measurement window must be two seconds"
    );
    ensure!(
        methodology
            .get("criterion_sample_size")
            .and_then(Value::as_u64)
            == Some(20),
        "Criterion sample size must be 20"
    );
    ensure!(
        methodology
            .get("session_repetitions")
            .and_then(Value::as_u64)
            == Some(3),
        "session repetitions must be three"
    );
    ensure!(
        string(methodology, "comparison_rule")? == "same_host_same_toolchain_back_to_back",
        "comparison rule must require same host and toolchain"
    );
    Ok(())
}

fn validate_thresholds(policy: &Map<String, Value>) -> Result<()> {
    ensure!(
        string(policy, "decision_metric")? == "median_of_three_sessions",
        "regression decision metric mismatch"
    );
    for (key, warning, failure) in [
        ("latency_pct", 5.0, 10.0),
        ("throughput_pct", 5.0, 10.0),
        ("binary_size_pct", 5.0, 10.0),
        ("clean_build_wall_time_pct", 10.0, 15.0),
        ("incremental_build_wall_time_pct", 10.0, 15.0),
        ("max_rss_pct", 10.0, 15.0),
    ] {
        let threshold = object(policy, key)?;
        ensure!(
            number(threshold, "warning")? == warning && number(threshold, "failure")? == failure,
            "regression threshold mismatch: {key}"
        );
    }
    ensure!(
        string(policy, "noise_rule")?
            == "failure_pct=max(configured_failure_pct,3x_baseline_cv_pct)",
        "noise rule mismatch"
    );
    Ok(())
}

fn validate_hosted_handoff(handoff: &Map<String, Value>) -> Result<()> {
    ensure!(
        handoff.get("authoritative").and_then(Value::as_bool) == Some(false),
        "local baseline must remain non-authoritative"
    );
    ensure!(
        string(handoff, "successor_task")? == "BPO-002",
        "hosted handoff must target BPO-002"
    );
    ensure!(
        string(handoff, "current_state")? == "hosted_gate_not_materialized",
        "hosted handoff state mismatch"
    );
    Ok(())
}

fn validate_non_inheritance(boundaries: &Map<String, Value>) -> Result<()> {
    ensure!(
        boundaries.len() == FORBIDDEN_CAPABILITIES.len(),
        "non-inheritance boundary count mismatch"
    );
    for key in FORBIDDEN_CAPABILITIES {
        ensure!(
            boundaries.get(*key) == Some(&Value::Bool(false)),
            "non-inheritance boundary must be explicit false: {key}"
        );
    }
    Ok(())
}

fn run_negative_selftest(baseline: &Value, root: &Path) -> Result<usize> {
    let cases = [
        (
            "wrong_profile",
            "methodology.profile",
            Value::from("bench"),
            "published baseline profile",
        ),
        (
            "low_samples",
            "methodology.criterion_sample_size",
            Value::from(10),
            "Criterion sample size",
        ),
        (
            "wrong_owner",
            "workloads.0.owner",
            Value::from("unknown"),
            "workload owner mismatch",
        ),
        (
            "missing_observation",
            "workloads.0.observations",
            Value::from(vec![1.0, 2.0]),
            "three session observations",
        ),
        (
            "wrong_median",
            "workloads.0.median",
            Value::from(1.0),
            "median does not match",
        ),
        (
            "weak_threshold",
            "regression_policy.latency_pct.failure",
            Value::from(20.0),
            "threshold mismatch",
        ),
        (
            "authoritative_local",
            "hosted_handoff.authoritative",
            Value::Bool(true),
            "must remain non-authoritative",
        ),
        (
            "inherited_submit",
            "non_inheritance.production_submit",
            Value::Bool(true),
            "explicit false",
        ),
    ];
    for (id, path, replacement, expected) in &cases {
        let mut mutated = baseline.clone();
        set_path(&mut mutated, path, replacement.clone())?;
        let error = validate_baseline(&mutated, root)
            .expect_err("negative performance mutation unexpectedly passed")
            .to_string();
        ensure!(
            error.contains(expected),
            "negative selftest failed for {id}: expected '{expected}', got '{error}'"
        );
    }
    Ok(cases.len())
}

fn set_path(value: &mut Value, dotted_path: &str, replacement: Value) -> Result<()> {
    let parts: Vec<_> = dotted_path.split('.').collect();
    let (leaf, parents) = parts.split_last().context("mutation path is empty")?;
    let mut cursor = value;
    for part in parents {
        cursor = match cursor {
            Value::Object(object) => object.get_mut(*part),
            Value::Array(array) => array.get_mut(part.parse::<usize>()?),
            _ => None,
        }
        .with_context(|| format!("missing mutation path: {dotted_path}"))?;
    }
    match cursor {
        Value::Object(object) => {
            object.insert((*leaf).to_string(), replacement);
        }
        Value::Array(array) => {
            let target = array
                .get_mut(leaf.parse::<usize>()?)
                .with_context(|| format!("missing mutation leaf: {dotted_path}"))?;
            *target = replacement;
        }
        _ => bail!("mutation parent must be object or array: {dotted_path}"),
    }
    Ok(())
}

fn repository_root(baseline_path: &Path) -> Result<PathBuf> {
    let canonical = baseline_path
        .canonicalize()
        .with_context(|| format!("cannot resolve {}", baseline_path.display()))?;
    canonical
        .ancestors()
        .find(|ancestor| ancestor.join("Cargo.toml").is_file())
        .map(Path::to_path_buf)
        .context("cannot locate repository root from performance baseline")
}

fn ensure_git_object(root: &Path, object: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["cat-file", "-e", &format!("{object}^{{commit}}")])
        .current_dir(root)
        .status()
        .context("failed to inspect baseline commit")?;
    ensure!(status.success(), "baseline commit is unavailable: {object}");
    Ok(())
}

fn load_json(path: &Path) -> Result<Value> {
    serde_json::from_str(&fs::read_to_string(path)?)
        .with_context(|| format!("invalid JSON: {}", path.display()))
}

fn object<'a>(value: &'a Map<String, Value>, key: &str) -> Result<&'a Map<String, Value>> {
    value
        .get(key)
        .and_then(Value::as_object)
        .with_context(|| format!("missing object: {key}"))
}

fn array<'a>(value: &'a Map<String, Value>, key: &str) -> Result<&'a Vec<Value>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .with_context(|| format!("missing array: {key}"))
}

fn string<'a>(value: &'a Map<String, Value>, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .with_context(|| format!("missing string: {key}"))
}

fn number(value: &Map<String, Value>, key: &str) -> Result<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .with_context(|| format!("missing number: {key}"))
}

fn number_array(value: &Map<String, Value>, key: &str) -> Result<Vec<f64>> {
    array(value, key)?
        .iter()
        .map(|entry| {
            entry
                .as_f64()
                .with_context(|| format!("{key} must contain numbers"))
        })
        .collect()
}

fn ensure_positive(value: &Map<String, Value>, key: &str, label: &str) -> Result<()> {
    let number = number(value, key)?;
    ensure!(
        number.is_finite() && number > 0.0,
        "{label} must be positive"
    );
    Ok(())
}

fn calculated_median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted[sorted.len() / 2]
}

fn coefficient_of_variation_pct(values: &[f64]) -> f64 {
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / values.len() as f64;
    variance.sqrt() / mean * 100.0
}

fn relative_difference(left: f64, right: f64) -> f64 {
    (left - right).abs() / right.abs().max(f64::EPSILON)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracked_baseline_and_negative_matrix_are_valid() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let baseline = root.join("docs/rust-cutover/governance/backend_performance_baseline.json");
        let counts = validate_backend_performance_baseline(&baseline, true).unwrap();
        assert_eq!(counts.workloads, 6);
        assert_eq!(counts.observations, 18);
        assert_eq!(counts.build_measurements, 2);
        assert_eq!(counts.binary_measurements, 2);
        assert_eq!(counts.negative_cases, 8);
    }
}
