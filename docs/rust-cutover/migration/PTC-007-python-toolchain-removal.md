# PTC-007 Python Toolchain Removal

Date: 2026-07-17
Executor: Codex

## Decision

NTPRO repository development and validation now use Rust, Cargo, shell, jq,
and pinned native tools. Python package management, test runners, wheels,
sdists, PyPI publication, legacy package containers, and Python-based
pre-commit environments are retired rather than carried as no-op targets.

## Supported Replacements

| Retired surface | Current authority |
| --- | --- |
| uv dependency sync | `cargo fetch --locked` |
| Python formatting and lint | `cargo fmt` and `cargo clippy` |
| pytest and coverage scripts | Cargo tests, nextest, and cargo-llvm-cov |
| wheel/sdist builds | Rust CLI binary workflow |
| pip/PyPI audit | cargo-audit, cargo-deny, cargo-vet, and OSV Scanner |
| Python pre-commit hooks | local Rust/system prek hooks |
| inline JSON helpers | jq and Rust governance commands |
| package Docker images | local Postgres/Redis integration services only |

## Boundaries

Historical `docs/rust-cutover/` records remain unchanged and may describe the
retired surfaces. The v0.32.0 release baseline is untouched. This migration
does not add runtime capability or authorize submit, mutation, adapter send,
live exchange, retry, remediation, or trading controls.
