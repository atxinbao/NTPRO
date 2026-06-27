# Installation

NTPRO is distributed as a Rust-only source workspace. The supported public
installation path is to clone the NTPRO repository, select a source point, and
build or install the Rust CLI with Cargo.

NTPRO does not publish or support a Python package, PyPI install path, Python
wheel, PyO3 bridge, Cython build, or mixed Rust/Python packaging path.

## Supported release path

The current formal Rust-only release is:

```text
ntpro-rust-only-v0.1.0
```

Clone that source point:

```bash
git clone --branch ntpro-rust-only-v0.1.0 --depth 1 https://github.com/atxinbao/NTPRO.git
cd NTPRO
```

For active v0.2.0 development, use `main` instead of the release tag.

## Platform notes

NTPRO v0.1.0 is a Rust-only release source point, not a packaged binary
distribution. NBIN-001 keeps v0.2.0 source-build first: users build the
`nautilus` binary from the repository checkout, and prebuilt binary release
assets remain deferred until a later dedicated task.

| Operating system | CPU architecture | Current status |
| --- | --- | --- |
| macOS 15.0 and later | ARM64 | Release-gate development platform. |
| Linux Ubuntu 22.04 and later | x86_64 | Source-build target; verify locally. |
| Linux Ubuntu 22.04 and later | ARM64 | Source-build target; verify locally. |
| Windows Server 2022 and later | x86_64 | Rust source-build target; prebuilt binary delivery deferred. |

If you need redis-backed cache or message-bus workflows, see
[Redis](#redis). Otherwise Redis is not required for CLI help, source builds,
or local documentation validation.

## Install Rust

Install [rustup](https://rustup.rs/):

```bash tab="Linux/macOS"
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
```

```powershell tab="Windows"
# Download and install rustup-init.exe from https://win.rustup.rs/x86_64
# Then start a new PowerShell session.
```

Install the pinned release toolchain:

```bash
rustup toolchain install 1.95.0
rustup override set 1.95.0
rustc --version
cargo --version
```

If `rustc --version` or `cargo --version` still reports a Homebrew compiler
or any version other than `1.95.0`, see
[NTPRO Rust Toolchain Verification](../rust-cutover/verification/toolchain.md).
The local verification scripts pin the Rust `1.95.0` toolchain before running
Cargo so stale PATH entries cannot silently validate the wrong compiler.

## Build from source

Run the CLI directly from the repository checkout:

```bash
cargo run -p nautilus-cli -- --help
```

Build the executable without installing it globally:

```bash
cargo build -p nautilus-cli --bin nautilus
./target/debug/nautilus --help
```

For an optimized local binary:

```bash
cargo build -p nautilus-cli --bin nautilus --release
./target/release/nautilus --help
```

## Install the CLI locally

Local Cargo installation from a checked-out NTPRO source tree is supported:

```bash
cargo install --path crates/cli --bin nautilus --locked --force
nautilus --help
```

This installs the `nautilus` executable from the `nautilus-cli` package into
Cargo's bin directory, normally `$HOME/.cargo/bin`.

NTPRO does not currently support `cargo install nautilus-cli` from crates.io,
because the NTPRO CLI package has not been published there as an NTPRO release.

## Binary release policy

The current supported release artifact is the tagged source tree. NTPRO does
not currently publish prebuilt `nautilus` binaries for GitHub Releases.

If prebuilt binaries are approved later, the expected artifact pattern is
documented in
[NTPRO Binary And Install Path Decision](../rust-cutover/release/binary_install_path.md).

Do not use old upstream R2 or package-host installer paths for NTPRO unless a
future NTPRO release task explicitly reintroduces them.

## Install native build tools

Linux builds should have `clang` and `lld` available:

```bash tab="Linux"
sudo apt-get update
sudo apt-get install clang lld
```

macOS builds require the Apple command line tools:

```bash tab="macOS"
xcode-select --install
```

Windows builds require the Visual Studio 2022 C++ build tools and the Rust
MSVC toolchain.

## Run the Rust CLI

The Rust CLI is the first supported product entrypoint:

```bash
cargo run -p nautilus-cli -- --help
cargo run -p nautilus-cli -- backtest --help
cargo run -p nautilus-cli -- sandbox --help
cargo run -p nautilus-cli -- live --help
```

These commands must not require Python, PyO3, Cython, or a Python virtual
environment.

After local `cargo install`, the equivalent executable commands are:

```bash
nautilus --help
nautilus backtest --help
nautilus sandbox --help
nautilus live --help
```

## Local verification

Run the fast local smoke before opening a PR:

```bash
scripts/ai/verify_fast.sh
```

`verify_fast.sh` is intentionally a fast smoke. By default it verifies the
pinned toolchain and `cargo fmt --check`; it is not release validation and is
not release evidence. It does not replace workspace `cargo check`, clippy,
golden traces, release gates, or strict provenance.

For compile/lint-oriented checks, run:

```bash
VERIFY_FAST_CARGO_CHECK=1 VERIFY_FAST_CLIPPY=1 scripts/ai/verify_fast.sh
scripts/ai/verify_full.sh
```

For release-oriented checks, run:

```bash
scripts/ai/check_rust_only_runtime.sh
scripts/ai/check_cython_removed.sh
scripts/ai/verify_release.sh
scripts/ai/verify_release_strict.sh v18
```

`verify_release.sh` is broader and slower than the fast gate. It is intended
for release evidence, not every local edit.

## Redis

Using [Redis](https://redis.io) is optional. It is only needed when a workflow
explicitly configures Redis as the backend for a cache database or
[message bus](../concepts/message_bus.md).

The minimum supported Redis version is 6.2.

For a quick local instance:

```bash
docker run -d --name redis -p 6379:6379 redis:latest
```

Manage the container with:

```bash
docker start redis
docker stop redis
```

## Precision mode

NTPRO keeps the Rust precision modes from the NautilusTrader lineage:

- **High precision**: 128-bit integers with up to 16 decimals of precision.
- **Standard precision**: 64-bit integers with up to 9 decimals of precision.

The default is standard precision unless the `high-precision` Rust feature is
enabled.

To enable high precision from a Rust dependency:

```toml
[dependencies]
nautilus_core = { version = "*", features = ["high-precision"] }
```

For workspace builds, prefer the existing Cargo feature configuration and
record the exact command in PR evidence.

## Unsupported install paths

The following are not NTPRO product entrypoints:

- `pip install` or `uv pip install`;
- PyPI or third-party Python package indexes;
- Python wheels or source distributions;
- `maturin`, PyO3, Cython, or `build.py` product builds;
- `cargo install nautilus-cli` from crates.io;
- prebuilt binary download commands;
- upstream NautilusTrader R2 package download paths;
- Docker/Jupyter images as the default NTPRO delivery path.

Local Python scripts may still exist under `scripts/` for repository control,
audits, or release evidence. They are helper tooling only, not user-facing
runtime APIs.
