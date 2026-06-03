# RREL-009 - Wire final golden trace replay gate

Milestone: R7 Release
Priority: P0
Default role: Release

## Goal

Wire the final golden trace replay gate so `scripts/ai/verify_release.sh` no
longer fails only because `GOLDEN_TRACE_REPLAY_COMMAND` is unset.

## Scope

- Define a deterministic final golden trace release replay/scope contract.
- Ensure every `tests/golden/*.jsonl` case is either bound to executable Rust
  replay evidence or explicitly scoped as schema-only seed evidence.
- Keep the release decision separate from human owner signoff.
- Update release evidence and golden trace gate documentation.

## Likely files

- `scripts/ai/run_golden_traces.sh`
- `scripts/ai/*golden*`
- `docs/rust-cutover/golden_trace/**`
- `docs/rust-cutover/release/**`
- `docs/rust-cutover/evidence/RREL-009.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RREL-009.json`

## Non-goals

- Do not mark the Rust-only cutover complete.
- Do not create or push a release tag.
- Do not sign the human owner signoff packet.
- Do not delete or weaken golden trace fixtures.
- Do not change trading semantics to satisfy the gate.

## Dependencies

- `RREL-008`

## Acceptance criteria

- Final golden trace replay/scope contract exists and is documented.
- `scripts/ai/run_golden_traces.sh` passes in final release mode without
  requiring an ad hoc external command.
- `scripts/ai/verify_release.sh` reaches the next release phase or passes; if it
  fails after the golden trace gate, the new blocker is documented.
- RREL-009 evidence states that owner signoff remains pending.

## Required commands

```bash
scripts/ai/run_golden_traces.sh
REQUIRE_GOLDEN_REPLAY=1 scripts/ai/run_golden_traces.sh
scripts/ai/verify_release.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RREL-009.md` with:

- task ID;
- summary;
- files changed;
- commands run;
- command results;
- behavior impact;
- public API impact;
- migration note status;
- rollback plan.
