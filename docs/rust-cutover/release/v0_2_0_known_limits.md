# NTPRO Rust-only v0.2.0 Known Limits

Date: 2026-06-09
Executor: Codex
Task ID: P0-001

## Scope

v0.2.0 is limited to:

```text
Local Multi-Node Runtime Foundation
```

It is a local, sandbox-first foundation for future Dashboard, control, and
runtime hardening work. It is not a production trading release.

## Product Limits

- Dashboard is not part of the v0.2.0 release claim.
- Production exchange connectivity is not included.
- Real account connectivity is not included.
- Real order submission is not included.
- Manual order entry is not included.
- Distributed multi-server deployment is not included.
- Prebuilt binary, Docker, PyPI, and Python wheel delivery are not included.
- Python, PyO3, and Cython remain unsupported product surfaces.

## Runtime Limits

- Supervisor and `ntpro-node` workflows are local sandbox workflows.
- Evidence uses local artifacts and sandbox smoke paths.
- Real adapter cancellation and real exchange behavior are outside the v0.2.0
  product claim.
- Stronger process lifecycle, artifact atomicity, shutdown hardening, and
  startup cancellation work is tracked by the v0.2.0 audit remediation tasks
  `P0-004` through `P0-008`.
- Plug-in cdylib loading remains a trusted local alpha boundary. Plug-ins are
  not sandboxed, are not productionized in v0.2, and are not part of the
  default product claim. Follow-up hardening is tracked in
  `docs/rust-cutover/security/plugin_unsafe_register.md`.

## Release Limits

The v0.2.0 source tag does not imply release artifact delivery. Future prebuilt
binary or Docker delivery requires a separate task, release policy, checksums,
platform matrix, and explicit release approval.
