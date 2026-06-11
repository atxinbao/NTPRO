# Plugin Unsafe Boundary Register

Date: 2026-06-11
Executor: Codex
Task: P1-005

## Purpose

This register tracks the trusted local alpha boundary for NTPRO plug-in cdylib
loading. It does not approve plug-ins as a production-ready or untrusted-code
safe product surface for v0.2.

## Current Boundary

- Plug-ins are compiled cdylibs loaded into the host process.
- Loading a plug-in can execute code through dynamic-library initialization and
  through the exported `nautilus_plugin_init` symbol.
- A configured plug-in path must be treated as trusted local operator input.
- `LiveNodeConfig.plugins[].sha256` exists and is validated when supplied, but
  v0.2 does not require a digest for every plug-in.
- v0.2 does not claim plug-in sandboxing, hot reload, untrusted third-party
  loading, production crash isolation, or production ABI stability.

## Follow-up Items

| ID | Area | Current state | Required close condition |
| --- | --- | --- | --- |
| `PLUG-UNSAFE-001` | Trusted path allowlist | Operators can configure local paths directly. | Define an allowlist policy for trusted plug-in directories and document how operators manage it. |
| `PLUG-UNSAFE-002` | Canonical path policy | Relative paths resolve through the process working directory. | Canonicalize configured paths before loading and reject path traversal or unexpected roots. |
| `PLUG-UNSAFE-003` | Symlink escape checks | No dedicated symlink escape policy is documented for product claims. | Add tests and documentation for symlink handling under the allowlist policy. |
| `PLUG-UNSAFE-004` | Mandatory sha256 | `sha256` is optional in config. | Require sha256 for production-marked plug-in loading, with tests for mismatch, missing digest, and digest normalization. |
| `PLUG-UNSAFE-005` | ABI and build identity | ABI/build-id diagnostics exist, but v0.2 alpha does not promise compatibility. | Define production ABI/build-id acceptance policy and fixture-backed mismatch tests. |
| `PLUG-UNSAFE-006` | Panic boundary | Panic guards exist for thunk paths, but production crash-isolation claims are not made. | Audit every host-to-plug-in call path and add tests for panic conversion and node survivability. |
| `PLUG-UNSAFE-007` | Crash isolation | Plug-ins run in-process. | Decide whether production plug-ins remain in-process trusted code or move to an isolated process model. |
| `PLUG-UNSAFE-008` | Cancellation and shutdown | Plug-in lifecycle behavior is not part of the v0.2 production claim. | Add lifecycle cancellation/shutdown tests before productizing plug-in runtime control. |

## Product Claim Rule

Until these items are closed by dedicated tasks, public release text must use
phrasing such as:

```text
trusted local alpha plug-in boundary
```

Public release text must not call plug-ins:

- production-ready;
- safe for untrusted third-party cdylibs;
- sandboxed;
- hot-reloadable;
- enabled by default for v0.2 product claims.
