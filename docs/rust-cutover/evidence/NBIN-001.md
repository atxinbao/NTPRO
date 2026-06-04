# NBIN-001 Release Binary And Install Path Evidence

Date: 2026-06-04
Executor: Codex
Task ID: NBIN-001
Risk: medium

## Scope

NBIN-001 defines how users install and run the NTPRO Rust CLI during the
v0.2.0 product-hardening phase.

This task does not publish binaries, create release tags, publish GitHub
Releases, publish crates to crates.io, or introduce Docker delivery.

## Context Reviewed

- `docs/rust-cutover/tasks/NBIN-001.md`
- `docs/rust-cutover/scope/v0_2_0_roadmap.md`
- `docs/getting_started/installation.md`
- `docs/getting_started/index.md`
- `docs/developer_guide/releases.md`
- `docs/rust-cutover/product/RUST_API_ENTRYPOINTS.md`
- `docs/rust-cutover/product/RUST_CLI_CONTRACT.md`
- `crates/cli/Cargo.toml`
- `scripts/cli/install.sh`

## Decisions Recorded

- Supported public path: clone NTPRO source and build with Cargo.
- Supported local install path:
  `cargo install --path crates/cli --bin nautilus --locked --force`.
- crates.io install path: not supported for v0.2.0.
- Binary name: `nautilus`.
- Cargo package: `nautilus-cli`.
- Release artifact strategy: tagged source tree only for now; prebuilt binary
  assets are deferred.
- Future binary asset naming pattern is documented for later approval.
- Platform scope is source-build first:
  - macOS ARM64 primary local release-gate platform;
  - Linux x86_64 and ARM64 source-build targets;
  - Windows x86_64 source-build target with prebuilt binary delivery deferred.
- Docker and Jupyter delivery remain deferred.
- Old upstream R2/package-host installer paths are not supported NTPRO product
  paths.

## Files Changed

- `README.md`
- `docs/getting_started/index.md`
- `docs/getting_started/installation.md`
- `docs/developer_guide/releases.md`
- `docs/rust-cutover/release/README.md`
- `docs/rust-cutover/release/binary_install_path.md`
- `docs/rust-cutover/evidence/NBIN-001.md`
- `.agentflow/state/task_status.json`
- `.agentflow/leases/NBIN-001.json`

## Validation Commands

```bash
cargo run -q -p nautilus-cli -- --help
cargo install --path crates/cli --bin nautilus --locked --force --root /tmp/ntpro-nbin-001-install
/tmp/ntpro-nbin-001-install/bin/nautilus --help
git diff --check
scripts/ai/validate_agentflow_roles.py
scripts/ai/verify_fast.sh
```

## Results

- First unpinned `cargo run -q -p nautilus-cli -- --help` attempt failed
  because the shell defaulted to `rustc 1.87.0`. This confirms the documented
  toolchain requirement.
- Pinned-toolchain `cargo run -q -p nautilus-cli -- --help`: passed and
  printed the Rust CLI command surface.
- `cargo install --path crates/cli --bin nautilus --locked --force --root /tmp/ntpro-nbin-001-install`:
  passed and installed `/tmp/ntpro-nbin-001-install/bin/nautilus`.
- `/tmp/ntpro-nbin-001-install/bin/nautilus --help`: passed and printed the
  Rust CLI command surface.
- `git diff --check`: passed.
- `scripts/ai/validate_agentflow_roles.py`: passed.
- `scripts/ai/verify_fast.sh`: passed with Cargo/Rust `1.95.0`.

## Behavior Impact

No trading behavior changed. This task documents and validates the user-facing
install/run path for the existing Rust CLI.

## Public API Impact

No Rust public API changed. The documented executable name remains `nautilus`.

## Migration Note Status

Python, PyO3, Cython, wheel, PyPI, upstream R2, crates.io install, prebuilt
binary download, Docker, and Jupyter paths are documented as unsupported NTPRO
product install paths for v0.2.0.

## Rollback Plan

Revert this PR to remove the NBIN-001 decision record, evidence, and install
documentation changes.
