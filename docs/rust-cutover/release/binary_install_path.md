# NTPRO Binary And Install Path Decision

Date: 2026-06-04
Executor: Codex
Task ID: NBIN-001

## Purpose

This record defines how users install and run the NTPRO Rust CLI during the
v0.2.0 product-hardening phase.

It is a decision record only. It does not publish binaries, create release
tags, create GitHub Releases, publish crates, or introduce Docker delivery.

## Decisions

### Supported Source Build Path

The supported public install path is a source checkout plus Cargo build:

```bash
git clone https://github.com/atxinbao/NTPRO.git
cd NTPRO
rustup toolchain install 1.95.0
rustup override set 1.95.0
cargo run -p nautilus-cli -- --help
```

For a fixed release source point, clone the release tag:

```bash
git clone --branch ntpro-rust-only-v0.2.0 --depth 1 https://github.com/atxinbao/NTPRO.git
cd NTPRO
rustup toolchain install 1.95.0
rustup override set 1.95.0
cargo run -p nautilus-cli -- --help
```

Active development after the v0.2.0 source release uses `main`.

### Local Cargo Install

Local `cargo install` from a checked-out source tree is supported:

```bash
cargo install --path crates/cli --bin nautilus --locked --force
nautilus --help
```

This installs the existing CLI package:

```text
package: nautilus-cli
binary:  nautilus
source:  crates/cli/src/bin/cli.rs
```

This support means "build and install from this repository checkout." It does
not mean the package is published on crates.io.

### crates.io Cargo Install

`cargo install nautilus-cli` from crates.io is not a supported NTPRO path in
v0.2.0.

Reason:

- NTPRO has not published `nautilus-cli` as an NTPRO-owned crates.io release.
- The current public release surface is the GitHub source repository and tags.
- Publishing crates would require a separate release policy, package ownership
  check, versioning decision, and release gate.

### Release Artifact Strategy

NTPRO v0.2.0 does not require prebuilt binary release artifacts.

The current release artifact is the tagged source tree. If a later task
approves prebuilt binaries, the expected GitHub Release asset pattern is:

```text
nautilus-<version>-<target-triple>.tar.gz
nautilus-<version>-<target-triple>.zip
checksums.txt
```

The archive should contain a single executable named `nautilus` plus any
required license and notice files. Checksums must be published with the release
assets before users are instructed to download them.

Any future prebuilt binary task must include at least:

- platform matrix;
- archive layout;
- checksum generation and verification;
- local smoke for `nautilus --help`;
- release notes update;
- rollback plan.

### Binary Naming

The product binary name is:

```text
nautilus
```

The Cargo package that provides it is:

```text
nautilus-cli
```

Documentation should refer to the executable as `nautilus`, and to the Cargo
package only when the build command requires `-p nautilus-cli`.

### Platform Scope

Current platform scope is source-build first:

| Platform | Architecture | v0.2.0 status |
| --- | --- | --- |
| macOS 15+ | ARM64 | Primary local release-gate platform. |
| Linux Ubuntu 22.04+ | x86_64 | Supported source-build target. |
| Linux Ubuntu 22.04+ | ARM64 | Supported source-build target. |
| Windows Server 2022+ | x86_64 | Source-build target; prebuilt binary delivery deferred. |

Prebuilt binary publication remains deferred until a dedicated binary release
task validates the target matrix.

### Docker Delivery

Docker delivery is explicitly deferred for v0.2.0.

Docker images, Jupyter images, and Docker-based install paths are not the
default NTPRO product delivery path. A later owner-approved task may define a
Docker policy, but NBIN-001 does not add one.

## Unsupported Paths

The following install paths are not supported NTPRO product paths in v0.2.0:

- `pip install`;
- Python wheels or source distributions;
- PyPI package publication;
- PyO3, maturin, or Cython product builds;
- `cargo install nautilus-cli` from crates.io;
- prebuilt binary download commands;
- R2 or upstream NautilusTrader package download paths;
- Docker or Jupyter image delivery.

## User-Facing Commands

After building or installing, users should verify the CLI with:

```bash
nautilus --help
nautilus backtest --help
nautilus sandbox --help
nautilus live --help
```

From a source checkout, the equivalent Cargo commands are:

```bash
cargo run -p nautilus-cli -- --help
cargo run -p nautilus-cli -- backtest --help
cargo run -p nautilus-cli -- sandbox --help
cargo run -p nautilus-cli -- live --help
```

## Follow-Up Work

Potential follow-up tasks:

- define and test GitHub Release binary assets;
- add a checksum verification command for future assets;
- add platform-specific install docs if prebuilt binaries are approved;
- decide whether crates.io publication is in scope for a future release;
- replace or retire stale upstream installer scripts that reference external
  package hosts not used by NTPRO.
