# NTPRO Rust Toolchain Verification

Date: 2026-06-04
Executor: Codex
Task ID: RHARD-002

## Required Toolchain

NTPRO verification uses Rust `1.95.0`.

Install it with:

```bash
rustup toolchain install 1.95.0
```

Set the repository override from the NTPRO root:

```bash
cd /Users/mac/Documents/NTPRO
rustup override set 1.95.0
```

## Homebrew Rust Pitfall

On macOS, Homebrew can put `/opt/homebrew/bin/rustc` and
`/opt/homebrew/bin/cargo` before rustup shims on `PATH`. In that case
`rustup override set 1.95.0` exists, but plain `rustc` and `cargo` may still
resolve to an older Homebrew compiler.

Check the active shell:

```bash
command -v rustc
rustc --version
command -v cargo
cargo --version
rustup override list
```

The expected compiler is:

```text
rustc 1.95.0
```

If the active compiler is older or says `Homebrew`, either put the rustup
toolchain before Homebrew on `PATH`:

```bash
export PATH="$HOME/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin:$PATH"
```

or run commands through rustup explicitly:

```bash
rustup run 1.95.0 cargo fmt --check
rustup run 1.95.0 cargo check --workspace
```

For local shells where Homebrew must stay on `PATH` for other projects, use a
path-scoped wrapper in a directory that already appears before Homebrew, such as
`$HOME/.local/bin`. The wrapper should only switch toolchains when `PWD` is
inside `/Users/mac/Documents/NTPRO`; outside NTPRO it should delegate back to the
normal Homebrew command. This keeps ordinary `cargo` and `rustc` commands inside
NTPRO on Rust `1.95.0` without changing unrelated workspaces.

## Verification Script Behavior

The NTPRO verification scripts source `scripts/ai/toolchain_env.sh` before
running Cargo. That helper:

- resolves `rustc` and `cargo` through `rustup which --toolchain 1.95.0`;
- prepends the selected toolchain `bin` directory to `PATH`;
- exports `RUSTC` to the matching compiler;
- fails early if the resolved compiler is not Rust `1.95.0`.

This keeps local verification from silently using Homebrew Rust `1.87.0` or
another stale compiler.

## Supported Commands

Fast local check:

```bash
scripts/ai/verify_fast.sh
```

Broader local check:

```bash
scripts/ai/verify_full.sh
```

Release check:

```bash
scripts/ai/verify_release.sh
```

Rust-only runtime surface check:

```bash
scripts/ai/check_rust_only_runtime.sh
```

CLI help check:

```bash
scripts/ai/verify_cli_help.sh
```

Golden trace check:

```bash
scripts/ai/run_golden_traces.sh
```
