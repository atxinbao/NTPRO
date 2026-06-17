# NTPRO Roadmap

Date: 2026-06-17
Executor: Codex

NTPRO is a Rust-only release workspace for the trading engine cutover from
NautilusTrader. The current public source release is
`ntpro-rust-only-v0.8.0`, and the next patch track is `v0.8.1`.

## Current Release Surface

Current published release:

```text
ntpro-rust-only-v0.8.0
```

Current capability boundary:

```text
authenticated Binance testnet read-only proof
env-only testnet credentials
redacted account-shape artifact evidence
no real funds
no real order submission
no production trading
```

`v0.8.0` builds on the v0.7 public read-only testnet connectivity proof and
adds an authenticated Binance testnet read-only account-shape proof. The current
public claim remains fail-closed, testnet-only, artifact-first, and explicitly
non-production.

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

## Next Safety Patch Track: v0.8.1

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

## Corrected Capability Sequence: v0.9.0 through v0.12.0

The previous idea of making `v0.9.0` a Binance testnet order lifecycle proof is
superseded. `v0.8.0` is still authenticated Binance testnet read-only proof,
and the current live smoke surface is still a sandbox/fixture-oriented local
node foundation. NTPRO must prove that `ntpro-node` can host strategy runtime
state before it attempts testnet order submission.

Corrected sequence:

```text
v0.9.0  = Strategy Runtime Foundation
v0.10.0 = Binance Testnet Order Proof
v0.11.0 = Production Read-Only + Shadow
v0.12.0 = Guarded Live Alpha
```

`v0.9.0` is the next capability track. It must make `ntpro-node` a headless
strategy runtime host that can load a strategy session config, run a built-in
demo strategy against fixture/mock market input, write signal/order-intent/risk
decision/audit artifacts, expose supervisor status, and show read-only
Dashboard state.

`v0.9.0` explicitly does not include:

- Binance testnet order submission;
- order cancel/replace/amend;
- production order submission;
- real funds;
- production trading;
- Dashboard order controls;
- strategy-driven live execution through an exchange adapter.

`v0.10.0` is the first track where Binance testnet order proof may be planned,
behind a separate explicit gate and with its own risk, redaction, and lifecycle
evidence.

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
