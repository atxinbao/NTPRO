// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
// -------------------------------------------------------------------------------------------------

use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail, ensure};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

const REQUIRED_TRACKED_AUTHORITY: &[&str] = &[
    "docs/rust-cutover/governance/backend_hygiene_authority_map.md",
    "docs/rust-cutover/governance/backend_fixture_inventory.json",
    "docs/rust-cutover/governance/post_freeze_backend_hygiene_closeout.md",
    "scripts/ai/check_backend_hygiene.sh",
    "crates/governance/src/backend_hygiene.rs",
];
const REQUIRED_IGNORE_RULES: &[&str] = &[
    "*target/",
    "*target-v2/",
    "build/",
    "dist/",
    ".DS_Store",
    "/release-publication-evidence/",
    "/graphify-out/",
];
const GENERATED_ROOTS: &[&str] = &[
    "target",
    "target-v2",
    "build",
    "dist",
    "release-publication-evidence",
    "graphify-out",
];
const EXPECTED_BUILD_OUTPUTS: &[&str] = &[
    "target",
    "target-v2",
    "build",
    "dist",
    ".coverage*",
    ".benchmarks*",
];
const EXPECTED_GENERATED_OUTPUTS: &[&str] = &["release-publication-evidence", "graphify-out"];
const EXPECTED_PROTECTED_OUTPUTS: &[&str] = &[
    ".codex",
    ".agentflow",
    ".understand-anything",
    "project.html",
    "tests/test_data/large",
    "tests/test_data/local",
];
const RETIRED_ACTIVE_MARKERS: &[&str] = &[
    "ci-pr-wheel",
    "scripts/control/*.sh",
    "scripts/ci/plan.sh",
    "uv sync",
    "uv run",
    "origin/develop",
    "docs/developer_guide/environment_setup.md",
    "CLA Assistant",
];
const RETIRED_GITIGNORE_EXCEPTIONS: &[&str] = &[
    "!.docker/entrypoint.sh",
    "!.pre-commit-hooks/check_formatting_py.sh",
    "!.pre-commit-hooks/check_no_legacy_paths_v2.sh",
    "!.pre-commit-hooks/check_pyo3_conventions.sh", // zero-python-pattern-definition
    "!scripts/check-no-build-packages.sh",
    "!scripts/ci/osv-severity-gate.sh",
    "!scripts/ci/plan.sh",
    "!scripts/ci/publish-release-attestation-siblings.sh",
    "!scripts/ci/publish-release-checksums.sh",
    "!scripts/ci/publish-wheels-delete-artifacts.sh",
    "!scripts/ci/publish-wheels-generate-index.sh",
    "!scripts/ci/publish-wheels-r2-remove-old-wheels.sh",
    "!scripts/ci/publish-wheels-r2-upload-index.sh",
    "!scripts/ci/publish-wheels-r2-upload-new-wheels.sh",
    "!scripts/ci/publish-wheels-r2-verify-files.sh",
    "!scripts/ci/retry-on-corruption.sh",
    "!scripts/ci/update-pyproject-version.sh",
    "!scripts/ci/validate-wheel-count.sh",
    "!scripts/install-capnp.sh",
    "!scripts/package-version.sh",
    "!scripts/purge-orphan-dev-wheels.sh",
    "!scripts/python-version.sh",
    "!scripts/regen-capnp.sh",
    "!scripts/test-coverage.sh",
    "!scripts/test-examples.sh",
    "!scripts/test-performance.sh",
    "!scripts/test.sh",
    "!scripts/uv-version.sh",
    "!scripts/control/*.sh",
];
const RETIRED_ROOT_CONFIGS: &[&str] = &[
    ".codecov.yml",
    ".codespellrc",
    ".dockerignore",
    ".gitlint",
    ".markdownlint.jsonc",
    ".taplo.toml",
    ".typos.toml",
    ".yamlfmt",
    ".yamllint.yaml",
];

/// BFH-007 guard 所读取的仓库权威文件。
pub struct BackendHygieneConfig {
    pub inventory: PathBuf,
    pub authority_map: PathBuf,
    pub contributing: PathBuf,
    pub gitignore: PathBuf,
    pub cargo_manifest: PathBuf,
    pub makefile: PathBuf,
}

/// repository hygiene 验证成功后的稳定计数。
pub struct BackendHygieneCounts {
    pub tracked_files: usize,
    pub fixture_entries: usize,
    pub tracked_fixture_hashes: usize,
    pub local_ignored_fixture_hashes: usize,
    pub negative_cases: usize,
}

struct InventoryCounts {
    entries: usize,
    tracked: usize,
    ignored: usize,
}

/// 验证 BFH-002 至 BFH-006 建立的生成物、配置、清理和 fixture 权威。
///
/// # Errors
///
/// 当生成物进入 Git、已退役入口回流、清理范围扩大、fixture 清单漂移或负向自测
/// 未能 fail closed 时返回错误。
pub fn validate_backend_hygiene(
    config: &BackendHygieneConfig,
    negative_selftest: bool,
) -> Result<BackendHygieneCounts> {
    let root = env::current_dir().context("failed to resolve repository root")?;
    let tracked = tracked_files(&root)?;
    validate_retired_root_configs(&tracked)?;
    validate_required_authority(&root, &tracked)?;

    let authority_map = read(&config.authority_map)?;
    validate_authority_map(&authority_map)?;
    let gitignore = read(&config.gitignore)?;
    validate_gitignore(&gitignore)?;
    let cargo_manifest = read(&config.cargo_manifest)?;
    validate_cargo_manifest(&cargo_manifest)?;
    let contributing = read(&config.contributing)?;
    validate_contributing(&contributing)?;
    let makefile = read(&config.makefile)?;
    validate_makefile(&makefile)?;

    validate_generated_state(&root, &tracked)?;
    let inventory = load_json(&config.inventory)?;
    let inventory_counts = validate_inventory_shape(&inventory)?;
    let (tracked_hashes, local_ignored_hashes) =
        validate_inventory_files(&root, &tracked, &inventory)?;

    let negative_cases = if negative_selftest {
        run_negative_selftests(
            &gitignore,
            &cargo_manifest,
            &contributing,
            &makefile,
            &inventory,
        )?
    } else {
        0
    };

    ensure!(
        tracked_hashes == inventory_counts.tracked,
        "tracked fixture verification count mismatch"
    );
    ensure!(
        local_ignored_hashes <= inventory_counts.ignored,
        "ignored fixture verification count mismatch"
    );

    Ok(BackendHygieneCounts {
        tracked_files: tracked.len(),
        fixture_entries: inventory_counts.entries,
        tracked_fixture_hashes: tracked_hashes,
        local_ignored_fixture_hashes: local_ignored_hashes,
        negative_cases,
    })
}

fn validate_retired_root_configs(tracked: &BTreeSet<String>) -> Result<()> {
    for retired in RETIRED_ROOT_CONFIGS {
        ensure!(
            !tracked.contains(*retired),
            "retired root configuration returned: {retired}"
        );
    }
    Ok(())
}

fn validate_required_authority(root: &Path, tracked: &BTreeSet<String>) -> Result<()> {
    for required in REQUIRED_TRACKED_AUTHORITY {
        ensure!(
            tracked.contains(*required) && root.join(required).is_file(),
            "required backend hygiene authority is not tracked: {required}"
        );
    }
    for task in 1..=7 {
        for class in ["tasks", "evidence"] {
            let relative = format!("docs/rust-cutover/{class}/BFH-{task:03}.md");
            ensure!(
                tracked.contains(&relative) && root.join(&relative).is_file(),
                "required BFH history is not tracked: {relative}"
            );
        }
    }
    Ok(())
}

fn validate_authority_map(text: &str) -> Result<()> {
    for marker in [
        "source_tree_plus_github_remote",
        "size alone does not authorize",
        "make distclean FORCE=1",
        "Phase 2 issues `#1120-#1126` are all blocked by BFH-007",
    ] {
        ensure!(
            text.contains(marker),
            "backend hygiene authority marker missing: {marker}"
        );
    }
    Ok(())
}

fn validate_gitignore(text: &str) -> Result<()> {
    let lines: BTreeSet<_> = text.lines().map(str::trim).collect();
    for required in REQUIRED_IGNORE_RULES {
        ensure!(
            lines.contains(required),
            "required generated-output ignore rule missing: {required}"
        );
    }
    for retired in RETIRED_GITIGNORE_EXCEPTIONS {
        ensure!(
            !lines.contains(retired),
            "retired Git ignore exception returned: {retired}"
        );
    }
    Ok(())
}

fn validate_cargo_manifest(text: &str) -> Result<()> {
    ensure!(
        !text.contains("[profile.ci-pr-wheel]") && !text.contains("profile.ci-pr-wheel"),
        "retired ci-pr-wheel profile returned"
    );
    for profile in ["[profile.ci-pr]", "[profile.release]", "[profile.bench]"] {
        ensure!(
            text.contains(profile),
            "required Cargo profile missing: {profile}"
        );
    }
    Ok(())
}

fn validate_contributing(text: &str) -> Result<()> {
    for marker in [
        "latest `main`",
        "rust-toolchain.toml",
        "make install-deps",
        "make install-tools",
        "scripts/ai/verify_fast.sh",
        "scripts/ai/check_zero_python_closeout.sh",
        "scripts/ai/verify_release.sh backend-freeze-baseline",
        "docs/rust-cutover/TASK_EXECUTION.md",
        "[Contributor License Agreement](CLA.md)",
    ] {
        ensure!(
            text.contains(marker),
            "current contributor route missing: {marker}"
        );
    }
    for marker in RETIRED_ACTIVE_MARKERS {
        ensure!(
            !text.contains(marker),
            "retired contributor route returned: {marker}"
        );
    }
    ensure!(
        text.contains("does not use Python, uv, or `pyproject.toml`"),
        "zero-Python contributor authority statement missing"
    );
    Ok(())
}

fn validate_makefile(text: &str) -> Result<()> {
    let build = make_variable(text, "CLEAN_BUILD_OUTPUTS")?;
    let generated = make_variable(text, "CLEAN_GENERATED_OUTPUTS")?;
    let protected = make_variable(text, "CLEAN_PROTECTED_OUTPUTS")?;
    ensure!(
        build == expected_set(EXPECTED_BUILD_OUTPUTS),
        "CLEAN_BUILD_OUTPUTS drifted: {build:?}"
    );
    ensure!(
        generated == expected_set(EXPECTED_GENERATED_OUTPUTS),
        "CLEAN_GENERATED_OUTPUTS drifted: {generated:?}"
    );
    ensure!(
        protected == expected_set(EXPECTED_PROTECTED_OUTPUTS),
        "CLEAN_PROTECTED_OUTPUTS drifted: {protected:?}"
    );
    ensure!(
        build.is_disjoint(&protected) && generated.is_disjoint(&protected),
        "cleanup allowlist overlaps protected local state"
    );
    for marker in [
        ".PHONY: clean-dry-run",
        ".PHONY: clean-generated-dry-run",
        ".PHONY: clean-generated",
        ".PHONY: distclean-dry-run",
        ".PHONY: distclean",
        "make clean-generated FORCE=1",
        "if [ \"$$FORCE\" != \"1\" ]",
        "rm -rf -- $(CLEAN_BUILD_OUTPUTS)",
        "rm -rf -- $(CLEAN_GENERATED_OUTPUTS)",
    ] {
        ensure!(
            text.contains(marker),
            "guarded cleanup marker missing: {marker}"
        );
    }
    for marker in [
        "git clean",
        "clean-build-artifacts",
        "clean-caches",
        "clean-builds",
    ] {
        ensure!(
            !text.contains(marker),
            "retired cleanup route returned: {marker}"
        );
    }
    Ok(())
}

fn make_variable(text: &str, name: &str) -> Result<BTreeSet<String>> {
    let mut values = BTreeSet::new();
    let mut found = false;
    for line in text.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed
            .strip_prefix(&format!("{name} :="))
            .or_else(|| trimmed.strip_prefix(&format!("{name} +=")))
        else {
            continue;
        };
        found = true;
        values.extend(rest.split_whitespace().map(ToOwned::to_owned));
    }
    ensure!(found, "missing Make variable: {name}");
    Ok(values)
}

fn expected_set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn validate_generated_state(root: &Path, tracked: &BTreeSet<String>) -> Result<()> {
    validate_no_tracked_generated(tracked)?;
    let mut finder_caches = Vec::new();
    find_named(root, root, ".DS_Store", &mut finder_caches)?;
    ensure!(
        finder_caches.is_empty(),
        "Finder cache exists: {finder_caches:?}"
    );
    Ok(())
}

fn validate_no_tracked_generated(tracked: &BTreeSet<String>) -> Result<()> {
    for relative in tracked {
        let first = relative.split('/').next().unwrap_or(relative);
        ensure!(
            !GENERATED_ROOTS.contains(&first)
                && !first.starts_with(".coverage")
                && !first.starts_with(".benchmarks")
                && first != ".DS_Store"
                && !relative.ends_with("/.DS_Store"),
            "generated output is tracked: {relative}"
        );
    }
    Ok(())
}

fn find_named(root: &Path, directory: &Path, name: &str, found: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(directory)
        .with_context(|| format!("failed to inspect {}", directory.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let entry_name = entry.file_name();
        let entry_name = entry_name.to_string_lossy();
        if entry_name == name {
            found.push(relative(root, &path));
            continue;
        }
        if entry.file_type()?.is_dir()
            && !matches!(
                entry_name.as_ref(),
                ".git" | "target" | "target-v2" | ".codex" | ".agentflow" | ".understand-anything"
            )
        {
            find_named(root, &path, name, found)?;
        }
    }
    Ok(())
}

fn validate_inventory_shape(inventory: &Value) -> Result<InventoryCounts> {
    ensure!(
        inventory.get("schema_version").and_then(Value::as_u64) == Some(1),
        "fixture inventory schema mismatch"
    );
    ensure!(
        inventory.get("task_id").and_then(Value::as_str) == Some("BFH-006"),
        "fixture inventory task mismatch"
    );
    let threshold = u64_field(inventory, "threshold_bytes")?;
    ensure!(threshold == 1_048_576, "fixture threshold drifted");
    let owners = object(inventory, "owners")?;
    let policies = object(inventory, "reconstruction_policies")?;
    let fixtures = array(inventory, "fixtures")?;
    let mut paths = BTreeSet::new();
    let mut tracked = 0;
    let mut ignored = 0;
    let mut tracked_bytes = 0_u64;
    let mut ignored_bytes = 0_u64;
    for fixture in fixtures {
        let fixture = fixture
            .as_object()
            .context("fixture inventory row must be an object")?;
        let path = string(fixture, "path")?;
        ensure!(
            paths.insert(path.to_string()),
            "duplicate fixture path: {path}"
        );
        ensure!(
            u64_field_object(fixture, "size_bytes")? >= threshold,
            "fixture below threshold"
        );
        let digest = string(fixture, "sha256")?;
        ensure!(
            digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "invalid fixture sha256: {path}"
        );
        let owner = string(fixture, "owner_id")?;
        ensure!(owners.contains_key(owner), "unknown fixture owner: {owner}");
        let policy = string(fixture, "reconstruction_policy_id")?;
        ensure!(
            policies.contains_key(policy),
            "unknown fixture reconstruction policy: {policy}"
        );
        ensure!(
            fixture.get("consumers").is_some_and(Value::is_array),
            "fixture consumers must be an array: {path}"
        );
        for field in ["format", "reachability", "disposition", "hash_policy"] {
            ensure!(!string(fixture, field)?.is_empty(), "empty {field}: {path}");
        }
        match string(fixture, "git_state")? {
            "tracked" => {
                tracked += 1;
                tracked_bytes += u64_field_object(fixture, "size_bytes")?;
                ensure!(
                    string(fixture, "disposition")? == "retain_tracked_quarantine",
                    "tracked fixture is not quarantined: {path}"
                );
            }
            "ignored" => {
                ignored += 1;
                ignored_bytes += u64_field_object(fixture, "size_bytes")?;
                ensure!(
                    string(fixture, "disposition")? == "retain_ignored_cache",
                    "ignored fixture disposition drifted: {path}"
                );
                ensure!(
                    !array_object(fixture, "consumers")?.is_empty(),
                    "ignored fixture lost active consumers: {path}"
                );
                ensure!(
                    fixture
                        .get("metadata_path")
                        .and_then(Value::as_str)
                        .is_some(),
                    "ignored fixture metadata path missing: {path}"
                );
            }
            state => bail!("unsupported fixture Git state: {state}"),
        }
    }
    let summary = object(inventory, "summary")?;
    ensure!(
        u64_field_object(summary, "fixture_count")? == fixtures.len() as u64
            && u64_field_object(summary, "tracked_count")? == tracked as u64
            && u64_field_object(summary, "ignored_count")? == ignored as u64
            && u64_field_object(summary, "tracked_bytes")? == tracked_bytes
            && u64_field_object(summary, "ignored_bytes")? == ignored_bytes
            && u64_field_object(summary, "total_bytes")? == tracked_bytes + ignored_bytes,
        "fixture inventory summary drifted"
    );
    let deletion_gate = object(inventory, "deletion_gate")?;
    ensure!(
        deletion_gate.get("size_only_deletion_allowed") == Some(&Value::Bool(false))
            && deletion_gate.get("network_response_as_fixture_replacement_allowed")
                == Some(&Value::Bool(false)),
        "fixture deletion gate must remain fail closed"
    );
    Ok(InventoryCounts {
        entries: fixtures.len(),
        tracked,
        ignored,
    })
}

fn validate_inventory_files(
    root: &Path,
    tracked: &BTreeSet<String>,
    inventory: &Value,
) -> Result<(usize, usize)> {
    let fixtures = array(inventory, "fixtures")?;
    let registered: BTreeSet<_> = fixtures
        .iter()
        .filter_map(|fixture| fixture.get("path").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .collect();
    let threshold = u64_field(inventory, "threshold_bytes")?;
    let checksums = load_json(&root.join("tests/test_data/large/checksums.json"))?;
    let checksums = checksums
        .as_object()
        .context("large fixture checksums must be an object")?;
    let baseline = string_value(inventory, "baseline_commit")?;
    ensure_git_object(root, baseline)?;

    let mut tracked_hashes = 0;
    let mut ignored_hashes = 0;
    for fixture in fixtures {
        let fixture = fixture
            .as_object()
            .context("fixture inventory row must be an object")?;
        let relative = string(fixture, "path")?;
        let file = root.join(relative);
        let state = string(fixture, "git_state")?;
        if state == "tracked" {
            ensure!(
                tracked.contains(relative),
                "fixture is no longer tracked: {relative}"
            );
            ensure!(file.is_file(), "tracked fixture is missing: {relative}");
            validate_file_identity(&file, fixture)?;
            ensure_git_blob(root, baseline, relative)?;
            tracked_hashes += 1;
        } else {
            ensure!(
                !tracked.contains(relative),
                "ignored fixture became tracked: {relative}"
            );
            ensure_git_ignored(root, relative)?;
            let metadata_path = string(fixture, "metadata_path")?;
            ensure!(
                tracked.contains(metadata_path) && root.join(metadata_path).is_file(),
                "ignored fixture metadata is not tracked: {metadata_path}"
            );
            let metadata = load_json(&root.join(metadata_path))?;
            ensure!(
                metadata.get("sha256").and_then(Value::as_str)
                    == fixture.get("sha256").and_then(Value::as_str)
                    && metadata.get("size_bytes").and_then(Value::as_u64)
                        == fixture.get("size_bytes").and_then(Value::as_u64),
                "ignored fixture metadata identity drifted: {relative}"
            );
            let name = Path::new(relative)
                .file_name()
                .and_then(|value| value.to_str())
                .context("fixture path has no UTF-8 filename")?;
            let expected_checksum = format!("sha256:{}", string(fixture, "sha256")?);
            ensure!(
                checksums.get(name).and_then(Value::as_str) == Some(expected_checksum.as_str()),
                "ignored fixture checksum authority drifted: {relative}"
            );
            if file.exists() {
                validate_file_identity(&file, fixture)?;
                ignored_hashes += 1;
            }
        }
    }
    let mut existing_large = BTreeSet::new();
    collect_large_files(root, &root.join("tests"), threshold, &mut existing_large)?;
    let unregistered: Vec<_> = existing_large.difference(&registered).collect();
    ensure!(
        unregistered.is_empty(),
        "large fixture is not registered: {unregistered:?}"
    );
    Ok((tracked_hashes, ignored_hashes))
}

fn validate_file_identity(path: &Path, fixture: &Map<String, Value>) -> Result<()> {
    let metadata = fs::metadata(path)?;
    let expected_size = u64_field_object(fixture, "size_bytes")?;
    ensure!(
        metadata.len() == expected_size,
        "fixture size drifted: {}",
        path.display()
    );
    let digest = format!("{:x}", Sha256::digest(fs::read(path)?));
    ensure!(
        digest == string(fixture, "sha256")?,
        "fixture hash drifted: {}",
        path.display()
    );
    Ok(())
}

fn collect_large_files(
    root: &Path,
    directory: &Path,
    threshold: u64,
    files: &mut BTreeSet<String>,
) -> Result<()> {
    for entry in fs::read_dir(directory)
        .with_context(|| format!("failed to inspect {}", directory.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_large_files(root, &path, threshold, files)?;
        } else if entry.metadata()?.len() >= threshold {
            files.insert(relative(root, &path));
        }
    }
    Ok(())
}

fn run_negative_selftests(
    gitignore: &str,
    cargo_manifest: &str,
    contributing: &str,
    makefile: &str,
    inventory: &Value,
) -> Result<usize> {
    expect_invalid(
        validate_gitignore(&gitignore.replace("/graphify-out/\n", "")).is_err(),
        "missing generated ignore",
    )?;
    expect_invalid(
        validate_gitignore(&format!("{gitignore}\n!scripts/ci/plan.sh\n")).is_err(),
        "retired ignore exception",
    )?;
    expect_invalid(
        validate_cargo_manifest(&format!("{cargo_manifest}\n[profile.ci-pr-wheel]\n")).is_err(),
        "retired Cargo profile",
    )?;
    expect_invalid(
        validate_contributing(&contributing.replace("latest `main`", "`develop`")).is_err(),
        "stale contributor branch",
    )?;
    expect_invalid(
        validate_makefile(&format!("{makefile}\nlegacy:\n\tgit clean -fxd\n")).is_err(),
        "unbounded cleanup",
    )?;
    expect_invalid(
        validate_makefile(&makefile.replace(
            "CLEAN_BUILD_OUTPUTS := target target-v2 build dist .coverage*",
            "CLEAN_BUILD_OUTPUTS := target target-v2 build dist .coverage* .codex",
        ))
        .is_err(),
        "protected cleanup overlap",
    )?;
    expect_invalid(
        validate_no_tracked_generated(&BTreeSet::from([
            "release-publication-evidence/result.json".to_string(),
        ]))
        .is_err(),
        "tracked generated output",
    )?;
    let mut missing_owner = inventory.clone();
    missing_owner["fixtures"][0]
        .as_object_mut()
        .context("negative fixture row must be an object")?
        .remove("owner_id");
    expect_invalid(
        validate_inventory_shape(&missing_owner).is_err(),
        "missing fixture owner",
    )?;
    let mut deletion_enabled = inventory.clone();
    deletion_enabled["deletion_gate"]["size_only_deletion_allowed"] = Value::Bool(true);
    expect_invalid(
        validate_inventory_shape(&deletion_enabled).is_err(),
        "size-only fixture deletion",
    )?;
    let tracked_fixture = array(inventory, "fixtures")?
        .iter()
        .find(|fixture| fixture.get("git_state").and_then(Value::as_str) == Some("tracked"))
        .and_then(Value::as_object)
        .context("negative selftest requires a tracked fixture")?;
    let mut hash_drift = tracked_fixture.clone();
    hash_drift.insert(
        "sha256".to_string(),
        Value::String(
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        ),
    );
    expect_invalid(
        validate_file_identity(Path::new(string(&hash_drift, "path")?), &hash_drift).is_err(),
        "fixture hash drift",
    )?;
    Ok(10)
}

fn expect_invalid(is_invalid: bool, label: &str) -> Result<()> {
    ensure!(is_invalid, "negative selftest accepted {label}");
    Ok(())
}

fn tracked_files(root: &Path) -> Result<BTreeSet<String>> {
    let output = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output()
        .context("failed to invoke git ls-files")?;
    ensure!(output.status.success(), "git ls-files failed");
    let text = String::from_utf8(output.stdout).context("tracked path list is not UTF-8")?;
    Ok(text
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn ensure_git_ignored(root: &Path, relative: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["check-ignore", "-q", "--", relative])
        .current_dir(root)
        .status()
        .context("failed to invoke git check-ignore")?;
    ensure!(status.success(), "fixture path is not ignored: {relative}");
    Ok(())
}

fn ensure_git_object(root: &Path, object: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["cat-file", "-e", &format!("{object}^{{commit}}")])
        .current_dir(root)
        .status()
        .context("failed to invoke git cat-file")?;
    ensure!(
        status.success(),
        "fixture baseline commit is unavailable: {object}"
    );
    Ok(())
}

fn ensure_git_blob(root: &Path, commit: &str, relative: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["cat-file", "-e", &format!("{commit}:{relative}")])
        .current_dir(root)
        .status()
        .context("failed to invoke git cat-file")?;
    ensure!(
        status.success(),
        "fixture is not reconstructable from baseline: {commit}:{relative}"
    );
    Ok(())
}

fn load_json(path: &Path) -> Result<Value> {
    let text = read(path)?;
    serde_json::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))
}

fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
}

fn object<'a>(value: &'a Value, key: &str) -> Result<&'a Map<String, Value>> {
    value
        .get(key)
        .and_then(Value::as_object)
        .with_context(|| format!("missing object: {key}"))
}

fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .with_context(|| format!("missing array: {key}"))
}

fn array_object<'a>(value: &'a Map<String, Value>, key: &str) -> Result<&'a Vec<Value>> {
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

fn string_value<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .with_context(|| format!("missing string: {key}"))
}

fn u64_field(value: &Value, key: &str) -> Result<u64> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .with_context(|| format!("missing integer: {key}"))
}

fn u64_field_object(value: &Map<String, Value>, key: &str) -> Result<u64> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .with_context(|| format!("missing integer: {key}"))
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_variable_collects_assignment_and_append() {
        let text = "CLEAN := target build\nCLEAN += dist\n";
        assert_eq!(
            make_variable(text, "CLEAN").unwrap(),
            expected_set(&["target", "build", "dist"])
        );
    }

    #[test]
    fn generated_root_detection_is_root_scoped() {
        assert!(GENERATED_ROOTS.contains(&"target"));
        assert!(!GENERATED_ROOTS.contains(&"docs"));
        assert!(
            validate_no_tracked_generated(&BTreeSet::from([".coverage.xml".to_string()])).is_err()
        );
        assert!(
            validate_no_tracked_generated(&BTreeSet::from(["docs/target/report.md".to_string()]))
                .is_ok()
        );
    }

    #[test]
    fn retired_root_configuration_is_fail_closed() {
        validate_retired_root_configs(&BTreeSet::new()).unwrap();
        assert!(
            validate_retired_root_configs(&BTreeSet::from([".codecov.yml".to_string()])).is_err()
        );
    }

    #[test]
    fn fixture_negative_matrix_is_fail_closed() {
        let inventory = serde_json::json!({
            "schema_version": 1,
            "task_id": "BFH-006",
            "threshold_bytes": 1048576,
            "owners": {"owner": {}},
            "reconstruction_policies": {"policy": {}},
            "fixtures": [{
                "path": "tests/a.bin",
                "size_bytes": 1048576,
                "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "git_state": "tracked",
                "format": "bin",
                "owner_id": "owner",
                "consumers": [],
                "reachability": "none",
                "disposition": "retain_tracked_quarantine",
                "hash_policy": "sha256",
                "reconstruction_policy_id": "policy"
            }],
            "summary": {
                "fixture_count": 1,
                "tracked_count": 1,
                "ignored_count": 0,
                "tracked_bytes": 1048576,
                "ignored_bytes": 0,
                "total_bytes": 1048576
            },
            "deletion_gate": {
                "size_only_deletion_allowed": false,
                "network_response_as_fixture_replacement_allowed": false
            }
        });
        assert!(validate_inventory_shape(&inventory).is_ok());
        let mut mutated = inventory;
        mutated["deletion_gate"]["size_only_deletion_allowed"] = Value::Bool(true);
        assert!(validate_inventory_shape(&mutated).is_err());
    }
}
