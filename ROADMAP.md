# NTPRO Roadmap

Date: 2026-06-14
Executor: Codex

NTPRO is a Rust-only release workspace for the trading engine cutover from
NautilusTrader. The current public source release is
`ntpro-rust-only-v0.6.0`, and the active hardening track is `v0.6.1`.

## Current Release Surface

Current published release:

```text
ntpro-rust-only-v0.6.0
```

Current capability boundary:

```text
Binance testnet dry-run runtime foundation
offline-only
no real funds
no real order submission
no production trading
```

`v0.6.0` absorbed the completed `v0.5.0` workflow artifact milestone. `v0.5.0`
is therefore part of the current release surface, but it is not a separate
public GitHub Release.

## Active Hardening Track: v0.6.1

`v0.6.1` is a hardening track for the published `v0.6.0` surface. It does not
expand the capability claim and must remain offline-only.

Planned work:

- align README, roadmap, release wording, Dashboard copy, and versioning docs;
- enforce a single source of truth for workflow `run_id`;
- harden `dry-run` and `connectivity-probe` wording so both remain offline;
- make workflow artifact browsing independent from supervisor registry state;
- audit child artifacts referenced by workflow manifests in Dashboard health;
- move v0.6 workflow smoke coverage into PR-stage CI;
- extract a shared workflow artifact contract for writer and reader;
- prepare v0.6.1 readiness report, release notes, and final gate.

v0.6.1 explicitly does not include:

- real Binance testnet network connection;
- real Binance testnet order submission;
- real account reconciliation;
- production Binance connectivity;
- production trading;
- remote or multi-user Dashboard operation;
- prebuilt binary or Docker release delivery.

## Next Capability Track: v0.7.0

`v0.7.0` is the first planned track allowed to cross from offline dry-run into
real Binance testnet read-only connectivity proof.

The only intended boundary change is:

```text
offline dry-run -> real Binance testnet read-only connectivity proof
```

Required constraints:

- no order submission;
- no real account trading;
- no production trading claim;
- Dashboard remains read-only and must not start probes;
- secrets are never written to artifacts, stdout, logs, docs, or PR bodies;
- default CI remains offline;
- manual online verification is opt-in only.

Real network access must fail closed unless all of these are true:

- `--allow-testnet-network` is passed;
- `NTPRO_ALLOW_TESTNET_NETWORK=1` is set;
- config environment is `testnet`;
- `order_submission = disabled`;
- `real_orders_submitted = false`.

The v0.7.0 primary online proof is HTTP read-only connectivity. WebSocket
read-only connectivity is optional/manual and must not become a default CI
release blocker.

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
