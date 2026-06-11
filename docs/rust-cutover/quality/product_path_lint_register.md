# Product Path Lint Register

Date: 2026-06-11
Executor: Codex
Task: P1-003

## Purpose

This register starts the warning-only rollout for product-reachable panic and
silent-result lints. The goal is to make risky patterns visible in the Rust
product path without turning every existing legacy hit into a blocking failure
in the first pass.

## Initial Product Path Scope

The first rollout applies to the Rust product entry and local runtime crates:

- `nautilus-cli`
- `nautilus-live`
- `nautilus-backtest`
- `nautilus-sandbox`

The package list is centralized in `scripts/ai/check_product_path_lints.sh` and
can be overridden with `NTPRO_PRODUCT_LINT_PACKAGES` for local investigation.
The script intentionally does not pass the workspace feature set by default:
the CI smoke workflow already runs workspace `cargo check` and clippy with
`NAUTILUS_RUST_FEATURES`. This product-path script is an additional
default-product-path lint sweep, so it avoids applying workspace-only features
to packages that do not define them.

## Warning Lints

| Lint | Rollout level | Reason | Blocking policy |
| --- | --- | --- | --- |
| `clippy::unwrap_used` | warn | Avoid unhandled panic paths in product-facing runtime/control code. | Warning-only until scoped product crates have clean baselines. |
| `clippy::expect_used` | warn | Avoid panic paths with assumed invariants at product input/runtime boundaries. | Warning-only until scoped product crates have clean baselines. |
| `clippy::indexing_slicing` | warn | Avoid unchecked indexing and slicing in product-reachable code. | Warning-only until indexed paths are classified. |
| `clippy::unused_result_ok` | warn | Avoid silently discarding errors by calling `.ok()` on `Result`. | Warning-only until existing intentional suppressions are documented. |

## Follow-up Register

| ID | Scope | Status | Close condition |
| --- | --- | --- | --- |
| `PPL-001` | `nautilus-cli` product command paths | Open | Product CLI command implementations either avoid these lint hits or document local `expect`/indexing invariants with targeted tests. |
| `PPL-002` | `nautilus-live` node/control paths | Open | Live node control, stop, startup, and supervisor-facing code has no product-boundary panic lint hits or documented exceptions. |
| `PPL-003` | `nautilus-backtest` runtime entry paths | Open | Backtest product entry paths remove or classify panic/indexing/result-discard hits. |
| `PPL-004` | `nautilus-sandbox` local demo adapter paths | Open | Sandbox runtime paths either remove hits or classify test-only/domain-invariant exceptions. |

## Current Policy

- CI runs workspace feature `cargo check` and clippy before this script.
- CI then runs `scripts/ai/check_product_path_lints.sh` for heavy Rust PRs.
- The script emits warnings but does not fail on existing hits.
- Local investigations can pass extra cargo arguments with
  `NTPRO_PRODUCT_LINT_EXTRA_CARGO_ARGS`.
- Existing hits must be handled by follow-up cleanup tasks, not drive-by rewrites
  in unrelated PRs.
- Once a scoped package is cleaned, that package may be moved from warning-only
  to deny in a dedicated PR with evidence.

## Non-goals

- This register does not claim the full workspace is clean for these lints.
- This register does not require rewriting every existing panic or indexing site
  in P1-003.
- This register does not change runtime behavior or trading semantics.
