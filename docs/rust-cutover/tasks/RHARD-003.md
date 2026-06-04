# RHARD-003 - CLI help contract

Milestone: v0.2.0 Hardening
Priority: P0
Default role: Rust Product Surface
Risk: medium

## Goal

Stabilize the Rust CLI product entrypoints.

## Scope

Validate and document help contracts for:

- `backtest`;
- `sandbox`;
- `live`;
- `data`;
- `config`;
- `database`.

## Likely files

- `crates/cli/`
- `docs/`
- `docs/rust-cutover/evidence/RHARD-003.md`

## Non-goals

- Do not implement full command behavior unless the task evidence proves it is
  already scoped.
- Do not change trading semantics.
- Do not restore Python CLI entrypoints.

## Dependencies

- `RHARD-007`

## Acceptance criteria

- CLI help output is captured.
- Supported, deferred, and missing CLI contracts are documented.
- Any missing command behavior is recorded as follow-up work.

## Required commands

```bash
cargo run -p nautilus-cli -- --help
cargo run -p nautilus-cli -- backtest --help
cargo run -p nautilus-cli -- sandbox --help
cargo run -p nautilus-cli -- live --help
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/RHARD-003.md`.
