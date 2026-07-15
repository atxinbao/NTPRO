# PTC-002 Rust Golden Trace Tooling Migration

Date: 2026-07-15
Executor: Codex

## Replacement

The supported golden-trace governance entrypoint is now:

```bash
scripts/ai/ntpro_governance.sh golden-trace TRACE --mode validate-only
scripts/ai/ntpro_governance.sh golden-trace TRACE --mode replay \
  --replay-command 'COMMAND {trace} {actual}'
scripts/ai/ntpro_governance.sh golden-trace-release-scope \
  --manifest docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json \
  --trace-glob 'tests/golden/*.jsonl'
```

The launcher executes the workspace binary `ntpro-governance`. A prebuilt
binary may be selected with `NTPRO_GOVERNANCE_BIN`; otherwise Cargo builds and
runs the locked workspace package.

## Retired Files

- `scripts/ai/golden_trace_runner.py`;
- `scripts/ai/validate_golden_trace_release_scope.py`.

Executable workflows and shell gates no longer call either file. Historical
release records may still name `golden_trace_runner.py` when checking the
contents of an immutable historical release manifest; those strings are audit
data, not executable dependencies.

## Compatibility

The Rust replacement preserves:

- JSONL comments and blank-line handling;
- validate-only output and replay command placeholders;
- replay output from either `{actual}` or standard output;
- exact expected/actual JSON comparison;
- release manifest and trace reconciliation;
- executable, validator-executable, and schema-only contracts;
- fail-closed rejection of blank owner and evidence fields.

This migration changes repository tooling only. It does not change runtime,
trading semantics, public API, backend capability, or the v0.32.0 baseline.
