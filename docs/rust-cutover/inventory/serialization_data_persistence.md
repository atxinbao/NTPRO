# Rust Serialization/Data/Persistence Inventory

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-004

## Summary

This inventory covers the Rust-only cutover gaps in:

- `crates/serialization`
- `crates/data`
- `crates/persistence`

Current state:

- `nautilus-serialization` provides Arrow, display Arrow, Cap'n Proto, SBE, and
  Python Arrow bindings.
- `nautilus-data` owns the Rust data engine, aggregation, option-chain
  aggregation, DeFi data path, and optional catalog streaming through
  `nautilus-persistence`.
- `nautilus-persistence` owns Parquet catalog storage, DataFusion query
  sessions, Feather streaming writers, custom data registration, object-store
  backends, and Python catalog/feather/wrangler bindings.
- RCORE-004 does not authorize Python, PyO3, Cython, FFI, schema, Cargo feature,
  data format, adapter, or persistence behavior removal. `removal_allowed =
  false`.

## Scope

In scope:

- Current Rust modules, feature flags, tests, and persistence/data-format
  boundaries in `serialization`, `data`, and `persistence`.
- Python/PyO3/FFI surfaces that block Rust-only closure.
- Cap'n Proto, SBE, Arrow, Parquet, Feather, DataFusion, object-store, and custom
  data parity gaps.
- Follow-up routing for RCORE/RREM/RTRACE/RADP tasks.

Non-goals:

- No crate code changes.
- No public API changes.
- No data format migration.
- No Python, PyO3, Cython, generated schema, or FFI removal.
- No adapter behavior changes.

## Inventory Snapshot

Observed from local scans:

- Rust source files under `src`: `serialization=67`, `data=25`,
  `persistence=24`.
- Rust files across crate trees, including tests/benches/generated code:
  `serialization=88`, `data=30`, `persistence=31`.
- Cap'n Proto has 11 bundled schema files under
  `crates/serialization/schemas/capnp` and 11 generated Rust files under
  `crates/serialization/generated/capnp`.
- Python-facing source files in the three crate trees: 16.
- Rust files mentioning PyO3/stub generation in these crate trees: 22.
- Integration test files under the three crate `tests/` directories: 11.
- TODO/FIXME matches in the three crate trees: 10.
- `unsafe` matches in the three crate trees: 60.

## Crate Surface Matrix

| Area | Representative files | Current Rust status | Observed gap |
| --- | --- | --- | --- |
| Arrow serialization | `crates/serialization/src/arrow/**` | Arrow schema, encode, decode, display, precision mismatch, and legacy raw correction helpers exist. | Needs release-gate matrix for standard/high precision, legacy raw correction, metadata preservation, and persisted catalog compatibility. |
| Cap'n Proto | `crates/serialization/schemas/capnp/**`, `generated/capnp/**`, `src/capnp/**` | Schemas are bundled; generated Rust files are checked in; build script can compile schemas when `capnp` is enabled. | README and module docs still state Cap'n Proto schemas/wire format are unstable. This is not release-ready without a schema stability decision. |
| SBE | `crates/serialization/src/sbe/**` | Cursor, writer, primitives, and market-data encode/decode utilities exist behind `sbe`. | SBE has unsafe writer/market code and no release-gate stability classification in this inventory scope. |
| Python serialization bridge | `crates/serialization/src/python/**` | PyO3 Arrow conversion functions expose Python-facing record batch bytes and schema maps. | Python/PyO3 remains an active product surface and cannot be removed in RCORE tasks. |
| Data engine | `crates/data/src/engine/**` | Rust `DataEngine` handles subscriptions, requests, aggregation, book snapshots, option chains, and optional catalog streaming. | Some dynamic data still bypasses the `Data` enum because C/Cython blocks enum closure. |
| Data aggregation | `crates/data/src/aggregation.rs` | Rust aggregation covers time, tick, volume, value, imbalance, runs, Renko, continuous futures, spread quote, and parity-heavy behavior. | Multiple comments intentionally mirror Cython behavior; full Rust-only closure needs parity evidence before Cython removal. |
| Data streaming | `crates/data/src/engine/streaming.rs`, `crates/data/Cargo.toml` | `streaming` feature enables `nautilus-persistence` and catalog-backed date-range handling. | Product contract for Rust-only catalog streaming remains implicit; test ownership should move to RCORE-005/RCORE-006. |
| Option chains | `crates/data/src/option_chains/**`, `src/python/option_chain_manager.rs` | Rust manager/aggregator exists; Python wrapper exposes the aggregator for Cython-owned lifecycle. | Python wrapper explicitly states Cython `DataEngine` owns lifecycle, so Rust-only lifecycle parity is not yet closed. |
| Parquet catalog | `crates/persistence/src/backend/catalog.rs`, `catalog_operations.rs`, `parquet.rs` | `ParquetDataCatalog` supports typed query/write, consolidation, interval checks, object-store paths, DataFusion, and Parquet helpers. | Instrument data handling is documented as not implemented for mixed catalog writes; object-store behavior needs fixture matrix. |
| Feather writer | `crates/persistence/src/backend/feather.rs` | Feather writer supports Arrow stream buffers, per-type/per-instrument routing, custom data, and object-store writes. | Rotation and error-type TODOs remain open. |
| DataFusion session | `crates/persistence/src/backend/session.rs` | `DataBackendSession` registers object stores/files and returns sorted query results. | Contains unsafe raw-parts reconstruction and query/session reset behavior that needs release-gate review. |
| Custom data | `crates/persistence/macros/src/custom.rs`, `backend/custom.rs`, `src/test_data.rs` | Custom data macro emits Arrow/schema/JSON traits and optional PyO3/stub support. | Rust-only custom data contract is not separate from PyO3/Arrow FFI boundary yet. |
| Python persistence bridge | `crates/persistence/src/python/**` | Python catalog, Feather writer, backend session, and wranglers are active. | Python catalog notes mixed data cannot cross PyO3 as a single type; Python bridge remains a blocker for Rust-only removal. |

## Gap Register

| ID | Gap | Evidence | Impact | Follow-up |
| --- | --- | --- | --- | --- |
| SDP-001 | Python/PyO3 surfaces remain active across serialization/data/persistence. | 16 `src/python/**` files and 22 Rust files mentioning PyO3/stub generation were observed. Cargo `python` and `extension-module` features remain in all three crate areas. | Rust-only removal cannot start from these crates without a dedicated Python/PyO3 removal plan. | RREM tasks; no removal in RCORE-004. |
| SDP-002 | Data engine still has C/Cython blocking notes for dynamic data enum closure. | `crates/data/src/engine/mod.rs` notes `InstrumentAny`, funding rates, instrument status, option greeks, and custom data should eventually enter `Data` after C/Cython blockers; `crates/data/tests/engine.rs` still notes temporary FFI API wrapper use until Cython is gone. | Rust product surface cannot claim full data engine independence while Cython/FFI assumptions remain. | RCORE-005 tests; RCORE-006 closure; RREM Cython gate. |
| SDP-003 | Cap'n Proto and SBE wire-format stability is explicitly not release-gated. | `crates/serialization/README.md`, `src/lib.rs`, and `src/capnp/mod.rs` warn schemas/wire formats are not stable. | Rust-only release cannot rely on these formats without a schema compatibility decision. | RTRACE schema/golden fixtures; RCORE-006 stability decision. |
| SDP-004 | Arrow fixed precision and legacy raw correction need cross-mode release evidence. | `crates/serialization/src/arrow/mod.rs` has `PrecisionMismatch`, `PRECISION_BYTES`, and legacy raw correction helpers; RCORE-003 tightened checked constructors. | Catalog decode behavior can differ across standard/high precision and legacy catalog data. | RCORE-005 high/standard precision tests; RCORE-006 policy. |
| SDP-005 | Instrument serialization and catalog mixed writes are not fully closed. | `crates/serialization/src/arrow/instrument/mod.rs` can return "Instrument type ... serialization not yet implemented"; `ParquetDataCatalog::write_data` documents instrument data handling as not implemented. | Instrument catalog parity is incomplete for Rust-only storage workflows. | RCORE-005 tests; RCORE-006 implementation or explicit deferral. |
| SDP-006 | Object-store/cloud persistence lacks a single support/defer matrix. | `nautilus-persistence` supports `cloud`; `parquet.rs` has S3, Azure, GCS, HTTP, and local object-store paths. | Release gate cannot distinguish supported local storage from deferred remote storage without fixture strategy. | RCORE-005 local/mock tests; RADP/NDB policy for external stores. |
| SDP-007 | Feather writer rotation and error model are still incomplete. | `crates/persistence/src/backend/feather.rs` has TODOs for next rotation timestamp, handling rotation config on buffer take, and fixing Arrow/object-store error type. | Streaming writer behavior is not fully specified for Rust-only release. | RCORE-006 implementation or scoped deferral. |
| SDP-008 | Unsafe blocks need targeted audit before release. | Unsafe occurs in persistence binary heap/session/PyO3 Arrow FFI paths and serialization SBE writer/market paths. | Rust-only release should not inherit unsafe data path behavior without documented justification and targeted tests. | Verification gatekeeper audit; RCORE-005 tests where feasible. |
| SDP-009 | Custom data registration couples Rust, Arrow, JSON, and optional PyO3. | `nautilus-persistence-macros` emits Arrow/schema/JSON traits and optional PyO3/stub code; `test_data.rs` uses `#[custom_data(pyo3)]`. | Rust-only custom data API is not independently documented from Python bridge behavior. | RCORE-005 custom data tests; RCORE-006 product contract. |
| SDP-010 | Catalog streaming is feature-coupled from data to persistence. | `nautilus-data` `streaming` feature depends on `nautilus-persistence`; `DataEngine` has a `CatalogMap` only under `streaming`. | Rust data runtime and persistence lifecycle boundaries need explicit product contract. | RCORE-006 boundary decision; NPROD/RPROD docs if exposed to users. |

## Existing Test Surface

Integration tests currently observed:

- `crates/serialization/tests/test_enums_capnp.rs`
- `crates/serialization/tests/test_identifiers_capnp.rs`
- `crates/serialization/tests/test_market_data_capnp.rs`
- `crates/serialization/tests/test_market_data_sbe.rs`
- `crates/serialization/tests/test_types_capnp.rs`
- `crates/data/tests/client.rs`
- `crates/data/tests/engine.rs`
- `crates/persistence/tests/test_catalog.rs`
- `crates/persistence/tests/test_feather.rs`

The three crate areas also have many inline `#[cfg(test)]` modules for Arrow,
SBE, aggregation, option chains, Parquet, Feather, and custom data. RCORE-004
does not add tests; RCORE-005 should convert the gap register above into a
targeted test matrix.

## Release Gate Decision

RCORE-004 is an inventory task only.

- `removal_allowed = false`
- No Python/PyO3/Cython/FFI deletion is allowed here.
- No Cap'n Proto, SBE, Arrow, Parquet, Feather, DataFusion, or object-store
  behavior changes are allowed here.
- RCORE-005 should add targeted Rust tests for the highest-risk gaps.
- RCORE-006 should either close the gaps or record explicit scope deferrals.
