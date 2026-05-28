# RPROD-013 Evidence

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-013

## Summary

Added Rust API entrypoint documentation for the current Rust product surface.
The document records the Cargo documentation command, generated doc roots,
CLI entrypoints, runnable Cargo smokes, and the primary Rust runtime crates
that owners should use before Python/PyO3/Cython removal gates.

## Files Changed

- `docs/rust-cutover/product/RUST_API_ENTRYPOINTS.md`
- `examples/rust/README.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RPROD-013.json`
- `docs/rust-cutover/evidence/RPROD-013.md`

## Commands Run

```bash
python3 scripts/ai/lease.py claim RPROD-013 --force --branch ai/RPROD-013-generate-or-validate-rust-documentation --agent-id Codex --path docs/rust-cutover/tasks/RPROD-013.md --path docs/rust-cutover/product/RUST_API_ENTRYPOINTS.md --path examples/rust/README.md --path docs/rust-cutover/evidence/RPROD-013.md --path .agentflow/state/task_status.json --path .agentflow/leases/RPROD-013.json
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo doc --workspace --exclude nautilus-pyo3 --features arrow,ffi,high-precision,streaming,defi --no-deps
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
```

## Command Results

- Lease claim succeeded for branch
  `ai/RPROD-013-generate-or-validate-rust-documentation`.
- Rust API entrypoint documentation was added under
  `docs/rust-cutover/product/`.
- Rust API docs generated successfully with `cargo doc`.
- Cargo reported generated docs under `target/doc/nautilus_analysis/index.html`
  and 41 other files.
- Required fast verification passed with `scripts/ai/verify_fast.sh`.
- Fast verification included toolchain discovery and `cargo fmt --check`.
- Optional fast cargo check and clippy remained intentionally skipped by the
  script defaults.

## Tests Added Or Updated

No Rust tests were added. This task validates documentation generation and the
required fast verification command.

## Behavior Impact

- Documents the current Rust product API entrypoints.
- Does not change runtime behavior, trading semantics, adapter behavior,
  persistence, public Rust APIs, Python APIs, PyO3, or Cython paths.
- Keeps CLI runtime execution blockers unchanged until later config-parser and
  runtime wiring tasks.

## Public API Impact

No public API change.

## Migration Note Status

No migration note required because this task adds documentation only.

## Rollback Plan

Revert the commit to remove the Rust API entrypoint documentation, examples
README link, and this evidence file. No runtime or generated data rollback is
required.
