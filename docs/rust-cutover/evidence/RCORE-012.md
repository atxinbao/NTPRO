# RCORE-012 Evidence - Close Rust Execution/Risk/Order Lifecycle Gaps

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-012
Risk: high

## Summary

Closed the RCORE-010/RCORE-011 execution/risk/order lifecycle inventory loop by
turning each remaining EROL gap into an explicit closeout decision.

Implemented:

- Added an RCORE-012 closeout matrix to
  `docs/rust-cutover/inventory/execution_risk_order_lifecycle.md`.
- Marked EROL-002 and EROL-003 as locally closed only for focused Rust
  RiskEngine accept/deny and queue-routing assertions added by RCORE-011.
- Marked EROL-001, EROL-005, EROL-006, EROL-007, EROL-009, and EROL-010 as
  explicitly scoped release blockers.
- Marked EROL-004 and EROL-008 as partial Rust evidence only, not release-ready
  parity.

No runtime implementation was changed. RCORE-012 is high risk because it decides
what execution/risk/order lifecycle evidence can and cannot be claimed for the
Rust-only gate. This PR must stop at `REVIEW_REQUIRED`, must not enable
auto-merge, and requires Verification & Release Gatekeeper review before merge.

## Files Changed

- `docs/rust-cutover/inventory/execution_risk_order_lifecycle.md`
- `docs/rust-cutover/evidence/RCORE-012.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-012.json`

## Commands Run

Task and planning:

```bash
mcp__shrimp_task_manager__.get_task_detail(...)
mcp__shrimp_task_manager__.analyze_task(...)
mcp__code_index__.list_indexes()
rg ...
sed ...
```

Required and final validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_fast.sh
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json
python3 -m json.tool .agentflow/leases/RCORE-012.json
python3 -m json.tool /Users/mac/.codex/shrimp-data/NTPRO/tasks.json
git diff --check
```

## Command Results

- `mcp__shrimp_task_manager__.get_task_detail`: confirmed RCORE-012 is
  `in_progress` in the isolated NTPRO Shrimp queue.
- `mcp__code_index__.list_indexes`: failed with `Transport closed`; local
  `rg`/`sed` was used as the fallback code search path.
- `rg`/`sed`: confirmed the remaining EROL gaps still map to documented
  golden-trace blockers, ignored emulator/risk tests, matching-engine TODOs,
  reconciliation Python-parity limitations, and active Python/PyO3/Cython
  surfaces.
- `scripts/ai/verify_full.sh`: passed and completed with
  `== verify_full complete ==`.
- `scripts/ai/verify_fast.sh`: passed and completed with
  `== verify_fast complete ==`.
- `python3 scripts/ai/validate_agentflow_roles.py`: passed with
  `agentflow role protocol validation passed`.
- JSON validation: passed for `.agentflow/state/task_status.json`,
  `.agentflow/leases/RCORE-012.json`, and the isolated Shrimp queue
  `/Users/mac/.codex/shrimp-data/NTPRO/tasks.json`.
- `git diff --check`: passed.

## Tests Added or Updated

No tests were added or updated. RCORE-012 did not change runtime behavior. It
records which existing Rust assertions are claimable and which lifecycle gaps
must remain scoped to later trace/runtime/removal gates.

## Behavior Impact

No trading behavior changed. No execution, risk, order manager, matching
engine, emulator, reconciliation, position, portfolio, adapter, persistence, or
public API behavior changed.

## Public API Impact

No public function signatures changed.

## Migration Note Status

No migration note is required because this task only updates cutover inventory,
evidence, and task state. Python/PyO3/Cython removal remains blocked.

## Gate Status

RCORE-012 is ready for Verification & Release Gatekeeper review. The task state
is `REVIEW_REQUIRED`; the lease is `PR_READY`; auto-merge must not be enabled.

## Rollback Plan

Revert the RCORE-012 closeout matrix, remove this evidence file, and revert the
RCORE-012 task state/lease updates. No runtime, schema, persisted data, adapter,
or public API rollback is required.
