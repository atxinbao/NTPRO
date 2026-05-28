# Core/Model Value Types Inventory

Date: 2026-05-29
Executor: Codex
Task ID: RCORE-001

## Summary

This inventory records the current Rust core/model value type surface for the
NTPRO Rust-first cutover. It is descriptive only: no runtime behavior, public
API, Cargo feature, Python, PyO3, Cython, or FFI surface changed in RCORE-001.

Current state:

- `nautilus-core` exposes foundational value primitives such as `UnixNanos`,
  `UUID4`, `StackStr`, `Params`, `AtomicTime`, correctness helpers, and
  serialization helpers.
- `nautilus-model` exposes fixed-point domain value types such as `Price`,
  `Quantity`, `Money`, `Currency`, `AccountBalance`, and `MarginBalance`.
- `nautilus-model` exposes identifier value types under
  `crates/model/src/identifiers/`, including `InstrumentId`, `Symbol`,
  `Venue`, account/order/trader/strategy IDs, and related serialization macros.
- Python/PyO3 remains compiled behind feature flags and still appears directly
  on many core/model value type definitions.
- C FFI and Cython header generation remain available behind the `ffi` feature.
- RCORE-001 does not authorize any Python, PyO3, Cython, FFI, or Cargo feature
  removal. `removal_allowed = false` for this inventory.

## Scope

Inspected paths:

- `crates/core/Cargo.toml`
- `crates/core/build.rs`
- `crates/core/src/lib.rs`
- `crates/core/src/nanos.rs`
- `crates/core/src/uuid.rs`
- `crates/core/src/string/stack_str.rs`
- `crates/model/Cargo.toml`
- `crates/model/build.rs`
- `crates/model/src/types/`
- `crates/model/src/identifiers/`
- `crates/model/src/defi/`

Non-goals:

- No Rust runtime implementation changes.
- No trading semantic changes.
- No public API changes.
- No Python/PyO3/Cython deletion.
- No adapter behavior classification.

## Current Value Type Map

### Core Primitives

| Area | Representative files | Current Rust status | Observed gap |
| --- | --- | --- | --- |
| Time value type | `crates/core/src/nanos.rs` | `UnixNanos` is a Rust value type with parsing, arithmetic, serialization, and property/unit tests. | Release evidence does not yet bind `UnixNanos` to a core/model value type gate. |
| UUID value type | `crates/core/src/uuid.rs` | `UUID4` is a Rust value type with RFC 4122 validation and serde/proptest coverage. | `UUID4` still carries PyO3 annotations behind the `python` feature. |
| Stack string | `crates/core/src/string/stack_str.rs` | `StackStr` supports checked Rust constructors and C-compatible storage. | C pointer constructors and FFI assumptions require explicit boundary classification before Rust-only release. |
| Core build surface | `crates/core/Cargo.toml`, `crates/core/build.rs` | Default feature set is empty, so Rust-only default builds can avoid Python/PyO3. | Optional `python`, `extension-module`, and `ffi` features still generate Python/Cython surfaces. |

### Model Numeric Value Types

| Area | Representative files | Current Rust status | Observed gap |
| --- | --- | --- | --- |
| Fixed precision | `crates/model/src/types/fixed.rs` | Standard precision is 9 decimal places; high precision is 16; DeFi precision can accept 18 via `WEI_PRECISION`. | Precision mode is feature-dependent and needs a release-gate test matrix before Rust-only closure. |
| Price | `crates/model/src/types/price.rs` | `Price` has checked constructors, raw constructors, serde, arithmetic, and high/standard precision branches. | `Price::from_raw` still has a v2 TODO for spurious bit validation. |
| Quantity | `crates/model/src/types/quantity.rs` | `Quantity` has checked/non-zero constructors, raw constructors, serde, arithmetic, and DeFi conversion paths. | `Quantity::from_raw` still has a v2 TODO for spurious bit validation. |
| Money | `crates/model/src/types/money.rs` | `Money` carries currency precision and has checked/raw/decimal/mantissa constructors plus serde and property tests. | `Money::from_raw_checked` still has a v2 TODO for spurious bit validation. |
| Currency and balances | `crates/model/src/types/currency.rs`, `crates/model/src/types/balance.rs` | `Currency`, `AccountBalance`, and `MarginBalance` are Rust value types with invariant checks. | Currency/balance invariants need explicit gate evidence across standard, high-precision, and DeFi feature modes. |

### Identifier Value Types

| Area | Representative files | Current Rust status | Observed gap |
| --- | --- | --- | --- |
| Identifier macros | `crates/model/src/identifiers/macros.rs` | Shared serde/from/as-ref macros reduce duplication for identifier wrappers. | Gate evidence does not yet enumerate every generated identifier wrapper. |
| Instrument identifiers | `crates/model/src/identifiers/instrument_id.rs` | `InstrumentId` parses `<symbol>.<venue>`, supports serde, and has Rust tests including UTF-8 symbol cases. | DeFi-specific parsing is feature-gated and should be included in the RCORE-002 test matrix if DeFi remains supported. |
| Symbol and venue | `crates/model/src/identifiers/symbol.rs`, `crates/model/src/identifiers/venue.rs` | `Symbol` accepts valid UTF-8; `Venue` accepts ASCII and optional DeFi venue parsing. | Rust-only product expectations for unchecked constructors and venue registry lookup should be documented or tested. |
| Other IDs | `crates/model/src/identifiers/*.rs` | Account, actor, client, order, position, strategy, trader, and venue order IDs have Rust wrappers and inline tests. | There is no consolidated value-type parity matrix that proves all ID wrappers under the same gate. |

## Legacy Surface Observations

The following observations are blockers for Rust-only removal, not actions to
take in RCORE-001.

- `crates/core/Cargo.toml` defines `python = ["pyo3", "pyo3-stub-gen", "strum"]`,
  `extension-module = ["python", "pyo3/extension-module"]`, and
  `ffi = ["cbindgen"]`.
- `crates/model/Cargo.toml` defines `python`, `python-arrow`, `extension-module`,
  `ffi`, `high-precision`, `defi`, `stubs`, and `cython-compat`.
- `crates/core/build.rs` can generate C headers and Cython definitions into
  `nautilus_trader/core/includes/core.h` and
  `nautilus_trader/core/rust/core.pxd` when `ffi` is enabled.
- `crates/model/build.rs` can generate C headers and Cython definitions into
  `nautilus_trader/core/includes/model.h` and
  `nautilus_trader/core/rust/model.pxd` when `ffi` is enabled; it also forwards
  high-precision mode into Cython definitions.
- `crates/core/src` and `crates/model/src` currently contain 124 files under
  `src/python/**`.
- `crates/core/src` and `crates/model/src` currently contain 44 files under
  `src/ffi/**`.
- 229 source files under `crates/core/src` and `crates/model/src` mention PyO3
  or PyO3 stub annotations.
- The C/Cython config files remain present:
  `crates/core/cbindgen.toml`, `crates/core/cbindgen_cython.toml`,
  `crates/model/cbindgen.toml`, and `crates/model/cbindgen_cython.toml`.

## Gap List

| Gap ID | Gap | Evidence | Impact | Follow-up |
| --- | --- | --- | --- | --- |
| CMVT-001 | Core/model value types are still dual-surfaced through PyO3 annotations and `src/python/**`. | `crates/core/src/uuid.rs`, `crates/model/src/types/price.rs`, `crates/model/src/types/quantity.rs`, `crates/model/src/types/money.rs`, `crates/model/src/identifiers/*.rs`; 124 Python files and 229 PyO3/stub mention files were observed. | Rust-only removal cannot start from core/model without a dedicated PyO3 inventory and release gate. | RREM-002, RREM-007; do not remove in RCORE-001. |
| CMVT-002 | Cython/FFI generation is still wired to core/model build scripts. | `crates/core/build.rs`, `crates/model/build.rs`, `cbindgen.toml`, `cbindgen_cython.toml`. | Rust-only closure must decide whether FFI remains product surface or removal target. | RREM-003, RREM-008; do not remove in RCORE-001. |
| CMVT-003 | Standard/high/DeFi precision behavior is implemented but not summarized as a release-gate matrix. | `crates/model/src/types/fixed.rs`, `price.rs`, `quantity.rs`, `money.rs`, `crates/model/Cargo.toml`. | Precision regressions are high-risk because they affect trading quantities, prices, balances, and DeFi amounts. | RCORE-002 should add or identify targeted tests for standard, high-precision, and DeFi modes. |
| CMVT-004 | Raw fixed-point spurious-bit validation remains explicitly deferred for `Price`, `Quantity`, and `Money`. | TODO markers in `crates/model/src/types/price.rs`, `quantity.rs`, and `money.rs`. | Raw constructors can accept values whose stored scale does not match the declared precision until follow-up closure. | RCORE-003 should either enforce validation or document an accepted compatibility exception. |
| CMVT-005 | Identifier wrappers have inline tests, but no consolidated parity inventory/test matrix. | `crates/model/src/identifiers/macros.rs`, `instrument_id.rs`, `symbol.rs`, `venue.rs`, and other ID wrapper files. | Product-surface code cannot yet point to one gate proving all core ID parsing/serde/display invariants. | RCORE-002 should enumerate identifier wrappers and run targeted Rust tests. |
| CMVT-006 | DeFi value paths depend on `defi` implying `high-precision` and U256 conversion helpers. | `crates/model/Cargo.toml`, `crates/model/src/types/fixed.rs`, `crates/model/src/defi/types/quantity.rs`, `crates/model/src/defi/wallet.rs`. | Rust-only scope must decide whether DeFi value types are supported, deferred, or adapter-scoped before release. | RCORE-002 for tests; RADP tasks for adapter policy; no removal here. |
| CMVT-007 | Checked and panicking constructors are mixed across Rust, FFI, and Python boundary expectations. | `UnixNanos::from_millis`, `UUID4::from`, `StackStr::from_c_ptr`, `Price::new`, `Quantity::new`, `Money::new`, and FFI type TODO markers. | Runtime/product callers need an explicit contract for checked vs panicking constructors before broad Rust-only API exposure. | RCORE-003 should classify constructor behavior and migration notes if public API changes. |
| CMVT-008 | Existing value-type tests are broad but not connected to release evidence. | Inline `#[cfg(test)]` modules and `rstest`/`proptest` uses in `nanos.rs`, `uuid.rs`, `stack_str.rs`, `price.rs`, `quantity.rs`, `money.rs`, and identifiers. | Local test presence is not enough for gatekeeper review unless commands and coverage scope are recorded. | RCORE-002 should publish targeted command evidence; RCORE-003 should close remaining gaps. |

## Release Gate Decision

RCORE-001 does not pass a removal gate.

Required before any Python/PyO3/Cython/core-model removal:

- RCORE-002 proves or scopes Rust tests for the value type inventory above.
- RCORE-003 closes or explicitly accepts remaining core/model value type gaps.
- RBTL and RADP tasks prove runtime and adapter parity where these value types
  are exercised.
- RREM tasks separately inventory and stage Python/PyO3/Cython removal.
- Verification & Release Gatekeeper approves the high/critical-risk gate.

## Verification Notes

RCORE-001 is docs-only inventory work. No crate tests were added or changed.
`scripts/ai/verify_fast.sh` remains the required command for this task, while
full value-type test closure belongs to RCORE-002 and RCORE-003.
