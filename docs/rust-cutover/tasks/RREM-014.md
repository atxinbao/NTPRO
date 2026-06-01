# RREM-014 - Remove Python examples and documentation surfaces

Date: 2026-06-02
Executor: Codex

## Goal

Remove remaining Python example and documentation code surfaces from the
Rust-only product path after the Python package surface removal.

## Owner

- owner_role: rust_product_surface_agent
- review_role: verification_release_gatekeeper
- risk_level: critical

## Scope

Allowed paths:

- `examples/**`
- `examples/rust/**`
- `docs/**/*.py`
- `docs/tutorials/assets/backtest_fx_bars/**`
- `docs/tutorials/assets/backtest_orderbook_binance/**`
- `docs/tutorials/assets/backtest_orderbook_bybit/**`
- `README.md`
- `docs/**/*.md`
- `examples/**/*.md`
- `tests/docs_tests/test_tutorials.py`
- `pyproject.toml`
- `docs/rust-cutover/tasks/RREM-014.md`
- `docs/rust-cutover/evidence/RREM-014.md`
- `docs/rust-cutover/migration/python_examples_docs_removed.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREM-014.json`

Prohibited paths:

- `tests/**` except `tests/docs_tests/test_tutorials.py`
- `crates/**`
- `Cargo.toml`
- `Cargo.lock`
- `crates/serialization/schemas/capnp/**`
- `crates/serialization/generated/capnp/**`

## Acceptance

- Python files under `examples/` are removed.
- Non-Rust legacy example trees under `examples/` are removed.
- Rust examples under `examples/rust/` remain available and documented.
- Python files under `docs/` are removed.
- Python tutorial-only assets for the removed Python tutorials are removed.
- Public docs no longer instruct users to run removed Python examples as the
  supported product path.
- Python tests are not deleted by this task.
- The docs tutorial test is retargeted to assert the removed Python docs
  scripts stay absent instead of executing deleted files.
- Cap'n Proto schemas and generated Rust code are not changed by this task.
- Remaining Python code is recorded as test/tooling residue or later cleanup.
- Work stops at `REVIEW_REQUIRED`; auto-merge is not enabled.

## Required evidence

Run or record why not run:

```bash
git ls-files 'examples/**/*.py' 'docs/**/*.py' 'examples/**/*.ipynb' 'docs/**/*.ipynb'
git ls-files 'examples/**' | sed -n '1,40p'
python3 -m json.tool .agentflow/state/task_status.json
python3 -m json.tool .agentflow/leases/RREM-014.json
python3 scripts/ai/validate_agentflow_roles.py
pytest tests/docs_tests/test_tutorials.py
cargo metadata --format-version=1
cargo fmt --check
scripts/ai/verify_fast.sh
scripts/ai/check_rust_only_runtime.sh
git diff --check
```

## Notes

This task intentionally does not remove Python tests. Python test cleanup and
Rust crate PyO3 annotation cleanup remain separate removal work.
