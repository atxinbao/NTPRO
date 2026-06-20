# NTPRO Roadmap

Date: 2026-06-20
Executor: Codex

NTPRO is a Rust-only release workspace for the trading engine cutover from
NautilusTrader. The current public source release is
`ntpro-rust-only-v0.11.0`, the next patch track is `v0.11.1`
release-surface hotfix cleanup, and the next capability track is `v0.12.0`
Guarded Live Alpha.

## Current Release Surface

Current published release:

```text
ntpro-rust-only-v0.11.0
```

Current capability boundary:

```text
Production Read-Only Contract + Offline Shadow Portfolio
production endpoint classification
production public read-only contract, offline fail-closed
owner-gated authenticated account snapshot contract, offline fail-closed
local shadow execution intent artifacts
local shadow portfolio snapshot artifacts
local shadow/read-only lifecycle state evidence
local reconciliation/manual-remediation event evidence
read-only Dashboard production shadow status
successful online production reads=0
production order counters fixed at zero
production order mutations attempted=0
no real funds
no production trading
no Dashboard order controls
```

`v0.11.0` builds on the v0.10 Binance spot sandbox order proof and adds the
Production Read-Only Contract + Offline Shadow Portfolio release package. The
current public claim remains artifact-first, offline by default,
read-only/shadow-only from production surfaces, read-only from Dashboard
surfaces, and explicitly non-production. It does not prove successful online
production public/account reads.

## Published Hardening Patch: v0.7.1

`v0.7.1` is the published hardening patch for the `v0.7.0` surface. It does
not expand the capability claim, does not add order submission, and keeps the
default local/CI path offline.

Completed hardening work:

- wire v0.7 default offline and manual-online preflight scripts into
  `verify_release.sh`, PR smoke, and hosted release gate;
- align Roadmap, readiness, and release-facing wording for the v0.7.1
  hardening release;
- normalize the v0.7 HTTP connectivity probe artifact path/schema contract;
- validate Binance `/api/v3/time` response shape before claiming HTTP
  connectivity success;
- split manual-online classification from manual-online connectivity proof;
- prepare v0.7.1 readiness notes and final gate evidence.

v0.7.1 explicitly does not include:

- real Binance testnet order submission;
- authenticated Binance testnet account access;
- real account reconciliation;
- production Binance connectivity;
- production trading;
- remote or multi-user Dashboard operation;
- prebuilt binary or Docker release delivery.

## Published Wording/Evidence Patch: v0.7.2

`v0.7.2` is the published wording and evidence patch for the `v0.7.1`
hardening surface. It does not expand the capability claim, does not add
authenticated account access, and keeps the default local/CI path offline.

Completed release-surface cleanup:

- finalize v0.7.2 release notes as published;
- finalize v0.7.2 readiness report as released/PASS;
- record formal tag, GitHub Release URL, hosted release gates, and publication
  flags;
- keep the no orders / no real funds / no production trading boundary explicit.

v0.7.2 explicitly does not include:

- real Binance testnet order submission;
- authenticated Binance testnet account access;
- real account reconciliation;
- production Binance connectivity;
- production trading;
- Dashboard-started network probes.

## Published Capability Track: v0.8.0

`v0.8.0` is the published capability track after the v0.7.2 wording and
evidence patch. It advances the boundary to authenticated Binance testnet
read-only proof.

The only intended boundary change is:

```text
public read-only testnet proof -> authenticated read-only testnet proof
```

Required constraints:

- no order submission;
- no account mutation;
- no real account trading;
- no production trading claim;
- Dashboard remains read-only and must not start probes;
- secrets are never written to artifacts, stdout, logs, docs, or PR bodies;
- default CI remains offline;
- manual online verification is opt-in only.

Authenticated read-only access must fail closed unless all of these are true:

- `--allow-testnet-network` is passed;
- `NTPRO_ALLOW_TESTNET_NETWORK=1` is set;
- config environment is `testnet`;
- `order_submission = disabled`;
- `real_orders_submitted = false`.
- required credential env vars are present;
- credential values are never persisted or printed.

The v0.8.0 proof must stay read-only. It must not place, cancel, amend, or query
through any endpoint that mutates account state.

Completed release closure:

- formal tag: `ntpro-rust-only-v0.8.0`;
- GitHub Release:
  `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.8.0`;
- hosted workflow-dispatch and tag-triggered release gates passed;
- closure evidence recorded in `docs/rust-cutover/evidence/V080-009.md`.

## Safety Patch Track: v0.8.1

`v0.8.1` is a safety and release-surface closure patch for the v0.8.0 line. It
must not add order submission, account mutation, production Binance
connectivity, real funds, production trading, or Dashboard-started network
probes.

The v0.8.1 patch scope is:

- align README and ROADMAP with the published v0.8.0 release surface;
- enforce `NTPRO_V08_MANUAL_ONLINE=1` inside the Rust authenticated runtime
  gate, not only in Bash verification scripts;
- expose authenticated read-only proof status in summary / manifest output;
- tighten authenticated response-shape naming and validation;
- publish v0.8.1 readiness and release notes as a safety/closure patch.

## Published Capability Track: v0.9.0

`v0.9.0` is the published local deterministic Strategy Runtime batch foundation
release. It proves that `ntpro-node` can load a local strategy session, consume
a bounded fixture/mock market input batch, write signal and shadow order-intent
artifacts, write shadow risk decisions, expose supervisor status, and render
read-only Dashboard state.

Completed release closure:

- formal tag: `ntpro-rust-only-v0.9.0`;
- GitHub Release:
  `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.9.0`;
- hosted workflow-dispatch and tag-triggered release gates passed;
- closure evidence recorded in `docs/rust-cutover/evidence/V090-014.md`.

`v0.9.0` explicitly does not include:

- persistent long-running Strategy Runtime semantics;
- Binance testnet order submission;
- order cancel/replace/amend;
- production order submission;
- real funds;
- production trading;
- Dashboard order controls;
- strategy-driven live execution through an exchange adapter.

## Historical Hardening Patch Track: v0.9.1

`v0.9.1` is Strategy Runtime Semantics & Audit Hardening for the published
v0.9.0 line. It must not add Binance testnet order submission or production
trading capability. Its scope is to make node/session/market/risk/heartbeat and
artifact audit semantics true before later Binance sandbox order proof work.

The v0.9.1 patch scope is:

- align README and ROADMAP with the published v0.9.0 release surface;
- mark v0.9 readiness/boundary wording as released rather than planning;
- unify StrategyNode config validation between CLI and node runtime;
- make StrategySession lifecycle semantics persistent until stop/pause/risk
  halt, instead of stopping before the node waits for shutdown;
- align node/session/market status transitions;
- keep heartbeat counters monotonic and sourced from one runtime snapshot;
- split kill-switch enabled/active semantics in config and artifacts;
- add Strategy Session manifest and artifact integrity audit;
- surface artifact/status conflicts as degraded in Supervisor and Dashboard;
- add integration, heartbeat, shutdown, and restart smoke coverage;
- add v0.9.1 release notes and readiness material after the hardening tasks;
- document that v0.10.0 is the Binance spot sandbox order proof release track.

## Published Capability Track: v0.10.0

`v0.10.0` is the published Binance spot sandbox order proof release. It proves
one owner-gated Spot Demo Mode LIMIT GTC submit/cancel lifecycle with redacted
artifacts, terminal reconciliation, production order counters fixed at zero,
and read-only Dashboard evidence display.

Completed release closure:

- formal tag: `ntpro-rust-only-v0.10.0`;
- GitHub Release:
  `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.10.0`;
- tag-triggered release gate passed;
- release notes and readiness report recorded under
  `docs/rust-cutover/release/v0_10_0_release_notes.md` and
  `docs/rust-cutover/release/v0_10_0_readiness_report.md`.

`v0.10.0` explicitly does not include:

- production Binance connectivity;
- production order submission, cancel, replace, amend, or live order
  management;
- automatic online order mutation;
- real funds;
- production trading;
- Dashboard order controls;
- production account reconciliation.

## Published Capability Track: v0.11.0

`v0.11.0` is the published Production Read-Only Contract + Offline Shadow
Portfolio release. It defines production endpoint classification, read-only
public/account snapshot contracts, offline fail-closed read artifacts, local
shadow execution/portfolio evidence, local shadow/read-only lifecycle and
reconciliation models, and read-only Dashboard production shadow status.

Completed release closure:

- formal tag: `ntpro-rust-only-v0.11.0`;
- GitHub Release:
  `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.11.0`;
- release notes and readiness report recorded under
  `docs/rust-cutover/release/v0_11_0_release_notes.md` and
  `docs/rust-cutover/release/v0_11_0_readiness_report.md`.

`v0.11.0` explicitly does not include:

- production order submission;
- production cancel, replace, amend, retry, or correction orders;
- successful online production public/account reads;
- production network-read runtime as completed capability;
- real funds;
- production trading;
- automatic production reconciliation or remediation;
- production order lifecycle parity;
- Dashboard order, cancel, replace, amend, or retry controls.

## Active Hardening Patch Track: v0.11.1

`v0.11.1` is the active release-surface hardening patch for the published
v0.11.0 line. It prepares readiness and release-note material for owner release
decision, but it does not create a tag or publish a GitHub Release by itself.

The v0.11.1 patch scope is:

- align v0.11 wording to contract/offline reality;
- add a central endpoint classifier API and deny tests;
- add a production shadow manifest contract;
- harden Dashboard production shadow artifact health;
- wire the public read probe into v11 offline release gates;
- clarify that `/api/v3/openOrders` and order-state reads are out of scope;
- normalize `read_allowed` artifact semantics with explicit
  `contract_ready` and `online_read_allowed` fields;
- prepare v0.11.1 readiness and release-note material.

`v0.11.1` explicitly does not include:

- production online read runtime;
- successful online production public/account reads;
- production open-order or order-state reads;
- production order submission or mutation;
- real funds;
- production trading;
- automatic production remediation;
- Dashboard order controls.

## Next Capability Track: v0.12.0

`v0.12.0` is the next capability track: Guarded Live Alpha. It requires a
separate scope decision and must not be inferred from v0.11.0 read-only/shadow
evidence.

## Corrected Capability Sequence: v0.9.0 through v0.12.0

The previous idea of making `v0.9.0` a Binance testnet order lifecycle proof is
superseded. `v0.9.0` is now published as Strategy Runtime Foundation, and
`v0.10.0` is now published as the Binance spot sandbox order proof release.

Corrected sequence:

```text
v0.9.0  = local deterministic Strategy Runtime batch foundation
v0.9.1  = Strategy Runtime Semantics & Audit Hardening
v0.10.0 = Binance Spot Sandbox Order Proof
v0.11.0 = Production Read-Only Contract + Offline Shadow Portfolio
v0.12.0 = Guarded Live Alpha
```

`v0.9.0` is the published batch foundation track. It makes `ntpro-node` load a
strategy session config, run a bounded built-in demo strategy against
fixture/mock market input, write signal/order-intent/risk decision/audit
artifacts, expose supervisor status, and show read-only Dashboard state.

`v0.9.1` is the hardening track that must make the runtime semantics honest:
node running implies session state is coherent, market exhaustion is not labeled
running, heartbeat counters do not regress, artifact gaps are visible, and
release gates verify the Supervisor/Dashboard path.

`v0.9.0` explicitly does not include:

- Binance testnet order submission;
- order cancel/replace/amend;
- production order submission;
- real funds;
- production trading;
- Dashboard order controls;
- strategy-driven live execution through an exchange adapter.

`v0.10.0` is the published track where Binance spot sandbox order proof was
completed behind explicit owner gates and with its own risk, redaction, and
lifecycle evidence.

`v0.11.0` is the published Production Read-Only Contract + Offline Shadow
Portfolio track. It is contract/offline-shadow only and must not claim
successful online production reads, submit, cancel, replace, amend, or
automatically correct production orders.

`v0.12.0` is the next Guarded Live Alpha track and requires a separate scope
decision before any live command or production trading capability is claimed.

## Product Surface Direction

Supported product surfaces:

- Rust workspace crates;
- Rust CLI commands and command contracts;
- Rust examples and documentation;
- Rust release verification scripts;
- Dashboard read-only local artifact surfaces;
- local Python helper scripts under `scripts/` only, used for repository
  control or release evidence.

Unsupported product surfaces:

- Python package installation;
- Python import/API usage;
- PyO3 bindings;
- Cython build or runtime paths;
- Python wheels, PyPI publication, or mixed Rust/Python packaging;
- Cap'n Proto serialization;
- production exchange trading claims without dedicated release evidence.

## Release Gate Direction

Before any next public release, these must agree:

- Shrimp task queue state;
- task evidence under `docs/rust-cutover/evidence/`;
- readiness report;
- release notes;
- README release surface;
- local verification commands;
- hosted GitHub release or PR checks when used as release evidence.

No release may describe a dry-run, fixture, mock, sandbox, or read-only probe as
production trading readiness.

## Versioning

NTPRO has multiple version identifiers with different meanings:

- release tags such as `ntpro-rust-only-v0.6.0`;
- Cargo workspace package version such as `0.58.0`;
- badge metadata in `version.json`.

The release tag is the product milestone identity. Cargo and badge metadata are
not proof of the current NTPRO release surface. See `docs/versioning.md`.
