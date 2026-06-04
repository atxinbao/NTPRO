# NTPRO v0.1.0 Post-Release Gap List

Date: 2026-06-04
Executor: Codex
Task ID: RHARD-001

## Purpose

`ntpro-rust-only-v0.1.0` is the first formal Rust-only release point. This
document lists the remaining gaps that should drive v0.2.0 work. It does not
implement any of the gaps.

The v0.2.0 focus remains:

```text
Rust-only product hardening and operator-ready foundation
```

## Summary

| Area | Current gap | Follow-up task | Owner role | Risk | v0.2.0 status |
| --- | --- | --- | --- | --- | --- |
| CLI | Help contracts need a single current record for `backtest`, `sandbox`, `live`, `data`, `config`, and `database`. | `RHARD-003` | Rust Product Surface | medium | executable |
| CLI | A minimal backtest command path is not yet presented as a complete user workflow with input, config, output, and expected result. | `RHARD-006` | Rust Product Surface | medium | executable |
| Examples | Rust examples exist, but the post-release user journey still needs a clearer sequence from CLI help to runnable backtest, sandbox, and live-init examples. | `RHARD-006`, `RHARD-004`, `RHARD-005` | Rust Product Surface | medium | executable |
| Docs | Public docs now point at Rust-only usage, but toolchain and verification choices still need a tighter operator-facing guide. | `RHARD-002`, `RHARD-007` | Verification | low/medium | executable |
| Docs | Install and run policy still needs a v0.2.0 decision for source build, `cargo install`, binary naming, artifact strategy, platform scope, and Docker deferral. | `NBIN-001` | Rust Product Surface | medium | executable |
| Adapters | Per-adapter inventories exist, but users need one support matrix with supported, sandbox-only, fixture-only, deferred, and removed categories. | `NADAPT-001` | Adapter & Integration | medium | executable |
| Verification | Release gates exist, but v0.2.0 needs a trace and performance expansion plan that separates required, optional, and deferred evidence. | `NTRACE-001` | Verification | medium | executable |
| Architecture | Current Rust-only architecture needs a consolidated map across product surface, node runtime, engines, cache, message bus, persistence, adapters, and gates. | `NARCH-001` | Control & Scope | low | executable |
| Architecture | Module boundaries need an audit before any refactor or dashboard-facing state extraction. | `NARCH-006` | Control & Scope | low | executable |
| Architecture | Core module contracts are not yet written in one place. | `NARCH-002` | Control & Scope | medium | executable |
| Architecture | Node lifecycle states and transitions need an explicit contract before dashboard/control work. | `NARCH-003` | Control & Scope | medium | executable |
| Architecture | Future dashboard-readable observability state needs a model without implementing telemetry yet. | `NARCH-004` | Control & Scope | medium | executable |
| Architecture | Control actions need a contract without adding live control endpoints. | `NARCH-005` | Control & Scope | medium | executable |
| Dashboard | Dashboard MVP scope should be locked, but UI implementation should not start in the v0.2.0 foundation track. | `NDASH-001` | Control & Scope | low | scope-only |

## CLI Gaps

Existing evidence:

- Rust CLI product contracts exist under `docs/rust-cutover/product/`.
- `scripts/ai/verify_cli_help.sh` exists.
- README and getting-started docs point users at the Rust CLI.

Remaining gaps:

- A post-v0.1.0 help snapshot should verify the actual current CLI command
  surface.
- `backtest`, `sandbox`, `live`, `data`, `config`, and `database` help should be
  documented in one place.
- Missing command behavior should be recorded as follow-up work, not hidden in
  the docs.

Next tasks:

- `RHARD-003`
- `RHARD-006`

## Examples Gaps

Existing evidence:

- Rust example directories exist under `examples/rust/`.
- Runnable Cargo examples exist under `crates/backtest/examples/` and
  `crates/live/examples/`.
- Getting-started docs point at Rust examples and Rust how-to guides.

Remaining gaps:

- The examples are not yet packaged as one clear post-release user path.
- Backtest, sandbox, and live-init examples need evidence that explains inputs,
  commands, outputs, and expected results.
- Legacy migration tutorial snippets must stay clearly separated from supported
  Rust product examples.

Next tasks:

- `RHARD-006`
- `RHARD-004`
- `RHARD-005`

## Documentation Gaps

Existing evidence:

- README now describes the formal Rust-only release workspace.
- Installation docs now describe the source/Cargo path.
- Rust-only migration and release notes exist.

Remaining gaps:

- Toolchain selection still needs a single hardening guide so agents do not use
  the wrong local compiler.
- Fast, full, release, Rust-only runtime, Cython-removal, and golden-trace
  verification choices need one operator-facing explanation.
- Binary/install policy needs a decision record before the project promises a
  packaged CLI install path.

Next tasks:

- `RHARD-002`
- `RHARD-007`
- `NBIN-001`

## Adapter Gaps

Existing evidence:

- Per-adapter gap inventories exist under `docs/rust-cutover/inventory/`.
- Adapter tasks through `RADP-024` recorded fixture and parity evidence.

Remaining gaps:

- Users still need a consolidated support matrix.
- Supported, sandbox-only, fixture-only, deferred, and removed categories should
  be explicit.
- Real API calls and credential-dependent behavior must not be required for
  support classification.

Next task:

- `NADAPT-001`

## Verification Gaps

Existing evidence:

- Fast, full, release, Rust-only runtime, Cython-removal, and golden trace
  scripts exist under `scripts/ai/`.
- `RREL-009` records final release verification.
- Golden trace release scope exists under `docs/rust-cutover/golden_trace/`.

Remaining gaps:

- Verification docs need to explain which command is appropriate for which
  work type.
- Trace and performance expansion should be planned before adding new release
  gates.
- Slow release checks should be labeled so automation does not treat them as
  every-edit checks.

Next tasks:

- `RHARD-007`
- `NTRACE-001`

## Architecture Gaps

Existing evidence:

- General architecture docs exist under `docs/concepts/architecture.md`.
- Rust-only target architecture docs exist under `docs/rust-cutover/`.
- Runtime, product, adapter, trace, and release evidence exists from v0.1.0.

Remaining gaps:

- NTPRO needs a current Rust-only architecture map after removal.
- Module boundaries need an audit before refactor or dashboard-facing data
  extraction.
- Core module contracts need to define responsibilities, inputs, outputs,
  state, lifecycle, errors, and dependency boundaries.
- Node lifecycle, observability state, and control action contracts are needed
  before dashboard work.

Next tasks:

- `NARCH-001`
- `NARCH-006`
- `NARCH-002`
- `NARCH-003`
- `NARCH-004`
- `NARCH-005`
- `NDASH-001`

## Blockers

No blocker prevents the next task, `RHARD-002`, from starting after
`RHARD-001` is merged and closed.

Known constraints for later tasks:

- Real exchange credentials must not be required for smoke or support matrix
  evidence.
- Dashboard UI implementation is not part of the v0.2.0 foundation track.
- Runtime telemetry and live control wiring are deferred until architecture
  contracts are complete.
- Docker delivery is deferred unless a later owner-approved task changes scope.

## Deferred From v0.2.0 Mainline

The following remain deferred:

- Dashboard UI implementation.
- Runtime telemetry implementation.
- Node control API implementation.
- Live control wiring.
- Manual order entry.
- Strategy parameter hot reload.
- Multi-user permissions.
- Docker delivery as a requirement.
- New Python/PyO3/Cython product surfaces.
- Additional removal work outside explicitly approved cleanup tasks.
