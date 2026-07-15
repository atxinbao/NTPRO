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

use std::{collections::BTreeSet, env, fs, path::Path};

use anyhow::{Context, Result, ensure};
use regex::Regex;

const REQUIRED_PATHS: &[&str] = &[
    "examples/rust/README.md",
    "examples/rust/backtest/README.md",
    "examples/rust/backtest/minimal_dry_run.toml",
    "examples/rust/backtest/minimal_engine_smoke.toml",
    "examples/rust/binance/testnet_dry_run.toml",
    "examples/rust/config/README.md",
    "examples/rust/data/README.md",
    "examples/rust/data/catalog_audit.toml",
    "examples/rust/data/fixtures/quotes.csv",
    "examples/rust/data/load_quotes.toml",
    "examples/rust/live/README.md",
    "examples/rust/live/live_init_smoke.toml",
    "examples/rust/sandbox/README.md",
    "examples/rust/sandbox/sandbox_smoke.toml",
];

/// Counts emitted by the Rust examples integrity guard.
pub struct RustExamplesCounts {
    pub required_paths: usize,
    pub toml_files: usize,
    pub readme_paths: usize,
}

/// Validates canonical Rust examples, all example TOML documents, and local
/// example paths referenced by Markdown.
///
/// # Errors
///
/// Returns an error when a required path or Markdown reference is missing, or
/// an example TOML document is invalid.
pub fn validate_rust_examples() -> Result<RustExamplesCounts> {
    let root = env::current_dir().context("failed to resolve repository root")?;
    for path in REQUIRED_PATHS {
        ensure!(root.join(path).is_file(), "missing canonical path: {path}");
    }
    let example_root = root.join("examples/rust");
    let toml_files = glob_files(&example_root.join("**/*.toml"))?;
    for path in &toml_files {
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        text.parse::<toml::Table>()
            .with_context(|| format!("invalid example TOML {}", relative(&root, path)))?;
    }

    let pattern = Regex::new("examples/rust/[A-Za-z0-9_./-]*")?;
    let mut references = BTreeSet::new();
    for markdown in glob_files(&example_root.join("**/*.md"))? {
        let text = fs::read_to_string(&markdown)
            .with_context(|| format!("failed to read {}", markdown.display()))?;
        references.extend(
            pattern
                .find_iter(&text)
                .map(|matched| matched.as_str().trim_end_matches(['.', '/']).to_string()),
        );
    }
    let missing: Vec<_> = references
        .iter()
        .filter(|reference| !root.join(reference).exists())
        .cloned()
        .collect();
    ensure!(missing.is_empty(), "missing README paths: {missing:?}");
    Ok(RustExamplesCounts {
        required_paths: REQUIRED_PATHS.len(),
        toml_files: toml_files.len(),
        readme_paths: references.len(),
    })
}

fn glob_files(pattern: &Path) -> Result<Vec<std::path::PathBuf>> {
    let pattern = pattern
        .to_str()
        .context("example glob path is not valid UTF-8")?;
    glob::glob(pattern)
        .with_context(|| format!("invalid example glob: {pattern}"))?
        .map(|entry| entry.with_context(|| format!("failed to expand example glob: {pattern}")))
        .collect()
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
    fn reference_suffixes_are_trimmed() {
        let pattern = Regex::new("examples/rust/[A-Za-z0-9_./-]*").unwrap();
        let matched = pattern.find("See examples/rust/live/README.md.").unwrap();
        assert_eq!(
            matched.as_str().trim_end_matches(['.', '/']),
            "examples/rust/live/README.md"
        );
    }
}
