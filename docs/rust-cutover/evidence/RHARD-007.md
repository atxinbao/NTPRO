# RHARD-007 Verification Cleanup Evidence

Date: 2026-06-04
Executor: Codex
Task ID: RHARD-007
Risk: low

## Scope

RHARD-007 documents which local verification command to use for each NTPRO
work type. It does not weaken release verification, make CI the only validation
source, or change trading behavior.

## Context Reviewed

- `docs/rust-cutover/tasks/RHARD-007.md`
- `docs/rust-cutover/verification/toolchain.md`
- `scripts/ai/verify_fast.sh`
- `scripts/ai/verify_full.sh`
- `scripts/ai/verify_release.sh`
- `scripts/ai/run_golden_traces.sh`
- `scripts/ai/check_rust_only_runtime.sh`
- `scripts/ai/check_cython_removed.sh`
- `scripts/ai/verify_cli_help.sh`

## Changes

- Added `docs/rust-cutover/verification/README.md`.
- Documented when to run:
  - `verify_fast.sh`;
  - `verify_full.sh`;
  - `verify_release.sh`;
  - `check_rust_only_runtime.sh`;
  - `check_cython_removed.sh`;
  - `run_golden_traces.sh`;
  - `verify_cli_help.sh`.
- Documented optional fast-mode Cargo check and clippy flags.
- Documented why release verification is expected to take substantially longer
  than fast checks.
- Documented PR evidence expectations by risk level.

## Commands Run

```bash
git diff --check
scripts/ai/verify_fast.sh
scripts/ai/validate_agentflow_roles.py
```

## Results

- `git diff --check`: passed.
- `scripts/ai/verify_fast.sh`: passed and printed the pinned Cargo/Rust
  `1.95.0` toolchain.
- `scripts/ai/validate_agentflow_roles.py`: passed.

## Behavior Impact

No trading behavior changed. This is documentation-only verification cleanup.

## Rollback Plan

Revert this PR to remove the verification guide and RHARD-007 evidence.
