# RHARD-002 Toolchain and Verification Path Hardening Evidence

Date: 2026-06-04
Executor: Codex
Task ID: RHARD-002
Risk: medium

## Scope

RHARD-002 prevents local verification from silently using the wrong Rust
compiler.

## Finding

Before this task, the active shell resolved Rust through Homebrew:

```text
rustc 1.87.0 (17067e9ac 2025-05-09) (Homebrew)
cargo 1.87.0 (Homebrew)
/opt/homebrew/bin/rustc
/opt/homebrew/bin/cargo
```

The workspace release path requires Rust `1.95.0`. `rustup` had
`1.95.0-aarch64-apple-darwin` installed, but plain `rustc` and `cargo` still
resolved to Homebrew because of PATH order.

## Changes

- Added `scripts/ai/toolchain_env.sh`.
- Updated Cargo-running verification scripts to source the toolchain helper:
  - `scripts/ai/verify_fast.sh`;
  - `scripts/ai/verify_full.sh`;
  - `scripts/ai/verify_release.sh`;
  - `scripts/ai/run_golden_traces.sh`;
  - `scripts/ai/check_rust_only_runtime.sh`;
  - `scripts/ai/verify_cli_help.sh`.
- Added `docs/rust-cutover/verification/toolchain.md`.
- Updated `docs/getting_started/installation.md`.
- Updated `README.md`.

The helper resolves `rustc` and `cargo` through:

```bash
rustup which rustc --toolchain 1.95.0
rustup which cargo --toolchain 1.95.0
```

It prepends the selected toolchain `bin` directory to `PATH`, exports `RUSTC`,
and fails early if the resolved compiler is not Rust `1.95.0`.

## Local Toolchain Setup

The repository override was set locally:

```bash
rustup override set 1.95.0
```

The plain shell still reported Homebrew Rust because `/opt/homebrew/bin` is
ahead of rustup shims, so the verification scripts now pin the toolchain
themselves instead of relying only on `rustup override`.

## Commands Run

```bash
rustc --version
cargo --version
command -v rustc
command -v cargo
rustup override list
rustup which rustc --toolchain 1.95.0
rustup which cargo --toolchain 1.95.0
```

Toolchain helper smoke:

```bash
bash -c 'source scripts/ai/toolchain_env.sh; command -v rustc; rustc --version; command -v cargo; cargo --version'
```

Result:

```text
/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/rustc
rustc 1.95.0 (59807616e 2026-04-14)
/Users/mac/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo
cargo 1.95.0 (f2d3ce0bd 2026-03-21)
```

Validation:

```bash
bash -n scripts/ai/toolchain_env.sh scripts/ai/verify_fast.sh scripts/ai/verify_full.sh scripts/ai/verify_release.sh scripts/ai/run_golden_traces.sh scripts/ai/check_rust_only_runtime.sh scripts/ai/verify_cli_help.sh
git diff --check
scripts/ai/validate_agentflow_roles.py
scripts/ai/verify_fast.sh
scripts/ai/check_rust_only_runtime.sh
bash -c 'source scripts/ai/toolchain_env.sh; cargo metadata --no-deps --format-version=1 >/tmp/ntpro-rhard-002-cargo-metadata.json; wc -c /tmp/ntpro-rhard-002-cargo-metadata.json'
```

Results:

- Shell syntax check: passed.
- `git diff --check`: passed.
- `validate_agentflow_roles.py`: passed.
- `verify_fast.sh`: passed and printed Cargo/Rust `1.95.0`.
- `check_rust_only_runtime.sh`: passed.
- `cargo metadata` through `toolchain_env.sh`: passed.

Supplemental command:

```bash
scripts/ai/verify_cli_help.sh
```

Result: started and reached `verify_cli_help: top-level help`, then was
terminated after several minutes with no additional output. This command is a
supplemental CLI compile/run check, not the required RHARD-002 validation
command. The process was terminated and no stale `verify_cli_help` or Cargo
process remained.

## Behavior Impact

No trading behavior changed. The change affects local verification toolchain
selection only.

## Rollback Plan

Revert this PR to remove `toolchain_env.sh` and return verification scripts to
their previous PATH-dependent behavior.
