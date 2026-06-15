# NTPRO

NTPRO is a Rust-only release workspace for a trading engine cutover from
NautilusTrader.

The current public milestone is:

```text
Current source tag: ntpro-rust-only-v0.7.1
Capability: Binance testnet read-only connectivity proof
Boundary: v0.7.1 hardening for manual online HTTP read-only proof; no orders, no real funds, no production trading
```

This tag is the current v0.7.1 source release point for the scoped Binance
testnet read-only connectivity proof hardening patch. It is published as a
GitHub Release for the tagged source tree:

```text
https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.7.1
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

v0.7.1 is the current formal release line. It builds on the earlier foundation
layers:

- `v0.4.x`: Binance sandbox product foundation;
- `v0.5.0`: local Binance sandbox workflow artifacts;
- `v0.6.0`: Binance testnet dry-run runtime foundation.
- `v0.6.1`: offline wording, Dashboard, artifact-contract, and PR smoke
  hardening.
- `v0.7.0`: Binance testnet public read-only connectivity proof.

`v0.5.0` was completed as a scoped readiness milestone and is absorbed into the
`v0.6.0` release tree. It is not published as a separate public GitHub Release.

`v0.6.1` aligned version wording, Dashboard copy, workflow artifact contracts,
offline-only probe semantics, and PR-stage smoke coverage.

The current release path supports:

- Rust CLI `workflow run --workflow binance-sandbox`;
- Rust CLI `workflow run --workflow binance-testnet --mode dry-run`;
- Rust CLI `workflow run --workflow binance-testnet --mode connectivity-probe`
  behind explicit manual online opt-in;
- deterministic local artifact directories and manifest / summary / events
  contracts;
- checked-in testnet dry-run config;
- env-var-only credential policy artifact;
- offline connectivity probe artifact and manual-online HTTP read-only probe
  artifact;
- optional/manual WebSocket read-only probe artifact schema;
- dry-run order lifecycle artifact;
- artifact-only reconciliation artifact;
- Dashboard read-only workflow and testnet workflow surfaces.

The v0.3.0 local Supervisor control console and the v0.4.x Binance sandbox
foundation remain part of validated release history, but they are no longer the
current public milestone.

Not included in the v0.7.1 product claim:

- Real Binance testnet order submission.
- Testnet order cancel, replace, amend, or live order management.
- Testnet account mutation.
- Real account reconciliation.
- Production Binance connectivity.
- Real account credential values in repository artifacts.
- Real funds.
- Production trading parity.
- Remote or multi-user Dashboard operation.
- Prebuilt binary or Docker release artifact delivery.

The v0.7.0 release introduced optional Binance testnet read-only network proof behind
manual gates. The implemented probe is a public HTTP
read-only `/api/v3/time` connectivity check; it requires
`--allow-testnet-network` plus `NTPRO_ALLOW_TESTNET_NETWORK=1` and never submits
orders. The workflow also emits an optional WebSocket read-only probe artifact,
but the default path records it as manual-only and does not open a WebSocket,
subscribe to streams, or make it a CI/release blocker. Credential values remain
environment-only: artifacts may record environment variable names and presence
booleans, but must not record API key or API secret values. Public read-only
probes must not require credentials. Authenticated read-only probes are
manual-online-only and still must not submit, cancel, replace, or amend orders.

## Binance Testnet Read-Only Boundary

The current v0.7.1 product boundary remains Binance testnet read-only
connectivity proof.
Default local and CI runs remain offline, artifact-first, Rust-only, and
explicitly non-production.

Manual online proof may connect only to Binance testnet public read-only HTTP
endpoints after explicit opt-in. This release does not store or load real API
key values, does not submit real orders, and does not claim live or production
trading readiness.

The scope and readiness documents are:

- `docs/rust-cutover/release/v0_5_0_workflow_artifacts_readiness_report.md`
- `docs/rust-cutover/release/v0_6_0_binance_testnet_dry_run_readiness_report.md`
- `docs/rust-cutover/release/v0_6_1_offline_hardening_readiness_report.md`
- `docs/rust-cutover/release/v0_6_1_release_notes.md`
- `docs/rust-cutover/release/v0_7_0_readonly_testnet_boundary.md`
- `docs/rust-cutover/release/v0_7_0_readonly_testnet_readiness_report.md`
- `docs/rust-cutover/release/v0_7_0_release_notes.md`
- `docs/versioning.md`

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
- `docs/rust-cutover/release/v0_4_0_binance_sandbox_readiness_report.md`
- `docs/rust-cutover/release/v0_5_0_workflow_artifacts_readiness_report.md`
- `docs/rust-cutover/release/v0_6_0_binance_testnet_dry_run_readiness_report.md`
- `docs/rust-cutover/release/v0_6_0_release_notes.md`
- `docs/rust-cutover/release/v0_6_1_offline_hardening_readiness_report.md`
- `docs/rust-cutover/release/v0_6_1_release_notes.md`
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

`ntpro-rust-only-v0.7.1` is the current Rust-only source release for the
Binance testnet read-only connectivity proof line. `v0.6.1` remains the v0.6
offline hardening closure, `v0.6.0` remains the Binance testnet dry-run runtime
foundation, `v0.5.0` remains a completed internal workflow-artifact milestone
absorbed into `v0.6.0`, `v0.4.1` remains the latest Binance sandbox public patch
baseline, `v0.3.0` remains the Local Supervisor Control Console baseline,
`v0.2.0` remains the local multi-node runtime foundation baseline, and
`v0.1.0` remains the first formal Rust-only cutover release and historical
baseline.

Before cutting the next release, review:

- current GitHub checks for the tagged commit;
- release notes;
- public README surface;
- Rust CLI help output;
- repository language display.

## License

This repository inherits the NautilusTrader license lineage. Review the license
files and upstream notices before distributing a final release artifact.
