# RREM-015 Evidence

Date: 2026-06-02
Executor: Codex
Task ID: RREM-015

## Goal

Remove the standalone Cap'n Proto serialization product surface and carefully
audit the remaining Python/PyO3/Cython cleanup surface without bulk-deleting
Python tests.

## Plain Summary

This PR removes NTPRO's own Cap'n Proto serialization feature. It deletes the
schemas, generated Rust code, conversion modules, feature flags, tests, benches,
regeneration scripts, CI wiring, Makefile commands, and docs for that format.

It does not delete the remaining Python tests or scripts. Those files are still
classified as follow-up work because many of them may encode behavior regression
coverage.

## Files Changed

- `Cargo.toml`, `Cargo.lock`
- `crates/serialization/**`
- `crates/common/**`
- `.github/**`
- `.pre-commit-config.yaml`
- `Makefile`
- `scripts/install-capnp.sh`, `scripts/regen-capnp.sh`
- `docs/getting_started/**`
- `docs/developer_guide/**`
- `docs/rust-cutover/**`

## Validation Commands

| Command | Result |
| --- | --- |
| `cargo metadata --format-version=1 --no-deps` | Pass |
| `cargo tree -i capnpc --workspace --all-features` | Expected fail: no `capnpc` package remains. |
| `cargo tree -i capnp --workspace --all-features` | Pass; shows only transitive `hypersync-client` dependency. |
| `cargo fmt --check` | Pass |
| `scripts/ai/validate_agentflow_roles.py` | Pass |
| `scripts/ai/verify_fast.sh` | Pass |
| `cargo check -p nautilus-common --all-targets` with Rust 1.95.0 | Pass |
| `cargo check -p nautilus-serialization --all-targets` with Rust 1.95.0 | Pass |
| `cargo check -p nautilus-serialization --all-targets --features sbe` with Rust 1.95.0 | Pass |
| `scripts/ai/check_cython_removed.sh` | Pass |
| `scripts/ai/check_no_cython_runtime.sh` | Expected fail; Rust crate Cython references remain. |
| `scripts/ai/check_rust_only_runtime.sh` | Expected fail; Rust crate PyO3/Cython references remain. |

Rust 1.95.0 was selected explicitly with:

```bash
RUSTC="$(rustup which rustc --toolchain 1.95.0)" rustup run 1.95.0 cargo check ...
```

The default Homebrew `cargo` path still points to Rust 1.87.0 in this local
shell, which is below the repository `rust-version = "1.95.0"` requirement.

## Cap'n Proto Scan

No active first-party Cap'n Proto entrypoints remain in Cargo manifests, Rust
source, CI, Makefile, pre-commit, tooling, scripts, or public install/developer
docs:

```bash
rg -n "capnp|Cap'n Proto|capnpc|regen-capnp|install-capnp|check-capnp" \
  Cargo.toml crates Makefile .github .pre-commit-config.yaml tools.toml \
  scripts docs/getting_started docs/developer_guide \
  --glob '!docs/rust-cutover/evidence/**'
```

Result: no matches.

`capnp` remains in `Cargo.lock` only because `hypersync-client` depends on it
transitively through the optional blockchain adapter path. The first-party
`nautilus-serialization` `capnp` feature and `capnpc` generator are removed.

## Python and C/C++ Audit

Tracked Python files remain:

```text
total=540
tests=534
scripts=6
other=0
```

Tracked C/C++ files:

```text
git ls-files '*.c' '*.h' '*.cpp' '*.hpp' -> 0
```

The remaining Python files are tests and local scripts only. They are not
deleted in RREM-015.

## Behavior Impact

Trading semantics, adapter behavior, data model semantics, order matching, and
persistence behavior are unchanged.

Downstream code that enabled `nautilus-serialization/capnp` or imported
`nautilus_serialization::capnp` must migrate to Arrow, Parquet/catalog storage,
JSON, MsgPack, SBE, or project-local serialization.

## Review Status

Risk level: critical.

This task must stop at `REVIEW_REQUIRED`. Auto-merge is disabled.
