# Rust Data and Catalog Examples

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-010

This directory is reserved for Rust-first data/catalog examples driven by the
`nautilus data` CLI.

## Command Contract

```bash
cargo run -q -p nautilus-cli -- data inspect --config examples/rust/data/catalog_audit.toml
cargo run -q -p nautilus-cli -- data validate --config examples/rust/data/catalog_audit.toml
cargo run -q -p nautilus-cli -- data load --config examples/rust/data/load_quotes.toml --run-id load-quotes --output runs/load-quotes
```

## Current Blocker

`data inspect`, `data validate`, and `data load` parse and expose help, but
execution returns an explicit blocker until Rust config parsing and catalog
inspection/loading wiring are implemented.

## Required Evidence For First Runnable Example

- The example uses a local fixture, local catalog, or adapter replay path with
  explicit adapter evidence.
- Unsupported data types and missing intervals produce explicit errors.
- Load writes only to the configured catalog target.
- The command does not import Python, require PyO3, or require Cython artifacts.
