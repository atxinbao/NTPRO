# NTPRO

NTPRO is a Rust-only release workspace for a trading engine cutover from
NautilusTrader.

The current source-tree milestone is:

```text
Current source tag: ntpro-rust-only-v0.25.0
Latest formal release: ntpro-rust-only-v0.25.0
Current source-tree readiness: ntpro-rust-only-v0.25.0 released
Current capability: v0.25.0 Monitoring, Incident, and Disaster-Recovery Foundation
Capability class: monitoring / incident / disaster-recovery foundation only
Next patch: v0.25.1 Reserved Patch Track
Next capability: v0.26.0 Reserved Capability Track
v0.26.0 entry: reserved next capability track only after v0.25.0 release evidence is published
Boundary: v0.25.0 publishes the monitoring, incident, runbook/audit, disaster-recovery preview, read-only Dashboard monitoring, SLO/freshness diagnostics, release gates, and strict provenance foundation. It does not include product-grade live trading, new production submit capability, production order mutation, execution adapter send, live exchange request, implicit retry, retry scheduler, automatic remediation, automatic recovery, strategy-driven production execution, shared approval consumption, real-funds proof in CI, or Dashboard order/approval/cancel/retry/submit/replace/amend/flatten/remediation/order-ticket controls.
```

`ntpro-rust-only-v0.25.0` is the latest formal GitHub Release. It is the
Monitoring, Incident, and Disaster-Recovery Foundation release. It preserves the
no-submit/no-Dashboard-controls boundary while closing the v25 monitoring,
incident, runbook/audit, DR preview, read-only Dashboard, SLO/freshness,
release gate, and strict provenance gaps. It is a monitoring and governance
foundation, not a product-grade live trading terminal or production execution
runtime.

The latest formal release is published as a GitHub Release for the v0.25.0
tagged source tree:

```text
https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.25.0
```

`v0.25.0` is the current formal monitoring / incident / DR foundation release.
It remains evidence and governance only, not a product-grade live trading
terminal or general production trading platform claim. It does not add
production submit, order mutation, execution adapter send, live exchange
request, ungated cancel/retry/replace/amend/flatten, implicit retry, retry
scheduler, automatic remediation, automatic recovery, bulk order execution,
strategy-driven production execution, shared approval consumption, or Dashboard
operation controls.

The next patch track is `v0.25.1`. The next capability track is `v0.26.0`.
`v0.26.0` does not automatically inherit production submit, mutation, adapter
send, live exchange request, retry scheduler, automatic remediation, or
Dashboard operation controls from `v0.25.0`.

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

v0.25.0 is the current formal release line. It is the monitoring / incident /
disaster-recovery foundation over the read-only Workbench and order-control
foundation line and builds on the earlier foundation layers:

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
- `v0.12.0`: Production Online Read-Only + Persistent Shadow release package
  with owner-gated production public/account `GET` read-only proof paths,
  redacted response-shape evidence, local shadow portfolio runtime, bounded
  shadow strategy session event artifact, local read-only reconciliation,
  Dashboard v0.12 production shadow read-only panel, and v0.12
  offline/manual-online preflight release gates.
- `v0.12.1`: Production Read-Only Evidence & Release Surface Hardening patch.
- `v0.13.0`: Guarded Live Alpha Preflight only, with bounded local shadow
  preflight loop evidence, owner-gated read-only proof-pack wrapper, kill-switch
  dry-run/manual approval artifact, Dashboard control-boundary evidence,
  Decimal/string amount-boundary evidence, and no-production-mutation release
  gate.
- `v0.14.0`: Production Order-State Read-Only + Live Alpha Dry-Run, with
  owner-gated production order-state read-only proof scope, local live-alpha
  dry-run evidence, local risk preflight, reconciliation golden traces,
  Dashboard read-only dry-run panel, and no-production-mutation release gate.
- `v0.15.0`: Guarded Live Alpha Mutation Scope + Execution Dry-Run Harness,
  with mutation endpoint classification, redacted local request preview,
  manual approval lifecycle, kill-switch runtime gate, local dry-run execution
  adapter evidence, incident/rollback artifacts, Dashboard read-only mutation
  preflight panel, and no-production-mutation release gate.
- `v0.16.0`: Minimum Owner-Approved Production Order Mutation Candidate, with
  one owner-approved tiny `LIMIT` `GTC` production order candidate, explicit
  runtime gates, owner-gated signing material, guarded HTTP send, response
  redaction, order-state readback, audit trail, kill-switch checks, no-retry
  failure semantics, Dashboard read-only evidence, and no general production
  trading claim.
- `v0.16.1`: Production Mutation Candidate Hardening patch readiness, with
  guarded-send counter clarification, post-send kill-switch second-read
  evidence, response-redaction source binding, non-marketable price safety,
  owner-run evidence accounting, CLI/classifier wording alignment, and no
  capability expansion from v0.16.0.
- `v0.17.0`: Production Reconciliation And Orphan Recovery Evidence, with
  local ledger persistence, redacted readback mapping, reconciliation
  classification, orphan order risk detection, restart recovery, read-only
  Dashboard evidence, failure incident semantics, and no actual cancel send.
- `v0.18.0`: Owner-Approved Cancel Recovery Preview, with cancel recovery
  intent, approval lifecycle, preview request/response artifacts,
  post-cancel readback contracts, failure/rollback evidence, Dashboard
  diagnostics, and no actual cancel send.
- `v0.18.1`: Release Surface & Provenance Hardening patch, with strict
  provenance evidence and published prerequisite release evidence.
- `v0.19.0`: Owner-Approved Single-Shot Actual Cancel, with one manual owner
  approval, one order, one venue, one execution attempt, risk gate, adapter
  boundary, post-cancel readback, failure evidence, read-only Dashboard audit,
  golden traces, and no production order submit lifecycle.
- `v0.20.0`: Owner-Approved Production Order Lifecycle Foundation, with
  pre-submit risk evidence, owner approval, one guarded submit attempt,
  response redaction, post-submit readback, failure/no-retry evidence,
  read-only Dashboard audit, golden traces, and no product-grade trading claim.
- `v0.20.1`: Production Order Lifecycle Release Closeout & Provenance Hardening
  patch, with release evidence, provenance hardening, and no capability
  expansion from v0.20.0.
- `v0.21.0`: Unified Read Model Foundation, with account, position, order,
  fill, risk, and Trader Terminal read-only Dashboard foundation evidence.
- `v0.21.1`: Unified Read Model Foundation Hardening Patch, with health status
  semantics, executable read-model replay, JSON Schema boundaries, read-model
  runtime bridge evidence, and v0.22 dependency proof.
- `v0.22.0`: Trader Terminal Workbench, with read-only first account,
  position, order, fill, risk, alerts, audit, provenance, gated manual
  operation-entry evidence, runtime degradation tests, release gates, and
  strict provenance.

`v0.5.0` was completed as a scoped readiness milestone and is absorbed into the
`v0.6.0` release tree. It is not published as a separate public GitHub Release.

`v0.6.1` aligned version wording, Dashboard copy, workflow artifact contracts,
offline-only probe semantics, and PR-stage smoke coverage. The historical
`v0.11.1` patch material remains release-surface hardening for the v0.11.0
line; it is not the current public source release line and does not add a
production trading capability.

The `v0.13.0` release advances beyond v0.12 read-only/shadow evidence into
Guarded Live Alpha preflight evidence only. The shadow preflight session is a
bounded local artifact loop with heartbeat, stop-file, and stale-data evidence. It
does not add production order submission, production order mutation,
production order-state reads, listenKey lifecycle, signed WebSocket user
streams, real funds, production trading, automatic production remediation,
risk/execution-grade live-alpha money math, or Dashboard order controls.

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
- owner-gated production public read-only online proof for allowlisted `GET`
  endpoints;
- owner-gated authenticated production account snapshot online proof for
  `GET /api/v3/account`;
- redacted production account response-shape evidence;
- local shadow portfolio runtime artifact;
- bounded local shadow strategy session event artifact;
- local read-only reconciliation classifications;
- Dashboard v0.12 production shadow read-only panel;
- v0.12 offline release gates and manual-online fail-closed preflight.
- bounded local v0.13 shadow preflight loop heartbeat/stop/stale-data evidence;
- v0.13 owner-gated production online read-only proof-pack wrapper, default
  offline and fail-closed;
- v0.13 kill-switch dry-run/manual approval artifact;
- v0.13 trader/ops Dashboard read-only/control boundary evidence;
- v0.13 Decimal/string-only amount preflight evidence;
- v0.13 no-production-mutation PR and release gate.

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

Not included in the v0.12.0 product claim:

- Production order submission.
- Production open-order or order-state reads such as `/api/v3/openOrders`.
- Production order submit, cancel, replace, amend, retry, correction, or live
  order management.
- Production listenKey lifecycle creation, keepalive, or close.
- Production signed WebSocket user stream execution.
- Strategy-driven production execution.
- Automatic production remediation.
- Production portfolio parity.
- Exchange-confirmed shadow fills or positions.
- Raw account response, raw balances, raw credentials, signatures, signed
  query, or signed URL persistence.
- Real funds.
- Production trading.
- Dashboard order buttons, order controls, credential input, or reconnect
  controls.

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
Runtime batch foundation plus release wording/evidence closure. The v0.11.0
release built on the v0.10.0 Binance spot sandbox order proof and added
Production Read-Only Contract + Offline Shadow Portfolio evidence only.

The v0.12.0 release source tree adds owner-gated production `GET` read-only
proof paths and persistent local shadow artifact evidence. The v0.13.0 release
source tree adds Guarded Live Alpha preflight evidence only. Default local, PR,
CI, and release-gate runs still remain offline unless the owner explicitly
enables the manual-online proof gates.

The v0.9 runtime may load a local strategy session, consume a bounded
fixture/mock input batch, write signal artifacts, write shadow order-intent and
risk-decision artifacts, and expose read-only supervisor/Dashboard status. The
v0.10.0 release proves one owner-gated Binance Spot Demo Mode submit/cancel
artifact package. The v0.11.0 release adds production read-only contracts and
local offline shadow evidence. The v0.12.0 release adds owner-gated
production online read-only proof paths and persistent shadow artifacts. The
v0.13.0 release adds Guarded Live Alpha Preflight evidence, but it does not
prove production order submission, production order mutation, production
order-state reads, listenKey lifecycle, real funds, production trading
readiness, automatic production remediation, or Dashboard order controls.

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
- `docs/rust-cutover/release/v0_12_0_boundary.md`
- `docs/rust-cutover/release/v0_12_0_release_gates.md`
- `docs/rust-cutover/release/v0_12_0_readiness_report.md`
- `docs/rust-cutover/release/v0_12_0_release_notes.md`
- `docs/rust-cutover/release/v0_12_1_readiness_report.md`
- `docs/rust-cutover/release/v0_12_1_release_notes.md`
- `docs/rust-cutover/release/v0_13_0_scope_decision.md`
- `docs/rust-cutover/release/v0_13_0_shadow_session_preflight.md`
- `docs/rust-cutover/release/v0_13_0_online_readonly_proof_pack.md`
- `docs/rust-cutover/release/v0_13_0_kill_switch_approval_artifact.md`
- `docs/rust-cutover/release/v0_13_0_dashboard_control_boundary.md`
- `docs/rust-cutover/release/v0_13_0_decimal_amount_boundary.md`
- `docs/rust-cutover/release/v0_13_0_no_production_mutation_gate.md`
- `docs/rust-cutover/release/v0_13_0_readiness_report.md`
- `docs/rust-cutover/release/v0_13_0_release_notes.md`
- `docs/versioning.md`

## Verification

Fast local validation:

```bash
scripts/ai/verify_fast.sh
```

`verify_fast.sh` is a fast smoke only: it checks the pinned Rust toolchain and
`cargo fmt --check` by default. A passing default fast smoke is not release
validation and is not release evidence.

Compile and lint validation:

```bash
VERIFY_FAST_CARGO_CHECK=1 VERIFY_FAST_CLIPPY=1 scripts/ai/verify_fast.sh
```

That check adds workspace compile/lint coverage, but still does not replace
release gates, full tests, or golden trace evidence.

Full test validation:

```bash
scripts/ai/verify_full.sh
```

Release gate validation:

```bash
scripts/ai/verify_release.sh
```

Strict binary provenance, when required by the release line:

```bash
scripts/ai/verify_release_strict.sh v18
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
- `docs/rust-cutover/release/v0_12_0_boundary.md`
- `docs/rust-cutover/release/v0_12_0_response_shape.md`
- `docs/rust-cutover/release/v0_12_0_shadow_portfolio_runtime.md`
- `docs/rust-cutover/release/v0_12_0_persistent_shadow_strategy_session.md`
- `docs/rust-cutover/release/v0_12_0_production_readonly_reconciliation.md`
- `docs/rust-cutover/release/v0_12_0_dashboard_production_shadow_readonly_panel.md`
- `docs/rust-cutover/release/v0_12_0_release_gates.md`
- `docs/rust-cutover/release/v0_12_0_readiness_report.md`
- `docs/rust-cutover/release/v0_12_0_release_notes.md`
- `docs/rust-cutover/release/v0_12_1_readiness_report.md`
- `docs/rust-cutover/release/v0_12_1_release_notes.md`
- `docs/rust-cutover/release/v0_13_0_scope_decision.md`
- `docs/rust-cutover/release/v0_13_0_shadow_session_preflight.md`
- `docs/rust-cutover/release/v0_13_0_online_readonly_proof_pack.md`
- `docs/rust-cutover/release/v0_13_0_kill_switch_approval_artifact.md`
- `docs/rust-cutover/release/v0_13_0_dashboard_control_boundary.md`
- `docs/rust-cutover/release/v0_13_0_decimal_amount_boundary.md`
- `docs/rust-cutover/release/v0_13_0_no_production_mutation_gate.md`
- `docs/rust-cutover/release/v0_13_0_readiness_report.md`
- `docs/rust-cutover/release/v0_13_0_release_notes.md`
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

`ntpro-rust-only-v0.20.1` is the latest formal GitHub Release for the
Production Order Lifecycle Release Closeout & Provenance Hardening Patch. It
preserves the v0.20.0 owner-approved foundation boundary and does not authorize
product-grade live trading, new submit capability, implicit retry, automatic
cancel/remediation, bulk order execution, strategy-driven production execution,
multi-account or multi-venue expansion, or Dashboard operation controls.
`v0.21.0` is the deferred next capability track. `v0.17.0` remains the Production Reconciliation
And Orphan Recovery Evidence baseline, `v0.12.1` remains the Production Read-Only Evidence
& Release Surface Hardening baseline, `v0.10.0` remains the Binance spot
sandbox order-proof baseline, `v0.9.0` remains the local deterministic Strategy
Runtime batch foundation baseline, `v0.8.0` remains the authenticated Binance
testnet read-only proof baseline, `v0.7.2` remains the wording/evidence closure
for the read-only connectivity proof line, `v0.6.1` remains the v0.6 offline
hardening closure, `v0.6.0` remains the Binance testnet dry-run runtime
foundation, `v0.5.0` remains a completed internal workflow-artifact milestone
absorbed into `v0.6.0`, `v0.4.1` remains the Binance sandbox public patch
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
