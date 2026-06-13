# NTPRO

NTPRO is a Rust-only release workspace for a trading engine cutover from
NautilusTrader.

The current public milestone is:

```text
Current source tag: ntpro-rust-only-v0.3.1
Capability: Local Supervisor Control Console Hardening
```

This tag is the current v0.3.1 source release point for the local sandbox-only
Supervisor control-console release line. It preserves the v0.3.0 local control
surface and closes the patch hardening queue for release accounting, release
gate behavior, hosted evidence, and control-surface wording. It is published as
a GitHub Release for the tagged source tree:

```text
https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.3.1
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

v0.3.1 is still a local-only Supervisor control-console release line. It does
not add a new trading product milestone. It keeps the same sandbox-only control
surface as v0.3.0 and hardens how that surface is documented, verified, and
released.

Supported shipped capability in the v0.3.1 claim:

- source-build Rust CLI and local Dashboard workflows for sandbox node
  registration;
- local `ntpro-node` process startup and shutdown;
- status / logs / metrics inspection;
- local two-node smoke evidence;
- local control actions:

- `start`
- `stop`
- `pause`
- `resume`
- `reconnect data source` as an explicit local sandbox `not_supported` result
- `reconnect execution gateway` as an explicit local sandbox `not_supported`
  result

Not included in the v0.3.1 product claim:

- Production exchange connectivity.
- Real account connectivity.
- Real order submission.
- Manual order entry.
- Production reconnect behavior.
- Distributed multi-server deployment.
- Remote or multi-user Dashboard operation.
- Prebuilt binary or Docker release artifact delivery.

Source-tree deltas that are present in the v0.3.1 tag but do not expand the
shipped capability claim:

- Dashboard UI copy localization.
- Supervisor / trader product-shape documents under `docs/architecture/`.
- v0.4 planning/task documents under `docs/rust-cutover/tasks/V04-*`.
- v0.4 scope/spec documents under `docs/rust-cutover/scope/` and
  `docs/rust-cutover/specs/`.
- supporting v0.4 planning evidence under `docs/rust-cutover/evidence/V04-*.md`.

Those files are part of the tagged source tree, but they do not mean that NTPRO
now claims a trader terminal, v0.4 product scope, production order entry, or
distributed runtime support in v0.3.1.

## Planned v0.4 Boundary

The planned v0.4 product boundary is Binance sandbox-only. It is fixture /
testnet / mock first, with no real funds, no production trading, and no real
order submission.

The scope and planning references include:

- `docs/rust-cutover/scope/v0_4_0_binance_sandbox_product_foundation.md`
- `docs/rust-cutover/specs/v0_4_strategy_contracts.md`

These are planning boundaries for the next milestone. They do not change the
current v0.3.1 product claim.

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
- `docs/rust-cutover/release/v0_3_1_supervisor_control_hardening_release_notes.md`
- `docs/rust-cutover/release/v0_3_1_supervisor_control_hardening_readiness_report.md`
- `docs/rust-cutover/release/v0_3_1_supervisor_control_hardening_closeout.md`
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

`ntpro-rust-only-v0.3.1` is the current Rust-only source release for the Local
Supervisor Control Console Hardening line. It does not expand the v0.3.0
product claim; it closes the patch release surface around release gate
hardening, hosted evidence, release accounting, and control-surface wording.

`v0.2.0` remains the local multi-node runtime foundation baseline, and
`v0.1.0` remains the first formal Rust-only cutover release and historical
baseline.

Before cutting a later release, review:

- current GitHub checks for the tagged commit;
- release notes;
- public README surface;
- Rust CLI help output;
- repository language display.

## License

This repository inherits the NautilusTrader license lineage. Review the license
files and upstream notices before distributing a final release artifact.
