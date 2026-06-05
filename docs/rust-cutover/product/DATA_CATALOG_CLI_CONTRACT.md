# Data and Catalog CLI Contract

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-007

Updated: 2026-06-05
Executor: Codex
Task ID: GH-156

Updated: 2026-06-06
Executor: Codex
Task ID: DRG-005

## Purpose

This contract refines the `nautilus data` surface from
`docs/rust-cutover/product/RUST_CLI_CONTRACT.md`. It defines the Rust-first
data/catalog command shape, config boundary, inspect and validate workflows,
load workflow, output contract, and known blockers for later implementation
tasks.

This document started as a product contract. GH-156 implements the first
local-file Rust path for `data inspect` and `data validate`. DRG-005 implements
the first scoped local fixture Rust path for `data load`.

## Current Baseline

`nautilus-cli` exposes `data` by default. After GH-156, `data inspect` and
`data validate` execute a scoped local-file/local-directory Rust path. They
parse the TOML config, validate the query shape, reject unsupported data types,
and inspect the configured `catalog.path` for existence, readability, size,
extension, and directory entries.

`data load` supports `source.kind = "fixture"` with
`source.schema = "quote_tick_csv_v1"` and `mapping.data_type = "QuoteTick"`.
It validates the CSV header and copies the fixture into the configured catalog
directory with an owner-visible summary artifact.

## Command Surface

The Rust-first data command must expose:

```text
nautilus data inspect --config <path> [--output <dir>]
nautilus data validate --config <path>
nautilus data load --config <path> [--run-id <id>] [--output <dir>]
```

`inspect`, `validate`, and `load` must parse the same config format. `inspect`
and `validate` must never run a strategy, start a live node, or connect to a
production venue. `load` must write only to the configured catalog target and
must use a scoped fixture, file, or adapter path with explicit adapter evidence.

Current GH-156 scope:

- `inspect` and `validate` support only `catalog.protocol = "file"`;
- `catalog.path` may point to a local file or directory;
- relative paths are first interpreted from the current working directory when
  they exist, otherwise relative to the config file directory;
- supported `data_type` values are the built-in data types listed in
  [Data Type Scope](#data-type-scope);
- no row decoding, Parquet schema validation, catalog interval lookup, adapter
  access, or external data source connection is performed.

Current DRG-005 load scope:

- `source.kind = "fixture"` only;
- `source.schema = "quote_tick_csv_v1"` only;
- `mapping.data_type = "QuoteTick"` only;
- `mapping.timestamp_column = "ts_init"` only;
- the CLI copies the fixture into the configured catalog directory and writes
  `summary.txt`;
- no adapter access, no Parquet row encoding, no row-level semantic decode, and
  no interval availability proof is performed.

## Inspect Workflow

`data inspect` reports catalog/source metadata without loading data into a
runtime engine.

Required checks:

- config file exists and parses;
- catalog path or source path is present;
- catalog storage protocol is supported by Rust code;
- requested data types are known built-in Nautilus data types or explicitly
  registered custom data types;
- requested instrument IDs, bar types, and identifiers are syntactically valid;
- requested time range is syntactically valid when present.

Required output:

- command name;
- config path;
- catalog path or source path;
- storage protocol;
- configured query count;
- data types requested;
- instrument or identifier filters;
- time range filters;
- discovered catalog directories or source files where available;
- owner-visible blocker when discovery cannot be completed.

GH-156 implements the local metadata subset: existence/readability, file size,
file extension, directory entry count, first discovered directory entries, query
count, requested data types, and configured query filters.

## Validate Workflow

`data validate` verifies that a catalog or configured source can satisfy the
requested data window before another command uses it.

Required checks:

- every `inspect` check passes;
- catalog path exists for local catalogs;
- each requested data type maps to a Rust catalog prefix;
- each requested instrument ID or identifier maps to the data type query shape;
- `start_time` is earlier than `end_time` when both are provided;
- requested intervals can be queried or missing intervals can be reported;
- `bar_spec` and `bar_types` are not contradictory;
- remote storage options are declared through config or environment variables
  without printing sensitive values;
- custom data is rejected unless a Rust custom data registry entry is present.

Allowed Rust integration points:

- `nautilus_persistence::backend::catalog::ParquetDataCatalog`;
- `nautilus_backtest::config::BacktestDataConfig`;
- `nautilus_backtest::node::BacktestNode::load_catalog`;
- `nautilus_data` catalog request and streaming boundaries when scoped by a
  runtime task;
- `nautilus_model::data::CatalogPathPrefix` and built-in data type mappings.

`validate` must not silently accept an unsupported source, unsupported data
type, missing catalog path, or unclassified adapter path.

GH-156 implements the local file/directory subset. It rejects missing catalog
paths, unreadable catalog paths, unsupported data types, malformed query shape,
and invalid time range ordering. It does not prove that the catalog can satisfy
every requested interval at the row level.

## Load Workflow

`data load` converts scoped input data into the configured catalog target.

Supported source classes must be introduced incrementally:

- fixture files committed to the repository;
- local CSV or Parquet files with documented schema mapping;
- adapter replay outputs that have adapter evidence;
- remote object-store inputs only after storage options and fixture validation
  are documented.

`load` must validate before writing. DRG-005 writes the source fixture into the
target catalog directory as the first Rust-only catalog bootstrap path. Later
tasks may promote Parquet writes through Rust catalog APIs such as
`ParquetDataCatalog::write_to_parquet`.

The first implementation may support only one source class and one data type,
but it must reject all unsupported combinations explicitly. It must not fall
back to Python wranglers, Python catalog code, PyO3, or Cython artifacts.

## Config Format

The first implementation should use TOML, matching the backtest, sandbox, and
live CLI contracts. JSON can be added later only as an explicit extension.

Minimum inspect/validate TOML shape:

```toml
[run]
id = "catalog-audit"
mode = "inspect"

[catalog]
path = "catalog/backtests/ema-cross"
protocol = "file"
batch_size = 5000

[[queries]]
data_type = "QuoteTick"
instrument_id = "AUD/USD.SIM"
start_time = "2025-01-01T00:00:00Z"
end_time = "2025-01-02T00:00:00Z"

[[queries]]
data_type = "Bar"
bar_type = "AUD/USD.SIM-1-MINUTE-BID-EXTERNAL"
start_time = "2025-01-01T00:00:00Z"
end_time = "2025-01-02T00:00:00Z"

[output]
dir = "runs/catalog-audit"
format = "text"
write_summary = true
```

Minimum load TOML shape:

```toml
[run]
id = "load-quotes"
mode = "load"

[catalog]
path = "catalog/backtests/ema-cross"
protocol = "file"
batch_size = 5000
compression = "snappy"

[source]
kind = "fixture"
path = "tests/fixtures/data/quotes.csv"
schema = "quote_tick_csv_v1"

[mapping]
data_type = "QuoteTick"
instrument_id = "AUD/USD.SIM"
timestamp_column = "ts_init"

[output]
dir = "runs/load-quotes"
write_summary = true
```

### Field Mapping

`run` identifies the workflow and maps to an owner-visible run ID.

`catalog` maps to `ParquetDataCatalog` construction. Local paths are the first
supported target. Remote URI support must be documented with storage options.

`queries` maps to `BacktestDataConfig`-style catalog queries:

- `data_type` maps to `NautilusDataType` or a scoped custom data type;
- `instrument_id` or `instrument_ids` maps to the query identifier;
- `bar_type` maps to explicit bar type queries;
- `start_time` and `end_time` map to query intervals;
- `filter_expr` can be added later only when the Rust catalog query path
  supports it.

`source` identifies load input. Adapter-backed sources remain under adapter
tasks and must not be assumed supported by this contract alone.

`mapping` defines how a source record becomes a Nautilus model data type. Each
mapping must name the Rust data type and schema contract used by the loader.

`output` controls owner-visible artifacts and must not affect market data
semantics.

## Data Type Scope

The initial implementation should classify every requested data type as one of:

- supported;
- blocked pending Rust loader or query implementation;
- blocked pending adapter evidence;
- blocked pending custom data registry.

Built-in data types that may be considered first:

- `QuoteTick`;
- `TradeTick`;
- `Bar`;
- `OrderBookDelta`;
- `OrderBookDepth10`;
- `InstrumentAny`;
- `FundingRateUpdate`.

Unsupported data types must be rejected explicitly instead of ignored.

## Output Contract

`inspect` must print or write:

- command name;
- run ID when configured;
- config path;
- catalog path or source path;
- requested data types;
- discovered catalog prefixes or source files;
- query filters;
- final status.

`validate` must print a concise success or failure summary:

```text
data.validate status=ok config=<path> queries=<n>
```

`load` must print or write:

- command name;
- run ID;
- config path;
- source path or adapter source;
- target catalog path;
- data type loaded;
- row or record count when known;
- output directory;
- started timestamp;
- completed or failed timestamp;
- final status.

Human-readable text is enough for the initial implementation. Machine-readable
JSON output can be added later as an explicit `--format json` option.

## Failure Behavior

The commands must use stable non-zero exits so automation can distinguish user
errors from catalog and loader failures.

Recommended exit codes:

- `2`: CLI usage or argument parse error;
- `3`: config parse or validation error;
- `4`: catalog path, source path, or storage backend unavailable;
- `5`: unsupported data type or identifier shape;
- `6`: requested data interval unavailable;
- `7`: source decode or mapping error;
- `8`: catalog write error;
- `9`: output artifact write error.

Every failure must name the failing section or operation when known.
Unsupported config sections must be rejected explicitly instead of ignored.

## Implementation Gates

The inspect/validate local-file path is considered usable when all of the
following pass:

```bash
cargo run -q -p nautilus-cli -- data --help
cargo run -q -p nautilus-cli -- data inspect --help
cargo run -q -p nautilus-cli -- data validate --help
cargo run -q -p nautilus-cli -- data load --help
cargo run -q -p nautilus-cli -- data load --config examples/rust/data/load_quotes.toml --output runs/load-quotes
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
```

The first successful data/catalog smoke must also prove:

- no Python import is required;
- no PyO3 or Cython build artifact is required;
- a local catalog or fixture source can be inspected from Rust;
- unsupported data types and missing intervals produce explicit blockers;
- load writes only to the configured catalog target and owner-visible output.

## Remaining Blockers

- Full Parquet/catalog row decoding and interval availability checks are not
  implemented in this CLI path.
- A shared Rust workflow config model is not yet complete across all commands.
- Adapter-backed data loading remains under adapter evidence.
- Custom data loading requires an explicit Rust custom data registry path.

These blockers should be closed by later RPROD/RADP/RCORE/RTRACE tasks, not
bypassed by Python fallback behavior.
