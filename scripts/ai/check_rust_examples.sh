#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

fail() {
  echo "rust examples integrity failed: $*" >&2
  exit 1
}

required_paths=(
  examples/rust/README.md
  examples/rust/backtest/README.md
  examples/rust/backtest/minimal_dry_run.toml
  examples/rust/backtest/minimal_engine_smoke.toml
  examples/rust/binance/testnet_dry_run.toml
  examples/rust/config/README.md
  examples/rust/data/README.md
  examples/rust/data/catalog_audit.toml
  examples/rust/data/fixtures/quotes.csv
  examples/rust/data/load_quotes.toml
  examples/rust/live/README.md
  examples/rust/live/live_init_smoke.toml
  examples/rust/sandbox/README.md
  examples/rust/sandbox/sandbox_smoke.toml
)

for path in "${required_paths[@]}"; do
  [[ -f "$path" ]] || fail "missing canonical path: $path"
done

python3 <<'PY'
import re
import sys
import tomllib
from pathlib import Path

root = Path.cwd()
example_root = root / "examples" / "rust"

for path in sorted(example_root.rglob("*.toml")):
    try:
        with path.open("rb") as handle:
            tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise SystemExit(f"invalid example TOML {path.relative_to(root)}: {exc}")

references = set()
pattern = re.compile(r"examples/rust/[A-Za-z0-9_./-]*")
for markdown in sorted(example_root.rglob("*.md")):
    text = markdown.read_text(encoding="utf-8")
    for match in pattern.findall(text):
        references.add(match.rstrip("./"))

missing = sorted(ref for ref in references if not (root / ref).exists())
if missing:
    for ref in missing:
        print(f"missing README path: {ref}", file=sys.stderr)
    raise SystemExit(1)

print(
    "rust_examples_integrity=pass "
    f"required_paths=14 toml_files={len(list(example_root.rglob('*.toml')))} "
    f"readme_paths={len(references)}"
)
PY
