# NTPRO Rust-only v0.2.0 Release Notes

Date: 2026-06-09
Executor: Codex
Task ID: P0-001

## Release Identity

```text
Current source tag: ntpro-rust-only-v0.2.0
Capability: Local Multi-Node Runtime Foundation
```

The v0.2.0 release is a tagged source release for the local multi-node runtime
foundation. It builds on the Rust-only cutover baseline and records that the
local supervisor and sandbox node process path can be validated from Rust-only
tooling.

## Included

- Rust-only source build path through Cargo.
- Local `nautilus` CLI commands for sandbox-first workflow validation.
- Local supervisor registry for sandbox node records.
- Local `ntpro-node` process start/stop/status/logs/metrics workflows.
- Two-node local sandbox smoke evidence.
- Rust-only runtime checks and release evidence.

## Not Included

v0.2.0 does not claim these capabilities:

- Dashboard release scope.
- Production exchange connectivity.
- Real account connectivity.
- Real order submission.
- Manual order entry.
- Distributed multi-server deployment.
- Prebuilt binary, Docker, PyPI, Python wheel, or other release artifact
  delivery.

## Validation Source

Primary readiness evidence:

- `docs/rust-cutover/release/v0_2_local_multi_node_readiness_report.md`
- `docs/rust-cutover/evidence/V02-010.md`

The release still requires strong local gate execution before future tag or
release publication decisions. `verify_fast.sh` remains a fast smoke only and
does not replace full release validation.

## Migration Note Status

No Python, PyO3, or Cython product surface is restored by v0.2.0. Users should
follow the Rust CLI, Rust crates, Rust examples, and Rust documentation paths.

