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

use std::{collections::BTreeSet, fs, path::Path};

use anyhow::{Context, Result, ensure};
use serde_json::{Map, Value, json};

const CONTRACT_SCHEMA: &str = "ntpro.backend_performance_hosted_contract.v1";
const RESULT_SCHEMA: &str = "ntpro.backend_benchmark_result.v1";
const COMPARISON_SCHEMA: &str = "ntpro.backend_benchmark_comparison.v1";
const EXPECTED_WORKLOADS: &[&str] = &[
    "core_stack_str",
    "model_price",
    "data_engine_ingest",
    "execution_matching_core",
    "live_runner_dispatch",
    "network_rate_limiter",
];
const EXPECTED_INFORMATIONAL: &str = "execution_matching_core";

/// Counts emitted after hosted benchmark contract validation.
pub struct BackendBenchmarkContractCounts {
    pub workloads: usize,
    pub stable: usize,
    pub informational: usize,
}

/// Comparison outcome emitted after artifact validation.
pub struct BackendBenchmarkComparison {
    pub workload: String,
    pub outcome: String,
    pub regression_pct: f64,
    pub effective_failure_pct: f64,
    pub merge_authority: bool,
    pub must_fail: bool,
}

/// Validates the BPO-002 hosted benchmark contract.
///
/// # Errors
///
/// Returns an error when workload classification, runner bounds, artifact
/// schema, trigger policy, threshold policy, or non-inheritance drifts.
pub fn validate_backend_benchmark_contract(
    contract_path: &Path,
) -> Result<BackendBenchmarkContractCounts> {
    let contract = load_json(contract_path)?;
    validate_contract(&contract)
}

/// Validates and compares one baseline/candidate artifact pair, then writes the
/// machine-readable comparison report.
///
/// # Errors
///
/// Returns an error when the contract or artifacts are invalid, the output
/// cannot be written. A stable regression is represented by `must_fail` in the
/// returned comparison after the report has been written.
pub fn compare_backend_benchmark_results(
    contract_path: &Path,
    baseline_path: &Path,
    candidate_path: &Path,
    output_path: &Path,
) -> Result<BackendBenchmarkComparison> {
    let contract = load_json(contract_path)?;
    validate_contract(&contract)?;
    let baseline = load_json(baseline_path)?;
    let candidate = load_json(candidate_path)?;
    let baseline = validate_result(&baseline)?;
    let candidate = validate_result(&candidate)?;

    ensure!(
        baseline.workload == candidate.workload,
        "baseline and candidate workload mismatch"
    );
    ensure!(
        baseline.commit != candidate.commit,
        "baseline and candidate commits must differ"
    );
    ensure!(
        baseline.command == candidate.command,
        "baseline and candidate command mismatch"
    );
    ensure!(
        baseline.runner_identity == candidate.runner_identity,
        "baseline and candidate runner identity mismatch"
    );

    let contract_object = contract.as_object().context("contract must be an object")?;
    let policy = object(contract_object, "regression_policy")?;
    let warning_pct = number(policy, "warning_pct")?;
    let failure_floor_pct = number(policy, "failure_floor_pct")?;
    let stability_max_cv_pct = number(policy, "stability_max_cv_pct")?;
    let cv_multiplier = number(policy, "baseline_cv_failure_multiplier")?;
    let effective_failure_pct = failure_floor_pct.max(baseline.cv_pct * cv_multiplier);
    let regression_pct = (candidate.median / baseline.median - 1.0) * 100.0;

    let stable_ids = string_set(contract_object, "stable_workloads")?;
    let preclassified_stable = stable_ids.contains(baseline.workload.as_str());
    let observed_stable =
        baseline.cv_pct <= stability_max_cv_pct && candidate.cv_pct <= stability_max_cv_pct;
    let merge_authority = preclassified_stable && observed_stable;
    let (outcome, reason) = if !preclassified_stable {
        (
            "informational",
            "workload is preclassified informational and cannot block merge",
        )
    } else if !observed_stable {
        (
            "noisy_informational",
            "observed session CV exceeds the hosted stability ceiling",
        )
    } else if regression_pct > effective_failure_pct {
        (
            "regression",
            "stable candidate median crossed the noise-adjusted failure threshold",
        )
    } else if regression_pct > warning_pct {
        (
            "warning",
            "stable candidate median crossed the warning threshold",
        )
    } else {
        ("pass", "candidate remained within the warning threshold")
    };
    let must_fail = outcome == "regression";

    let report = json!({
        "schema_version": COMPARISON_SCHEMA,
        "task_id": "BPO-002",
        "workload_id": &baseline.workload,
        "baseline_commit": &baseline.commit,
        "candidate_commit": &candidate.commit,
        "baseline_median_ns": baseline.median,
        "candidate_median_ns": candidate.median,
        "baseline_cv_pct": baseline.cv_pct,
        "candidate_cv_pct": candidate.cv_pct,
        "regression_pct": regression_pct,
        "warning_pct": warning_pct,
        "failure_floor_pct": failure_floor_pct,
        "effective_failure_pct": effective_failure_pct,
        "preclassified_stable": preclassified_stable,
        "observed_stable": observed_stable,
        "merge_authority": merge_authority,
        "outcome": outcome,
        "reason": reason,
        "runner_identity": &baseline.runner_identity,
        "command": &baseline.command,
        "non_inheritance": object(contract_object, "non_inheritance")?,
    });
    write_json(output_path, &report)?;

    Ok(BackendBenchmarkComparison {
        workload: baseline.workload,
        outcome: outcome.to_string(),
        regression_pct,
        effective_failure_pct,
        merge_authority,
        must_fail,
    })
}

fn validate_contract(value: &Value) -> Result<BackendBenchmarkContractCounts> {
    let contract = value.as_object().context("contract must be an object")?;
    ensure!(
        string(contract, "schema_version")? == CONTRACT_SCHEMA,
        "hosted benchmark contract schema mismatch"
    );
    ensure!(string(contract, "task_id")? == "BPO-002", "task mismatch");
    ensure!(
        string(contract, "status")? == "active",
        "hosted benchmark contract must be active"
    );
    ensure!(
        string(contract, "classification")? == "v33-separately-scoped",
        "classification mismatch"
    );
    let runner = object(contract, "runner")?;
    ensure!(
        string(runner, "image")? == "ubuntu-22.04",
        "runner image mismatch"
    );
    ensure!(
        string(runner, "rust_toolchain")? == "1.95.0",
        "runner Rust toolchain mismatch"
    );
    ensure!(
        runner.get("timeout_minutes").and_then(Value::as_u64) == Some(60),
        "runner timeout mismatch"
    );
    ensure!(
        runner.get("session_repetitions").and_then(Value::as_u64) == Some(3),
        "runner session repetition mismatch"
    );
    ensure!(
        runner.get("cache_enabled").and_then(Value::as_bool) == Some(true),
        "runner cache policy mismatch"
    );
    let triggers = string_set(contract, "triggers")?;
    ensure!(
        triggers
            == BTreeSet::from([
                "pull_request".to_string(),
                "schedule".to_string(),
                "workflow_dispatch".to_string(),
            ]),
        "hosted benchmark trigger set mismatch"
    );
    let stable = string_set(contract, "stable_workloads")?;
    let informational = string_set(contract, "informational_workloads")?;
    ensure!(
        informational == BTreeSet::from([EXPECTED_INFORMATIONAL.to_string()]),
        "informational workload set mismatch"
    );
    ensure!(
        stable.is_disjoint(&informational),
        "stable and informational workloads overlap"
    );
    let all: BTreeSet<_> = stable.union(&informational).cloned().collect();
    ensure!(
        all == EXPECTED_WORKLOADS.iter().map(ToString::to_string).collect(),
        "hosted workload set mismatch"
    );
    let policy = object(contract, "regression_policy")?;
    ensure!(
        number(policy, "warning_pct")? == 5.0
            && number(policy, "failure_floor_pct")? == 10.0
            && number(policy, "stability_max_cv_pct")? == 5.0
            && number(policy, "baseline_cv_failure_multiplier")? == 3.0,
        "hosted regression policy mismatch"
    );
    ensure!(
        string(policy, "noisy_policy")? == "informational_non_blocking",
        "noisy policy must remain informational"
    );
    let artifact = object(contract, "artifact")?;
    ensure!(
        string(artifact, "result_schema")? == RESULT_SCHEMA
            && string(artifact, "comparison_schema")? == COMPARISON_SCHEMA,
        "artifact schema mismatch"
    );
    ensure!(
        artifact.get("retention_days").and_then(Value::as_u64) == Some(30),
        "artifact retention mismatch"
    );
    let boundaries = object(contract, "non_inheritance")?;
    ensure!(boundaries.len() == 9, "non-inheritance count mismatch");
    for (key, value) in boundaries {
        ensure!(
            value == &Value::Bool(false),
            "non-inheritance boundary must be false: {key}"
        );
    }
    Ok(BackendBenchmarkContractCounts {
        workloads: all.len(),
        stable: stable.len(),
        informational: informational.len(),
    })
}

struct ResultRecord {
    workload: String,
    commit: String,
    command: String,
    runner_identity: Value,
    median: f64,
    cv_pct: f64,
}

fn validate_result(value: &Value) -> Result<ResultRecord> {
    let result = value
        .as_object()
        .context("benchmark result must be an object")?;
    ensure!(
        string(result, "schema_version")? == RESULT_SCHEMA,
        "benchmark result schema mismatch"
    );
    ensure!(
        string(result, "task_id")? == "BPO-002",
        "result task mismatch"
    );
    let workload = string(result, "workload_id")?.to_string();
    ensure!(
        EXPECTED_WORKLOADS.contains(&workload.as_str()),
        "unexpected result workload"
    );
    let commit = string(result, "commit_sha")?.to_string();
    ensure!(is_sha(&commit), "result commit must be a full SHA");
    let observations = number_array(result, "observations_ns")?;
    ensure!(
        observations.len() == 3
            && observations
                .iter()
                .all(|value| value.is_finite() && *value > 0.0),
        "result requires three positive observations"
    );
    let median = number(result, "median_ns")?;
    let cv_pct = number(result, "coefficient_of_variation_pct")?;
    ensure!(
        median.is_finite() && median > 0.0,
        "result median must be positive"
    );
    ensure!(
        cv_pct.is_finite() && cv_pct >= 0.0,
        "result CV must be non-negative"
    );
    ensure!(
        relative_difference(median, calculated_median(&observations)) <= 0.001,
        "result median mismatch"
    );
    ensure!(
        (cv_pct - coefficient_of_variation_pct(&observations)).abs() <= 0.02,
        "result CV mismatch"
    );
    let methodology = object(result, "methodology")?;
    ensure!(
        string(methodology, "profile")? == "bench-lto"
            && methodology.get("warmup_seconds").and_then(Value::as_u64) == Some(1)
            && methodology
                .get("measurement_seconds")
                .and_then(Value::as_u64)
                == Some(2)
            && methodology.get("sample_size").and_then(Value::as_u64) == Some(20)
            && methodology
                .get("session_repetitions")
                .and_then(Value::as_u64)
                == Some(3)
            && string(methodology, "estimate")? == "slope.point_estimate",
        "result methodology mismatch"
    );
    let runner_identity = object(result, "runner_identity")?;
    for key in ["os", "architecture", "cpu", "rustc", "cargo"] {
        ensure!(
            !string(runner_identity, key)?.is_empty(),
            "runner identity field must not be empty: {key}"
        );
    }
    Ok(ResultRecord {
        workload,
        commit,
        command: string(result, "command")?.to_string(),
        runner_identity: Value::Object(runner_identity.clone()),
        median,
        cv_pct,
    })
}

fn is_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn load_json(path: &Path) -> Result<Value> {
    serde_json::from_str(&fs::read_to_string(path)?)
        .with_context(|| format!("invalid JSON: {}", path.display()))
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))
        .with_context(|| format!("cannot write {}", path.display()))
}

fn object<'a>(value: &'a Map<String, Value>, key: &str) -> Result<&'a Map<String, Value>> {
    value
        .get(key)
        .and_then(Value::as_object)
        .with_context(|| format!("missing object: {key}"))
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
    value
        .get(key)
        .and_then(Value::as_array)
        .with_context(|| format!("missing array: {key}"))?
        .iter()
        .map(|entry| {
            entry
                .as_f64()
                .with_context(|| format!("{key} must contain numbers"))
        })
        .collect()
}

fn string_set(value: &Map<String, Value>, key: &str) -> Result<BTreeSet<String>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .with_context(|| format!("missing array: {key}"))?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(ToString::to_string)
                .with_context(|| format!("{key} must contain strings"))
        })
        .collect()
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
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn compares_stable_and_informational_results() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let contract =
            root.join("docs/rust-cutover/governance/backend_performance_hosted_contract.json");
        let counts = validate_backend_benchmark_contract(&contract).unwrap();
        assert_eq!(counts.workloads, 6);
        assert_eq!(counts.stable, 5);
        assert_eq!(counts.informational, 1);

        let temp = tempdir().unwrap();
        for (workload, expected) in [
            ("core_stack_str", "warning"),
            ("execution_matching_core", "informational"),
        ] {
            let baseline = result(workload, &"1".repeat(40), [10.0, 10.1, 9.9]);
            let candidate = result(workload, &"2".repeat(40), [10.6, 10.7, 10.5]);
            let baseline_path = temp.path().join(format!("{workload}-base.json"));
            let candidate_path = temp.path().join(format!("{workload}-candidate.json"));
            let output_path = temp.path().join(format!("{workload}-comparison.json"));
            write_json(&baseline_path, &baseline).unwrap();
            write_json(&candidate_path, &candidate).unwrap();
            let comparison = compare_backend_benchmark_results(
                &contract,
                &baseline_path,
                &candidate_path,
                &output_path,
            )
            .unwrap();
            assert_eq!(comparison.outcome, expected);
            assert!(!comparison.must_fail);
            assert!(output_path.is_file());
        }
    }

    #[test]
    fn stable_regression_blocks_but_noisy_result_does_not() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let contract =
            root.join("docs/rust-cutover/governance/backend_performance_hosted_contract.json");
        let temp = tempdir().unwrap();
        let baseline_path = temp.path().join("baseline.json");
        let candidate_path = temp.path().join("candidate.json");
        let output_path = temp.path().join("comparison.json");
        write_json(
            &baseline_path,
            &result("core_stack_str", &"1".repeat(40), [10.0, 10.1, 9.9]),
        )
        .unwrap();
        write_json(
            &candidate_path,
            &result("core_stack_str", &"2".repeat(40), [11.2, 11.3, 11.1]),
        )
        .unwrap();
        let regression = compare_backend_benchmark_results(
            &contract,
            &baseline_path,
            &candidate_path,
            &output_path,
        )
        .unwrap();
        assert_eq!(regression.outcome, "regression");
        assert!(regression.merge_authority);
        assert!(regression.must_fail);

        write_json(
            &candidate_path,
            &result("core_stack_str", &"2".repeat(40), [8.0, 12.0, 14.0]),
        )
        .unwrap();
        let noisy = compare_backend_benchmark_results(
            &contract,
            &baseline_path,
            &candidate_path,
            &output_path,
        )
        .unwrap();
        assert_eq!(noisy.outcome, "noisy_informational");
        assert!(!noisy.merge_authority);
        assert!(!noisy.must_fail);
    }

    fn result(workload: &str, commit: &str, values: [f64; 3]) -> Value {
        let median = calculated_median(&values);
        let cv = coefficient_of_variation_pct(&values);
        json!({
            "schema_version": RESULT_SCHEMA,
            "task_id": "BPO-002",
            "workload_id": workload,
            "commit_sha": commit,
            "captured_at": "2026-07-20T00:00:00Z",
            "command": "same command",
            "runner_identity": {
                "os": "Linux",
                "architecture": "x86_64",
                "cpu": "test",
                "rustc": "1.95.0",
                "cargo": "1.95.0"
            },
            "methodology": {
                "profile": "bench-lto",
                "warmup_seconds": 1,
                "measurement_seconds": 2,
                "sample_size": 20,
                "session_repetitions": 3,
                "estimate": "slope.point_estimate"
            },
            "observations_ns": values,
            "median_ns": median,
            "coefficient_of_variation_pct": cv
        })
    }
}
