# GH-156 - Data inspect/validate CLI evidence

Date: 2026-06-05
Executor: Codex

## Task

- GitHub issue: <https://github.com/atxinbao/NTPRO/issues/156>
- Branch: `codex/audit-data-cli`
- Owner role: Rust Product Surface Agent
- Review role: Verification & Release Gatekeeper
- Risk level: Medium

## Plain Chinese summary

这次把 `nautilus data inspect` 和 `nautilus data validate` 从“只有命令，
执行时会报未实现”推进成了真实可运行的本地检查入口。

现在它们可以读取 TOML 配置，检查 `catalog.path` 指向的本地文件或目录是否
存在、是否可读，并输出文件大小、扩展名、目录项数量、请求的数据类型和查询
过滤条件。

这不是完整 data ingest：不会解析 market data 行内容，不会写 catalog，不会
连接 adapter，也不会把数据接入 backtest/live runtime。`data load` 仍然明确
保持未实现 blocker。

## Goal

Implement the first real Rust-only local data file/directory
`inspect`/`validate` path for post-audit issue #156.

## Files changed

- `crates/cli/src/data.rs`
- `crates/cli/src/lib.rs`
- `crates/cli/src/config.rs`
- `crates/cli/src/opt.rs`
- `docs/rust-cutover/product/DATA_CATALOG_CLI_CONTRACT.md`
- `docs/rust-cutover/product/CLI_HELP_CONTRACT.md`
- `docs/rust-cutover/product/RUST_CLI_CONTRACT.md`
- `docs/rust-cutover/product/RUST_PRODUCT_SURFACE_REPORT.md`
- `docs/rust-cutover/migration/python_to_rust_workflow_map.md`
- `examples/rust/data/README.md`
- `docs/rust-cutover/evidence/GH-156-DATA-INSPECT-VALIDATE-CLI.md`

## Implementation boundary

- `data inspect` parses TOML config and inspects `catalog.path`.
- `data validate` parses TOML config, validates query shape, and checks
  `catalog.path` readability.
- `catalog.protocol` supports only `file`.
- `catalog.path` may point to a local file or local directory.
- Supported `data_type` values are built-in Rust CLI data types:
  `QuoteTick`, `TradeTick`, `Bar`, `OrderBookDelta`, `OrderBookDepth10`,
  `InstrumentAny`, and `FundingRateUpdate`.
- `data load` remains an explicit blocker.
- `config validate --kind data` continues to validate TOML shape only; it does
  not require catalog path existence.

## Behavior impact

Users can now run a first Rust-only preflight before backtest/live work:

- inspect a local configured file or directory;
- validate that the file/directory is present and readable;
- reject unsupported data types and malformed query shape;
- write `inspection.txt` when `data inspect --output <dir>` is provided.

No trading semantics change. No runtime workflow is started by these commands.

## Public API impact

No Rust public API change. CLI behavior for `data inspect` and `data validate`
changes from explicit blocker to scoped local metadata validation.

## Migration note status

Product and migration docs were updated to record the new partial data CLI
status. No separate breaking-change migration note is required because this
enables previously blocked Rust CLI paths.

## Validation

### Targeted CLI unit tests

Command:

```bash
cargo fmt --all && source scripts/ai/toolchain_env.sh && CARGO_BUILD_JOBS=2 cargo test -p nautilus-cli --lib -- --nocapture
```

Result summary:

```text
running 41 tests
test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### CLI smoke

Command summary:

```bash
source scripts/ai/toolchain_env.sh
cargo run -q -p nautilus-cli -- data inspect --config <temp>/data-file.toml --output <temp>/out-inspect
cargo run -q -p nautilus-cli -- data validate --config <temp>/data-file.toml
cargo run -q -p nautilus-cli -- data validate --config <temp>/data-dir.toml
cargo run -q -p nautilus-cli -- data validate --config <temp>/data-missing.toml
cargo run -q -p nautilus-cli -- data validate --config <temp>/data-unsupported.toml
cargo run -q -p nautilus-cli -- data load --config <temp>/data-load.toml
```

Result summary:

- Valid local CSV file inspect returned `data.inspect status=ok`.
- Valid local CSV file validate returned `data.validate status=ok`.
- Valid local directory validate returned `data.validate status=ok`.
- Missing local catalog path was rejected.
- Unsupported `CustomPythonData` data type was rejected.
- `data load` still returned the expected explicit blocker.
- `data inspect --output` wrote `inspection.txt` with command, status, run ID,
  config path, catalog path, protocol, kind, query count, data types, query
  filters, file size, and extension.

### Cargo check

Command:

```bash
source scripts/ai/toolchain_env.sh && cargo check -p nautilus-cli
```

Result summary:

```text
Finished `dev` profile [unoptimized] target(s) in 27.26s
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
rg -n "data inspect.*not implemented|data validate.*not implemented|data inspect.*blocker|data validate.*blocker|data inspect.*deferred|data validate.*deferred|data inspect.*return explicit blocker|data validate.*return explicit blocker" crates/cli/src docs/rust-cutover/product docs/rust-cutover/migration examples/rust/data -S
```

Result summary:

- `git diff --check` returned no whitespace errors.
- Residual search returned no stale `data inspect` / `data validate` blocker
  claims in the scoped public docs and CLI source.

## Remaining risks

- No market data row decoding is implemented.
- No Parquet schema validation is implemented.
- No catalog interval availability query is implemented.
- No `data load` behavior is implemented.
- No adapter or external source access is implemented.

## Rollback plan

Revert the PR. `data inspect` and `data validate` return to explicit blocker
behavior. No persisted runtime data, catalog data, or trading state is affected.
