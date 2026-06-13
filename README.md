# NTPRO

NTPRO is a Rust-only release workspace for a trading engine cutover from
NautilusTrader.

The current public milestone is:

```text
Current source tag: ntpro-rust-only-v0.3.0
Capability: Local Supervisor Control Console
```

This tag is the current v0.3.0 source release point for the local sandbox-only
Supervisor control console. It is published as a GitHub Release for the tagged
source tree:

```text
https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.3.0
```

## Current Status

NTPRO is now treated as a Rust-only product surface.

Supported product surfaces:

- Rust workspace crates.
- Rust CLI commands.
- Rust examples and documentation.
- Rust release verification scripts.
- Local Python helper scripts under `scripts/` only, used for repository
  control and release evidence.

Unsupported product surfaces:

- Python package installation.
- Python import/API usage.
- PyO3 bindings.
- Cython build or runtime paths.
- Python wheels, PyPI publication, or mixed Rust/Python packaging.
- Cap'n Proto serialization.

## Rust Toolchain

The release gate is validated with the pinned Rust toolchain:

```bash
rustup toolchain install 1.95.0
rustup override set 1.95.0
```

The repository can also be built through the local pinned toolchain path when
that is how the workspace is configured:

```bash
PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH"
```

If Homebrew `rustc` or `cargo` appears before rustup on `PATH`, the local
verification scripts source `scripts/ai/toolchain_env.sh` to force Rust
`1.95.0` before running Cargo. See
`docs/rust-cutover/verification/toolchain.md`.

## User Entrypoint

Use the Rust CLI as the first product entrypoint:

```bash
cargo run -p nautilus-cli -- --help
cargo run -p nautilus-cli -- backtest --help
cargo run -p nautilus-cli -- sandbox --help
cargo run -p nautilus-cli -- live --help
```

The release verification gate checks that the CLI exposes the Rust product
commands without requiring Python.

## Install Path

NTPRO is source-build first. From a checked-out repository, run the CLI through
Cargo or install the binary locally:

```bash
cargo run -p nautilus-cli -- --help
cargo install --path crates/cli --bin nautilus --locked --force
nautilus --help
```

The current binary name is `nautilus`, provided by the `nautilus-cli` package.
NTPRO does not currently publish prebuilt binaries, Python wheels, PyPI
packages, or Docker images as product delivery paths.

## Current Capability Boundary

v0.3.0 is a local-only Supervisor control-console release. It supports
source-build Rust CLI and local Dashboard workflows for sandbox node
registration, local `ntpro-node` process startup and shutdown, status/log/
metrics inspection, local two-node smoke evidence, and local control actions:

- `start`
- `stop`
- `pause`
- `resume`
- `reconnect data source` as an explicit local sandbox `not_supported` result
- `reconnect execution gateway` as an explicit local sandbox `not_supported`
  result

Not included in the v0.3.0 product claim:

- Production exchange connectivity.
- Real account connectivity.
- Real order submission.
- Manual order entry.
- Production reconnect behavior.
- Distributed multi-server deployment.
- Remote or multi-user Dashboard operation.
- Prebuilt binary or Docker release artifact delivery.

## Planned v0.4 Boundary

The planned v0.4 product boundary is Binance sandbox-only. It is fixture /
testnet / mock first, with no real funds, no production trading, and no real
order submission.

The scope contract is:

- `docs/rust-cutover/scope/v0_4_0_binance_sandbox_product_foundation.md`

This is a planning boundary for the next milestone. It does not change the
current v0.3.0 product claim.

## Verification

Fast local validation:

```bash
scripts/ai/verify_fast.sh
```

`verify_fast.sh` is a fast smoke only: it checks the pinned Rust toolchain and
`cargo fmt --check` by default. It does not run workspace `cargo check`,
clippy, golden traces, or release validation unless optional flags or stronger
scripts are used.

Full release validation:

```bash
scripts/ai/verify_release.sh
```

Rust-only surface checks:

```bash
scripts/ai/check_rust_only_runtime.sh
scripts/ai/check_cython_removed.sh
```

Golden trace validation:

```bash
scripts/ai/run_golden_traces.sh
REQUIRE_GOLDEN_REPLAY=1 scripts/ai/run_golden_traces.sh
```

## Documentation

Core cutover documents:

- `docs/rust-cutover/CONTRACT.md`
- `docs/rust-cutover/DEFINITION_OF_DONE.md`
- `docs/rust-cutover/TASK_EXECUTION.md`
- `docs/rust-cutover/AGENT_ROLES.md`

Release documents:

- `docs/rust-cutover/release/v0_2_0_release_notes.md`
- `docs/rust-cutover/release/v0_2_0_known_limits.md`
- `docs/rust-cutover/release/v0_2_local_multi_node_readiness_report.md`
- `docs/rust-cutover/release/v0_3_0_supervisor_control_readiness_report.md`
- `docs/rust-cutover/release/rust_only_release_notes.md`
- `docs/rust-cutover/release/final_release_verification.md`
- `docs/rust-cutover/release/final_completion_report.md`
- `docs/rust-cutover/release/human_owner_signoff_packet.md`
- `docs/rust-cutover/release/release_candidate_tag_plan.md`

Migration documents:

- `docs/rust-cutover/migration/rust_only_migration_guide.md`
- `docs/rust-cutover/migration/python_to_rust_workflow_map.md`
- `docs/rust-cutover/migration/python_test_scope_map.md`

## Examples

Rust examples live under:

```text
examples/rust/
```

Start with:

- `examples/rust/backtest_cli_smoke.rs`
- `examples/rust/live_cli_smoke.rs`
- `examples/rust/sandbox_cli_smoke.rs`

## Release Notes

`ntpro-rust-only-v0.3.0` is the current Rust-only source release for the Local
Supervisor Control Console. v0.2.0 remains the local multi-node runtime
foundation baseline, and v0.1.0 remains the first formal Rust-only cutover
release and historical baseline.

Before cutting a later release, review:

- current GitHub checks for the tagged commit;
- release notes;
- public README surface;
- Rust CLI help output;
- repository language display.

## License

This repository inherits the NautilusTrader license lineage. Review the license
files and upstream notices before distributing a final release artifact.
