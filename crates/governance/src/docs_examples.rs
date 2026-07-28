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
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use percent_encoding::percent_decode_str;
use regex::Regex;

const PUBLIC_ROOTS: &[&str] = &[
    "docs/concepts",
    "docs/developer_guide",
    "docs/getting_started",
    "docs/governance",
    "docs/how_to",
    "docs/integrations",
    "docs/product",
    "docs/tutorials",
    "docs/rust-cutover/governance",
    "docs/rust-cutover/migration",
];
const INTEGRATIONS: &[(&str, &str)] = &[
    ("architect_ax", "architect_ax"),
    ("betfair", "betfair"),
    ("binance", "binance"),
    ("bitmex", "bitmex"),
    ("bybit", "bybit"),
    ("coinbase", "coinbase"),
    ("databento", "databento"),
    ("deribit", "deribit"),
    ("dydx", "dydx"),
    ("hyperliquid", "hyperliquid"),
    ("ib", "interactive_brokers"),
    ("kraken", "kraken"),
    ("okx", "okx"),
    ("polymarket", "polymarket"),
    ("tardis", "tardis"),
];
const CONCEPT_PAGES: &[&str] = &[
    "execution",
    "instruments",
    "live",
    "logging",
    "orders",
    "portfolio",
    "positions",
    "strategies",
    "synthetics",
];
const ROOT_MARKDOWN_SURFACE: &[&str] = &["AGENTS.md", "README.md"];

/// Counts emitted by the docs/examples governance guard.
pub struct DocsExamplesCounts {
    pub markdown_files: usize,
    pub local_links: usize,
    pub image_links: usize,
    pub integration_pages: usize,
    pub python_fences_classified: usize,
    pub concept_pages: usize,
    pub tutorial_assets: usize,
}

/// Validates local links, Rust authority markers, tutorial routes, and assets
/// across the retained public docs surface.
///
/// # Errors
///
/// Returns an error for a missing local target, missing Rust authority marker,
/// retired Python product route, or tutorial asset drift.
pub fn validate_docs_examples() -> Result<DocsExamplesCounts> {
    let root = env::current_dir().context("failed to resolve repository root")?;
    validate_root_documentation_surface(&root)?;
    ensure!(
        !root.join("docs/api_reference").exists(),
        "retired docs/api_reference exists"
    );
    ensure!(
        !root.join("docs/developer_guide/python.md").exists(),
        "retired Python developer guide exists"
    );
    ensure!(
        find_named(&root.join("docs"), ".DS_Store")?.is_none()
            && find_named(&root.join("examples"), ".DS_Store")?.is_none(),
        "Finder cache exists under docs/ or examples/"
    );

    let all_docs = glob_files(&root.join("docs/**/*.md"))?;
    let retired_hits: Vec<_> = all_docs
        .iter()
        .filter_map(|path| {
            let text = fs::read_to_string(path).ok()?;
            text.contains("/docs/python-api-latest/")
                .then(|| relative(&root, path))
        })
        .collect();
    ensure!(
        retired_hits.is_empty(),
        "retired Python API URL remains: {retired_hits:?}"
    );

    let mut markdown_files = Vec::new();
    for public_root in PUBLIC_ROOTS {
        markdown_files.extend(glob_files(&root.join(format!("{public_root}/**/*.md")))?);
    }
    markdown_files.sort();
    markdown_files.dedup();

    let markdown_link = Regex::new(r"(!?)\[[^\]]*\]\(([^)]+)\)")?;
    let mut local_links = 0;
    let mut image_links = 0;
    let mut missing = Vec::new();
    for markdown in &markdown_files {
        let text = fs::read_to_string(markdown)
            .with_context(|| format!("failed to read {}", markdown.display()))?;
        for captures in markdown_link.captures_iter(&text) {
            let is_image = &captures[1] == "!";
            let target = first_target(&captures[2]);
            if target.is_empty() || is_external(target, is_image) {
                continue;
            }
            let candidates = local_candidates(&root, markdown, target)?;
            if candidates.is_empty() {
                continue;
            }
            if is_image {
                image_links += 1;
            } else {
                local_links += 1;
            }
            if !candidates.iter().any(|path| path.exists()) {
                let kind = if is_image { "image " } else { "" };
                missing.push(format!("{}: {kind}{target}", relative(&root, markdown)));
            }
        }
    }
    if !missing.is_empty() {
        bail!(
            "missing local targets: {}\n{}",
            missing.len(),
            missing.join("\n")
        );
    }

    let mut python_fences = 0;
    for (page_name, crate_name) in INTEGRATIONS {
        let page = root.join(format!("docs/integrations/{page_name}.md"));
        let text = fs::read_to_string(&page)
            .with_context(|| format!("failed to read {}", page.display()))?;
        ensure!(
            first_lines(&text, 16).contains(":::warning[Rust-only authority]"),
            "missing integration authority: {}",
            relative(&root, &page)
        );
        ensure!(
            root.join(format!("crates/adapters/{crate_name}/Cargo.toml"))
                .is_file(),
            "missing Rust adapter crate: {crate_name}"
        );
        python_fences += text.matches("```python").count();
    }

    for page_name in CONCEPT_PAGES {
        let page = root.join(format!("docs/concepts/{page_name}.md"));
        let text = fs::read_to_string(&page)
            .with_context(|| format!("failed to read {}", page.display()))?;
        ensure!(
            first_lines(&text, 14).contains(":::warning[Rust-only authority]"),
            "missing concept authority: {}",
            relative(&root, &page)
        );
    }

    for section in ["docs/tutorials", "docs/how_to"] {
        for page in glob_files(&root.join(format!("{section}/**/*.md")))? {
            let text = fs::read_to_string(&page)
                .with_context(|| format!("failed to read {}", page.display()))?;
            ensure!(
                !text.contains("```python") && !text.contains("from nautilus_trader"),
                "Python product route remains: {}",
                relative(&root, &page)
            );
        }
    }

    let tutorial_root = root.join("docs/tutorials");
    let asset_files: BTreeSet<_> = glob_files(&tutorial_root.join("assets/**/*"))?
        .into_iter()
        .filter(|path| path.is_file())
        .map(|path| relative(&tutorial_root, &path))
        .collect();
    let asset_pattern = Regex::new(r"assets/[A-Za-z0-9_./-]+\.(?:png|jpg|jpeg|gif|svg)")?;
    let mut asset_refs = BTreeSet::new();
    for page in glob_files(&tutorial_root.join("**/*.md"))? {
        let text = fs::read_to_string(&page)
            .with_context(|| format!("failed to read {}", page.display()))?;
        asset_refs.extend(
            asset_pattern
                .find_iter(&text)
                .map(|matched| matched.as_str().to_string()),
        );
    }
    ensure!(
        asset_files == asset_refs,
        "tutorial asset drift: missing={:?} orphan={:?}",
        asset_refs.difference(&asset_files).collect::<Vec<_>>(),
        asset_files.difference(&asset_refs).collect::<Vec<_>>()
    );

    Ok(DocsExamplesCounts {
        markdown_files: markdown_files.len(),
        local_links,
        image_links,
        integration_pages: INTEGRATIONS.len(),
        python_fences_classified: python_fences,
        concept_pages: CONCEPT_PAGES.len(),
        tutorial_assets: asset_files.len(),
    })
}

fn validate_root_documentation_surface(root: &Path) -> Result<()> {
    let mut markdown = BTreeSet::new();
    for entry in fs::read_dir(root).context("failed to read repository root")? {
        let entry = entry.context("failed to read repository root entry")?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|extension| extension == "md") {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .context("invalid root Markdown filename")?;
            markdown.insert(name.to_string());
        }
    }
    let expected: BTreeSet<_> = ROOT_MARKDOWN_SURFACE
        .iter()
        .map(|name| (*name).to_string())
        .collect();
    ensure!(
        markdown == expected,
        "root Markdown surface drifted: expected {expected:?}, found {markdown:?}"
    );

    let project = fs::read_to_string(root.join("project.html"))
        .context("missing root project.html documentation entrypoint")?;
    let changelog = fs::read_to_string(root.join("changelog.html"))
        .context("missing root changelog.html documentation entrypoint")?;
    ensure!(
        project.contains("changelog.html"),
        "project.html must link changelog.html"
    );
    ensure!(
        changelog.contains("project.html"),
        "changelog.html must link project.html"
    );
    Ok(())
}

fn glob_files(pattern: &Path) -> Result<Vec<PathBuf>> {
    let pattern = pattern
        .to_str()
        .context("governance glob path is not valid UTF-8")?;
    glob::glob(pattern)
        .with_context(|| format!("invalid governance glob: {pattern}"))?
        .map(|entry| entry.with_context(|| format!("failed to expand governance glob: {pattern}")))
        .collect()
}

fn find_named(root: &Path, name: &str) -> Result<Option<PathBuf>> {
    if !root.exists() {
        return Ok(None);
    }
    for entry in fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))? {
        let path = entry?.path();
        if path.file_name().and_then(|value| value.to_str()) == Some(name) {
            return Ok(Some(path));
        }
        if path.is_dir()
            && let Some(found) = find_named(&path, name)?
        {
            return Ok(Some(found));
        }
    }
    Ok(None)
}

fn first_target(raw: &str) -> &str {
    let raw = raw.trim();
    if let Some(rest) = raw.strip_prefix('<')
        && let Some(end) = rest.find('>')
    {
        return &rest[..end];
    }
    raw.split_whitespace().next().unwrap_or("")
}

fn is_external(target: &str, image: bool) -> bool {
    target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with('#')
        || (!image && target.starts_with("mailto:"))
        || (image && target.starts_with("data:"))
}

fn local_candidates(root: &Path, markdown: &Path, target: &str) -> Result<Vec<PathBuf>> {
    let target = target.split('#').next().unwrap_or("");
    if target.is_empty() {
        return Ok(Vec::new());
    }
    let decoded = percent_decode_str(target)
        .decode_utf8()
        .with_context(|| format!("invalid percent-encoded local target: {target}"))?;
    let path = if decoded.starts_with('/') {
        root.join(decoded.trim_start_matches('/'))
    } else {
        markdown
            .parent()
            .context("markdown path has no parent")?
            .join(decoded.as_ref())
    };
    let mut candidates = vec![path.clone()];
    if path.extension().is_none() {
        candidates.push(path.with_extension("md"));
        candidates.push(path.join("index.md"));
    }
    Ok(candidates)
}

fn first_lines(text: &str, count: usize) -> String {
    text.lines().take(count).collect::<Vec<_>>().join("\n")
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
    fn parses_angle_and_plain_targets() {
        assert_eq!(
            first_target("<path with spaces.md> title"),
            "path with spaces.md"
        );
        assert_eq!(first_target("path.md title"), "path.md");
    }

    #[test]
    fn expands_extensionless_candidates() {
        let root = Path::new("/repo");
        let markdown = Path::new("/repo/docs/page.md");
        let candidates = local_candidates(root, markdown, "guide#section").unwrap();
        assert_eq!(candidates[0], Path::new("/repo/docs/guide"));
        assert_eq!(candidates[1], Path::new("/repo/docs/guide.md"));
        assert_eq!(candidates[2], Path::new("/repo/docs/guide/index.md"));
    }

    #[test]
    fn root_markdown_surface_is_explicit() {
        assert_eq!(ROOT_MARKDOWN_SURFACE, ["AGENTS.md", "README.md"]);
    }
}
