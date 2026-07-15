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

use std::{fs, path::Path};

use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};

const RETIRED_TOOLS: &[&str] = &[
    "scripts/ai/lease.py",
    "scripts/ai/validate_agentflow_roles.py",
    "scripts/control/dispatch_next.py",
    "scripts/control/close_merged_pr.py",
    "scripts/ai/inventory_cython.py",
];
const ACTIVE_AUTHORITY_FILES: &[&str] = &[
    "Makefile",
    ".github/pull_request_template.md",
    "docs/rust-cutover/TASK_EXECUTION.md",
    "docs/rust-cutover/AGENT_ROLES.md",
];
const ACTIVE_AUTHORITY_DIRS: &[&str] = &[".github/workflows", ".github/actions", "scripts"];
const INVENTORY_PATH: &str = "docs/rust-cutover/inventory/cython_inventory.csv";
const INVENTORY_SHA256: &str = "e8a43c7cb7ccb9b029d979572968693a329931b20816bbdde4ee28b5162ab701";
const RETIREMENT_DOC: &str = "docs/rust-cutover/automation/PR_AUTODISPATCH.md";
const MIGRATION_DOC: &str =
    "docs/rust-cutover/migration/PTC-005-control-plane-tooling-retirement.md";

/// Counts emitted after legacy control-plane tooling retirement validation.
pub struct ControlPlaneRetirementCounts {
    pub retired_tools: usize,
    pub authority_files: usize,
    pub inventory_rows: usize,
}

/// Validates that legacy Python control tools are absent and their current
/// authority has moved to GitHub-native, auditable commands.
///
/// # Errors
///
/// Returns an error when a retired tool remains, an active caller still names
/// one, the current protocol still requires local AgentFlow state, or the
/// retained Cython inventory snapshot drifts.
pub fn validate_control_plane_retirement() -> Result<ControlPlaneRetirementCounts> {
    validate_tree(Path::new("."))
}

fn validate_tree(root: &Path) -> Result<ControlPlaneRetirementCounts> {
    for relative in RETIRED_TOOLS {
        ensure!(
            !root.join(relative).exists(),
            "retired control-plane tool still exists: {relative}"
        );
    }

    let mut authority_files = Vec::new();
    for relative in ACTIVE_AUTHORITY_FILES {
        let path = root.join(relative);
        ensure!(path.is_file(), "missing active authority file: {relative}");
        authority_files.push(path);
    }
    for relative in ACTIVE_AUTHORITY_DIRS {
        collect_files(&root.join(relative), &mut authority_files)?;
    }
    authority_files.sort();
    authority_files.dedup();

    for path in &authority_files {
        let bytes = fs::read(path)
            .with_context(|| format!("failed to read authority file: {}", path.display()))?;
        let text = String::from_utf8_lossy(&bytes);
        ensure_no_retired_references(path, &text)?;
    }

    for relative in [
        ".github/pull_request_template.md",
        "docs/rust-cutover/TASK_EXECUTION.md",
        "docs/rust-cutover/AGENT_ROLES.md",
    ] {
        let text = read(root, relative)?;
        for retired_state in [".agentflow", "Lease file:", "claim a lease"] {
            ensure!(
                !text.contains(retired_state),
                "current protocol still requires retired local state {retired_state}: {relative}"
            );
        }
    }

    let retirement = read(root, RETIREMENT_DOC)?;
    for marker in [
        "Status: RETIRED",
        "gh issue list",
        "git switch -c codex/",
        "gh pr view",
        "Closes #<ISSUE_NUMBER>",
        "No local queue or lease file is authoritative",
    ] {
        ensure!(
            retirement.contains(marker),
            "retirement document missing marker: {marker}"
        );
    }

    let migration = read(root, MIGRATION_DOC)?;
    for marker in [
        "GitHub is the only active control plane",
        "Historical evidence is retained",
        "cython_inventory.csv",
    ] {
        ensure!(
            migration.contains(marker),
            "migration document missing marker: {marker}"
        );
    }

    let inventory = fs::read(root.join(INVENTORY_PATH))
        .with_context(|| format!("failed to read retained inventory: {INVENTORY_PATH}"))?;
    ensure!(
        hex_sha256(&inventory) == INVENTORY_SHA256,
        "retained Cython inventory hash drifted"
    );
    let inventory_text = String::from_utf8(inventory).context("Cython inventory is not UTF-8")?;
    ensure!(
        inventory_text
            .starts_with("path,kind,lines,imports,has_cdef_class,has_cpdef,has_cimport\r\n"),
        "Cython inventory header drifted"
    );
    let inventory_rows = inventory_text.lines().count();
    ensure!(inventory_rows == 244, "Cython inventory row count drifted");

    Ok(ControlPlaneRetirementCounts {
        retired_tools: RETIRED_TOOLS.len(),
        authority_files: authority_files.len(),
        inventory_rows,
    })
}

fn collect_files(directory: &Path, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
    ensure!(
        directory.is_dir(),
        "missing authority directory: {}",
        directory.display()
    );
    for entry in fs::read_dir(directory)
        .with_context(|| format!("failed to read directory: {}", directory.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_files(&path, files)?;
        } else if file_type.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn read(root: &Path, relative: &str) -> Result<String> {
    fs::read_to_string(root.join(relative)).with_context(|| format!("failed to read {relative}"))
}

fn ensure_no_retired_references(path: &Path, text: &str) -> Result<()> {
    for retired in RETIRED_TOOLS {
        ensure!(
            !text.contains(retired),
            "active authority still references {retired}: {}",
            path.display()
        );
    }
    Ok(())
}

fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_retired_tool_reference() {
        let error = ensure_no_retired_references(
            Path::new("supported.sh"),
            "run scripts/ai/lease.py claim",
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("scripts/ai/lease.py"));
    }

    #[test]
    fn retained_inventory_identity_is_explicit() {
        assert_eq!(INVENTORY_SHA256.len(), 64);
        assert_eq!(RETIRED_TOOLS.len(), 5);
    }
}
