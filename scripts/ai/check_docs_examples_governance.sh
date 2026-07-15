#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

fail() {
  echo "docs/examples governance failed: $*" >&2
  exit 1
}

[[ ! -e docs/api_reference ]] || fail "retired docs/api_reference exists"
[[ ! -e docs/developer_guide/python.md ]] || fail "retired Python developer guide exists"

if find docs examples -name .DS_Store -print -quit | grep -q .; then
  fail "Finder cache exists under docs/ or examples/"
fi

python3 <<'PY'
import re
import sys
from pathlib import Path
from urllib.parse import unquote

root = Path.cwd()
docs_root = root / "docs"
public_roots = [
    docs_root / "concepts",
    docs_root / "developer_guide",
    docs_root / "getting_started",
    docs_root / "how_to",
    docs_root / "integrations",
    docs_root / "tutorials",
    docs_root / "rust-cutover" / "governance",
    docs_root / "rust-cutover" / "migration",
]
markdown_files = sorted(
    path for public_root in public_roots for path in public_root.rglob("*.md")
)

retired_url = "/docs/python-api" + "-latest/"
retired_hits = []
for path in docs_root.rglob("*.md"):
    if retired_url in path.read_text(encoding="utf-8"):
        retired_hits.append(path.relative_to(root).as_posix())
if retired_hits:
    raise SystemExit(f"retired Python API URL remains: {retired_hits}")

link_pattern = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")
image_pattern = re.compile(r"!\[[^\]]*\]\(([^)]+)\)")
missing = []
local_link_count = 0
image_link_count = 0

def first_target(raw):
    raw = raw.strip()
    if raw.startswith("<") and ">" in raw:
        return raw[1 : raw.index(">")]
    return raw.split()[0] if raw else ""

def candidates(markdown, target):
    target = unquote(target.split("#", 1)[0])
    if not target:
        return []
    path = root / target.lstrip("/") if target.startswith("/") else markdown.parent / target
    result = [path]
    if not path.suffix:
        result.extend([Path(f"{path}.md"), path / "index.md"])
    return result

for markdown in markdown_files:
    text = markdown.read_text(encoding="utf-8")
    for raw in link_pattern.findall(text):
        target = first_target(raw)
        if target.startswith(("http://", "https://", "mailto:", "#")):
            continue
        paths = candidates(markdown, target)
        if not paths:
            continue
        local_link_count += 1
        if not any(path.exists() for path in paths):
            missing.append(f"{markdown.relative_to(root)}: {target}")
    for raw in image_pattern.findall(text):
        target = first_target(raw)
        if target.startswith(("http://", "https://", "data:")):
            continue
        paths = candidates(markdown, target)
        if not paths:
            continue
        image_link_count += 1
        if not any(path.exists() for path in paths):
            missing.append(f"{markdown.relative_to(root)}: image {target}")

if missing:
    print("\n".join(missing), file=sys.stderr)
    raise SystemExit(f"missing local targets: {len(missing)}")

integration_map = {
    "architect_ax": "architect_ax",
    "betfair": "betfair",
    "binance": "binance",
    "bitmex": "bitmex",
    "bybit": "bybit",
    "coinbase": "coinbase",
    "databento": "databento",
    "deribit": "deribit",
    "dydx": "dydx",
    "hyperliquid": "hyperliquid",
    "ib": "interactive_brokers",
    "kraken": "kraken",
    "okx": "okx",
    "polymarket": "polymarket",
    "tardis": "tardis",
}
python_fences = 0
for page_name, crate_name in integration_map.items():
    page = docs_root / "integrations" / f"{page_name}.md"
    text = page.read_text(encoding="utf-8")
    if ":::warning[Rust-only authority]" not in "\n".join(text.splitlines()[:16]):
        raise SystemExit(f"missing integration authority: {page.relative_to(root)}")
    if not (root / "crates" / "adapters" / crate_name / "Cargo.toml").is_file():
        raise SystemExit(f"missing Rust adapter crate: {crate_name}")
    python_fences += text.count("```python")

concept_pages = [
    "execution",
    "instruments",
    "live",
    "logging",
    "orders",
    "portfolio",
    "positions",
    "strategies",
    "synthetics",
]
for page_name in concept_pages:
    page = docs_root / "concepts" / f"{page_name}.md"
    text = page.read_text(encoding="utf-8")
    if ":::warning[Rust-only authority]" not in "\n".join(text.splitlines()[:14]):
        raise SystemExit(f"missing concept authority: {page.relative_to(root)}")

for section in (docs_root / "tutorials", docs_root / "how_to"):
    for page in section.rglob("*.md"):
        text = page.read_text(encoding="utf-8")
        if "```python" in text or "from nautilus_trader" in text:
            raise SystemExit(f"Python product route remains: {page.relative_to(root)}")

tutorial_root = docs_root / "tutorials"
asset_files = {
    path.relative_to(tutorial_root).as_posix()
    for path in (tutorial_root / "assets").rglob("*")
    if path.is_file()
}
asset_refs = set()
asset_pattern = re.compile(r"assets/[A-Za-z0-9_./-]+\.(?:png|jpg|jpeg|gif|svg)")
for page in tutorial_root.rglob("*.md"):
    asset_refs.update(asset_pattern.findall(page.read_text(encoding="utf-8")))
if asset_files != asset_refs:
    raise SystemExit(
        f"tutorial asset drift: missing={sorted(asset_refs - asset_files)} "
        f"orphan={sorted(asset_files - asset_refs)}"
    )

print(
    "docs_examples_governance=pass "
    f"markdown_files={len(markdown_files)} local_links={local_link_count} "
    f"image_links={image_link_count} integration_pages={len(integration_map)} "
    f"python_fences_classified={python_fences} concept_pages={len(concept_pages)} "
    f"tutorial_assets={len(asset_files)}"
)
PY

scripts/ai/check_rust_examples.sh
scripts/ai/check_backend_freeze_baseline.sh
