// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
// -------------------------------------------------------------------------------------------------

use std::{collections::HashSet, fs, path::Path, process::Command};

use anyhow::{Context, Result, bail, ensure};
use serde_json::Value;
use sha2::{Digest, Sha256};

const SCHEMA_VERSION: &str = "ntpro.historical_release_executable_retirement.v1";
const EXPECTED_TASK: &str = "PTC-006";
const EXPECTED_STATUS: &str = "review_required";
const EXPECTED_RISK: &str = "high";
const EXPECTED_RETIRED: usize = 263;
const EXPECTED_TOOLING: usize = 233;
const EXPECTED_TAGS: usize = 49;
const STRICT_RELEASE_SCRIPT: &str = "scripts/ai/verify_release_strict.sh";
const ACTIVE_AUTHORITY: &[&str] = &[
    ".github/workflows",
    ".github/actions",
    "scripts/ai/verify_release.sh",
    "scripts/ai/verify_fast.sh",
    "scripts/ai/check_github_release_published.sh",
    "scripts/ai/verify_release_publish_after_gate.sh",
    "Makefile",
];

/// Counts emitted after historical executable retirement validation.
pub struct HistoricalReleaseRetirementCounts {
    pub retired: usize,
    pub tooling: usize,
    pub tags: usize,
    pub restored_blobs: usize,
    pub negative_cases: usize,
}

/// Validates the source-controlled retirement inventory and its Git recovery path.
///
/// # Errors
///
/// Returns an error when manifest identity or counts drift, a retired executable
/// reappears, an active authority references one, or Git cannot reconstruct an
/// inventoried blob with the recorded identity.
pub fn validate_historical_release_retirement(
    manifest_path: &Path,
    negative_selftest: bool,
) -> Result<HistoricalReleaseRetirementCounts> {
    let bytes = fs::read(manifest_path)
        .with_context(|| format!("failed to read manifest: {}", manifest_path.display()))?;
    let manifest: Value = serde_json::from_slice(&bytes).context("manifest is not valid JSON")?;
    let inventory = validate_manifest_shape(&manifest)?;

    let negative_cases = if negative_selftest {
        run_negative_selftests(&manifest)?
    } else {
        0
    };

    for entry in &inventory.entries {
        ensure!(
            !Path::new(&entry.path).exists(),
            "retired historical executable still exists: {}",
            entry.path
        );
    }
    validate_active_authority(&inventory.entries)?;
    validate_tags(&inventory.tags)?;

    let mut restored_blobs = 0;
    for entry in &inventory.entries {
        let object = format!("{}:{}", inventory.source_commit, entry.path);
        let output = Command::new("git")
            .args(["show", &object])
            .output()
            .with_context(|| format!("failed to invoke git for {}", entry.path))?;
        ensure!(
            output.status.success(),
            "Git cannot restore {} from {}",
            entry.path,
            inventory.source_commit
        );
        ensure!(
            hex_sha256(&output.stdout) == entry.sha256,
            "restored SHA-256 drifted: {}",
            entry.path
        );
        ensure!(
            output.stdout.len() == entry.bytes,
            "restored byte count drifted: {}",
            entry.path
        );
        ensure!(
            contains_tooling(&output.stdout) == entry.contains_tooling,
            "tooling classification drifted: {}",
            entry.path
        );

        let blob = git_stdout(["rev-parse", &object])?;
        ensure!(
            blob == entry.blob_sha,
            "restored Git blob drifted: {}",
            entry.path
        );
        let last_change = git_stdout([
            "log",
            "-1",
            "--format=%H",
            &inventory.source_commit,
            "--",
            &entry.path,
        ])?;
        ensure!(
            last_change == entry.last_change_commit,
            "last change commit drifted: {}",
            entry.path
        );
        restored_blobs += 1;
    }

    Ok(HistoricalReleaseRetirementCounts {
        retired: inventory.entries.len(),
        tooling: inventory.tooling,
        tags: inventory.tags.len(),
        restored_blobs,
        negative_cases,
    })
}

struct Inventory {
    source_commit: String,
    tags: Vec<String>,
    entries: Vec<RetiredEntry>,
    tooling: usize,
}

#[derive(Clone)]
struct RetiredEntry {
    path: String,
    sha256: String,
    blob_sha: String,
    last_change_commit: String,
    bytes: usize,
    contains_tooling: bool,
}

fn validate_manifest_shape(manifest: &Value) -> Result<Inventory> {
    ensure_string(manifest, "schema_version", SCHEMA_VERSION)?;
    ensure_string(manifest, "task_id", EXPECTED_TASK)?;
    ensure_string(manifest, "status", EXPECTED_STATUS)?;
    ensure_string(manifest, "risk", EXPECTED_RISK)?;
    ensure!(
        usize_field(manifest, "github_issue")? == 1101,
        "GitHub issue identity drifted"
    );

    let baseline = manifest
        .get("baseline")
        .context("manifest missing baseline")?;
    let source_commit = string_field(baseline, "pre_retirement_source_commit")?.to_owned();
    ensure_hex(&source_commit, 40, "source commit")?;
    ensure!(
        string_field(baseline, "history_authority")? == "immutable_release_tags_plus_git_history",
        "history authority drifted"
    );
    let tags = string_array(baseline, "release_tags")?;
    ensure!(tags.len() == EXPECTED_TAGS, "release tag count drifted");
    ensure_unique(&tags, "release tag")?;

    let retirement = manifest
        .get("retirement")
        .context("manifest missing retirement")?;
    ensure!(
        usize_field(retirement, "historical_executable_count")? == EXPECTED_RETIRED,
        "declared retired executable count drifted"
    );
    ensure!(
        string_field(retirement, "restore_source_commit")? == source_commit,
        "restore source commit differs from baseline"
    );
    ensure!(
        retirement.get("source_tree_required_after_retirement") == Some(&Value::Bool(false)),
        "retired executables must remain absent from the source tree"
    );

    let preserved = string_array(manifest, "preserved_authority")?;
    for required in [
        "docs/rust-cutover/**",
        "tests/golden/**",
        "Rust integration tests",
        "immutable Git tags",
        "published GitHub Releases",
        "hosted workflow run references",
    ] {
        ensure!(
            preserved.iter().any(|value| value == required),
            "preserved authority missing: {required}"
        );
    }

    let values = manifest
        .get("retired_executables")
        .and_then(Value::as_array)
        .context("manifest missing retired_executables")?;
    ensure!(
        values.len() == EXPECTED_RETIRED,
        "retired executable count drifted"
    );
    let mut entries = Vec::with_capacity(values.len());
    let mut tooling = 0;
    for value in values {
        let path = string_field(value, "path")?.to_owned();
        ensure!(
            path == STRICT_RELEASE_SCRIPT
                || (path.starts_with("scripts/ai/verify_v") && path.ends_with(".sh")),
            "unexpected retired executable path: {path}"
        );
        let sha256 = string_field(value, "sha256")?.to_owned();
        ensure_hex(&sha256, 64, &format!("SHA-256 for {path}"))?;
        let blob_sha = string_field(value, "git_blob_sha")?.to_owned();
        ensure_hex(&blob_sha, 40, &format!("Git blob SHA for {path}"))?;
        let last_change_commit = string_field(value, "last_change_commit")?.to_owned();
        ensure_hex(
            &last_change_commit,
            40,
            &format!("last change commit for {path}"),
        )?;
        let bytes = usize_field(value, "bytes")?;
        let contains_tooling = value
            .get("contains_python_tooling")
            .and_then(Value::as_bool)
            .with_context(|| format!("tooling classification must be boolean for {path}"))?;
        let restore_command = string_field(value, "restore_command")?;
        let expected_restore = format!("git show {source_commit}:{path} > {path}");
        ensure!(
            restore_command == expected_restore,
            "restore command drifted for {path}"
        );
        if contains_tooling {
            tooling += 1;
        }
        entries.push(RetiredEntry {
            path,
            sha256,
            blob_sha,
            last_change_commit,
            bytes,
            contains_tooling,
        });
    }
    let paths: Vec<_> = entries.iter().map(|entry| entry.path.clone()).collect();
    ensure_unique(&paths, "retired executable")?;
    ensure!(
        tooling == EXPECTED_TOOLING,
        "tooling classification count drifted"
    );

    Ok(Inventory {
        source_commit,
        tags,
        entries,
        tooling,
    })
}

fn run_negative_selftests(manifest: &Value) -> Result<usize> {
    let mutations = [
        ("schema_version", Value::String("invalid".to_owned())),
        ("status", Value::String("released".to_owned())),
        ("risk", Value::String("medium".to_owned())),
    ];
    let mut cases = 0;
    for (field, value) in mutations {
        let mut mutated = manifest.clone();
        mutated[field] = value;
        ensure!(
            validate_manifest_shape(&mutated).is_err(),
            "negative selftest accepted mutated {field}"
        );
        cases += 1;
    }
    let mut duplicated = manifest.clone();
    let values = duplicated["retired_executables"]
        .as_array_mut()
        .context("negative fixture missing entries")?;
    values[1]["path"] = values[0]["path"].clone();
    ensure!(
        validate_manifest_shape(&duplicated).is_err(),
        "negative selftest accepted duplicate path"
    );
    Ok(cases + 1)
}

fn validate_active_authority(entries: &[RetiredEntry]) -> Result<()> {
    let retired: HashSet<_> = entries.iter().map(|entry| entry.path.as_str()).collect();
    let mut files = Vec::new();
    for authority in ACTIVE_AUTHORITY {
        collect_files(Path::new(authority), &mut files)?;
    }
    for path in files {
        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read active authority: {}", path.display()))?;
        ensure!(
            !text.contains("verify_release_strict.sh"),
            "active authority references the retired strict release executable: {}",
            path.display()
        );
        ensure!(
            !(text.contains("verify_v") && text.contains(".sh")),
            "active authority references a version-specific release executable: {}",
            path.display()
        );
        for retired_path in &retired {
            ensure!(
                !text.contains(retired_path),
                "active authority references retired executable {retired_path}: {}",
                path.display()
            );
        }
    }
    Ok(())
}

fn collect_files(path: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
    ensure!(
        path.exists(),
        "missing active authority: {}",
        path.display()
    );
    if path.is_file() {
        files.push(path.to_owned());
        return Ok(());
    }
    for entry in fs::read_dir(path)
        .with_context(|| format!("failed to read authority directory: {}", path.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_files(&path, files)?;
        } else if entry.file_type()?.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn validate_tags(expected: &[String]) -> Result<()> {
    let output = git_stdout(["tag", "--list", "ntpro-rust-only-v*"])?;
    let actual: HashSet<_> = output.lines().collect();
    for tag in expected {
        ensure!(
            actual.contains(tag.as_str()),
            "immutable release tag missing: {tag}"
        );
    }
    Ok(())
}

fn git_stdout<const N: usize>(args: [&str; N]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context("failed to invoke git")?;
    if !output.status.success() {
        bail!(
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)
        .context("git output is not UTF-8")?
        .trim()
        .to_owned())
}

fn ensure_string(value: &Value, field: &str, expected: &str) -> Result<()> {
    ensure!(string_field(value, field)? == expected, "{field} drifted");
    Ok(())
}

fn string_field<'a>(value: &'a Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .with_context(|| format!("manifest field {field} must be a string"))
}

fn usize_field(value: &Value, field: &str) -> Result<usize> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|number| usize::try_from(number).ok())
        .with_context(|| format!("manifest field {field} must be an unsigned integer"))
}

fn string_array(value: &Value, field: &str) -> Result<Vec<String>> {
    value
        .get(field)
        .and_then(Value::as_array)
        .with_context(|| format!("manifest field {field} must be an array"))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToOwned::to_owned)
                .with_context(|| format!("manifest field {field} must contain strings"))
        })
        .collect()
}

fn ensure_unique(values: &[String], label: &str) -> Result<()> {
    let unique: HashSet<_> = values.iter().collect();
    ensure!(unique.len() == values.len(), "duplicate {label} entry");
    Ok(())
}

fn ensure_hex(value: &str, length: usize, label: &str) -> Result<()> {
    ensure!(
        value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "invalid {label}"
    );
    Ok(())
}

fn contains_tooling(bytes: &[u8]) -> bool {
    let text = String::from_utf8_lossy(bytes).to_ascii_lowercase();
    ["python", "uv run", "pytest", "ruff", "pip-audit"]
        .iter()
        .any(|marker| text.contains(marker))
}

fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retirement_path_scope_is_explicit() {
        assert_eq!(EXPECTED_RETIRED, 263);
        assert_eq!(EXPECTED_TOOLING, 233);
        assert_eq!(EXPECTED_TAGS, 49);
    }
}
