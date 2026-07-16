// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
// -------------------------------------------------------------------------------------------------

use std::{fs, path::Path, process::Command};

use anyhow::{Context, Result, bail, ensure};
use regex::Regex;

const FORBIDDEN_TRACKED_EXTENSIONS: &[&str] = &["py", "pyi", "pyx", "pxd", "ipynb", "pyc"];
const FORBIDDEN_TRACKED_NAMES: &[&str] = &[
    "pyproject.toml",
    "uv.lock",
    "Pipfile",
    "Pipfile.lock",
    "poetry.lock",
    "pdm.lock",
    "tox.ini",
    ".python-version",
];
const FORBIDDEN_ROOT_PATHS: &[&str] =
    &["pyproject.toml", "uv.lock", ".venv", "venv", "__pycache__"];
const RETAINED_HISTORICAL_AUTHORITY: &[&str] = &[
    "docs/rust-cutover/CONTRACT.md",
    "docs/rust-cutover/DEFINITION_OF_DONE.md",
    "docs/rust-cutover/TASK_EXECUTION.md",
    "docs/rust-cutover/AGENT_ROLES.md",
    "docs/rust-cutover/governance/python_tooling_baseline.json",
    "docs/rust-cutover/governance/python_tooling_authority_map.md",
    "docs/rust-cutover/governance/historical_release_executable_retirement.json",
    "docs/rust-cutover/inventory/cython_inventory.csv",
];
const REQUIRED_ACTIVE_AUTHORITY: &[&str] = &[
    "Makefile",
    ".github/workflows/rust-cutover-smoke.yml",
    ".github/workflows/security-audit.yml",
    "crates/governance/src/lib.rs",
    "crates/governance/src/main.rs",
    "crates/governance/src/zero_python.rs",
    "scripts/ai/check_zero_python_closeout.sh",
    "scripts/ai/verify_release.sh",
];
const MINIMUM_HISTORICAL_DOCS: usize = 1_925;
const GUARD_ENTRY: &str = "scripts/ai/check_zero_python_closeout.sh";

/// Counts emitted after repository-wide zero-Python validation.
pub struct ZeroPythonCloseoutCounts {
    pub tracked_files: usize,
    pub active_scripts: usize,
    pub workflow_actions: usize,
    pub historical_docs: usize,
    pub negative_cases: usize,
}

#[derive(Clone, Copy)]
enum ActiveSurface {
    Script,
    WorkflowAction,
}

/// Validates that active repository tooling no longer depends on Python.
///
/// Historical migration and release evidence under `docs/rust-cutover/` is
/// retained and intentionally not text-cleaned.
///
/// # Errors
///
/// Returns an error when tracked Python source or manifests remain, a local
/// Python environment/cache exists, supported scripts execute Python tooling,
/// workflow/action files retain Python or wheel execution, or required
/// historical authority is missing.
pub fn validate_zero_python_closeout(negative_selftest: bool) -> Result<ZeroPythonCloseoutCounts> {
    validate_tree(Path::new("."), negative_selftest)
}

fn validate_tree(root: &Path, negative_selftest: bool) -> Result<ZeroPythonCloseoutCounts> {
    let tracked = tracked_files(root)?;
    let mut violations = Vec::new();

    for relative in FORBIDDEN_ROOT_PATHS {
        if root.join(relative).exists() {
            violations.push(format!("forbidden local Python path exists: {relative}"));
        }
    }
    collect_generated_python_artifacts(root, root, &mut violations)?;

    let mut active_scripts = 0;
    let mut workflow_actions = 0;
    let mut historical_docs = 0;
    for relative in &tracked {
        if has_forbidden_extension(relative) {
            violations.push(format!("forbidden tracked Python file: {relative}"));
        }
        if has_forbidden_manifest(relative) {
            violations.push(format!("forbidden tracked Python manifest: {relative}"));
        }

        if relative.starts_with("docs/rust-cutover/") {
            historical_docs += 1;
        }

        let surface = active_surface(relative);
        if let Some(surface) = surface {
            match surface {
                ActiveSurface::Script => active_scripts += 1,
                ActiveSurface::WorkflowAction => workflow_actions += 1,
            }
            if matches!(surface, ActiveSurface::WorkflowAction)
                && has_forbidden_workflow_path(relative)
            {
                violations.push(format!(
                    "forbidden Python/wheel workflow or action path: {relative}"
                ));
            }
            let path = root.join(relative);
            let text = fs::read_to_string(&path)
                .with_context(|| format!("failed to read active surface: {relative}"))?;
            for finding in scan_active_text(surface, &text)? {
                violations.push(format!("{relative}: {finding}"));
            }
        }
    }

    for required in RETAINED_HISTORICAL_AUTHORITY {
        ensure!(
            tracked.iter().any(|path| path == required),
            "required historical authority is not tracked: {required}"
        );
    }
    for required in REQUIRED_ACTIVE_AUTHORITY {
        ensure!(
            tracked.iter().any(|path| path == required),
            "required zero-Python authority is not tracked: {required}"
        );
    }
    for caller in [
        "scripts/ai/verify_release.sh",
        ".github/workflows/rust-cutover-smoke.yml",
    ] {
        let text = fs::read_to_string(root.join(caller))
            .with_context(|| format!("failed to read zero-Python caller: {caller}"))?;
        ensure!(
            text.contains(GUARD_ENTRY),
            "zero-Python guard is not wired into required caller: {caller}"
        );
    }
    for task in 1..=7 {
        for class in ["tasks", "evidence"] {
            let relative = format!("docs/rust-cutover/{class}/PTC-{task:03}.md");
            ensure!(
                tracked.iter().any(|path| path == &relative),
                "required PTC history is not tracked: {relative}"
            );
        }
    }
    ensure!(
        historical_docs >= MINIMUM_HISTORICAL_DOCS,
        "historical docs/rust-cutover authority count decreased: {historical_docs} < {MINIMUM_HISTORICAL_DOCS}"
    );

    if !violations.is_empty() {
        bail!(
            "zero-Python closeout violations:\n{}",
            violations.join("\n")
        );
    }

    let negative_cases = if negative_selftest {
        run_negative_selftests()?
    } else {
        0
    };

    Ok(ZeroPythonCloseoutCounts {
        tracked_files: tracked.len(),
        active_scripts,
        workflow_actions,
        historical_docs,
        negative_cases,
    })
}

fn tracked_files(root: &Path) -> Result<Vec<String>> {
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
        .filter(|path| root.join(path).is_file())
        .map(ToOwned::to_owned)
        .collect())
}

fn has_forbidden_extension(relative: &str) -> bool {
    Path::new(relative)
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| {
            FORBIDDEN_TRACKED_EXTENSIONS
                .iter()
                .any(|extension| value.eq_ignore_ascii_case(extension))
        })
}

fn has_forbidden_manifest(relative: &str) -> bool {
    let Some(name) = Path::new(relative)
        .file_name()
        .and_then(|value| value.to_str())
    else {
        return false;
    };
    let lower = name.to_ascii_lowercase();
    FORBIDDEN_TRACKED_NAMES
        .iter()
        .any(|forbidden| lower == forbidden.to_ascii_lowercase())
        || (lower.starts_with("requirements") && lower.ends_with(".txt"))
}

fn active_surface(relative: &str) -> Option<ActiveSurface> {
    let path = Path::new(relative);
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);
    if relative == "Makefile"
        || (relative.starts_with(".pre-commit-config.")
            && matches!(extension.as_deref(), Some("yml" | "yaml")))
        || (relative.starts_with("scripts/")
            && matches!(extension.as_deref(), Some("sh" | "bash" | "zsh" | "fish")))
    {
        return Some(ActiveSurface::Script);
    }
    if (relative.starts_with(".github/workflows/") || relative.starts_with(".github/actions/"))
        && matches!(extension.as_deref(), Some("yml" | "yaml"))
    {
        return Some(ActiveSurface::WorkflowAction);
    }
    None
}

fn has_forbidden_workflow_path(relative: &str) -> bool {
    let lower = relative.to_ascii_lowercase();
    ["python", "wheel", "pypi", "maturin"] // zero-python-pattern-definition
        .iter()
        .any(|marker| lower.contains(marker))
}

fn scan_active_text(surface: ActiveSurface, text: &str) -> Result<Vec<String>> {
    let command = Regex::new(
        r"(?ix)(?:^|[;&|({!]\s*|\$\(\s*|run:\s*)\s*[@+-]?(?:(?:(?:if|then|elif|while|until|do|exec|command|sudo|xargs|time|nohup)\s+)|(?:timeout\s+\S+\s+)|(?:nice(?:\s+-\S+)*\s+)|(?:(?:\S*/)?env(?:\s+-\S+)*\s+))*(?:[A-Za-z_][A-Za-z0-9_]*=\S+\s+)*(?:\S*/)?(?:python(?:\d+(?:\.\d+)*)?|py|uvx?|pytest|ruff|pip(?:3|x|-audit)?)(?:\s|$)",
    )?;
    let python_shebang = Regex::new(r"(?i)^#!\s*(?:\S*/)?(?:env\s+)?python(?:\d+(?:\.\d+)*)?\s*$")?;
    let workflow = Regex::new(
        r"(?ix)(?:uses:\s*\S*(?:setup-python|setup-uv|wheel|maturin)|python-version\s*:|shell:\s*(?:\S*/)?python(?:\d+(?:\.\d+)*)?|(?:cibuildwheel|publish-wheels|common-wheel-build|upload-artifact-wheel))", // zero-python-pattern-definition
    )?;
    let mut findings = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if python_shebang.is_match(trimmed) {
            findings.push(format!("line {} uses a Python shebang", index + 1));
            continue;
        }
        if trimmed.starts_with('#') {
            continue;
        }
        if command.is_match(line) {
            findings.push(format!("line {} executes Python tooling", index + 1));
        }
        if matches!(surface, ActiveSurface::WorkflowAction) && workflow.is_match(line) {
            findings.push(format!(
                "line {} retains Python/wheel workflow tooling",
                index + 1
            ));
        }
    }
    Ok(findings)
}

fn collect_generated_python_artifacts(
    root: &Path,
    directory: &Path,
    violations: &mut Vec<String>,
) -> Result<()> {
    for entry in fs::read_dir(directory)
        .with_context(|| format!("failed to inspect directory: {}", directory.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if entry.file_type()?.is_dir() {
            if matches!(name.as_ref(), ".git" | "target" | "target-v2") {
                continue;
            }
            if matches!(name.as_ref(), ".venv" | "venv" | "__pycache__") {
                let relative = path.strip_prefix(root).unwrap_or(&path);
                violations.push(format!(
                    "forbidden local Python directory exists: {}",
                    relative.display()
                ));
                continue;
            }
            collect_generated_python_artifacts(root, &path, violations)?;
        } else if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("pyc"))
        {
            let relative = path.strip_prefix(root).unwrap_or(&path);
            violations.push(format!(
                "forbidden local Python bytecode exists: {}",
                relative.display()
            ));
        }
    }
    Ok(())
}

fn run_negative_selftests() -> Result<usize> {
    let cases = [
        (ActiveSurface::Script, "python3 tool.py\n"),
        (
            ActiveSurface::Script,
            "if env MODE=test .venv/bin/python script.py; then :; fi\n",
        ),
        (ActiveSurface::Script, "uv run pytest tests\n"),
        (
            ActiveSurface::WorkflowAction,
            "steps:\n  - uses: actions/setup-python@deadbeef\n",
        ),
        (
            ActiveSurface::WorkflowAction,
            "steps:\n  - uses: ./.github/actions/common-wheel-build\n",
        ),
        (ActiveSurface::Script, "@python3 forbidden.py\n"),
        (
            ActiveSurface::Script,
            "timeout 5s /usr/bin/env python3 forbidden.py\n",
        ),
        (ActiveSurface::Script, "#!/usr/bin/python3\n"),
        (ActiveSurface::WorkflowAction, "shell: python3\n"),
    ];
    for (surface, fixture) in cases {
        ensure!(
            !scan_active_text(surface, fixture)?.is_empty(),
            "negative selftest accepted Python tooling fixture: {fixture:?}"
        );
    }
    ensure!(
        scan_active_text(
            ActiveSurface::Script,
            "grep -E 'python|pytest' retained-history.txt\necho historical Python evidence retained\n"
        )?
        .is_empty(),
        "negative selftest rejected a non-executable historical mention"
    );
    Ok(cases.len() + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_tracked_python_extensions_case_insensitively() {
        assert!(has_forbidden_extension("scripts/tool.py"));
        assert!(has_forbidden_extension("notebook.IPYNB"));
        assert!(!has_forbidden_extension("docs/history.md"));
    }

    #[test]
    fn rejects_nested_python_manifests() {
        assert!(has_forbidden_manifest("tools/pyproject.toml"));
        assert!(has_forbidden_manifest("tools/PYPROJECT.TOML"));
        assert!(has_forbidden_manifest("helper/requirements-dev.txt"));
        assert!(has_forbidden_manifest("Pipfile.lock"));
        assert!(!has_forbidden_manifest("Cargo.lock"));
    }

    #[test]
    fn classifies_only_supported_execution_surfaces() {
        assert!(matches!(
            active_surface("scripts/verify.sh"),
            Some(ActiveSurface::Script)
        ));
        assert!(matches!(
            active_surface("scripts/verify.ZSH"),
            Some(ActiveSurface::Script)
        ));
        assert!(matches!(
            active_surface(".github/workflows/build.yml"),
            Some(ActiveSurface::WorkflowAction)
        ));
        assert!(active_surface("docs/rust-cutover/evidence/PTC-001.md").is_none());
    }

    #[test]
    fn rejects_python_and_wheel_workflow_paths() {
        assert!(has_forbidden_workflow_path(
            ".github/actions/common-wheel-build/action.yml"
        ));
        assert!(has_forbidden_workflow_path(
            ".github/workflows/publish-pypi.yml"
        ));
        assert!(!has_forbidden_workflow_path(".github/workflows/build.yml"));
    }

    #[test]
    fn negative_execution_matrix_is_fail_closed() {
        assert_eq!(run_negative_selftests().unwrap(), 10);
    }
}
