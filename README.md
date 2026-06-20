# NTPRO

NTPRO is a Rust-only release workspace for a trading engine cutover from
NautilusTrader.

The current public milestone is:

```text
Current source tag: ntpro-rust-only-v0.11.0
Capability: Production Read-Only Contract + Offline Shadow Portfolio
Boundary: production endpoint classification and read-only contracts, offline fail-closed public/account read artifacts, local shadow execution/portfolio evidence, Dashboard read-only production shadow status, successful online production reads=0, production orders submitted=0, production order mutations attempted=0, no real funds, no production trading, no Dashboard order controls
```

This tag is the current v0.11.0 source release point for the scoped Production
Read-Only Contract + Offline Shadow Portfolio line. It defines production
read-only classification/contracts and offline evidence. It is not evidence of
successful online production reads. It is published as a GitHub Release for the
tagged source tree:

```text
https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.11.0
```

The next patch track is `v0.11.1`, a release-surface hotfix line for the
published v0.11.0 boundary. It must not expand v0.11.0 into production order
submission, production order mutation, real funds, production trading,
automatic online remediation, or Dashboard order controls.
The v0.11.1 readiness and release-note material is prepared for owner release
decision, but this source tree does not by itself create the
`ntpro-rust-only-v0.11.1` tag or publish a GitHub Release.

The next capability track after the v0.11.x line is `v0.12.0` Guarded Live
Alpha. Its boundary requires a separate scope decision and must not be inferred
from the v0.11.0 read-only/shadow-only release.

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

v0.11.0 is the current formal release line. It builds on the earlier foundation
layers:

- `v0.4.x`: Binance sandbox product foundation;
- `v0.5.0`: local Binance sandbox workflow artifacts;
- `v0.6.0`: Binance testnet dry-run runtime foundation.
- `v0.6.1`: offline wording, Dashboard, artifact-contract, and PR smoke
  hardening.
- `v0.7.0`: Binance testnet public read-only connectivity proof.
- `v0.7.1`: release-gate and artifact-contract hardening for the v0.7 proof.
- `v0.7.2`: wording and evidence closure for the v0.7 read-only proof line.
- `v0.8.0`: authenticated Binance testnet read-only account-shape proof.
- `v0.9.0`: local deterministic Strategy Runtime batch foundation with
  fixture/mock market input, signal artifacts, shadow order-intent/risk-decision
  artifacts, supervisor read-only status, and Dashboard read-only artifact
  display.
- `v0.10.0`: Binance spot sandbox order proof with owner-gated Spot Demo Mode
  submit/cancel evidence, redacted execution artifacts, terminal reconciliation,
  read-only Dashboard evidence display, and production order counters fixed at
  zero.
- `v0.11.0`: Production Read-Only Contract + Offline Shadow Portfolio release
  package with endpoint classification, production read-only contracts, offline
  fail-closed public/account read artifacts, local shadow
  execution/portfolio artifacts, shadow/read-only lifecycle and reconciliation
  models, Dashboard read-only production shadow status, and production order
  counters fixed at zero.

`v0.5.0` was completed as a scoped readiness milestone and is absorbed into the
`v0.6.0` release tree. It is not published as a separate public GitHub Release.

`v0.6.1` aligned version wording, Dashboard copy, workflow artifact contracts,
offline-only probe semantics, and PR-stage smoke coverage. The `v0.11.1` patch
track is reserved for release-surface hotfixes after the formal v0.11.0
publication; it does not add a production trading capability.

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
- authenticated Binance testnet `GET /api/v3/account` read-only proof artifact
  with redacted account-shape evidence;
- env-var-only testnet credential policy for authenticated read-only proof;
- synthetic secret leak scan for v0.8 generated output;
- local Strategy Session runtime through `ntpro-node`;
- fixture/mock market stream input;
- strategy signal JSONL artifacts;
- shadow order-intent JSONL artifacts;
- shadow-mode risk decision artifacts with actual order submission disabled;
- strategy session audit and summary artifacts;
- Dashboard read-only workflow and testnet workflow surfaces.
- disabled-by-default `[testnet_order]` execution config contract;
- offline order risk preflight;
- redacted signed Binance spot sandbox request preview;
- offline `/api/v3/order/test` preflight report;
- owner-gated Binance Spot Demo Mode submit/cancel proof artifact;
- terminal `CANCELED` order lifecycle reconciliation evidence;
- read-only Dashboard order-proof display.
- production endpoint classification for read-only/shadow-only boundaries;
- production public read-only probe contract, offline fail-closed by default
  with `network_attempted=false`;
- authenticated production account snapshot contract, owner-gated, redacted,
  and offline fail-closed by default with `network_attempted=false`;
- local shadow execution intent artifacts;
- local shadow portfolio snapshot artifacts;
- local shadow/read-only order lifecycle state model;
- local read-only/shadow reconciliation event model;
- Dashboard read-only production shadow status.

The v0.3.0 local Supervisor control console and the v0.4.x Binance sandbox
foundation remain part of validated release history, but they are no longer the
current public milestone.

Not included in the v0.11.0 product claim:

- Production order submission.
- Production open-order or order-state reads such as `/api/v3/openOrders`.
- Production order submit, cancel, replace, amend, or live order management.
- Successful online production public or account reads.
- Production network-read runtime presented as completed capability.
- Automatic online order mutation.
- Production account mutation.
- Automatic production reconciliation or remediation.
- Real account credential values in repository artifacts.
- Real funds.
- Production trading parity.
- Remote or multi-user Dashboard operation.
- Dashboard order buttons, order controls, or credential input.
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

## Strategy Runtime Boundary

The v0.9.0 product boundary remains the historical local deterministic Strategy
Runtime batch foundation plus release wording/evidence closure. The current
v0.11.0 release builds on the v0.10.0 Binance spot sandbox order proof and adds
Production Read-Only Contract + Offline Shadow Portfolio evidence only. Default
local and CI runs remain offline, artifact-first, Rust-only,
read-only/shadow-only, and explicitly non-production.

The v0.9 runtime may load a local strategy session, consume a bounded
fixture/mock input batch, write signal artifacts, write shadow order-intent and
risk-decision artifacts, and expose read-only supervisor/Dashboard status. The
v0.10.0 release proves one owner-gated Binance Spot Demo Mode submit/cancel
artifact package. The v0.11.0 release adds production read-only contracts and
local offline shadow evidence, but it does not prove successful online
production reads, production order submission, production order mutation, real
funds, production trading readiness, automatic production remediation, or
Dashboard order controls.

The scope and readiness documents are:

- `docs/rust-cutover/release/v0_5_0_workflow_artifacts_readiness_report.md`
- `docs/rust-cutover/release/v0_6_0_binance_testnet_dry_run_readiness_report.md`
- `docs/rust-cutover/release/v0_6_1_offline_hardening_readiness_report.md`
- `docs/rust-cutover/release/v0_6_1_release_notes.md`
- `docs/rust-cutover/release/v0_7_0_readonly_testnet_boundary.md`
- `docs/rust-cutover/release/v0_7_0_readonly_testnet_readiness_report.md`
- `docs/rust-cutover/release/v0_7_0_release_notes.md`
- `docs/rust-cutover/release/v0_7_1_release_gate_hardening_readiness_report.md`
- `docs/rust-cutover/release/v0_7_1_release_notes.md`
- `docs/rust-cutover/release/v0_7_2_readiness_report.md`
- `docs/rust-cutover/release/v0_7_2_release_notes.md`
- `docs/rust-cutover/release/v0_8_0_authenticated_readonly_boundary.md`
- `docs/rust-cutover/release/v0_8_0_authenticated_readonly_readiness_report.md`
- `docs/rust-cutover/release/v0_8_0_release_notes.md`
- `docs/rust-cutover/release/v0_8_1_readiness_report.md`
- `docs/rust-cutover/release/v0_8_1_release_notes.md`
- `docs/rust-cutover/release/v0_9_0_strategy_runtime_boundary.md`
- `docs/rust-cutover/release/v0_9_0_strategy_runtime_readiness_report.md`
- `docs/rust-cutover/release/v0_9_0_release_notes.md`
- `docs/rust-cutover/release/v0_9_1_readiness_report.md`
- `docs/rust-cutover/release/v0_9_1_release_notes.md`
- `docs/rust-cutover/release/v0_10_0_order_boundary.md`
- `docs/rust-cutover/release/v0_10_0_readiness_report.md`
- `docs/rust-cutover/release/v0_10_0_release_notes.md`
- `docs/rust-cutover/release/v0_11_0_boundary.md`
- `docs/rust-cutover/release/v0_11_0_readiness_report.md`
- `docs/rust-cutover/release/v0_11_0_release_notes.md`
- `docs/rust-cutover/release/v0_11_1_readiness_report.md`
- `docs/rust-cutover/release/v0_11_1_release_notes.md`
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
- `docs/rust-cutover/release/v0_7_0_readonly_testnet_boundary.md`
- `docs/rust-cutover/release/v0_7_0_readonly_testnet_readiness_report.md`
- `docs/rust-cutover/release/v0_7_0_release_notes.md`
- `docs/rust-cutover/release/v0_7_1_release_gate_hardening_readiness_report.md`
- `docs/rust-cutover/release/v0_7_1_release_notes.md`
- `docs/rust-cutover/release/v0_7_2_readiness_report.md`
- `docs/rust-cutover/release/v0_7_2_release_notes.md`
- `docs/rust-cutover/release/v0_8_0_authenticated_readonly_boundary.md`
- `docs/rust-cutover/release/v0_8_0_authenticated_readonly_readiness_report.md`
- `docs/rust-cutover/release/v0_8_0_release_notes.md`
- `docs/rust-cutover/release/v0_8_1_readiness_report.md`
- `docs/rust-cutover/release/v0_8_1_release_notes.md`
- `docs/rust-cutover/release/v0_9_0_strategy_runtime_boundary.md`
- `docs/rust-cutover/release/v0_9_0_strategy_runtime_readiness_report.md`
- `docs/rust-cutover/release/v0_9_0_release_notes.md`
- `docs/rust-cutover/release/v0_9_1_readiness_report.md`
- `docs/rust-cutover/release/v0_9_1_release_notes.md`
- `docs/rust-cutover/release/v0_10_0_order_boundary.md`
- `docs/rust-cutover/release/v0_10_0_readiness_report.md`
- `docs/rust-cutover/release/v0_10_0_release_notes.md`
- `docs/rust-cutover/release/v0_11_0_boundary.md`
- `docs/rust-cutover/release/v0_11_0_readiness_report.md`
- `docs/rust-cutover/release/v0_11_0_release_notes.md`
- `docs/rust-cutover/release/v0_11_1_readiness_report.md`
- `docs/rust-cutover/release/v0_11_1_release_notes.md`
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

`ntpro-rust-only-v0.11.0` is the current Rust-only source release for the
Production Read-Only Contract + Offline Shadow Portfolio line. `v0.11.1` is the next
release-surface hotfix track and does not add production order submission,
production order mutation, real funds, production trading, automatic production
remediation, or Dashboard order controls. `v0.10.0` remains the Binance spot
sandbox order-proof baseline, `v0.9.0` remains the local deterministic Strategy
Runtime batch foundation baseline, `v0.8.0` remains the authenticated Binance
testnet read-only proof baseline, `v0.7.2` remains the wording/evidence closure
for the read-only connectivity proof line, `v0.6.1` remains the v0.6 offline
hardening closure, `v0.6.0` remains the Binance testnet dry-run runtime
foundation, `v0.5.0` remains a completed internal workflow-artifact milestone
absorbed into `v0.6.0`, `v0.4.1` remains the Binance sandbox public patch
baseline, `v0.3.0` remains the Local Supervisor Control Console baseline,
`v0.2.0` remains the local multi-node runtime foundation baseline, and `v0.1.0`
remains the first formal Rust-only cutover release and historical baseline.

Before cutting the next release, review:

- current GitHub checks for the tagged commit;
- release notes;
- public README surface;
- Rust CLI help output;
- repository language display.

## License

This repository inherits the NautilusTrader license lineage. Review the license
files and upstream notices before distributing a final release artifact.
