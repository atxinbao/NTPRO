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
file to validate. `--output` writes an owner-visible `validation.txt` artifact
for automation and review evidence.

## Validation Contract

`config validate` must parse and validate the same config shape as the matching
workflow command:

- `--kind backtest` maps to `nautilus backtest validate`;
- `--kind sandbox` maps to `nautilus sandbox validate`;
- `--kind live` validates the Rust live init smoke TOML boundary without
  starting a node;
- `--kind data` validates the Rust data/catalog TOML boundary without
  inspecting, loading, or querying a catalog.

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

When `--output <dir>` is provided, the command writes:

```text
<dir>/validation.txt
```

with the command name, status, selected kind, and config path.

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

- Backtest and sandbox validation reuse their current minimal Rust CLI config
  parsers. They are not yet unified behind a shared validation trait.
- Live validation is limited to the Rust live init smoke TOML boundary and does
  not start a node, classify production adapters, or connect to any venue.
- Data validation is limited to the Rust data/catalog TOML boundary and does
  not inspect catalog contents, load source data, or query storage.

These blockers should be closed by later product/runtime tasks. They must not
be bypassed with Python, PyO3, or Cython fallback behavior.
