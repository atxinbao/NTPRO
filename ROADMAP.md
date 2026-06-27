# NTPRO Roadmap

Date: 2026-06-27
Executor: Codex

NTPRO is a Rust-only release workspace for the trading engine cutover from
NautilusTrader. The current public source release is
`ntpro-rust-only-v0.18.0`, the Owner-Approved Cancel Recovery Preview release.
The next patch track is `v0.18.1`. The next capability track is `v0.19.0`,
which is the follow-up line for owner-approved single-shot actual cancel.

## Current Release Surface

Current published release:

```text
ntpro-rust-only-v0.18.0
```

Current capability boundary:

```text
Owner-Approved Cancel Recovery Preview
v0.17 reconciliation and orphan-risk evidence lineage
cancel recovery intent contract
owner approval lifecycle evidence
preview request/response artifact contracts
post-cancel readback contract
failure and partial-success semantics
rollback evidence
read-only Dashboard cancel recovery diagnostics
default local/PR/release execution offline and fail-closed
actual cancel send disabled
automatic cancel disabled
automatic remediation disabled
no strategy-driven production execution
no cancel/replace/amend/retry/correction/flatten
no listenKey lifecycle
no real-funds proof in CI
no multi-account or multi-venue execution
no Dashboard order controls
no Dashboard cancel controls
```

`v0.18.0` builds on the v0.17 Production Reconciliation And Orphan Recovery
Evidence line. It does not submit another order and does not enable actual
cancel execution. It adds preview-only evidence for cancel recovery intent,
owner approval, request/response handling, post-cancel readback contracts,
failure/rollback semantics, release gates, and Dashboard diagnostics.

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

## Historical Hardening Patch Track: v0.11.1

`v0.11.1` is the historical release-surface hardening patch material for the
published v0.11.0 line. It prepared readiness and release-note material for
owner release decision, but it did not become the current public source release
line after v0.12.0 was published.

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

## Published Capability Track: v0.12.0

`v0.12.0` is the published Production Online Read-Only + Persistent Shadow
release. It includes owner-gated production `GET` read-only proof paths and
persistent local shadow artifact evidence.

The v0.12.0 release scope is:

- owner-gated production public read-only online proof;
- owner-gated authenticated production account snapshot read-only proof;
- redacted production account response-shape evidence;
- local shadow portfolio runtime artifact;
- bounded local shadow strategy session event artifact;
- local read-only reconciliation classifications;
- Dashboard v0.12 production shadow read-only panel;
- v0.12 offline release gates and manual-online fail-closed preflight.

`v0.12.0` explicitly does not include:

- production order submission;
- production cancel, replace, amend, retry, or correction orders;
- production open-order or order-state reads;
- listenKey lifecycle access;
- strategy-driven production execution;
- automatic production remediation;
- production portfolio parity;
- real funds;
- production trading;
- Dashboard order controls.

## Published Capability Track: v0.13.0

`v0.13.0` is the published Guarded Live Alpha Preflight line. It keeps the
release default offline/fail-closed and adds preflight evidence only before any
future live-alpha execution decision.

`v0.13.0` includes:

- v0.13 Guarded Live Alpha Preflight scope decision;
- bounded local shadow preflight loop heartbeat/stop/stale-data evidence;
- owner-gated production online read-only proof-pack wrapper;
- kill-switch dry-run/manual approval artifact;
- trader/ops Dashboard read-only/control boundary evidence;
- Decimal/string-only amount preflight evidence;
- no-production-mutation PR and release gate;
- readiness report and release notes.

`v0.13.0` explicitly does not include:

- production order submission;
- production cancel, replace, amend, retry, or correction orders;
- production open-order or order-state reads;
- listenKey lifecycle access;
- signed WebSocket user stream runtime;
- strategy-driven production execution;
- automatic production remediation;
- production portfolio parity;
- live-alpha risk/execution-grade money math;
- real funds;
- production trading;
- Dashboard order controls.

`v0.12.1` remains the published hardening baseline for the v0.12 production
read-only/shadow-only line. `v0.13.0` remains the Guarded Live Alpha Preflight
line, `v0.14.0` remains the Production Order-State Read-Only + Live Alpha
Dry-Run line, and `v0.15.0` remains the Guarded Live Alpha Mutation Scope +
Execution Dry-Run Harness line. These are superseded by the v0.16.0
owner-approved single-order candidate line for current public release-surface
wording.

## Published Capability Track: v0.15.0

`v0.15.0` is the published Guarded Live Alpha Mutation Scope + Execution
Dry-Run Harness line. It keeps release defaults offline/fail-closed and adds
production mutation endpoint classification, redacted local request-preview
artifacts, manual approval, kill-switch runtime gating, dry-run execution
adapter evidence, incident/rollback evidence, and Dashboard read-only mutation
preflight only.

`v0.15.0` includes:

- v0.15 production mutation scope decision;
- production mutation endpoint classifier gate;
- redacted live order request dry-run preview;
- manual approval lifecycle artifact;
- kill switch runtime gate;
- execution adapter isolation artifact;
- dry-run mutation golden traces;
- manual incident, rollback, and emergency-stop artifacts;
- Dashboard live-alpha mutation preflight read-only panel;
- v0.15 release gates;
- readiness report and release notes.

`v0.15.0` explicitly does not include:

- production request sending;
- production order submission;
- production cancel, replace, amend, retry, or correction orders;
- production order-test submission;
- production execution adapter calls;
- default production network execution;
- listenKey lifecycle access;
- signed WebSocket user stream runtime;
- strategy-driven production execution;
- automatic production remediation;
- real funds;
- production trading;
- Dashboard order controls.

## Published Capability Track: v0.16.0

`v0.16.0` is the published Minimum Owner-Approved Production Order Mutation
Candidate line. It keeps release defaults offline/fail-closed and adds the
smallest production mutation candidate boundary: one owner-approved tiny
`LIMIT` `GTC` production order candidate with explicit gates, redacted
evidence, readback, audit, kill-switch enforcement, and terminal no-retry
failure semantics.

`v0.16.0` includes:

- v0.16 production mutation scope contract;
- owner-approved runtime gates;
- production signing-material approval evidence;
- single `LIMIT` `GTC` request builder;
- guarded production HTTP send path behind explicit gates;
- production mutation response redaction;
- post-submit order-state readback proof contract;
- kill-switch checks around the send boundary;
- production mutation audit trail;
- failure-mode and no-retry semantics;
- Dashboard production mutation evidence read-only panel;
- v0.16 release gates;
- readiness report and release notes.

`v0.16.0` explicitly does not include:

- strategy-driven production execution;
- multiple production orders;
- batch production orders;
- `MARKET` orders;
- production cancel, replace, amend, retry, correction, or flatten;
- automatic production remediation;
- listenKey lifecycle access;
- signed WebSocket user stream runtime;
- multi-account or multi-venue execution;
- VWAP/POV/Iceberg execution algorithms;
- real-funds proof in CI;
- production trading platform claim;
- Dashboard order controls.

## Published Capability Track: v0.17.0

`v0.17.0` is the published Production Reconciliation And Orphan Recovery
Evidence line. It preserves the v0.16 single owner-approved production mutation
candidate boundary and adds local/offline evidence for reconciliation,
orphan-risk detection, restart recovery, read-only Dashboard visibility, and
incident classification.

`v0.17.0` includes:

- local production order ledger;
- redacted exchange readback mapper;
- local-vs-exchange reconciliation classifier;
- orphan order risk detector;
- restart recovery evidence;
- owner-approved cancel recovery boundary documentation without cancel
  execution;
- read-only Dashboard reconciliation and orphan-risk evidence panel;
- failure and incident semantics;
- v0.17 aggregate release gates;
- readiness report and release notes.

`v0.17.0` explicitly does not include:

- network readback execution;
- new production order submission;
- additional production order mutation;
- actual cancel send;
- automatic cancel;
- automatic orphan cleanup;
- retry, replace, amend, correction, flatten, or remediation;
- Dashboard order controls;
- Dashboard cancel controls;
- Dashboard credential input;
- multi-account production execution;
- multi-venue production execution;
- VWAP/POV/Iceberg execution algorithms;
- listenKey creation, keepalive, or close lifecycle;
- real-funds proof in CI;
- general production trading platform claim.

## Published Capability Track: v0.18.0

`v0.18.0` is the published Owner-Approved Cancel Recovery Preview line. It
preserves the v0.17 reconciliation and orphan-risk evidence boundary while
adding preview-only cancel recovery contracts, owner approval lifecycle
evidence, failure/rollback evidence, release gates, and Dashboard diagnostics.

`v0.18.0` includes:

- cancel recovery intent contract;
- owner approval lifecycle evidence;
- preview request/response artifact contracts;
- post-cancel readback contract;
- failure and partial-success semantics;
- rollback evidence;
- read-only Dashboard cancel recovery diagnostics;
- v0.18 aggregate release gates;
- readiness report and release notes.

`v0.18.0` explicitly does not include:

- actual cancel send;
- automatic cancel;
- automatic remediation;
- retry, replace, amend, correction, flatten, or remediation;
- Dashboard order controls;
- Dashboard cancel controls;
- Dashboard credential input;
- strategy-driven production execution;
- multi-account production execution;
- multi-venue production execution;
- listenKey creation, keepalive, or close lifecycle;
- real-funds proof in CI;
- general production trading platform claim.

`v0.18.1` is the release surface and provenance hardening patch for this
preview-only line. `v0.19.0` is the next capability track for owner-approved
single-shot actual cancel.

## Corrected Capability Sequence: v0.9.0 through v0.16.0

The previous idea of making `v0.9.0` a Binance testnet order lifecycle proof is
superseded. `v0.9.0` is now published as Strategy Runtime Foundation, and
`v0.10.0` is now published as the Binance spot sandbox order proof release.

Corrected sequence:

```text
v0.9.0  = local deterministic Strategy Runtime batch foundation
v0.9.1  = Strategy Runtime Semantics & Audit Hardening
v0.10.0 = Binance Spot Sandbox Order Proof
v0.11.0 = Production Read-Only Contract + Offline Shadow Portfolio
v0.12.0 = Production Online Read-Only + Persistent Shadow
v0.12.1 = Production Read-Only Evidence & Release Surface Hardening
v0.13.0 = Guarded Live Alpha Preflight only
v0.14.0 = Production Order-State Read-Only + Live Alpha Dry-Run
v0.15.0 = Guarded Live Alpha Mutation Scope + Execution Dry-Run Harness
v0.16.0 = Minimum Owner-Approved Production Order Mutation Candidate
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

`v0.12.0` is the Production Online Read-Only + Persistent Shadow release. It
must not be described as production trading readiness, production order
submission readiness, real-funds readiness, production portfolio parity, or
Dashboard order-control readiness.

`v0.12.1` is the hardening-only patch track for v0.12 release evidence and
release surface. It must not be described as a live-alpha, production trading,
production order mutation, listenKey lifecycle, signed WebSocket user stream
runtime, real-funds, production portfolio parity, or Dashboard order-control
release.

`v0.13.0` is the published Guarded Live Alpha Preflight line. The V130-001
scope decision limits it to preflight evidence only before any live command or
production trading capability is claimed. `v0.14.0` is the published follow-up
read-only/dry-run line. `v0.15.0` is the published mutation-scope/request-preview
dry-run line. `v0.16.0` is the published owner-approved single-order candidate
line. `v0.17.0` is the published reconciliation and orphan-risk evidence line.
`v0.18.0` is the published owner-approved cancel recovery preview line.
`v0.18.1` is the release surface and provenance hardening patch. `v0.19.0` is
the next capability track for owner-approved single-shot actual cancel.

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
