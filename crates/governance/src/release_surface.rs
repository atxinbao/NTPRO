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
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail, ensure};
use regex::Regex;

/// Inputs for the current release-surface guard.
pub struct ReleaseSurfaceConfig {
    pub current_version: String,
    pub current_tag: String,
    pub governance_track: String,
    pub next_capability: String,
    pub current_capability: String,
    pub allow_missing_tag: bool,
}

/// Validates current release wording, release document identity, and stale
/// current-release references.
///
/// # Errors
///
/// Returns an error when a required file, tag, marker, or current-release
/// statement is missing or stale.
pub fn validate_release_surface(config: &ReleaseSurfaceConfig) -> Result<()> {
    let stem = format!("v{}", config.current_version.trim_start_matches('v')).replace('.', "_");
    let notes = format!("docs/rust-cutover/release/{stem}_release_notes.md");
    let readiness = format!("docs/rust-cutover/release/{stem}_readiness_report.md");
    let required = [
        "README.md",
        "docs/product/roadmap.md",
        "docs/versioning.md",
        "docs/rust-cutover/release/README.md",
        notes.as_str(),
        readiness.as_str(),
    ];
    for path in required {
        ensure!(Path::new(path).is_file(), "missing required file: {path}");
    }

    if !git_tag_exists(&config.current_tag)? {
        if config.allow_missing_tag {
            println!(
                "release_surface_current_guard=pre_tag_mode missing_tag={}",
                config.current_tag
            );
        } else {
            bail!("missing local git tag: {}", config.current_tag);
        }
    }

    let readme = read("README.md")?;
    let roadmap = read("docs/product/roadmap.md")?;
    let versioning = read("docs/versioning.md")?;
    let release_index = read("docs/rust-cutover/release/README.md")?;
    let notes_text = read(&notes)?;
    let readiness_text = read(&readiness)?;

    require_contains(
        &readme,
        &format!("Current source tag: {}", config.current_tag),
        "README current source tag",
    )?;
    require_contains(
        &readme,
        &format!(
            "https://github.com/atxinbao/NTPRO/releases/tag/{}",
            config.current_tag
        ),
        "README current GitHub Release URL",
    )?;
    require_contains(
        &readme,
        "No backend patch is scheduled.",
        "README backend patch status",
    )?;
    require_contains(
        &readme,
        &format!("`{}`", config.governance_track),
        "README post-baseline governance track",
    )?;
    require_contains(
        &readme,
        &format!("The next capability family is `{}`", config.next_capability),
        "README next capability family",
    )?;

    require_contains(
        &roadmap,
        &format!(
            "`{}`, the {} release",
            config.current_tag, config.current_capability
        ),
        "ROADMAP current release and patch track",
    )?;
    require_contains(
        &roadmap,
        "No backend patch is scheduled.",
        "ROADMAP backend patch status",
    )?;
    require_contains(
        &roadmap,
        &format!("`{}`", config.governance_track),
        "ROADMAP post-baseline governance track",
    )?;
    require_contains(
        &roadmap,
        &format!("## Published Capability Track: {}", config.current_version),
        "ROADMAP published capability track",
    )?;
    require_contains(
        &roadmap,
        &format!("The next capability family is `{}`", config.next_capability),
        "ROADMAP next capability family",
    )?;

    require_contains(
        &versioning,
        &format!("`{}` 是当前正式公开发布点", config.current_version),
        "versioning current release statement",
    )?;
    require_contains(
        &versioning,
        &config.current_tag,
        "versioning current release tag",
    )?;
    require_contains(
        &versioning,
        "none scheduled; baseline-invalidity exception only",
        "versioning backend patch status",
    )?;
    require_contains(
        &versioning,
        &config.governance_track,
        "versioning post-baseline governance track",
    )?;
    require_contains(
        &versioning,
        &config.next_capability,
        "versioning next capability family",
    )?;

    let readiness_name = Path::new(&readiness)
        .file_name()
        .and_then(|name| name.to_str())
        .context("invalid readiness report path")?;
    let readiness_released =
        format!("`{readiness_name}` - released readiness report for the formal");
    let readiness_ready =
        format!("`{readiness_name}` - release gate readiness report for the formal");
    ensure!(
        release_index.contains(&readiness_released) || release_index.contains(&readiness_ready),
        "missing release index current readiness report"
    );
    let notes_name = Path::new(&notes)
        .file_name()
        .and_then(|name| name.to_str())
        .context("invalid release notes path")?;
    require_contains(
        &release_index,
        &format!("`{notes_name}` - release notes for the formal"),
        "release index current release notes",
    )?;
    require_contains(
        &release_index,
        &format!("`{}` GitHub Release", config.current_tag),
        "release index current tag",
    )?;

    ensure!(
        notes_text.contains("Status: RELEASE GATE READY")
            || notes_text.contains("Status: RELEASED"),
        "missing current release notes status"
    );
    require_contains(
        &notes_text,
        &format!("Tag: `{}`", config.current_tag),
        "release notes tag",
    )?;
    require_contains(
        &notes_text,
        &format!("Release name: `NTPRO Rust-only {}`", config.current_version),
        "release notes release name",
    )?;
    require_contains(
        &readiness_text,
        &format!("Milestone: `{}`", config.current_tag),
        "readiness report milestone",
    )?;
    ensure!(
        [
            "Status: PASS",
            "Status: RELEASED",
            "Status: RELEASE GATE READY"
        ]
        .iter()
        .any(|marker| readiness_text.contains(marker)),
        "missing current readiness report release status"
    );

    reject_stale_current_release_wording(
        &config.current_version,
        &["README.md", "docs/product/roadmap.md", "docs/versioning.md"],
    )
}

fn read(path: &str) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read {path}"))
}

fn require_contains(text: &str, needle: &str, description: &str) -> Result<()> {
    ensure!(text.contains(needle), "missing {description}: {needle}");
    Ok(())
}

fn git_tag_exists(tag: &str) -> Result<bool> {
    let status = Command::new("git")
        .args(["rev-parse", "-q", "--verify", &format!("{tag}^{{commit}}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("failed to execute git rev-parse")?;
    Ok(status.success())
}

fn reject_stale_current_release_wording(current_version: &str, files: &[&str]) -> Result<()> {
    let release = Regex::new(r"(?:ntpro-rust-only-)?v(\d+)\.(\d+)\.(\d+)")?;
    let current = parse_version(&release, current_version)?;
    let context = Regex::new(
        "(?i)current|当前|当前正式公开发布点|current public|current source|current published",
    )?;
    let multiline = Regex::new(
        "(?i)current public milestone is|current published release line is|current release line is",
    )?;
    let mut errors = Vec::new();
    for path in files {
        let text = read(path)?;
        let mut pending_context = 0_u8;
        for (index, line) in text.lines().enumerate() {
            if multiline.is_match(line) {
                pending_context = 4;
            }
            if context.is_match(line) || pending_context > 0 {
                for captures in release.captures_iter(line) {
                    let version = capture_version(&captures)?;
                    if version < current {
                        errors.push(format!(
                            "{path}:{}: stale current release wording -> {line}",
                            index + 1
                        ));
                    }
                }
            }
            pending_context = pending_context.saturating_sub(1);
        }
    }
    if !errors.is_empty() {
        bail!(errors.join("\n"));
    }
    Ok(())
}

fn parse_version(regex: &Regex, value: &str) -> Result<(u64, u64, u64)> {
    let captures = regex
        .captures(value)
        .with_context(|| format!("invalid release version: {value}"))?;
    capture_version(&captures)
}

fn capture_version(captures: &regex::Captures<'_>) -> Result<(u64, u64, u64)> {
    Ok((
        captures[1].parse()?,
        captures[2].parse()?,
        captures[3].parse()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_current_wording_is_rejected() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("README.md");
        fs::write(&path, "Current published release line is\nv0.31.0\n").unwrap();

        let error = reject_stale_current_release_wording("v0.32.0", &[path.to_str().unwrap()])
            .unwrap_err()
            .to_string();
        assert!(error.contains("stale current release wording"));
    }

    #[test]
    fn historical_wording_without_current_context_is_allowed() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("history.md");
        fs::write(&path, "Historical release v0.31.0 remains auditable.\n").unwrap();

        reject_stale_current_release_wording("v0.32.0", &[path.to_str().unwrap()]).unwrap();
    }
}
