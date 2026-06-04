# CLI Help Contract

Date: 2026-06-04
Executor: Codex
Task ID: RHARD-003

## Purpose

This document records the current Rust CLI help-level product contract for
NTPRO v0.2.0 hardening. It separates commands that have stable `--help` output
from commands whose runtime behavior is still intentionally deferred.

Help-level support means the command exists, parses, prints help successfully,
and can be used as a stable user-visible entrypoint. It does not mean the
runtime workflow is fully implemented.

## Default Help Surface

The default `nautilus` binary exposes these top-level commands:

```text
nautilus
  backtest
  sandbox
  live
  data
  config
  database
```

`blockchain` remains behind the `defi` feature and is not part of the default
RHARD-003 help contract.

## Contract Matrix

| Command | Help status | Runtime status | Notes |
| --- | --- | --- | --- |
| `nautilus --help` | supported | n/a | Lists `backtest`, `sandbox`, `live`, `data`, `config`, and `database`. |
| `nautilus backtest --help` | supported | deferred | Exposes `validate` and `run`; execution returns explicit blocker messages. |
| `nautilus backtest validate --help` | supported | deferred | Requires `--config <CONFIG>`. |
| `nautilus backtest run --help` | supported | deferred | Requires `--config <CONFIG>` and accepts `--run-id` plus `--output`. |
| `nautilus sandbox --help` | supported | deferred | Exposes `validate` and `run`; execution returns explicit blocker messages. |
| `nautilus sandbox validate --help` | supported | deferred | Requires `--config <CONFIG>`. |
| `nautilus sandbox run --help` | supported | deferred | Requires `--config <CONFIG>` and accepts `--run-id` plus `--output`. |
| `nautilus live --help` | supported | deferred | Exposes `validate` and `run`; execution returns explicit blocker messages. |
| `nautilus live validate --help` | supported | deferred | Requires `--config <CONFIG>`. |
| `nautilus live run --help` | supported | deferred | Requires `--config <CONFIG>` and accepts `--run-id` plus `--output`. |
| `nautilus data --help` | supported | deferred | Exposes `inspect`, `validate`, and `load`; execution returns explicit blocker messages. |
| `nautilus data inspect --help` | supported | deferred | Requires `--config <CONFIG>` and accepts `--output`. |
| `nautilus data validate --help` | supported | deferred | Requires `--config <CONFIG>`. |
| `nautilus data load --help` | supported | deferred | Requires `--config <CONFIG>` and accepts `--run-id` plus `--output`. |
| `nautilus config --help` | supported | deferred | Exposes `validate`; execution returns explicit blocker messages. |
| `nautilus config validate --help` | supported | deferred | Requires `--kind <backtest\|sandbox\|live\|data>` and `--config <CONFIG>`, accepts `--output`. |
| `nautilus database --help` | supported | supported | Existing Postgres operations remain part of the Rust CLI surface. |
| `nautilus database init --help` | supported | supported | Accepts optional Postgres connection and schema flags. |
| `nautilus database drop --help` | supported | supported | Accepts optional Postgres connection and schema flags. |

## Deferred Behavior

These product commands are help-stable but not runtime-complete:

- `backtest validate`;
- `backtest run`;
- `sandbox validate`;
- `sandbox run`;
- `live validate`;
- `live run`;
- `data inspect`;
- `data validate`;
- `data load`;
- `config validate`.

They intentionally return owner-visible blocker errors from Rust code. Later
v0.2.0 tasks should replace those blockers with scoped Rust implementations
only when config parsing, runtime wiring, adapter classification, and evidence
requirements are ready.

## Missing or Out of Scope

The following are not part of the RHARD-003 completion criteria:

- full backtest, sandbox, live, data, and config runtime behavior;
- machine-readable CLI output;
- release binaries or installers;
- `defi` feature-gated blockchain help in the default build;
- Python CLI entrypoints.

Missing runtime behavior must be tracked as follow-up product/runtime tasks,
not bypassed by Python fallback behavior.

## Verification

RHARD-003 captured `--help` output for every command in the contract matrix and
added parser coverage for the database command surface. Evidence is recorded in
`docs/rust-cutover/evidence/RHARD-003.md`.
