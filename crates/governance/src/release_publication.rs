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
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

type ReleaseSections = (Map<String, Value>, Map<String, Value>, Map<String, Value>);

/// Inputs for source-controlled publish-after-gate binding validation.
pub struct ReleaseBindingConfig {
    pub manifest: PathBuf,
    pub closeout: PathBuf,
    pub version: String,
    pub tag: String,
    pub name: String,
    pub gate_run_id: u64,
    pub tag_sha: String,
}

/// Identity returned after release binding validation.
pub struct ReleaseBindingResult {
    pub release_tag: String,
    pub release_gate_run_id: u64,
    pub tag_sha: String,
    pub negative_cases: usize,
}

/// Validates source-controlled current-release publish-after-gate evidence and
/// four deterministic negative mutations.
///
/// # Errors
///
/// Returns an error when planned/published identity, hosted gate ordering,
/// source evidence, or current-release binding drifts.
pub fn validate_release_binding(config: &ReleaseBindingConfig) -> Result<ReleaseBindingResult> {
    ensure!(
        config.manifest.is_file(),
        "missing current release manifest: {}",
        config.manifest.display()
    );
    ensure!(
        config.closeout.is_file(),
        "missing current release closeout: {}",
        config.closeout.display()
    );
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(&config.manifest)
            .with_context(|| format!("failed to read {}", config.manifest.display()))?,
    )
    .with_context(|| format!("invalid JSON: {}", config.manifest.display()))?;
    let closeout_text = fs::read_to_string(&config.closeout)
        .with_context(|| format!("failed to read {}", config.closeout.display()))?;
    validate_manifest(&manifest, &closeout_text, config)?;

    let mut missing_closeout = manifest.clone();
    if let Some(object) = missing_closeout.as_object_mut() {
        object.remove("post_publication_closeout");
        object.remove("post_release_closeout");
    }
    expect_invalid(
        &missing_closeout,
        &closeout_text,
        config,
        "missing current closeout proof",
    )?;

    let mut stale = manifest.clone();
    set_published_at(&mut stale, "2026-07-08T20:00:00Z")?;
    expect_invalid(
        &stale,
        &closeout_text,
        config,
        "stale publication timestamp",
    )?;

    let mut sha_mismatch = manifest.clone();
    set_gate_head_sha(&mut sha_mismatch, "deadbeef")?;
    expect_invalid(&sha_mismatch, &closeout_text, config, "gate sha mismatch")?;

    let mut fixture_only = manifest;
    fixture_only["publication_governance"]["historical_fixture_only_current_release_proof_allowed"] =
        Value::Bool(true);
    expect_invalid(&fixture_only, &closeout_text, config, "fixture-only proof")?;

    Ok(ReleaseBindingResult {
        release_tag: config.tag.clone(),
        release_gate_run_id: config.gate_run_id,
        tag_sha: config.tag_sha.clone(),
        negative_cases: 4,
    })
}

/// Returns true when `left` is at or after `right` as RFC3339 timestamps.
///
/// # Errors
///
/// Returns an error when either timestamp is empty or invalid RFC3339.
pub fn timestamp_ge(left: &str, right: &str) -> Result<bool> {
    let left = parse_timestamp(left)?;
    let right = parse_timestamp(right)?;
    Ok(left >= right)
}

/// Builds the normalized and raw SHA-256 report for a GitHub release body and
/// its tracked release notes.
///
/// # Errors
///
/// Returns an error when the release JSON or notes file cannot be read.
pub fn release_body_hash_report(release_json: &str, notes_path: &Path) -> Result<String> {
    let release: Value =
        serde_json::from_str(release_json).context("invalid GitHub release JSON")?;
    let notes = fs::read_to_string(notes_path)
        .with_context(|| format!("failed to read {}", notes_path.display()))?;
    let body = release.get("body").and_then(Value::as_str).unwrap_or("");
    let normalized_body = normalize(body);
    let normalized_notes = normalize(&notes);
    Ok([
        "release_body_hash_semantics=normalized_sha256".to_string(),
        "release_body_normalization=line_rstrip_and_outer_strip".to_string(),
        format!(
            "release_body_normalized_sha256={}",
            sha256(&normalized_body)
        ),
        format!(
            "tracked_release_notes_normalized_sha256={}",
            sha256(&normalized_notes)
        ),
        format!(
            "release_body_normalized_sha256_matches_tracked_release_notes={}",
            normalized_body == normalized_notes
        ),
        format!("release_body_raw_sha256={}", sha256(body)),
        format!("tracked_release_notes_raw_sha256={}", sha256(&notes)),
        format!(
            "release_body_raw_sha256_matches_tracked_release_notes={}",
            body == notes
        ),
        "release_body_raw_sha256_is_acceptance_rule=false".to_string(),
    ]
    .join("\n"))
}

fn validate_manifest(
    manifest: &Value,
    closeout_text: &str,
    config: &ReleaseBindingConfig,
) -> Result<()> {
    ensure!(
        manifest.get("product_version").and_then(Value::as_str) == Some(config.version.as_str()),
        "product version mismatch"
    );
    let (planned, published, closeout) = canonical_sections(manifest)?;
    let governance = object(manifest, "publication_governance")?;
    let requirements = object(manifest, "post_publication_requirements")?;

    ensure!(
        string(&planned, "tag")? == config.tag,
        "planned tag mismatch"
    );
    ensure!(
        string(&planned, "name")? == config.name,
        "planned release name mismatch"
    );
    ensure!(
        string(&published, "tag")? == config.tag,
        "published tag mismatch"
    );
    ensure!(
        string(&published, "name")? == config.name,
        "published release name mismatch"
    );
    ensure!(
        published.get("draft") == Some(&Value::Bool(false)),
        "published release must not be draft"
    );
    ensure!(
        published.get("prerelease") == Some(&Value::Bool(false)),
        "published release must not be prerelease"
    );
    let published_sha = string(&published, "tag_sha")?;
    ensure!(!published_sha.is_empty(), "published tag_sha missing");
    ensure!(
        published_sha == config.tag_sha,
        "published tag sha mismatch"
    );
    let published_at = string(&published, "published_at")?;
    ensure!(!published_at.is_empty(), "published_at missing");

    ensure!(
        string(governance, "publication_evidence_strategy")? == "source_tree_plus_github_remote",
        "publication evidence strategy mismatch"
    );
    ensure!(
        governance.get("local_generated_evidence_required_in_source_tree")
            == Some(&Value::Bool(false)),
        "generated evidence must not be required in source tree"
    );
    ensure!(
        governance.get("remote_reconstruction_required") == Some(&Value::Bool(true)),
        "remote reconstruction requirement missing"
    );
    ensure!(
        governance.get("release_gate_success_before_publication_required")
            == Some(&Value::Bool(true)),
        "gate-before-publication requirement missing"
    );
    ensure!(
        governance.get("public_release_requires_successful_hosted_gate_for_same_tag_commit")
            == Some(&Value::Bool(true)),
        "same-tag hosted gate requirement missing"
    );
    ensure!(
        governance.get("current_release_publish_after_gate_binding_required")
            == Some(&Value::Bool(true)),
        "current release binding requirement missing"
    );
    ensure!(
        governance.get("historical_fixture_only_current_release_proof_allowed")
            == Some(&Value::Bool(false)),
        "fixture-only proof must not be allowed"
    );

    ensure!(
        closeout.get("release_gate_run_id").and_then(Value::as_u64) == Some(config.gate_run_id),
        "current release gate run id mismatch"
    );
    ensure!(
        string(&closeout, "release_gate_status")? == "completed",
        "release gate status mismatch"
    );
    ensure!(
        string(&closeout, "release_gate_conclusion")? == "success",
        "release gate conclusion mismatch"
    );
    ensure!(
        string(&closeout, "release_gate_workflow_name")? == "Rust Cutover Release Gate",
        "release gate workflow mismatch"
    );
    ensure!(
        string(&closeout, "release_gate_head_sha")? == published_sha,
        "gate head sha must match published tag sha"
    );
    ensure!(
        closeout.get("published_after_hosted_gate") == Some(&Value::Bool(true)),
        "published_after_hosted_gate must be true"
    );
    ensure!(
        closeout.get("generated_evidence_is_sole_proof") == Some(&Value::Bool(false)),
        "generated evidence must not be sole proof"
    );
    ensure!(
        closeout.get("source_controlled_closeout_evidence") == Some(&Value::Bool(true)),
        "source-controlled closeout evidence missing"
    );
    ensure!(
        string(&closeout, "current_release_publish_after_gate_binding")? == "pass",
        "current release binding status missing"
    );

    let gate_completed = string(&closeout, "release_gate_completed_at")?;
    ensure!(
        timestamp_ge(published_at, gate_completed)?,
        "published_at is before hosted gate completion"
    );

    ensure!(
        requirements.get("publication_after_hosted_gate_required") == Some(&Value::Bool(true)),
        "post-publication ordering requirement missing"
    );
    ensure!(
        requirements.get("github_release_published_required") == Some(&Value::Bool(true)),
        "published release requirement missing"
    );
    ensure!(
        requirements.get("source_controlled_closeout_evidence_required")
            == Some(&Value::Bool(true)),
        "source closeout requirement missing"
    );

    for marker in [
        "release publication after gate = pass".to_string(),
        "release publish after gate current-release binding = pass".to_string(),
        format!("release_gate_run_id = {}", config.gate_run_id),
        "published_at is public publication proof = true".to_string(),
        "historical fixture-only current-release proof allowed = false".to_string(),
    ] {
        ensure!(
            closeout_text.contains(&marker),
            "closeout marker missing: {marker}"
        );
    }
    Ok(())
}

fn canonical_sections(manifest: &Value) -> Result<ReleaseSections> {
    let planned = object(manifest, "planned_release")?.clone();
    if let Some(published) = manifest.get("published_release").and_then(Value::as_object)
        && !published.is_empty()
    {
        let closeout = object(manifest, "post_publication_closeout")?.clone();
        return Ok((planned, published.clone(), closeout));
    }

    let release_closeout = object(manifest, "post_release_closeout")?;
    let github_release = object_value(release_closeout, "github_release")?;
    let tag = object_value(release_closeout, "tag")?;
    let hosted_gate = object_value(release_closeout, "hosted_release_gate")?;
    let evidence = object_value(release_closeout, "publication_evidence")?;
    let published = map([
        (
            "tag",
            github_release.get("tag").cloned().unwrap_or(Value::Null),
        ),
        (
            "name",
            github_release.get("name").cloned().unwrap_or(Value::Null),
        ),
        (
            "github_release_url",
            github_release.get("url").cloned().unwrap_or(Value::Null),
        ),
        (
            "target_commitish",
            github_release
                .get("target_commitish")
                .cloned()
                .unwrap_or(Value::Null),
        ),
        (
            "draft",
            github_release.get("draft").cloned().unwrap_or(Value::Null),
        ),
        (
            "prerelease",
            github_release
                .get("prerelease")
                .cloned()
                .unwrap_or(Value::Null),
        ),
        (
            "tag_sha",
            tag.get("peeled_commit").cloned().unwrap_or(Value::Null),
        ),
        (
            "published_at",
            github_release
                .get("published_at")
                .cloned()
                .unwrap_or(Value::Null),
        ),
    ]);
    let closeout = map([
        (
            "release_gate_run_id",
            hosted_gate.get("run_id").cloned().unwrap_or(Value::Null),
        ),
        (
            "release_gate_status",
            hosted_gate.get("status").cloned().unwrap_or(Value::Null),
        ),
        (
            "release_gate_conclusion",
            hosted_gate
                .get("conclusion")
                .cloned()
                .unwrap_or(Value::Null),
        ),
        (
            "release_gate_workflow_name",
            hosted_gate.get("workflow").cloned().unwrap_or(Value::Null),
        ),
        (
            "release_gate_head_sha",
            hosted_gate.get("head_sha").cloned().unwrap_or(Value::Null),
        ),
        (
            "release_gate_completed_at",
            hosted_gate
                .get("completed_at")
                .cloned()
                .unwrap_or(Value::Null),
        ),
        (
            "published_after_hosted_gate",
            Value::Bool(
                evidence.get("status").and_then(Value::as_str) == Some("published_after_gate"),
            ),
        ),
        (
            "generated_evidence_is_sole_proof",
            evidence
                .get("generated_publication_evidence_sole_proof_allowed")
                .cloned()
                .unwrap_or(Value::Null),
        ),
        (
            "source_controlled_closeout_evidence",
            Value::Bool(
                release_closeout
                    .get("closeout_evidence_path")
                    .and_then(Value::as_str)
                    .is_some_and(|value| !value.is_empty()),
            ),
        ),
        (
            "current_release_publish_after_gate_binding",
            evidence
                .get("current_release_publish_after_gate_binding")
                .cloned()
                .unwrap_or(Value::Null),
        ),
    ]);
    Ok((planned, published, closeout))
}

fn set_published_at(manifest: &mut Value, value: &str) -> Result<()> {
    if manifest
        .get("published_release")
        .is_some_and(Value::is_object)
    {
        manifest["published_release"]["published_at"] = json!(value);
    } else {
        ensure!(
            manifest.get("post_release_closeout").is_some(),
            "missing post release closeout"
        );
        manifest["post_release_closeout"]["github_release"]["published_at"] = json!(value);
    }
    Ok(())
}

fn set_gate_head_sha(manifest: &mut Value, value: &str) -> Result<()> {
    if manifest
        .get("published_release")
        .is_some_and(Value::is_object)
        && manifest
            .get("post_publication_closeout")
            .is_some_and(Value::is_object)
    {
        manifest["post_publication_closeout"]["release_gate_head_sha"] = json!(value);
    } else {
        ensure!(
            manifest.get("post_release_closeout").is_some(),
            "missing post release closeout"
        );
        manifest["post_release_closeout"]["hosted_release_gate"]["head_sha"] = json!(value);
    }
    Ok(())
}

fn expect_invalid(
    manifest: &Value,
    closeout: &str,
    config: &ReleaseBindingConfig,
    label: &str,
) -> Result<()> {
    ensure!(
        validate_manifest(manifest, closeout, config).is_err(),
        "negative self-test unexpectedly allowed {label}"
    );
    Ok(())
}

fn parse_timestamp(value: &str) -> Result<OffsetDateTime> {
    ensure!(!value.is_empty(), "timestamp is empty");
    OffsetDateTime::parse(value, &Rfc3339)
        .with_context(|| format!("invalid RFC3339 timestamp: {value}"))
}

fn normalize(value: &str) -> String {
    value
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn sha256(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
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

fn string<'a>(value: &'a Map<String, Value>, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .with_context(|| format!("{key} must be a string"))
}

fn map<const N: usize>(entries: [(&str, Value); N]) -> Map<String, Value> {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_order_accepts_offsets() {
        assert!(timestamp_ge("2026-07-02T18:05:00+08:00", "2026-07-02T10:00:00Z").unwrap());
        assert!(!timestamp_ge("2026-07-02T09:59:59Z", "2026-07-02T10:00:00Z").unwrap());
    }

    #[test]
    fn body_hash_normalization_ignores_outer_and_trailing_space() {
        let directory = tempfile::tempdir().unwrap();
        let notes = directory.path().join("notes.md");
        fs::write(&notes, "# Release\nBody\n").unwrap();
        let report =
            release_body_hash_report(r#"{"body":"\n# Release  \nBody\n\n"}"#, &notes).unwrap();
        assert!(
            report.contains("release_body_normalized_sha256_matches_tracked_release_notes=true")
        );
        assert!(report.contains("release_body_raw_sha256_matches_tracked_release_notes=false"));
    }
}
