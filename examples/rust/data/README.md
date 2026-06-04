# Rust Data and Catalog Examples

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-010

Updated: 2026-06-05
Executor: Codex
Task ID: GH-156

This directory is reserved for Rust-first data/catalog examples driven by the
`nautilus data` CLI.

## Command Contract

```bash
cargo run -q -p nautilus-cli -- data inspect --config examples/rust/data/catalog_audit.toml
cargo run -q -p nautilus-cli -- data validate --config examples/rust/data/catalog_audit.toml
cargo run -q -p nautilus-cli -- data load --config examples/rust/data/load_quotes.toml --run-id load-quotes --output runs/load-quotes
```

## Current Status

`data inspect` and `data validate` now run a scoped local file/directory
metadata path. They parse TOML config, validate query shape, reject unsupported
data types, and inspect the configured `catalog.path` for existence,
readability, file size, extension, and directory entries.

`data load` still parses and exposes help, but execution returns an explicit
blocker until Rust catalog loading is implemented.

## Required Evidence For First Runnable Example

- The example uses a local fixture, local catalog, or adapter replay path with
  explicit adapter evidence.
- Unsupported data types and missing intervals produce explicit errors.
- Load writes only to the configured catalog target when a later load task
  implements it.
- The command does not import Python, require PyO3, or require Cython artifacts.
