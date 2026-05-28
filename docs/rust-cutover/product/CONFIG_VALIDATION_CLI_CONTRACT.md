# Config Validation CLI Contract

Date: 2026-05-28
Executor: Codex
Task ID: RPROD-009

## Purpose

This contract defines the shared Rust-first config validation entrypoint for
NTPRO product workflows. It complements the workflow-local validation commands:

```text
nautilus backtest validate --config <path>
nautilus sandbox validate --config <path>
nautilus live validate --config <path>
nautilus data validate --config <path>
```

The shared command is useful for automation that wants one stable validation
surface before selecting the runtime workflow.

## Command Surface

The Rust-first config command must expose:

```text
nautilus config validate --kind <backtest|sandbox|live|data> --config <path> [--output <dir>]
```

`--kind` selects the workflow config contract. `--config` points to the config
file to validate. `--output` can write owner-visible validation artifacts when a
later implementation adds report generation.

## Validation Contract

`config validate` must parse and validate the same config shape as the matching
workflow command:

- `--kind backtest` maps to `nautilus backtest validate`;
- `--kind sandbox` maps to `nautilus sandbox validate`;
- `--kind live` maps to `nautilus live validate`;
- `--kind data` maps to `nautilus data validate`.

The command must not import Python, call Python package code, require PyO3, or
require Cython build artifacts. It must reject unsupported config sections
explicitly instead of silently ignoring them.

## Output Contract

`config validate` must print a concise success or failure summary:

```text
config.validate status=ok kind=<kind> config=<path>
```

When validation fails, the error must name the failing section and field when
known. Human-readable text is enough for the initial implementation.
Machine-readable JSON output can be added later as an explicit format option.

## Failure Behavior

Recommended exit codes:

- `2`: CLI usage or argument parse error;
- `3`: config file missing, unreadable, or parse error;
- `4`: unsupported workflow kind or unsupported config section;
- `5`: workflow-specific validation error;
- `6`: output artifact write error.

## Implementation Gates

The command is not considered usable until all of the following pass:

```bash
cargo run -q -p nautilus-cli -- config --help
cargo run -q -p nautilus-cli -- config validate --help
cargo run -q -p nautilus-cli -- config validate --kind backtest --config config/backtest.toml
cargo run -q -p nautilus-cli -- config validate --kind sandbox --config config/sandbox.toml
cargo run -q -p nautilus-cli -- config validate --kind live --config config/live.toml
PATH="/opt/homebrew/opt/rustup/bin:$PATH" scripts/ai/verify_fast.sh
```

The first non-blocker implementation must also prove:

- no Python import is required;
- no PyO3 or Cython build artifact is required;
- the selected `--kind` maps to a Rust config model;
- failures report owner-visible section and field details.

## Known Blockers

- A shared Rust CLI TOML config parser has not been added.
- Backtest, sandbox, live, and data config models are not yet unified behind a
  shared CLI validation trait.
- Workflow-local validate commands currently expose the parser surface but
  intentionally return blockers.

These blockers should be closed by later RPROD/RCORE tasks, not bypassed by
Python fallback behavior.
