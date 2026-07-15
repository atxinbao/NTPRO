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
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail, ensure};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

const EXPECTED_TAG: &str = "ntpro-rust-only-v0.32.0";
const EXPECTED_TAG_OBJECT: &str = "b9a66f12ede051968723ace22b3f06a8e7ac5a09";
const EXPECTED_COMMIT: &str = "2b955cb8a989827e3351c08c3d82d9578253e1f6";
const EXPECTED_BOUNDARIES: &[&str] = &[
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "cancel_order_allowed",
    "replace_order_allowed",
    "amend_order_allowed",
    "flatten_position_allowed",
    "execution_adapter_call_allowed",
    "adapter_send_allowed",
    "live_exchange_request_allowed",
    "network_attempted",
    "retry_scheduler_enabled",
    "automatic_remediation_allowed",
    "automatic_operation_action_allowed",
    "automatic_recovery_allowed",
    "dashboard_operation_controls_enabled",
    "dashboard_trading_controls_enabled",
    "admin_workbench_operation_controls_enabled",
    "admin_workbench_trading_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "manual_operation_submit_allowed",
    "backend_go_live_claim",
    "actual_backend_production_go_live_allowed",
    "frontend_completion_claim",
    "product_grade_trading_terminal_claim",
    "product_grade_live_trading_terminal_claim",
    "default_production_execution_allowed",
];
const EXPECTED_INHERITANCE_FALSE: &[&str] = &[
    "inherits_backend_go_live_claim",
    "inherits_production_submit",
    "inherits_production_mutation",
    "inherits_adapter_send",
    "inherits_live_exchange_request",
    "inherits_retry_scheduler",
    "inherits_automatic_remediation",
    "inherits_dashboard_trading_controls",
    "inherits_admin_workbench_trading_controls",
    "inherits_trader_terminal_order_ticket",
];

/// Files which define the frozen backend baseline.
pub struct BackendFreezeConfig {
    pub registry: PathBuf,
    pub release_manifest: PathBuf,
    pub policy: PathBuf,
    pub readme: PathBuf,
    pub roadmap: PathBuf,
    pub versioning: PathBuf,
}

/// Counts emitted after backend freeze validation.
pub struct BackendFreezeCounts {
    pub tag: String,
    pub commit: String,
    pub boundaries: usize,
    pub source_hashes: usize,
    pub negative_cases: usize,
}

/// Validates the v0.32.0 backend freeze registry and deterministic negative
/// mutation suite.
///
/// # Errors
///
/// Returns an error when tag identity, source evidence, boundary fields,
/// release scope, immutability, or v0.33 non-inheritance drifts.
pub fn validate_backend_freeze(
    config: &BackendFreezeConfig,
    negative_selftest: bool,
) -> Result<BackendFreezeCounts> {
    for path in [
        &config.registry,
        &config.release_manifest,
        &config.policy,
        &config.readme,
        &config.roadmap,
        &config.versioning,
    ] {
        ensure!(path.is_file(), "missing required file: {}", path.display());
    }
    let registry = load_json(&config.registry)?;
    let manifest = load_json(&config.release_manifest)?;
    let texts = FreezeTexts {
        policy: read(&config.policy)?,
        readme: read(&config.readme)?,
        roadmap: read(&config.roadmap)?,
        versioning: read(&config.versioning)?,
    };
    let source_hashes = validate_registry(&registry, &manifest, &texts)?;
    let negative_cases = if negative_selftest {
        run_negative_selftest(&registry, &manifest, &texts)?
    } else {
        0
    };
    Ok(BackendFreezeCounts {
        tag: EXPECTED_TAG.to_string(),
        commit: EXPECTED_COMMIT.to_string(),
        boundaries: EXPECTED_BOUNDARIES.len(),
        source_hashes,
        negative_cases,
    })
}

struct FreezeTexts {
    policy: String,
    readme: String,
    roadmap: String,
    versioning: String,
}

fn validate_registry(registry: &Value, manifest: &Value, texts: &FreezeTexts) -> Result<usize> {
    ensure!(
        registry.get("schema_version").and_then(Value::as_str)
            == Some("ntpro.backend_freeze_registry.v1"),
        "registry schema mismatch"
    );
    ensure!(
        registry.get("task_id").and_then(Value::as_str) == Some("BFG-001"),
        "registry task mismatch"
    );
    ensure!(
        registry.get("status").and_then(Value::as_str) == Some("active"),
        "registry status must be active"
    );

    let baseline = object(registry, "baseline")?;
    let tag = object_value(baseline, "tag")?;
    ensure!(
        string(baseline, "version")? == "v0.32.0",
        "baseline version mismatch"
    );
    ensure!(
        string(baseline, "release_status")? == "released_and_frozen",
        "baseline release status mismatch"
    );
    let tag_name = string(tag, "name")?;
    ensure!(tag_name == EXPECTED_TAG, "baseline tag mismatch");
    ensure!(
        string(tag, "object_sha")? == EXPECTED_TAG_OBJECT,
        "baseline tag object mismatch"
    );
    ensure!(
        string(tag, "peeled_commit_sha")? == EXPECTED_COMMIT,
        "baseline commit mismatch"
    );
    let (local_tag_object, local_commit) = resolve_tag(tag_name)?;
    ensure!(
        local_tag_object == EXPECTED_TAG_OBJECT,
        "local tag object mismatch"
    );
    ensure!(
        local_commit == EXPECTED_COMMIT,
        "local peeled tag commit mismatch"
    );

    let release = object(registry, "github_release")?;
    ensure!(
        release.get("draft") == Some(&Value::Bool(false))
            && release.get("prerelease") == Some(&Value::Bool(false)),
        "release publication flags drifted"
    );
    ensure!(
        string(release, "url")?.ends_with(EXPECTED_TAG),
        "release URL mismatch"
    );
    let gate = object(registry, "hosted_release_gate")?;
    ensure!(
        gate.get("run_id").and_then(Value::as_u64) == Some(29_371_898_609),
        "release gate run mismatch"
    );
    ensure!(
        string(gate, "head_sha")? == EXPECTED_COMMIT,
        "release gate head mismatch"
    );
    ensure!(
        string(gate, "status")? == "completed" && string(gate, "conclusion")? == "success",
        "release gate status mismatch"
    );

    let scope = object(registry, "release_scope")?;
    ensure!(
        scope.get("milestone_number").and_then(Value::as_u64) == Some(30)
            && string(scope, "milestone_state")? == "closed",
        "release milestone mismatch"
    );
    let issues = array(scope, "exact_issue_numbers")?;
    let expected_issues: Vec<Value> = (1042_u64..=1051).map(Value::from).collect();
    ensure!(
        issues == expected_issues.as_slice(),
        "release issue set mismatch"
    );
    ensure!(
        scope.get("all_issues_closed") == Some(&Value::Bool(true)),
        "release issues must remain closed"
    );

    let registry_boundaries = object(registry, "boundary_flags")?;
    let manifest_boundaries = object(manifest, "boundary_flags")?;
    let expected: BTreeSet<_> = EXPECTED_BOUNDARIES.iter().copied().collect();
    ensure!(
        registry_boundaries
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            == expected,
        "registry boundary field set mismatch"
    );
    ensure!(
        manifest_boundaries
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            == expected,
        "release manifest boundary field set mismatch"
    );
    for key in EXPECTED_BOUNDARIES {
        ensure!(
            registry_boundaries.get(*key) == Some(&Value::Bool(false)),
            "boundary must remain explicit false: {key}"
        );
    }
    ensure!(
        registry_boundaries == manifest_boundaries,
        "registry and manifest boundaries differ"
    );

    let source = object(registry, "source_evidence")?;
    ensure!(
        string(source, "audit_strategy")? == "source_tree_plus_github_remote",
        "audit strategy mismatch"
    );
    ensure!(
        source.get("local_generated_evidence_required") == Some(&Value::Bool(false)),
        "local generated evidence must remain optional"
    );
    ensure!(
        source.get("generated_evidence_sole_proof_allowed") == Some(&Value::Bool(false)),
        "generated evidence cannot be sole proof"
    );
    ensure!(
        source.get("remote_reconstruction_required") == Some(&Value::Bool(true)),
        "remote reconstruction must remain required"
    );
    let source_files = array(source, "files")?;
    ensure!(
        source_files.len() == 4,
        "registered source evidence set mismatch"
    );
    for item in source_files {
        let item = item
            .as_object()
            .context("source evidence item must be an object")?;
        let path = Path::new(string(item, "path")?);
        ensure!(
            path.is_file(),
            "registered source evidence missing: {}",
            path.display()
        );
        let digest = format!("{:x}", Sha256::digest(fs::read(path)?));
        ensure!(
            digest == string(item, "sha256")?,
            "registered source evidence hash mismatch: {}",
            path.display()
        );
    }

    let next = object(registry, "next_track_contract")?;
    ensure!(
        next.get("default_patch_scheduled") == Some(&Value::Bool(false)),
        "backend patch must remain unscheduled"
    );
    ensure!(
        next.get("default_patch_version") == Some(&Value::Null),
        "default patch version must remain null"
    );
    ensure!(
        string(next, "governance_track")? == "backend-freeze-governance",
        "governance track mismatch"
    );
    ensure!(
        string(next, "next_capability_track")? == "v0.33.0+",
        "next capability family mismatch"
    );
    ensure!(
        string(next, "capability_entry")? == "separately_scoped_only",
        "v0.33+ entry must remain separately scoped"
    );
    for key in EXPECTED_INHERITANCE_FALSE {
        ensure!(
            next.contains_key(*key),
            "missing inheritance boundary: {key}"
        );
        ensure!(
            next.get(*key) == Some(&Value::Bool(false)),
            "v0.33+ inheritance must remain false: {key}"
        );
    }

    let immutability = object(registry, "immutability")?;
    for key in [
        "published_tag_rewrite_allowed",
        "published_release_rewrite_allowed",
        "baseline_release_package_routine_edit_allowed",
    ] {
        ensure!(
            immutability.get(key) == Some(&Value::Bool(false)),
            "immutability boundary drifted: {key}"
        );
    }
    ensure!(
        immutability.get("backend_patch_requires_proven_baseline_invalidity")
            == Some(&Value::Bool(true)),
        "patch exception proof requirement missing"
    );
    ensure!(
        immutability.get("backend_freeze_exception_issue_required") == Some(&Value::Bool(true)),
        "freeze exception issue requirement missing"
    );

    for marker in [
        EXPECTED_TAG,
        EXPECTED_COMMIT,
        "source_tree_plus_github_remote",
        "There is no scheduled v0.32.1 backend patch",
        "separately_scoped_only",
    ] {
        ensure!(
            texts.policy.contains(marker),
            "freeze policy marker missing: {marker}"
        );
    }
    for (label, text) in [("README", &texts.readme), ("ROADMAP", &texts.roadmap)] {
        ensure!(
            text.contains("No backend patch is scheduled."),
            "{label} backend patch status missing"
        );
        ensure!(
            text.contains("backend-freeze-governance"),
            "{label} governance track missing"
        );
        ensure!(
            text.contains("v0.33.0+"),
            "{label} capability family missing"
        );
    }
    ensure!(
        texts
            .versioning
            .contains("none scheduled; baseline-invalidity exception only"),
        "versioning backend patch status missing"
    );
    ensure!(
        texts.versioning.contains("backend-freeze-governance")
            && texts.versioning.contains("v0.33.0+"),
        "versioning governance route missing"
    );
    Ok(source_files.len())
}

fn run_negative_selftest(registry: &Value, manifest: &Value, texts: &FreezeTexts) -> Result<usize> {
    let cases = [
        (
            "missing_boundary",
            Mutation::Delete,
            "boundary_flags.actual_backend_production_go_live_allowed",
            Value::Null,
            "registry boundary field set mismatch",
        ),
        (
            "submit_enabled",
            Mutation::Set,
            "boundary_flags.production_order_submission_allowed",
            Value::Bool(true),
            "boundary must remain explicit false",
        ),
        (
            "mutation_enabled",
            Mutation::Set,
            "boundary_flags.production_order_mutation_allowed",
            Value::Bool(true),
            "boundary must remain explicit false",
        ),
        (
            "adapter_call_enabled",
            Mutation::Set,
            "boundary_flags.execution_adapter_call_allowed",
            Value::Bool(true),
            "boundary must remain explicit false",
        ),
        (
            "adapter_send_enabled",
            Mutation::Set,
            "boundary_flags.adapter_send_allowed",
            Value::Bool(true),
            "boundary must remain explicit false",
        ),
        (
            "live_request_enabled",
            Mutation::Set,
            "boundary_flags.live_exchange_request_allowed",
            Value::Bool(true),
            "boundary must remain explicit false",
        ),
        (
            "retry_enabled",
            Mutation::Set,
            "boundary_flags.retry_scheduler_enabled",
            Value::Bool(true),
            "boundary must remain explicit false",
        ),
        (
            "remediation_enabled",
            Mutation::Set,
            "boundary_flags.automatic_remediation_allowed",
            Value::Bool(true),
            "boundary must remain explicit false",
        ),
        (
            "recovery_enabled",
            Mutation::Set,
            "boundary_flags.automatic_recovery_allowed",
            Value::Bool(true),
            "boundary must remain explicit false",
        ),
        (
            "dashboard_controls_enabled",
            Mutation::Set,
            "boundary_flags.dashboard_trading_controls_enabled",
            Value::Bool(true),
            "boundary must remain explicit false",
        ),
        (
            "admin_controls_enabled",
            Mutation::Set,
            "boundary_flags.admin_workbench_trading_controls_enabled",
            Value::Bool(true),
            "boundary must remain explicit false",
        ),
        (
            "terminal_ticket_enabled",
            Mutation::Set,
            "boundary_flags.trader_terminal_order_ticket_enabled",
            Value::Bool(true),
            "boundary must remain explicit false",
        ),
        (
            "manual_submit_enabled",
            Mutation::Set,
            "boundary_flags.manual_operation_submit_allowed",
            Value::Bool(true),
            "boundary must remain explicit false",
        ),
        (
            "backend_go_live_enabled",
            Mutation::Set,
            "boundary_flags.actual_backend_production_go_live_allowed",
            Value::Bool(true),
            "boundary must remain explicit false",
        ),
        (
            "wrong_tag",
            Mutation::Set,
            "baseline.tag.name",
            Value::String("ntpro-rust-only-v0.32.1".to_string()),
            "baseline tag mismatch",
        ),
        (
            "wrong_commit",
            Mutation::Set,
            "baseline.tag.peeled_commit_sha",
            Value::String("0000000000000000000000000000000000000000".to_string()),
            "baseline commit mismatch",
        ),
        (
            "wrong_source_hash",
            Mutation::Set,
            "source_evidence.files.0.sha256",
            Value::String("0".repeat(64)),
            "registered source evidence hash mismatch",
        ),
        (
            "inherited_submit",
            Mutation::Set,
            "next_track_contract.inherits_production_submit",
            Value::Bool(true),
            "v0.33+ inheritance must remain false",
        ),
        (
            "missing_inheritance",
            Mutation::Delete,
            "next_track_contract.inherits_trader_terminal_order_ticket",
            Value::Null,
            "missing inheritance boundary",
        ),
        (
            "scheduled_patch",
            Mutation::Set,
            "next_track_contract.default_patch_scheduled",
            Value::Bool(true),
            "backend patch must remain unscheduled",
        ),
    ];
    for (case_id, operation, path, value, expected) in &cases {
        let mut mutated = registry.clone();
        mutate(&mut mutated, *operation, path, value.clone())?;
        let error = validate_registry(&mutated, manifest, texts)
            .expect_err("negative backend freeze mutation unexpectedly passed")
            .to_string();
        ensure!(
            error.contains(expected),
            "negative selftest failed for unexpected reason: {case_id}\nexpected: {expected}\nactual: {error}"
        );
    }
    Ok(cases.len())
}

#[derive(Clone, Copy)]
enum Mutation {
    Delete,
    Set,
}

fn mutate(root: &mut Value, operation: Mutation, dotted_path: &str, value: Value) -> Result<()> {
    let parts: Vec<_> = dotted_path.split('.').collect();
    let (leaf, parents) = parts.split_last().context("mutation path is empty")?;
    let mut cursor = root;
    for part in parents {
        cursor = match cursor {
            Value::Object(object) => object.get_mut(*part),
            Value::Array(array) => part
                .parse::<usize>()
                .ok()
                .and_then(|index| array.get_mut(index)),
            _ => None,
        }
        .with_context(|| format!("missing mutation path: {dotted_path}"))?;
    }
    match (operation, cursor) {
        (Mutation::Delete, Value::Object(object)) => {
            object
                .remove(*leaf)
                .with_context(|| format!("missing mutation leaf: {dotted_path}"))?;
        }
        (Mutation::Delete, Value::Array(array)) => {
            let index = leaf.parse::<usize>()?;
            ensure!(index < array.len(), "missing mutation leaf: {dotted_path}");
            array.remove(index);
        }
        (Mutation::Set, Value::Object(object)) => {
            object.insert((*leaf).to_string(), value);
        }
        (Mutation::Set, Value::Array(array)) => {
            let index = leaf.parse::<usize>()?;
            let target = array
                .get_mut(index)
                .with_context(|| format!("missing mutation leaf: {dotted_path}"))?;
            *target = value;
        }
        _ => bail!("mutation parent must be an object or array: {dotted_path}"),
    }
    Ok(())
}

fn load_json(path: &Path) -> Result<Value> {
    let text = read(path)?;
    serde_json::from_str(&text).with_context(|| format!("cannot load {}", path.display()))
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))
}

fn object<'a>(value: &'a Value, key: &str) -> Result<&'a Map<String, Value>> {
    value
        .get(key)
        .and_then(Value::as_object)
        .with_context(|| format!("{key} must be an object"))
}

fn object_value<'a>(value: &'a Map<String, Value>, key: &str) -> Result<&'a Map<String, Value>> {
    value
        .get(key)
        .and_then(Value::as_object)
        .with_context(|| format!("{key} must be an object"))
}

fn array<'a>(value: &'a Map<String, Value>, key: &str) -> Result<&'a [Value]> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .with_context(|| format!("{key} must be an array"))
}

fn string<'a>(value: &'a Map<String, Value>, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .with_context(|| format!("{key} must be a string"))
}

fn resolve_tag(tag: &str) -> Result<(String, String)> {
    let object = git_rev_parse(&format!("refs/tags/{tag}"))
        .with_context(|| format!("missing local baseline tag: {tag}"))?;
    let commit = git_rev_parse(&format!("{tag}^{{}}"))
        .with_context(|| format!("missing local baseline tag: {tag}"))?;
    Ok((object, commit))
}

fn git_rev_parse(revision: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", revision])
        .stderr(Stdio::null())
        .output()
        .context("failed to execute git rev-parse")?;
    ensure!(
        output.status.success(),
        "git revision not found: {revision}"
    );
    String::from_utf8(output.stdout)
        .context("git rev-parse output is not UTF-8")
        .map(|value| value.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dotted_mutation_supports_objects_and_arrays() {
        let mut value = json!({"items": [{"enabled": false}]});
        mutate(
            &mut value,
            Mutation::Set,
            "items.0.enabled",
            Value::Bool(true),
        )
        .unwrap();
        assert_eq!(value.pointer("/items/0/enabled"), Some(&Value::Bool(true)));
        mutate(&mut value, Mutation::Delete, "items.0.enabled", Value::Null).unwrap();
        assert!(value.pointer("/items/0/enabled").is_none());
    }
}
