# Installation

NTPRO is distributed as a Rust-only source workspace. The supported public
installation path is to clone the NTPRO repository, select the release tag, and
build or run the Rust workspace with Cargo.

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
distribution. Platform support is therefore validated through local source
builds and release verification scripts.

| Operating system | CPU architecture | Current status |
| --- | --- | --- |
| macOS 15.0 and later | ARM64 | Release-gate development platform. |
| Linux Ubuntu 22.04 and later | x86_64 | Source-build target; verify locally. |
| Linux Ubuntu 22.04 and later | ARM64 | Source-build target; verify locally. |
| Windows Server 2022 and later | x86_64 | Rust source-build target; formal binary policy is deferred to `NBIN-001`. |

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

## Local verification

Run the fast local gate before opening a PR:

```bash
scripts/ai/verify_fast.sh
```

For release-oriented checks, run:

```bash
scripts/ai/check_rust_only_runtime.sh
scripts/ai/check_cython_removed.sh
scripts/ai/verify_release.sh
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
- Docker/Jupyter images as the default NTPRO delivery path.

Local Python scripts may still exist under `scripts/` for repository control,
audits, or release evidence. They are helper tooling only, not user-facing
runtime APIs.
