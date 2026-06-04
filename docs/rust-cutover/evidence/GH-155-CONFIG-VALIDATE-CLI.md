# GH-155 - Config validate CLI evidence

Date: 2026-06-05
Executor: Codex

## Task

- GitHub issue: <https://github.com/atxinbao/NTPRO/issues/155>
- Branch: `codex/audit-config-validate-cli`
- Owner role: Rust Product Surface Agent
- Review role: Verification & Release Gatekeeper
- Risk level: Medium

## Plain Chinese summary

这次把 `nautilus config validate` 从“只定义了命令但还没实现”推进成
真实可运行的最小 Rust 路径。现在它可以按 `--kind` 校验 backtest、
sandbox、live-smoke 和 data/catalog 的 TOML 配置，并且可以用
`--output` 写出 `validation.txt` 作为自动化证据。

这不是完整 runtime wiring：不会启动 backtest engine，不会启动 live
node，不会连接交易所，也不会 inspect/load 真实 catalog。它只是先把
“配置文件是否存在、能否解析、关键字段是否符合当前 Rust-only 最小契约”
这条用户入口做实。

## Goal

Implement the first real Rust-only `config validate` path for post-release
audit issue #155.

## Files changed

- `crates/cli/src/backtest.rs`
- `crates/cli/src/config.rs`
- `crates/cli/src/sandbox.rs`
- `crates/cli/src/lib.rs`
- `crates/cli/src/opt.rs`
- `docs/rust-cutover/product/CONFIG_VALIDATION_CLI_CONTRACT.md`
- `docs/rust-cutover/product/RUST_CLI_CONTRACT.md`
- `docs/rust-cutover/product/CLI_HELP_CONTRACT.md`
- `docs/rust-cutover/product/RUST_PRODUCT_SURFACE_REPORT.md`
- `docs/rust-cutover/migration/python_to_rust_workflow_map.md`
- `docs/rust-cutover/evidence/GH-155-CONFIG-VALIDATE-CLI.md`

## Implementation boundary

- `--kind backtest` reuses the existing minimal backtest config parser.
- `--kind sandbox` reuses the existing minimal sandbox config parser.
- `--kind live` validates the Rust live init smoke TOML shape without starting
  a node.
- `--kind data` validates the Rust data/catalog TOML shape without inspecting,
  loading, or querying catalog storage.
- `--output <dir>` writes `<dir>/validation.txt`.

## Behavior impact

`nautilus config validate` now returns success for scoped valid Rust TOML
configs and clear errors for missing, unreadable, malformed, or structurally
invalid configs.

No trading semantics change. No runtime workflow is started by this command.

## Public API impact

No Rust public API change. CLI behavior for `config validate` changes from an
explicit blocker to a scoped validation implementation.

## Migration note status

Existing migration/product docs were updated to describe the new partial
config validation status. No separate breaking-change migration note is
required because this enables a previously blocked CLI path.

## Validation

### Targeted CLI unit tests

Command:

```bash
cargo fmt --all && source scripts/ai/toolchain_env.sh && CARGO_BUILD_JOBS=2 cargo test -p nautilus-cli --lib -- --nocapture
```

Result summary:

```text
running 35 tests
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### CLI smoke

Command summary:

```bash
source scripts/ai/toolchain_env.sh
cargo run -q -p nautilus-cli -- config validate --kind backtest --config <temp>/backtest.toml --output <temp>/out-backtest
cargo run -q -p nautilus-cli -- config validate --kind sandbox --config examples/rust/sandbox/sandbox_smoke.toml
cargo run -q -p nautilus-cli -- config validate --kind live --config examples/rust/live/live_init_smoke.toml
cargo run -q -p nautilus-cli -- config validate --kind data --config <temp>/data.toml
cargo run -q -p nautilus-cli -- config validate --kind data --config <temp>/bad-data.toml
```

Result summary:

- Valid backtest config returned `config.validate status=ok kind=backtest`.
- Valid sandbox config returned `config.validate status=ok kind=sandbox`.
- Valid live smoke config returned `config.validate status=ok kind=live`.
- Valid data/catalog config returned `config.validate status=ok kind=data`.
- Invalid data config was rejected with `queries must contain at least one data query`.
- Backtest `--output` wrote `validation.txt` with command, status, kind, and config path.

### Cargo check

Command:

```bash
source scripts/ai/toolchain_env.sh && cargo check -p nautilus-cli
```

Result summary:

```text
Finished `dev` profile [unoptimized] target(s) in 16.40s
```

### Rust-only runtime check

Command:

```bash
scripts/ai/check_rust_only_runtime.sh
```

Result summary:

```text
== rust-only-runtime: ok ==
```

### Cython removal check

Command:

```bash
scripts/ai/check_cython_removed.sh
```

Result summary:

```text
== cython-removed: ok ==
```

### Fast verification

Command:

```bash
scripts/ai/verify_fast.sh
```

Result summary:

```text
== verify_fast: rust fmt ==
== verify_fast complete ==
```

`verify_fast.sh` still skips workspace cargo check and clippy by default.

### Diff and residual blocker text checks

Commands:

```bash
git diff --check
rg -n "config validate.*blocker|shared parser/validator remains blocked|config validate.*not implemented|config validate.*deferred" docs/rust-cutover/product docs/rust-cutover/migration crates/cli/src -S
```

Result summary:

- `git diff --check` returned no whitespace errors.
- Residual search found one migration-table line in
  `docs/rust-cutover/migration/python_to_rust_workflow_map.md`.
- The remaining line is intentionally classified as a migration-status warning:
  `config validate` itself now works for scoped Rust TOML validation, while
  shared workflow config models and runtime-specific validation remain
  incomplete.

## Remaining risks

- Full `backtest run` runtime wiring is still deferred.
- Full `live validate/run` lifecycle and adapter classification remain
  deferred.
- Full `data inspect/validate/load` catalog behavior remains deferred to #156.
- Config validators are not yet unified behind a shared trait.

## Rollback plan

Revert the PR. The command returns to explicit blocker behavior. No persisted
runtime data or trading state is affected.
