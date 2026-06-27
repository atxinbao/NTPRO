# Releases

Date: 2026-06-04
Executor: Codex

This guide documents the NTPRO Rust-only release process.

NTPRO is not using the upstream NautilusTrader Python wheel, PyPI, R2 package,
or Docker image release path. The current product release surface is the Rust
source repository, Git tags, GitHub Releases, Cargo-built CLI binary, Rust
examples, and local verification evidence.

## Current Release Model

The current formal release is:

```text
ntpro-rust-only-v0.1.0
```

The supported install artifact for this release is the tagged source tree. See:

- `docs/getting_started/installation.md`
- `docs/rust-cutover/release/binary_install_path.md`
- `docs/rust-cutover/release/rust_only_release_notes.md`
- `docs/rust-cutover/release/final_release_verification.md`

## Branch And Tag Policy

NTPRO uses `main` as the public development branch.

Release tags are created only after explicit owner approval. Tags are not
created by routine task automation.

Pre-release tags may use names such as:

```text
ntpro-rust-only-rc.1
ntpro-rust-only-rc.2
ntpro-rust-only-rc.3
```

Formal release tags use names such as:

```text
ntpro-rust-only-v0.1.0
```

## Release Artifact Policy

Current supported artifact:

```text
GitHub source tag
```

Current supported user build path:

```bash
git clone --branch ntpro-rust-only-v0.1.0 --depth 1 https://github.com/atxinbao/NTPRO.git
cd NTPRO
rustup toolchain install 1.95.0
rustup override set 1.95.0
cargo run -p nautilus-cli -- --help
```

Local install from a checked-out source tree is supported:

```bash
cargo install --path crates/cli --bin nautilus --locked --force
nautilus --help
```

Unsupported release artifacts:

- Python wheels;
- Python source distributions;
- PyPI publication;
- PyO3 or Cython extension builds;
- upstream NautilusTrader R2 package downloads;
- Docker or Jupyter images as the default delivery path;
- crates.io `cargo install nautilus-cli`;
- prebuilt GitHub binary downloads.

## Binary Naming

The product executable is:

```text
nautilus
```

The Cargo package is:

```text
nautilus-cli
```

Documentation should describe user commands with `nautilus` after installation
and with `cargo run -p nautilus-cli -- ...` when running from source.

## Future Binary Assets

Prebuilt binary assets are deferred. If a future owner-approved task adds them,
the expected artifact pattern is:

```text
nautilus-<version>-<target-triple>.tar.gz
nautilus-<version>-<target-triple>.zip
checksums.txt
```

A future binary release task must prove:

- target platform matrix;
- archive layout;
- checksum generation and verification;
- `nautilus --help` smoke after unpacking;
- release notes update;
- rollback plan.

## Platform Scope

| Platform | Architecture | Current release status |
| --- | --- | --- |
| macOS 15+ | ARM64 | Primary local release-gate platform. |
| Linux Ubuntu 22.04+ | x86_64 | Supported source-build target. |
| Linux Ubuntu 22.04+ | ARM64 | Supported source-build target. |
| Windows Server 2022+ | x86_64 | Source-build target; prebuilt binary delivery deferred. |

## Required Verification

Fast local smoke:

```bash
scripts/ai/verify_fast.sh
```

This is a fast smoke only. By default it covers the pinned Rust toolchain and
`cargo fmt --check`; it is not release validation, is not release evidence,
and does not replace workspace `cargo check`, clippy, golden traces,
`verify_release.sh`, or strict provenance.

Compile and lint gate:

```bash
VERIFY_FAST_CARGO_CHECK=1 VERIFY_FAST_CLIPPY=1 scripts/ai/verify_fast.sh
```

Full test gate:

```bash
scripts/ai/verify_full.sh
```

Release-oriented gate:

```bash
scripts/ai/check_rust_only_runtime.sh
scripts/ai/check_cython_removed.sh
scripts/ai/verify_release.sh
scripts/ai/verify_release_strict.sh v18
```

Golden trace gate:

```bash
scripts/ai/run_golden_traces.sh
REQUIRE_GOLDEN_REPLAY=1 scripts/ai/run_golden_traces.sh
```

Use `verify_release.sh` for release evidence. It is expected to be slower than
the fast gate.

## Release Checklist

Before opening a release PR:

- [ ] Update release notes.
- [ ] Confirm README and installation docs match the release state.
- [ ] Confirm `docs/rust-cutover/release/final_release_verification.md` is
  current.
- [ ] Run the required local release gates or document any blocker.
- [ ] Confirm no Python, PyO3, Cython, wheel, PyPI, or Docker path is presented
  as an NTPRO product entrypoint.

Before creating a tag:

- [ ] Owner has explicitly approved the tag name.
- [ ] The release PR has merged to `main`.
- [ ] `main` is synced locally.
- [ ] Release notes identify the exact source point.
- [ ] No open release-blocking checks or manual gates remain.

Before publishing a GitHub Release:

- [ ] Owner has explicitly approved publishing.
- [ ] The tag exists on the remote.
- [ ] Release title is agreed.
- [ ] Release notes are current on `main`.
- [ ] Pre-release vs final-release status is explicit.

## Release Notes

Release notes live under:

```text
docs/rust-cutover/release/rust_only_release_notes.md
```

For future releases, keep notes concise and user-facing:

- what changed;
- what did not change;
- install path;
- verification result;
- unsupported paths;
- known blockers or deferred work.

## Manual Gates

The following actions require explicit owner approval:

- creating a release tag;
- publishing a GitHub Release;
- publishing prebuilt binaries;
- publishing crates to crates.io;
- changing the public release contract;
- reintroducing Docker as a supported product delivery path.

Routine task automation must stop and report instead of performing these
actions.
