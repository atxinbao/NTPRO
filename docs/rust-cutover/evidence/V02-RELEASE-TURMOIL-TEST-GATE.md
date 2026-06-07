# V02 Release Turmoil Test Gate Evidence

Date: 2026-06-07
Executor: Codex

## Task

Local task: V02 release gate blocker cleanup.

## Goal

Keep the v0.2 release gate deterministic by preventing Cargo from launching
network turmoil integration-test binaries when the release feature set does not
enable the `turmoil` simulation feature.

## Findings

- `scripts/ai/verify_release.sh` delegates to `scripts/ai/verify_full.sh`.
- `verify_full.sh` runs:
  - `cargo test --workspace --lib --tests --features "arrow,ffi,high-precision,streaming,defi" ...`
- `nautilus-network` enables `transport-sockudo` by default, but the release
  feature set does not enable `turmoil`.
- Before this change Cargo still auto-discovered and launched
  `crates/network/tests/turmoil_sockudo.rs` as an empty/simulation-disabled test
  binary during the release gate.
- The `turmoil_sockudo` test binary stalled even under `--list`, before any
  test function ran, leaving release verification blocked after the larger
  workspace tests had already progressed successfully.
- Explicit `cargo test -p nautilus-network --test turmoil_sockudo -- --list`
  now fails fast with a Cargo feature-gate error instead of starting the test
  binary.

## Changes

- `crates/network/Cargo.toml`
  - Disables integration-test auto-discovery for `nautilus-network` with
    `autotests = false`.
  - Explicitly lists the regular non-turmoil integration tests so default
    network test coverage stays active.
  - Adds explicit Cargo test target metadata for:
    - `property_backoff`
    - `property_ratelimiter`
    - `turmoil_socket`
    - `turmoil_websocket`
    - `turmoil_sockudo`
    - `websocket_proxy`
  - Adds `required-features = ["turmoil"]` for the generic turmoil targets.
  - Adds `required-features = ["turmoil", "transport-sockudo"]` for the sockudo
    turmoil target.

## Command Output Summary

- `cargo test -p nautilus-network --test turmoil_sockudo -- --list`
  - Result: exits with `exit_status=101`.
  - Output: Cargo reports target `turmoil_sockudo` requires the features
    `turmoil`, `transport-sockudo`.
- `cargo test -p nautilus-network --lib --tests -- --skip logging::logger::tests::serial_tests --skip logging::macros::tests::test_colored_logging_macros --skip logging::macros::tests::test_default_macro_captures_module_path --skip serial_tests`
  - Result: passed.
  - Summary:
    - `nautilus_network` lib: 307 passed.
    - `property_backoff`: 7 passed.
    - `property_ratelimiter`: 10 passed.
    - `websocket_proxy`: 4 passed.
  - `turmoil_*` integration targets did not run in the default network test path.
- `scripts/ai/verify_fast.sh`
  - Result: passed.
- `git diff --check`
  - Result: passed.

## Behavior Impact

No runtime or trading behavior changes.

The change only affects Cargo test target selection. Dedicated turmoil
integration tests are no longer part of the default release path unless the
matching simulation features are explicitly enabled.

## Public API Impact

No public API changes.

## Migration Note

No migration note required. Developers who want to work on these simulation
tests must explicitly enable the required features, for example:

```bash
cargo test -p nautilus-network --features "turmoil,transport-sockudo" --test turmoil_sockudo
```

## Rollback Plan

Remove the explicit `[[test]]` target metadata from `crates/network/Cargo.toml`
and rerun `scripts/ai/verify_release.sh`.
