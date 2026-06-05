# CLI Help Contract

Date: 2026-06-04
Executor: Codex
Task ID: RHARD-003

Updated: 2026-06-06
Executor: Codex
Task ID: DRG-005

## Purpose

This document records the current Rust CLI help-level product contract for
NTPRO v0.2.0 hardening. It separates commands that have stable `--help` output
from commands whose runtime behavior is still intentionally deferred.

Help-level support means the command exists, parses, prints help successfully,
and can be used as a stable user-visible entrypoint. It does not mean the
runtime workflow is fully implemented.

Runtime capability status is tracked in
`docs/rust-cutover/product/CLI_CAPABILITY_MATRIX.md` using the fixed statuses
`implemented`, `simulated_demo`, `metadata_only`, and `deferred`.

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
| `nautilus backtest --help` | supported | mixed | Exposes `validate` and `run`; see the capability matrix for dry-run vs full-run status. |
| `nautilus backtest validate --help` | supported | metadata_only | Requires `--config <CONFIG>`. |
| `nautilus backtest run --help` | supported | metadata_only / implemented | Requires `--config <CONFIG>` and accepts `--run-id`, `--output`, and `--dry-run`; RHARD-006 supports metadata-only dry-run and DRG-005 supports the scoped engine-smoke run. |
| `nautilus sandbox --help` | supported | simulated_demo | Exposes `validate` and `run` for the local simulated demo. |
| `nautilus sandbox validate --help` | supported | simulated_demo | Requires `--config <CONFIG>`; validates the RHARD-004 demo config. |
| `nautilus sandbox run --help` | supported | simulated_demo | Requires `--config <CONFIG>` and accepts `--run-id` plus `--output`; writes simulated demo artifacts without starting a real `LiveNode`. |
| `nautilus live --help` | supported | implemented | Exposes `validate` and `run` for the scoped live-init sandbox smoke path. |
| `nautilus live validate --help` | supported | implemented | Requires `--config <CONFIG>` and validates the live-init smoke TOML boundary. |
| `nautilus live run --help` | supported | implemented | Requires `--config <CONFIG>` and accepts `--run-id` plus `--output`; starts/stops a sandbox `LiveNode` without external venue access. |
| `nautilus data --help` | supported | metadata_only / implemented | Exposes `inspect`, `validate`, and `load`; GH-156 supports local file/directory inspect/validate and DRG-005 supports local QuoteTick fixture load. |
| `nautilus data inspect --help` | supported | metadata_only | Requires `--config <CONFIG>` and accepts `--output`; inspects local catalog metadata. |
| `nautilus data validate --help` | supported | metadata_only | Requires `--config <CONFIG>`; validates local catalog readability and query shape. |
| `nautilus data load --help` | supported | implemented | Requires `--config <CONFIG>` and accepts `--run-id` plus `--output`; loads a local QuoteTick fixture into a catalog directory. |
| `nautilus config --help` | supported | supported | Exposes `validate`. |
| `nautilus config validate --help` | supported | supported | Requires `--kind <backtest\|sandbox\|live\|data>` and `--config <CONFIG>`, accepts `--output`. |
| `nautilus database --help` | supported | supported | Existing Postgres operations remain part of the Rust CLI surface. |
| `nautilus database init --help` | supported | supported | Accepts optional Postgres connection and schema flags. |
| `nautilus database drop --help` | supported | supported | Accepts optional Postgres connection and schema flags. |

## Runtime Boundary

`backtest validate` and `backtest run --dry-run` now support the RHARD-006
metadata-only minimal path. `sandbox validate` and `sandbox run` now support the
RHARD-004 local simulated demo path. `config validate` now supports a scoped
Rust TOML validation path for backtest, sandbox, live-smoke, and data/catalog
configs. `data inspect` and `data validate` now support the GH-156 local
file/directory metadata path. DRG-005 adds the scoped non-dry-run backtest
engine-smoke path, live-init sandbox start/stop path, and local QuoteTick
fixture load path.

These paths are not full trading workflow claims. Later v0.2.0 tasks must still
promote arbitrary strategy loading, production live adapters, adapter-backed
data load, and full catalog row decoding with separate evidence.

## Missing or Out of Scope

The following are not part of the RHARD-003 completion criteria:

- full backtest, sandbox, live, and data runtime behavior;
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
