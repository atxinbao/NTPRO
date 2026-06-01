# RREM-014 Evidence - Remove Python Examples And Documentation Surfaces

Date: 2026-06-02
Executor: Codex
Task ID: RREM-014
Risk: critical
PR: pending

## Summary

Removed the legacy Python examples and Python documentation code surfaces from
the Rust-only cutover workspace:

- removed non-Rust example trees under `examples/`;
- retained `examples/rust/**` as the only supported top-level example surface;
- removed runnable Python documentation scripts under `docs/`;
- removed legacy tutorial panel renderer scripts;
- removed image assets that only belonged to deleted Python tutorial pages;
- updated public docs and integration docs that pointed at deleted Python
  example scripts;
- retargeted `tests/docs_tests/test_tutorials.py` to assert the removed Python
  docs scripts stay absent instead of executing deleted files.

This is a critical removal task. It stops at `REVIEW_REQUIRED`; auto-merge is
not enabled.

## Files Changed

Deleted:

- 184 tracked legacy Python example/documentation files and Python-only
  tutorial assets.
- Deleted example surfaces include `examples/backtest/**`, `examples/live/**`,
  `examples/sandbox/**`, `examples/other/**`, and `examples/utils/**`.
- Deleted documentation code surfaces include `docs/getting_started/*.py`,
  `docs/how_to/*.py`, `docs/tutorials/*.py`, and
  `docs/tutorials/assets/**/render_panels.py`.

Updated:

- `.agentflow/leases/RREM-014.json`
- `.agentflow/state/task_status.json`
- `README.md`
- `docs/concepts/greeks.md`
- `docs/concepts/message_bus.md`
- `docs/getting_started/index.md`
- `docs/how_to/index.md`
- `docs/integrations/*.md`
- `docs/tutorials/*.md`
- `examples/rust/README.md`
- `examples/rust/backtest/README.md`
- `pyproject.toml`
- `tests/docs_tests/test_tutorials.py`
- `docs/rust-cutover/tasks/RREM-014.md`
- `docs/rust-cutover/migration/python_examples_docs_removed.md`
- `docs/rust-cutover/evidence/RREM-014.md`

## Commands Run

```bash
git ls-files 'examples/**/*.py' 'docs/**/*.py' 'examples/**/*.ipynb' 'docs/**/*.ipynb'
git ls-files 'examples/**'
python3 -m json.tool .agentflow/state/task_status.json
python3 -m json.tool .agentflow/leases/RREM-014.json
python3 scripts/ai/validate_agentflow_roles.py
pytest tests/docs_tests/test_tutorials.py
uv run pytest tests/docs_tests/test_tutorials.py
python3 -m py_compile tests/docs_tests/test_tutorials.py
python3 - <<'PY'
from pathlib import Path
repo = Path.cwd()
removed = [
    repo / 'docs/getting_started/quickstart.py',
    repo / 'docs/getting_started/backtest_low_level.py',
    repo / 'docs/tutorials/backtest_fx_bars.py',
    repo / 'docs/getting_started/backtest_high_level.py',
    repo / 'docs/tutorials/backtest_orderbook_binance.py',
    repo / 'docs/tutorials/backtest_orderbook_bybit.py',
    repo / 'docs/how_to/loading_external_data.py',
    repo / 'docs/how_to/data_catalog_databento.py',
]
missing = [str(p.relative_to(repo)) for p in removed if p.exists()]
py_files = [str(p.relative_to(repo)) for p in (repo / 'docs').rglob('*.py')]
if missing or py_files:
    print('unexpected_existing=', missing)
    print('docs_py_files=', py_files)
    raise SystemExit(1)
print('OK removed doc Python surfaces are absent')
print('OK docs tree has no Python scripts')
PY
cargo metadata --format-version=1
cargo fmt --check
scripts/ai/verify_fast.sh
scripts/ai/check_rust_only_runtime.sh
git diff --check
python3 - <<'PY'
from pathlib import Path
from collections import defaultdict
root = Path.cwd()
files = [Path(line) for line in __import__('subprocess').check_output(['git','ls-files','*.py'], text=True).splitlines()]
by_top = defaultdict(lambda: [0,0])
for path in files:
    top = path.parts[0] if path.parts else '.'
    by_top[top][0] += 1
    by_top[top][1] += (root / path).stat().st_size
for top, (count, size) in sorted(by_top.items()):
    print(f'{top}\t{count}\t{size}')
print(f'TOTAL\t{len(files)}\t{sum((root / p).stat().st_size for p in files)}')
PY
git ls-files '*.[ch]' '*.cc' '*.cpp' '*.hpp' '*.hxx' '*.cxx'
git ls-files '*.capnp'
```

## Command Results

- `git ls-files 'examples/**/*.py' 'docs/**/*.py' 'examples/**/*.ipynb' 'docs/**/*.ipynb'`:
  passed with no output.
- `git ls-files 'examples/**'`: passed; only `examples/rust/**` remains.
- `python3 -m json.tool .agentflow/state/task_status.json`: passed.
- `python3 -m json.tool .agentflow/leases/RREM-014.json`: passed.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed.
- `pytest tests/docs_tests/test_tutorials.py`: not runnable because `pytest`
  is not installed on the local PATH.
- `uv run pytest tests/docs_tests/test_tutorials.py`: not runnable because the
  local `uv` is `0.11.12` while this repo requires `0.11.14`.
- `python3 -m py_compile tests/docs_tests/test_tutorials.py`: passed.
- Direct Python absence assertion for removed docs scripts and `docs/**/*.py`:
  passed.
- `cargo metadata --format-version=1`: passed.
- `cargo fmt --check`: passed.
- `scripts/ai/verify_fast.sh`: passed.
- `git diff --check`: passed.
- `scripts/ai/check_rust_only_runtime.sh`: failed as expected because active
  Rust crate paths still retain PyO3 annotations and Cython generation/parity
  references outside the RREM-014 examples/docs scope.

## Remaining Language Surface Inventory

Tracked Python after this task:

```text
scripts  6    30811 bytes
tests    534  9268417 bytes
TOTAL    540  9299228 bytes
```

Tracked C/C++/header files after this task:

```text
0
```

Tracked Cap'n Proto schemas after this task:

```text
crates/serialization/schemas/capnp/commands/data.capnp
crates/serialization/schemas/capnp/commands/trading.capnp
crates/serialization/schemas/capnp/common/base.capnp
crates/serialization/schemas/capnp/common/enums.capnp
crates/serialization/schemas/capnp/common/identifiers.capnp
crates/serialization/schemas/capnp/common/types.capnp
crates/serialization/schemas/capnp/data/market.capnp
crates/serialization/schemas/capnp/events/account.capnp
crates/serialization/schemas/capnp/events/order.capnp
crates/serialization/schemas/capnp/events/position.capnp
crates/serialization/schemas/capnp/events/system.capnp
```

Cap'n Proto remains in scope for Rust serialization and is not removed by this
task.

## Residual Release Blockers

- Python files remain under `tests/` and `scripts/`. Test cleanup and tool
  replacement are separate follow-up tasks.
- Active Rust crates still contain PyO3 annotations and Cython parity/build
  references. This continues to block `scripts/ai/check_rust_only_runtime.sh`.
- Product markdown still contains historical Python/PyO3/Cython guidance in
  broad conceptual, installation, developer, and integration docs. RREM-014
  removes Python example/doc code files and broken links to removed example
  scripts, but it does not rewrite the whole documentation set.
- One Rust example comment still mentions a deleted Python example path:
  `crates/adapters/hyperliquid/examples/node_outcome_exec_tester.rs`. Crate
  files are prohibited in this task and should be handled by a crate-scope
  cleanup task.

## Behavior Impact

Users can no longer run the removed Python example scripts or Python
documentation snippets from this workspace. The supported example surface is
now `examples/rust/**`.

No trading semantics, matching logic, risk rules, portfolio accounting, adapter
runtime behavior, persistence format, Cargo workspace structure, Cap'n Proto
schema, or Rust public API was intentionally changed.

## Public API Impact

Breaking documentation/example change: Python examples and runnable Python docs
snippets are removed from the workspace.

Rust crate and CLI APIs were not intentionally changed by this task.

## Migration Note Status

Added `docs/rust-cutover/migration/python_examples_docs_removed.md`.

## Rollback Plan

Revert this PR to restore the removed Python examples, documentation scripts,
and Python-only tutorial assets.
