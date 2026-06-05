# NAUDIT-003 - Unignore passing production-bug cache tests

Milestone: v0.2.0 Audit Backlog
Priority: P0
Default role: Rust Core Runtime
Risk: medium

## Goal

Restore regression coverage for the two common cache tests marked as production
bugs when targeted ignored test runs show they pass.

## Scope

- Remove `#[ignore = "Production bug: ..."]` from:
  - `test_order_when_rejected`
  - `test_order_when_filled`
- Run the restored tests as normal tests.
- Update ignored-test risk documentation to reflect the restored coverage.

## Likely files

- `crates/common/src/cache/tests.rs`
- `docs/rust-cutover/verification/ignored_tests_risk_register.md`
- `docs/rust-cutover/evidence/NAUDIT-003.md`

## Non-goals

- Do not fix all ignored tests in one PR.
- Do not change cache behavior unless the restored tests reveal a real failure.
- Do not delete tests.

## Dependencies

- `GH-160`

## Acceptance criteria

- The two cache tests run without `--ignored`.
- Ignored-test risk register no longer lists these two as active ignored
  production bugs.
- No trading semantics are changed unless required by a failing restored test.

## Required commands

```bash
cargo test -p nautilus-common --lib test_order_when_rejected -- --nocapture
cargo test -p nautilus-common --lib test_order_when_filled -- --nocapture
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NAUDIT-003.md`.
