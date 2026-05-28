# RPROD-012 Evidence

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-012

## Summary

Added a Rust Cargo smoke for the live/sandbox product surface. The smoke builds
a `nautilus-live` `LiveNode` in `Sandbox` mode, verifies the initial lifecycle
state is owner-visible, and prints that no Python runtime or external venue
connection is required.

The Rust CLI `sandbox validate` and `sandbox run` command surfaces remain
blocked until later config-parser and runtime wiring tasks. This task records a
direct Cargo path for the current Rust sandbox node construction smoke instead
of adding a placeholder CLI implementation.

## Files Changed

- `crates/live/Cargo.toml`
- `crates/live/examples/sandbox_node_smoke.rs`
- `examples/rust/sandbox/README.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RPROD-012.json`
- `docs/rust-cutover/evidence/RPROD-012.md`

## Commands Run

```bash
python3 scripts/ai/lease.py claim RPROD-012 --force --branch ai/RPROD-012-add-rust-live-or-sandbox-example-smoke --agent-id Codex --path docs/rust-cutover/tasks/RPROD-012.md --path crates/live/Cargo.toml --path crates/live/examples/sandbox_node_smoke.rs --path examples/rust/sandbox/README.md --path docs/rust-cutover/evidence/RPROD-012.md --path .agentflow/state/task_status.json --path .agentflow/leases/RPROD-012.json
PATH="/opt/homebrew/opt/rustup/bin:$PATH" cargo run -p nautilus-live --no-default-features --features node --example sandbox-node-smoke
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_full.sh
```

## Command Results

- Lease claim succeeded for branch
  `ai/RPROD-012-add-rust-live-or-sandbox-example-smoke`.
- Added explicit `sandbox-node-smoke` example target requiring the `node`
  feature.
- Cargo built and ran the Rust sandbox node smoke successfully.
- Cargo executed `target/debug/examples/sandbox-node-smoke`.
- Sandbox node smoke evidence from the command output:
  - `node_name=SandboxNodeSmoke`;
  - `trader_id=SANDBOX-SMOKE-001`;
  - `environment=Sandbox`;
  - `state=Idle`;
  - `running=false`;
  - `python_required=false`;
  - `external_venue_connection=false`;
  - process exit code: `0`.
- Required full verification passed with `scripts/ai/verify_full.sh`.
- Full verification included fast checks, `cargo fmt --check`, targeted Rust
  checks, `cargo clippy -p nautilus-cli`, Rust test execution, golden trace
  schema validation, and Rust documentation generation.
- No verification failures were observed.

## Tests Added Or Updated

No Rust tests were added. The verification target for this task is a Cargo
example smoke run plus the required full verification command.

## Behavior Impact

- Adds a Rust-only sandbox node construction smoke under `nautilus-live`.
- Documents the current runnable Rust sandbox smoke under
  `examples/rust/sandbox/`.
- Does not connect to a production venue.
- Does not change trading semantics, adapter behavior, persistence, public Rust
  APIs, Python APIs, PyO3, or Cython paths.
- Keeps CLI execution blockers unchanged until the later CLI runtime wiring
  tasks can connect config files to Rust live/sandbox node models.

## Public API Impact

No public API change.

## Migration Note Status

No migration note required because this task adds an example-only Cargo smoke.

## Rollback Plan

Revert the commit to remove the Cargo example target, the sandbox README smoke
reference, and this evidence file. No runtime or generated data rollback is
required.
