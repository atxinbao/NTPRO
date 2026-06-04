# Post-release Serialization Arrow Link Cleanup Evidence

- Date: 2026-06-05
- Executor: Codex
- Local task name: `POST-RELEASE-SERIALIZATION-ARROW-LINK-CLEANUP`
- Formal task file: not present in the Shrimp queue; this is a scoped
  post-release public surface cleanup.

## Goal

Update the Arrow precision mismatch diagnostic so users are sent to NTPRO
precision-mode documentation instead of the upstream NautilusTrader website.

## Scope

Changed:

- `crates/serialization/src/arrow/mod.rs`
- `docs/rust-cutover/evidence/POST-RELEASE-SERIALIZATION-ARROW-LINK-CLEANUP.md`

Not changed:

- Arrow schema encoding or decoding;
- precision-mode validation logic;
- `high-precision` or Cargo feature behavior;
- runtime catalog compatibility behavior;
- CLI, dashboard, control API, or adapter behavior;
- migration or release status.

## Changes

- Replaced the `PrecisionMismatch` error message URL:
  - from `https://nautilustrader.io/docs/latest/getting_started/installation#precision-mode`
  - to `https://github.com/atxinbao/NTPRO/blob/main/docs/getting_started/installation.md#precision-mode`
- Reused the same NTPRO precision-mode documentation link already used by:
  - `crates/serialization/src/lib.rs`
  - `crates/serialization/README.md`

## Validation

| Command | Result | Notes |
|---------|--------|-------|
| `rg -n "nautilustrader\\.io" crates/serialization/src/arrow/mod.rs` | passed | No matches; command exited 1 because `rg` found no lines. |
| `rg -n "github.com/atxinbao/NTPRO/blob/main/docs/getting_started/installation.md#precision-mode" crates/serialization/src/arrow/mod.rs crates/serialization/src/lib.rs crates/serialization/README.md` | passed | The Arrow diagnostic now uses the same NTPRO precision link as the serialization crate docs. |
| `scripts/ai/check_rust_only_runtime.sh` | passed | Reported `rust-only-runtime: ok`. |
| `scripts/ai/check_cython_removed.sh` | passed | Reported `cython-removed: ok`. |
| `scripts/ai/verify_fast.sh` | passed | Used cargo/rustc 1.95.0; workspace cargo check and clippy skipped by fast-mode defaults. |
| `source scripts/ai/toolchain_env.sh && cargo check -p nautilus-serialization` | passed | Finished `dev` profile in 1m 53s. |
| `git diff --check` | passed | No whitespace errors. |

## Behavior Impact

The only runtime-visible change is the help URL included in the precision
mismatch error message. Serialization behavior, Arrow schema handling, catalog
data compatibility, precision-byte validation, and trading semantics are
unchanged.

## Public API Impact

No public Rust API changed. No exported types, functions, modules, Cargo
features, crate names, or binary names changed.

## Migration Note Status

No new migration note is required. This PR supports the existing Rust-only
migration posture by making the precision-mode help link point at NTPRO-owned
documentation.

## Rollback Plan

Revert this PR to restore the previous upstream help URL. No runtime, API,
dependency, or data migration is required.
