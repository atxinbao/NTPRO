# RCORE-006 Evidence - Close Rust Serialization/Data/Persistence Gaps

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-006
Risk: high

## Summary

Closed the RCORE serialization/data/persistence runtime scope by recording an
explicit closure matrix for every RCORE-004 `SDP-*` gap.

The closure matrix separates locally proven Rust runtime coverage from items
that require later gates. Local Rust runtime coverage is closed for:

- Arrow/Parquet fixed precision catalog boundaries;
- `InstrumentAny` catalog persistence and query paths;
- custom data catalog roundtrip and data-engine request paths;
- internal `nautilus-data` to `nautilus-persistence` catalog streaming.

The following remain explicit scoped deferrals, not done work:

- Python/PyO3/Cython removal;
- Cap'n Proto and SBE release-stability claims;
- remote/cloud object-store fixture support;
- Feather rotation and consolidated error-model release claims;
- unsafe audit;
- product-surface exposure.

RCORE-006 is high risk by role policy because serialization, data, and
persistence define stored and replayed trading data. This PR must stop at
`REVIEW_REQUIRED`, must not enable auto-merge, and requires Verification &
Release Gatekeeper review before merge.

## Files Changed

- `crates/persistence/src/backend/catalog.rs`
- `docs/rust-cutover/inventory/serialization_data_persistence.md`
- `docs/rust-cutover/evidence/RCORE-006.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/RCORE-006.json`

## Commands Run

Dispatch:

```bash
python3 scripts/control/dispatch_next.py --max-risk high
```

Planning and discovery:

```bash
mcp__shrimp_task_manager__.get_task_detail(taskId="3b98308f-f357-4f79-b3f7-0d9be5a20182")
mcp__shrimp_task_manager__.analyze_task(...)
mcp__shrimp_task_manager__.reflect_task(...)
mcp__code_index__.search_code(directory="/Users/mac/Documents/NTPRO", query="PrecisionMismatch OR PRECISION_BYTES OR ParquetDataCatalog OR write_data")
rg -n "SDP-|removal_allowed|Release Gate Decision|Gap Register|RCORE-005|RCORE-006|defer|deferred|scope" docs/rust-cutover -g '*.md'
rg -n "not yet implemented|Cannot encode mixed|Instrument type|write_data|InstrumentAny|TODO|rotation|FIXME|unsafe" crates/serialization/src/arrow/instrument crates/persistence/src/backend crates/data/src/engine -g '*.rs'
rg -n "test_.*catalog|write_data|InstrumentAny|custom|feather|rotation|object" crates/persistence/tests crates/data/tests crates/serialization/tests -g '*.rs'
```

Focused validation:

```bash
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc rustup run 1.95.0-aarch64-apple-darwin cargo test -p nautilus-serialization --features arrow,high-precision test_validate_precision_bytes --lib
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc rustup run 1.95.0-aarch64-apple-darwin cargo test -p nautilus-persistence --features high-precision --test test_catalog test_rust_quote_tick_catalog_roundtrip_preserves_fixed_precision
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc rustup run 1.95.0-aarch64-apple-darwin cargo test -p nautilus-persistence --test test_catalog test_write_instruments
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc rustup run 1.95.0-aarch64-apple-darwin cargo test -p nautilus-data --features streaming --test engine test_request_instrument_catalog_uses_latest_record
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc rustup run 1.95.0-aarch64-apple-darwin cargo test -p nautilus-data --features streaming --test engine test_request_custom_data_catalog_plus_client_split
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc rustup run 1.95.0-aarch64-apple-darwin cargo test -p nautilus-data --features streaming --test engine test_request_quotes_catalog_plus_client_split
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc rustup run 1.95.0-aarch64-apple-darwin cargo test -p nautilus-persistence --test test_feather test_write_data_enum_all_types
```

Required/final validation:

```bash
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_full.sh
```

Post-evidence metadata validation:

```bash
RUSTC=/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc rustup run 1.95.0-aarch64-apple-darwin cargo fmt --check
python3 scripts/ai/validate_agentflow_roles.py
python3 -m json.tool .agentflow/state/task_status.json >/dev/null
python3 -m json.tool .agentflow/leases/RCORE-006.json >/dev/null
git diff --check
PATH="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH" RUSTC="/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc" scripts/ai/verify_fast.sh
```

## Command Results

- `dispatch_next.py --max-risk high`: dispatched RCORE-006 on branch
  `ai/RCORE-006-close-rust-serialization-data-persistence-gaps`.
- Shrimp isolated queue: confirmed RCORE-006 is the single in-progress task.
- Code-Index returned no matches for one broad query, so local `rg` scans were
  used for the focused gap and test-surface review.
- High-precision Arrow precision tests: passed, 2 tests.
- High-precision `QuoteTick` catalog precision roundtrip: passed, 1 test.
- Instrument catalog focused tests: passed, 2 tests.
- Data-engine streaming instrument catalog focused test: passed, 1 test.
- Data-engine streaming custom data catalog-plus-client focused test: passed,
  1 test.
- Data-engine streaming quote catalog-plus-client focused test: passed, 1 test.
- Feather mixed `Data` writer focused test: passed, 1 test.
- `scripts/ai/verify_full.sh`: passed. It completed `verify_fast`,
  workspace clippy, workspace Rust tests, log-global tests, golden trace
  validation, golden trace replay tests, and rustdoc generation.
- Post-evidence metadata validation: passed. `cargo fmt --check`,
  agentflow role validation, JSON validation, `git diff --check`, and
  `scripts/ai/verify_fast.sh` completed successfully.

## Tests Added or Updated

No Rust tests were added or updated in RCORE-006.

RCORE-006 closes or scopes gaps by connecting existing Rust test evidence and
RCORE-005 focused tests to the RCORE-004 `SDP-*` gap register. The only crate
source change is a documentation correction in `ParquetDataCatalog::write_data_enum`
that points instrument persistence to `write_instruments` and query APIs.

## Behavior Impact

No runtime behavior changed. No trading semantics changed. No adapter behavior
changed. No data format, schema, persistence writer, persistence reader, Cargo
feature, Python, PyO3, Cython, or FFI file was deleted or moved.

The `write_data_enum` Rustdoc no longer claims instrument data is not
implemented. It now states the actual contract: instruments are not `Data` enum
variants and must use the dedicated `write_instruments` / `query_instruments`
APIs.

## Public API Impact

No public function signatures changed.

The only API-facing change is Rustdoc clarification for existing catalog
methods.

## Migration Note Status

No migration note is required because no public API signature or persisted data
format changed.

## Rollback Plan

Revert the `write_data_enum` Rustdoc correction, remove the RCORE-006 closure
matrix and gate decision from
`docs/rust-cutover/inventory/serialization_data_persistence.md`, remove this
evidence file, and restore the RCORE-006 `.agentflow` metadata. No persisted
data migration is required.
