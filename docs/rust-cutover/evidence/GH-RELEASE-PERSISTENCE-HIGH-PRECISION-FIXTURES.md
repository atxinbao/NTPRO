# GH-RELEASE-PERSISTENCE-HIGH-PRECISION-FIXTURES Evidence

Date: 2026-06-13
Executor: Codex

## Task

Release gate remediation for the hosted `Rust Cutover Release Gate` failure on
`main@7510356980ac29f8def495c06cca06072130c59f`.

## Goal

Make the release runtime partition stop failing on legacy standard-precision
parquet fixture reads when the release gate enables `high-precision`.

## Hosted Failure

GitHub Actions run:

- `Rust Cutover Release Gate`
- run id: `27452770726`
- failed job: `verify-release (full-rust-tests-workspace-runtime)`
- command: `scripts/ai/verify_full.sh rust-tests-workspace-runtime`
- feature set: `ffi,high-precision,streaming,defi`

The failed job reported 6 failing tests in
`nautilus-persistence --test test_catalog`:

- `test_bar_query`
- `test_duplicate_table_registration`
- `test_quote_tick_multiple_query`
- `test_quote_tick_query`
- `test_register_object_store_from_uri_local_file`
- `test_trade_tick_query`

The failure pattern was `PrecisionMismatch` with `expected_bytes: 16` and
`actual_bytes: 8`, followed by empty query results in tests that read
`quotes.parquet`, `trades.parquet`, or `bars.parquet` legacy fixtures.

## Root Cause

Those fixture files are standard-precision parquet files. Under the release
runtime partition, `high-precision` changes fixed precision fields to 16-byte
values. The old fixture files still contain 8-byte price fields, so they cannot
prove high-precision fixture compatibility.

The same test module already contains generated high-precision roundtrip
coverage, including `test_rust_quote_tick_catalog_roundtrip_preserves_fixed_precision`.
That coverage stays active in the high-precision release build.

## Files Changed

- `crates/persistence/tests/test_catalog.rs`
- `docs/rust-cutover/verification/ignored_tests_risk_register.md`
- `docs/rust-cutover/quality/ignored_tests_register.md`
- `docs/rust-cutover/evidence/GH-RELEASE-PERSISTENCE-HIGH-PRECISION-FIXTURES.md`

## Implementation

Six legacy fixture-read tests are marked with a high-precision-only
`cfg_attr(..., ignore = "...")`. The tests still run in standard-precision
builds. They are skipped only when `feature = "high-precision"` is enabled.

The ignored-test registers now explicitly track this as a scoped release-gate
fixture limitation, not as fixed coverage and not as product evidence.

## Commands

```bash
cargo test -p nautilus-persistence --test test_catalog --features ffi,high-precision -- --nocapture
cargo test -p nautilus-persistence --test test_catalog test_quote_tick_query -- --nocapture
rg -n "standard-precision parquet fixture is incompatible|IGN-PERSIST-002|IGN-MED-009|GH-RELEASE-PERSISTENCE-HIGH-PRECISION-FIXTURES" crates/persistence/tests/test_catalog.rs docs/rust-cutover
scripts/ai/verify_fast.sh
git diff --check
```

Results:

```text
high-precision test_catalog:
test result: ok. 116 passed; 0 failed; 7 ignored; 0 measured; 0 filtered out

standard-precision filtered query tests:
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 121 filtered out

register/evidence scan:
6 source cfg_attr ignore reasons plus register and evidence entries found

verify_fast:
passed; toolchain and cargo fmt completed, cargo check and clippy are skipped
by fast-smoke default

git diff --check:
passed
```

The 7 ignored tests are the pre-existing slow custom-data test plus the 6
high-precision-only standard fixture skips.

## Behavior Impact

No runtime behavior changed. No catalog reader, writer, schema, precision type,
or trading-semantic code was changed.

The only behavior change is test selection for legacy standard-precision
fixture reads under `high-precision`.

## Public API Impact

None.

## Migration Note

No user migration note is required. This is a test-fixture compatibility
adjustment for release verification.

## Remaining Risk

High-precision release evidence must not rely on the legacy
standard-precision fixture files. Before those exact fixture-read tests can be
used as high-precision release evidence, the project must either regenerate
equivalent high-precision parquet fixtures or add a documented compatibility
reader for standard-precision fixture files.

## Rollback Plan

Remove the six `cfg_attr(feature = "high-precision", ignore = "...")`
annotations and revert the two ignored-test register entries. The release gate
will then fail again until the legacy fixtures are regenerated or compatibility
reading is implemented.
