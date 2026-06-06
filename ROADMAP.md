# NTPRO Roadmap

NTPRO is a Rust-only release workspace for the trading engine cutover from
NautilusTrader. The formal Rust-only source release point is
`ntpro-rust-only-v0.1.0`; work after that tag is post-release hardening and
v0.2 planning.

This roadmap describes the current NTPRO product direction. Historical
NautilusTrader Python, PyO3, Cython, wheel, and PyPI paths are not NTPRO product
surfaces.

## Current Product Boundary

Supported product surfaces:

- Rust workspace crates.
- Rust CLI commands and command contracts.
- Rust examples and documentation.
- Rust release verification scripts and golden trace evidence.
- Local Python helper scripts under `scripts/` only, used for repository
  control or release evidence.

Unsupported product surfaces:

- Python package installation.
- Python import/API usage.
- PyO3 bindings.
- Cython build or runtime paths.
- Python wheels, PyPI publication, or mixed Rust/Python packaging.
- Cap'n Proto serialization.

## v0.2 Priorities

### 1. Rust CLI Product Entrypoint

Goal: make the CLI honest, useful, and easy to validate from Cargo.

Planned work:

- Keep the CLI capability matrix explicit: implemented, simulated demo, or
  deferred.
- Finish real `config validate` and `data inspect/validate` user paths.
- Continue wiring supported backtest and live/sandbox runtime paths without
  presenting stubs as completed trading workflows.
- Keep `--help` text aligned with actual behavior.

Non-goal for this phase: claiming full backtest/live runtime coverage before
the wiring and evidence exist.

### 2. Rust Examples And Documentation

Goal: let a new user follow Rust-only docs without falling into legacy Python
setup instructions.

Planned work:

- Expand Rust examples for supported CLI and crate paths.
- Keep public docs aligned with the Rust-only contract.
- Mark any retained upstream Python tutorials as legacy or remove them from the
  active user path.
- Keep migration notes clear when an old Python/PyO3/Cython path has no Rust
  replacement yet.

### 3. Adapter Support Matrix

Goal: make supported, experimental, deferred, and removed integrations visible.

Planned work:

- Maintain adapter classification docs.
- Add fixture, mock, dry-run, or sandbox evidence for supported adapters.
- Avoid real exchange API requirements in automated evidence.
- Keep secrets out of code and test fixtures.

### 4. Release Delivery

Goal: make source builds and local binary installation predictable before
adding heavier distribution channels.

Planned work:

- Keep `cargo install --path crates/cli --bin nautilus --locked --force` as the
  source-build install path.
- Continue strengthening release verification and GitHub smoke gates.
- Decide whether a Rust-only binary release workflow is needed for v0.2 or
  should wait for a later release.
- Keep Docker images, Python wheels, and PyPI publication out of the NTPRO
  product release path until dedicated Rust-only workflows are approved.

### 5. Runtime Hardening And Regression Evidence

Goal: reduce product-reachable surprises before expanding the product surface.

Planned work:

- Convert product-boundary panics into explicit errors or unsupported statuses.
- Keep internal invariants classified rather than rewriting them blindly.
- Maintain ignored-test risk registers and close high-impact ignored tests in
  scoped slices.
- Extend golden trace and smoke evidence for product-critical flows.

## v0.2 Readiness Gates

v0.2 should not be tagged until these are true:

- `scripts/ai/verify_release.sh` passes or any skipped portion has an explicit
  owner-approved scope decision.
- Public docs do not describe Python, PyO3, or Cython as current NTPRO product
  entrypoints.
- The CLI capability matrix is current.
- Supported examples and adapter claims have local evidence.
- Open high-risk audit blockers are closed or formally scoped out.
- Release notes describe what is supported, simulated, deferred, and not
  supported.

## Out Of Scope For v0.2

- Python package, Python API, PyO3 binding, Cython, wheel, or PyPI support.
- A dashboard UI.
- Distributed or massively parallel backtest orchestration.
- Production Docker image publication.
- Full live trading adapter parity across every upstream venue.
- Unscoped trading-semantic changes without golden trace coverage.

## Contribution Direction

Contributions should follow the Rust-only product boundary:

- Prefer Rust crates, Rust CLI, Rust examples, and Rust documentation.
- For adapters, start with classification, fixtures, mock validation, and clear
  support boundaries.
- Do not revive Python/PyO3/Cython product paths in new work.
- Do not present a stub, dry-run, or simulated demo as a completed runtime
  workflow.

## Future Phases

After v0.2, the project can consider:

- stronger binary release automation;
- broader adapter parity;
- deeper trace coverage;
- richer CLI-driven operational workflows;
- optional dashboard/control-plane design only after runtime status and control
  APIs are ready.
