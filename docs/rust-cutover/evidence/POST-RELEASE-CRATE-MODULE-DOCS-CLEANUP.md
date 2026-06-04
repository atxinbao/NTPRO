# Post-release Crate Module Docs Cleanup Evidence

- Date: 2026-06-05
- Executor: Codex
- Local task name: `POST-RELEASE-CRATE-MODULE-DOCS-CLEANUP`
- Formal task file: not present in the Shrimp queue; Shrimp reported 0 pending
  and 0 in-progress tasks before this slice started.

## Goal

Clean docs.rs-facing crate module documentation in `crates/**/src/lib.rs` so the
public crate pages identify the workspace as NTPRO instead of linking users to
the upstream NautilusTrader site.

This is a documentation-only slice. It does not change runtime behavior, Cargo
features, crate names, binary names, ABI symbol names, dependency graphs, adapter
behavior, or trading semantics.

## Tooling Notes

- `mcp__shrimp_task_manager.list_tasks` showed 0 pending and 0 in-progress tasks.
- `mcp__shrimp_task_manager.analyze_task` was run for this local cleanup slice.
- The Shrimp analysis response requested `reflect_task`, but that tool was not
  exposed in the current MCP tool list, so this PR records the decision and
  proceeds as a scoped post-release cleanup.

## Scope

Changed:

- 39 files under `crates/**/src/lib.rs`.
- Module-level `//!` documentation that linked to `nautilustrader.io`.
- Module-level high-precision documentation links.
- Plain module-doc product wording such as `NautilusTrader ecosystem`,
  `NautilusTrader applications`, and `Nautilus host`.

Not changed:

- copyright/license headers containing `https://nautechsystems.io`;
- crate names such as `nautilus-model`;
- constants and symbols such as `NAUTILUS_PLUGIN_INIT_SYMBOL`;
- non-`lib.rs` source files;
- integration/how-to docs;
- runtime implementation.

## Files Changed

The modified files are the crate root module docs for:

- adapters: Architect AX, Betfair, Binance, BitMEX, Blockchain, Bybit, Coinbase,
  Databento, Deribit, dYdX, Hyperliquid, Interactive Brokers, Kraken, OKX,
  Polymarket, Sandbox, Tardis;
- core workspace crates: analysis, backtest, cli, common, core, cryptography,
  data, event_store, execution, indicators, infrastructure, live, model,
  network, persistence, plugin, portfolio, risk, serialization, system,
  testkit, trading.

## Validation

| Command | Result | Notes |
|---------|--------|-------|
| `scripts/ai/check_rust_only_runtime.sh` | passed | Reported `rust-only-runtime: ok`. |
| `scripts/ai/check_cython_removed.sh` | passed | Reported `cython-removed: ok`. |
| `scripts/ai/verify_fast.sh` | passed | Used cargo/rustc 1.95.0. Workspace cargo check and clippy were skipped by fast-mode defaults. |
| `source scripts/ai/toolchain_env.sh && cargo doc --workspace --no-deps` | passed | Finished dev profile in 19m 06s; generated workspace docs. |
| `rg -n "NautilusTrader|nautilustrader\\.io|nautechsystems/nautilus_trader|High-precision mode\\]\\(https://nautilustrader\\.io" crates/**/src/lib.rs` | passed | No matches. |
| `rg -n "Python control plane|PyO3|Cython|uv pip install|nautechsystems/nautilus_trader|nautilustrader.io" README.md docs crates Cargo.toml` | completed | Match count is classified below instead of requiring zero hits. |

## `rg` Classification

Before this evidence file was added, the full command reported 1245 matching
lines. Pattern counts at that point were:

| Pattern | Files | Matches | Classification |
|---------|-------|---------|----------------|
| `Python control plane` | 1 | 2 | retained only in prior evidence text |
| `PyO3` | 305 | 911 | migration/history/legacy warnings plus old deep docs |
| `Cython` | 293 | 803 | migration/history/legacy warnings plus old deep docs |
| `uv pip install` | 8 | 11 | unsupported/migration notes plus old integration docs outside this PR |
| `nautechsystems/nautilus_trader` | 37 | 61 | old tutorial/integration/source-code links plus prior evidence classification |
| `nautilustrader.io` | 3 | 5 | one non-`lib.rs` source diagnostic, one Tardis integration note, and prior evidence classification |

### Cleaned

- `crates/**/src/lib.rs` no longer contains `NautilusTrader`,
  `nautilustrader.io`, `nautechsystems/nautilus_trader`, or upstream
  high-precision documentation links.
- docs.rs-facing crate root pages now describe the product as NTPRO.
- High-precision links now point to the NTPRO repository documentation.

### Reasonably Retained

- Copyright/license headers still contain `https://nautechsystems.io` because
  this project retains NautilusTrader license lineage.
- Existing `NAUTILUS_*` ABI constants and crate names are unchanged because this
  PR is not a binary/API renaming task.
- Prior evidence files retain historical counts and classifications.

### Follow-up Cleanup

- `crates/serialization/src/arrow/mod.rs` still contains a non-`lib.rs`
  diagnostic link to upstream precision documentation. That should be handled
  in a non-root source-doc cleanup slice.
- `docs/integrations/tardis.md` still contains an upstream docs link. Integration
  docs need adapter-specific cleanup so support status remains accurate.
- `docs/integrations/**`, `docs/how_to/**`, and deep concept pages still contain
  legacy upstream links or Python/PyO3/Cython context outside this PR.

## Behavior Impact

No runtime behavior changed. This PR only changes Rust module documentation.

## Public API Impact

No Rust public API changed. Crate names, feature flags, ABI constants, exported
symbols, functions, structs, and modules are unchanged.

## Migration Note Status

The migration posture is unchanged: NTPRO is the Rust-only product surface, while
Python/PyO3/Cython/upstream links may remain only as legacy, migration, historical,
or explicitly deferred follow-up context.

## Rollback Plan

Revert this PR to restore the previous crate module documentation. No runtime,
dependency, data, or API migration is required.
