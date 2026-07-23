# nautilus-serialization

[![Rust Cutover Smoke](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml/badge.svg?branch=main)](https://github.com/atxinbao/NTPRO/actions/workflows/rust-cutover-smoke.yml)
[![Documentation](https://img.shields.io/docsrs/nautilus-serialization)](https://docs.rs/nautilus-serialization/latest/nautilus-serialization/)
[![crates.io version](https://img.shields.io/crates/v/nautilus-serialization.svg)](https://crates.io/crates/nautilus-serialization)
![license](https://img.shields.io/github/license/atxinbao/NTPRO?color=blue)

Data serialization and format conversion for NTPRO.

The `nautilus-serialization` crate provides data serialization capabilities for converting
trading data between different formats including Apache Arrow, Parquet, JSON, MsgPack, and SBE.
This enables efficient data storage, retrieval, and interoperability across different systems:

- **Apache Arrow integration**: Schema definitions and encoding/decoding for market data types.
- **Parquet file operations**: High-performance columnar storage for historical data analysis.
- **Record batch processing**: Efficient batch operations for time-series data.
- **Schema management**: Type-safe schema definitions with metadata preservation.
- **Cross-format conversion**: Data interchange between Arrow, Parquet, and native types.
- **SBE decode utilities**: Zero-copy cursor, shared decode errors, and generic var/group decoders for SBE parsers.

> [!WARNING]
>
> SBE schemas are not yet stable and may break between releases.

## NTPRO scope

NTPRO is an open-source, Rust-native
engine for multi-asset, multi-venue trading systems.

The system spans research, deterministic simulation, and live execution within a single
event-driven architecture, providing research-to-live semantic parity.

## Feature flags

This crate provides feature flags to control source code inclusion during compilation:

- `high-precision`: Enables [high-precision mode](https://github.com/atxinbao/NTPRO/blob/main/docs/getting_started/installation.md#precision-mode) to use 128-bit value types.
- `arrow`: Enables Apache Arrow schema definitions and RecordBatch encoding/decoding.
- `display`: Enables display-friendly Arrow encoders for market data (requires `arrow`).
- `sbe`: Enables generic SBE (Simple Binary Encoding) decode utilities.

## Serialization format comparison

This crate supports these serialization formats for market data types. Choose the format based on your use case:

| Format       | Serialize | Deserialize | Size      | Use case                                    |
|--------------|-----------|-------------|-----------|---------------------------------------------|
| JSON         | ~332ns    | ~779ns      | 174 bytes | Human-readable output, debugging, APIs.     |
| MsgPack      | ~375ns    | ~634ns      | 134 bytes | Compact storage, network transmission.      |
| Arrow        | TBD       | TBD         | Columnar  | Batch processing, Parquet, IPC, analytics.  |

Performance numbers shown for `QuoteTick` serialization (measured on AMD Ryzen 9 7950X).
MsgPack offers the smallest size among the compact row formats, while Arrow is optimized
for batch processing rather than individual messages.

### Usage examples

#### JSON serialization

```rust
use nautilus_core::serialization::Serializable;
use nautilus_model::data::QuoteTick;

let quote = QuoteTick { /* ... */ };

// Serialize to JSON
let json_bytes = quote.to_json_bytes()?;

// Deserialize from JSON
let decoded = QuoteTick::from_json_bytes(&json_bytes)?;
```

#### MsgPack serialization

```rust
use nautilus_core::serialization::{ToMsgPack, FromMsgPack};
use nautilus_model::data::QuoteTick;

let quote = QuoteTick { /* ... */ };

// Serialize to MsgPack
let msgpack_bytes = quote.to_msgpack_bytes()?;

// Deserialize from MsgPack
let decoded = QuoteTick::from_msgpack_bytes(&msgpack_bytes)?;
```

## Benchmarking

This crate has two benchmark tracks:

- `serialization_comparison` compares JSON and MsgPack for a smaller set of types.
- `sbe_decoding` measures SBE cursor decode utilities.

### format comparison benchmarks

Run benchmarks to compare JSON and MsgPack:

```bash
# Compare row formats for QuoteTick
cargo bench -p nautilus-serialization --bench serialization_comparison -- QuoteTick

# Compare row formats for TradeTick
cargo bench -p nautilus-serialization --bench serialization_comparison -- TradeTick

# Compare row formats for Bar
cargo bench -p nautilus-serialization --bench serialization_comparison -- Bar

# Run all comparison benchmarks
cargo bench -p nautilus-serialization --bench serialization_comparison
```

### SBE benchmarks

Run the SBE microbenchmarks:

```bash
# Run SBE cursor decode microbenchmarks
cargo bench -p nautilus-serialization --no-default-features --features sbe --bench sbe_decoding
```

## Documentation

See [the docs](https://docs.rs/nautilus-serialization) for more detailed usage.

## License

The NTPRO workspace retains NautilusTrader license lineage and is available under the [GNU Lesser General Public License v3.0](https://www.gnu.org/licenses/lgpl-3.0.en.html).

---

NTPRO is a Rust-only release workspace derived from NautilusTrader. It retains NautilusTrader license lineage; review the repository license files and release notes before distribution or operational use.

© 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
