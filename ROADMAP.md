# NTPRO Roadmap

Date: 2026-06-14
Executor: Codex

NTPRO is a Rust-only release workspace for the trading engine cutover from
NautilusTrader. The current public source release is
`ntpro-rust-only-v0.7.1`, and the next capability track is `v0.8.0`.

## Current Release Surface

Current published release:

```text
ntpro-rust-only-v0.7.1
```

Current capability boundary:

```text
real Binance testnet public HTTP read-only connectivity proof
default offline, manual online gate only
no authenticated account access
no real funds
no real order submission
no production trading
```

`v0.7.0` builds on the v0.6 testnet dry-run foundation and keeps v0.5 workflow
artifact coverage in the release tree. The current public claim is limited to
public read-only testnet connectivity proof behind fail-closed gates.

## Published Hardening Patch: v0.7.1

`v0.7.1` is the published hardening patch for the `v0.7.0` surface. It does
not expand the capability claim, does not add order submission, and keeps the
default local/CI path offline.

Completed hardening work:

- wire v0.7 default offline and manual-online preflight scripts into
  `verify_release.sh`, PR smoke, and hosted release gate;
- align Roadmap, readiness, and release-facing wording with v0.7.1 as the
  current public release;
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

## Next Capability Track: v0.8.0

`v0.8.0` is the next planned capability track after the v0.7.1 hardening
release. Its
intended direction is authenticated Binance testnet read-only proof.

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
