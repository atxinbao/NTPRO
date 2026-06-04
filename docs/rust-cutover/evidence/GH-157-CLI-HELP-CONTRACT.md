# GH-157 - CLI help contract cleanup evidence

Date: 2026-06-05
Executor: Codex

## Task

- GitHub issue: <https://github.com/atxinbao/NTPRO/issues/157>
- Branch: `codex/audit-cli-help-contract`
- Owner role: Rust Product Surface Agent
- Review role: Verification & Release Gatekeeper
- Risk level: Medium

## Plain Chinese summary

这次只改 CLI 的帮助文案和对应测试。以前 `live run --help`、
`data inspect --help`、`config validate --help` 这些命令看起来像已经能跑
真实流程，但执行时会提示还没实现。现在 help 会直接说明这些命令目前是
contract/validation 边界，真实 runtime wiring 还没实现。`backtest run`
也明确只有 metadata-only `--dry-run` 是当前已实现的最小路径。

## Goal

Align user-visible CLI help with the current implementation boundary for
post-release audit issue #157.

## Files changed

- `crates/cli/src/opt.rs`
- `docs/rust-cutover/evidence/GH-157-CLI-HELP-CONTRACT.md`

## Behavior impact

Docs/help-text only for CLI descriptions. Command parsing and command execution
behavior do not change.

## Public API impact

No Rust public API change.

## Migration note status

No migration note required. The change narrows CLI help wording and does not
change accepted arguments or runtime behavior.

## Validation

### Targeted CLI unit tests

Command:

```bash
source scripts/ai/toolchain_env.sh && CARGO_BUILD_JOBS=2 cargo test -p nautilus-cli --lib opt::tests -- --nocapture
```

Result summary:

```text
running 24 tests
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out
```

Note: an earlier untimed targeted test was stopped after it spent several
minutes compiling dependencies without reaching test execution. The bounded
rerun completed successfully.

### CLI help smoke

Commands:

```bash
source scripts/ai/toolchain_env.sh && cargo run -p nautilus-cli -- live run --help
source scripts/ai/toolchain_env.sh && cargo run -p nautilus-cli -- data inspect --help
source scripts/ai/toolchain_env.sh && cargo run -p nautilus-cli -- config validate --help
source scripts/ai/toolchain_env.sh && cargo run -p nautilus-cli -- backtest run --help
```

Result summary:

- `live run --help` says runtime wiring is not implemented yet.
- `data inspect --help` says implementation is not implemented yet.
- `config validate --help` says implementation is not implemented yet.
- `backtest run --help` says only the metadata-only dry-run path is available
  and engine runtime wiring is not implemented yet.

### Cargo check

Command:

```bash
source scripts/ai/toolchain_env.sh && cargo check -p nautilus-cli
```

Result summary:

```text
Finished `dev` profile [unoptimized] target(s) in 16.61s
```

### Fast verification

Command:

```bash
scripts/ai/verify_fast.sh
```

Result summary:

```text
== verify_fast: rust fmt ==
== verify_fast: cargo check skipped; set VERIFY_FAST_CARGO_CHECK=1 to run the legacy mixed-workspace check ==
== verify_fast: clippy skipped; set VERIFY_FAST_CLIPPY=1 to run it in fast mode ==
== verify_fast complete ==
```

## Rollback plan

Revert the PR. Runtime behavior and persisted data are unaffected.
