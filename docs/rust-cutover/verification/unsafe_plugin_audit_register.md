# Unsafe And Plugin Audit Register

Date: 2026-06-05
Executor: Codex
Task ID: NAUDIT-007

## Purpose

This register records unsafe and plug-in boundary risks that must be resolved or
explicitly scoped before NTPRO documents plug-ins as stable product
functionality.

This is an audit register only. It does not change the actor registry, plug-in
loader, FFI ABI, live node startup behavior, or plug-in public API.

## Product Status

Plug-ins remain an unstable extension surface for NTPRO v0.2.

Allowed wording:

- early alpha;
- unstable;
- process-startup only;
- not a stable product extension API;
- release-gate or follow-up implementation required.

Forbidden wording until the gates below pass:

- stable plug-in product surface;
- safe hot reload;
- trusted production extension path;
- guaranteed cross-version ABI compatibility;
- verified operator-controlled live plug-in cancellation.

## Scan Scope

The register is based on read-only scans of:

- `crates/common/src/actor/registry.rs`;
- `crates/plugin/src/**`;
- `crates/live/src/node.rs`;
- `crates/plugin/README.md`;
- `docs/rust-cutover/quality/ignored_tests_register.md`.

Representative commands:

```bash
rg -n "unsafe|UnsafeCell|extern \"C\"|libloading|ManuallyDrop|catch_unwind|sha256|dlclose|ABI|ValidatedPluginManifest" \
  crates/common/src/actor/registry.rs crates/plugin/src crates/live/src/node.rs -g '*.rs'
rg -n "plug-in|plugin|ABI|unsafe|SHA-256|sha256|dlclose|hot reload|early alpha" \
  crates/plugin/README.md docs/rust-cutover/quality/ignored_tests_register.md
```

## Status Definitions

| Status | Meaning |
| --- | --- |
| `documented_limit` | The current limitation is documented and acceptable only while the surface remains non-stable. |
| `open_gate` | Productization requires more implementation or executable evidence. |
| `manual_platform` | Evidence exists only through manual or platform-specific validation. |
| `release_blocker_if_productized` | This must block release if plug-ins are claimed as stable product functionality. |

## High Impact Register

| ID | Area | Evidence source | Current status | Owner role | Required evidence | Productization gate |
| --- | --- | --- | --- | --- | --- | --- |
| UNSAFE-HIGH-001 | Actor registry mutable aliasing through `Rc<UnsafeCell<dyn Actor>>` and `ActorRef<T>` deref/deref_mut. | `crates/common/src/actor/registry.rs:73`, `:90`, `:97`, `:235`; module docs already state aliasing is not prevented. | `release_blocker_if_productized` for plug-in actor stability. | Rust Core Runtime Agent | Either a guard/dispatch redesign that prevents overlapping mutable access, or a release-scope decision proving plug-in actor callbacks cannot create overlapping guards. | Do not call plug-in actor execution stable until aliasing behavior has regression evidence or an explicit scope decision. |
| PLUGIN-HIGH-001 | Plug-in path trust and `dlopen` side effects. | `crates/plugin/src/loader.rs:323` calls `Library::new`; comment says caller must trust the configured path. | `open_gate` | Verification & Release Gatekeeper | Config/path policy, canonicalization or allowlist rules, fixture proving rejected paths, and docs for operator-controlled plug-in directories. | Stable plug-in docs require a path trust policy before loading arbitrary configured cdylibs. |
| PLUGIN-HIGH-002 | SHA-256 verification is optional. | `crates/live/src/node.rs:1752` returns `Ok(())` when `PluginConfig.sha256` is absent. | `open_gate` | Verification & Release Gatekeeper | Decision whether SHA-256 is mandatory for product runs, plus tests for missing, matching, and mismatched hashes. | Stable product use requires mandatory hash verification or a documented local-dev exception that cannot be confused with production. |
| PLUGIN-HIGH-003 | ABI and manifest compatibility. | `crates/plugin/src/loader.rs:386`; `ValidatedPluginManifest`; `crates/plugin/README.md` says ABI/API are early alpha and `NAUTILUS_PLUGIN_ABI_VERSION` remains `1`. | `documented_limit` | Verification & Release Gatekeeper | Cross-version rejection tests, build-id compatibility matrix, and release notes tying plug-ins to the exact host version. | Stable compatibility claims require ABI versioning policy and fixture coverage across stale, mismatched, and malformed manifests. |
| PLUGIN-HIGH-004 | No unload or hot reload; library handles are intentionally leaked with `ManuallyDrop`. | `crates/plugin/src/loader.rs:218`; README says plug-ins load at process startup and live for process lifetime. | `documented_limit` | Verification & Release Gatekeeper | Explicit process-lifetime documentation plus tests proving no docs/CLI path advertises hot reload. | Do not document hot reload or unload until pointer lifetime and teardown behavior are redesigned and tested. |
| PLUGIN-HIGH-005 | Panic boundary is partly guarded, but infallible FFI thunks abort on panic. | `crates/plugin/src/panic.rs`; `crates/plugin/src/bridge/actor.rs`; `crates/plugin/src/bridge/strategy.rs`. | `open_gate` | Verification & Release Gatekeeper | Regression tests for actor, strategy, custom data, and host callback panic behavior, including which paths return `PluginErrorCode::Panic` and which abort. | Stable docs must state panic outcomes and include executable evidence for every exposed callback family. |
| PLUGIN-HIGH-006 | Live node plug-in loading happens during startup and has no separate cancellation/unload contract. | `crates/live/src/node.rs:391`; `crates/live/src/node.rs:434`; `crates/plugin/src/loader.rs:267`. | `release_blocker_if_productized` for operator-controlled live extension lifecycle. | Rust Core Runtime Agent | Startup cancellation and shutdown behavior tests for configured plug-ins, plus a documented no-unload/no-reload lifecycle. | Dashboard/control or live product claims must not imply plug-ins can be stopped, replaced, or unloaded independently. |

## Medium Impact Register

| ID | Area | Evidence source | Current status | Owner role | Required evidence | Productization gate |
| --- | --- | --- | --- | --- | --- | --- |
| PLUGIN-MED-001 | Platform cdylib smoke is manual/platform-specific. | `docs/rust-cutover/quality/ignored_tests_register.md` entry `IGN-PLUGIN-001`; `crates/plugin/tests/load_example_cdylib.rs`. | `manual_platform` | Verification & Release Gatekeeper | Release-mode Linux/macOS/Windows smoke or a scoped platform matrix. | Do not use default `verify_fast` as proof that cdylib loading works on all release platforms. |
| PLUGIN-MED-002 | FFI primitive ownership and string/slice lifetimes. | `crates/plugin/src/boundary.rs`; `BorrowedStr`, `Slice`, `OwnedBytes`, `drop_owned_bytes`. | `open_gate` | Verification & Release Gatekeeper | Boundary fixture tests for null pointer, non-zero length, invalid UTF-8, drop function ownership, and panicking drop behavior. | Stable ABI docs require ownership/lifetime evidence for every primitive crossing the boundary. |
| PLUGIN-MED-003 | Host vtable and `HostContext` pointer lifetime. | `crates/plugin/src/loader.rs:270`; `crates/plugin/src/bridge/registry.rs`; `crates/plugin/src/bridge/host.rs`. | `open_gate` | Rust Core Runtime Agent | Tests proving context creation/drop symmetry, null context rejection, and callback behavior after rejected manifest load. | Stable host callback docs require context lifetime evidence and null/stale pointer rejection tests. |
| PLUGIN-MED-004 | Receiver-local `Ustr` interning across host and cdylib. | `crates/plugin/README.md` identifier interning section. | `documented_limit` | Verification & Release Gatekeeper | Fixture coverage for command identifiers and event identifiers crossing the boundary. | Stable docs may keep receiver-local interning only if every exposed identifier path has a test or scoped exception. |
| PLUGIN-MED-005 | Custom data vtable/drop behavior. | `crates/plugin/src/bridge/custom_data.rs`; `crates/plugin/src/surfaces/custom_data.rs`. | `open_gate` | Verification & Release Gatekeeper | Custom data clone, JSON, drop, panic, and non-plug-in rejection tests. | Custom data plug-ins must not be documented as stable until drop and rejection behavior is covered. |

## Productization Preconditions

Before plug-ins can be documented as stable NTPRO product functionality:

1. Plug-in docs must keep the surface explicitly unstable until all high-impact
   rows above are either resolved or release-gate scoped out.
2. Production plug-in loading must have a path trust policy and SHA-256 policy.
3. ABI/build-id compatibility must be tested with stale and mismatched plug-ins.
4. Panic behavior must be documented per callback family.
5. Live lifecycle docs must state process-startup-only behavior unless unload or
   hot reload is implemented and tested.
6. Manual platform cdylib smoke must not be treated as covered by fast
   verification.

## Follow-Up Task Candidates

| Candidate | Risk | Scope |
| --- | --- | --- |
| `NPLUGIN-001` | medium/high | Make plug-in path trust and SHA-256 policy explicit in config/docs/tests. |
| `NPLUGIN-002` | medium | Add ABI/build-id mismatch fixture coverage and release matrix notes. |
| `NPLUGIN-003` | medium/high | Expand panic-boundary regression coverage across actor, strategy, custom data, and host callbacks. |
| `NPLUGIN-004` | high | Decide or redesign actor registry aliasing before plug-in actor stability claims. |
| `NPLUGIN-005` | high | Define live plug-in startup cancellation, shutdown, unload, and no-hot-reload lifecycle evidence. |

## Current Decision

For v0.2 readiness, plug-ins are not a stable productized extension surface.
They may remain documented as early alpha / unstable only, with this register as
the release-gate reference for future plugin hardening work.
