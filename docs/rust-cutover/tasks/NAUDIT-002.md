# NAUDIT-002 - CLI capability matrix and stub honesty cleanup

Milestone: v0.2.0 Audit Backlog
Priority: P0
Default role: Rust Product Surface
Risk: medium

## Goal

Make the Rust CLI public product surface accurately report what is implemented,
what is a simulated demo, and what remains deferred.

## Scope

- Create or update a CLI capability matrix with categories:
  - implemented;
  - simulated_demo;
  - metadata_only;
  - deferred.
- Cover at least:
  - `backtest validate`
  - `backtest run`
  - `sandbox validate`
  - `sandbox run`
  - `live validate`
  - `live run`
  - `data inspect`
  - `data validate`
  - `data load`
  - `config validate`
- Fix misleading `sandbox run` artifact wording before real `LiveNode` wiring
  exists.
- Keep CLI help and product docs aligned with implementation boundaries.

## Likely files

- `crates/cli/src/`
- `docs/rust-cutover/product/`
- `examples/rust/`

## Non-goals

- Do not implement full live runtime wiring.
- Do not implement dashboard UI or control API.
- Do not change trading semantics.

## Dependencies

- `RHARD-003`
- `RHARD-004`
- `RHARD-005`
- `RHARD-006`

## Acceptance criteria

- User-facing CLI docs do not overstate stubbed or simulated commands.
- `sandbox run` does not imply a real node lifecycle when it only writes
  simulated artifacts.
- Targeted CLI tests or help smoke passes.

## Required commands

```bash
cargo test -p nautilus-cli
cargo check -p nautilus-cli
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NAUDIT-002.md`.
