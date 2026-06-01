# Cap'n Proto Feature Removed

Date: 2026-06-02
Executor: Codex
Task ID: RREM-015

## What changed

NTPRO removed the optional Cap'n Proto serialization feature from the Rust
workspace.

Removed user-facing commands and feature flags:

- `cargo build -p nautilus-serialization --features capnp`
- `cargo test -p nautilus-serialization --features capnp`
- `make check-capnp-schemas`
- `make regen-capnp`
- `scripts/install-capnp.sh`
- `scripts/regen-capnp.sh`

Removed repository surfaces:

- Cap'n Proto schemas under `crates/serialization/schemas/capnp/**`
- generated Rust bindings under `crates/serialization/generated/capnp/**`
- conversion code under `crates/serialization/src/capnp/**`
- common crate Cap'n Proto forwarding modules
- Cap'n Proto tests, benchmarks, CI install action, and cache plumbing

## What did not change

The existing Rust serialization paths remain:

- Arrow and display Arrow support;
- Parquet/catalog persistence;
- JSON and MsgPack helpers from `nautilus-core`;
- SBE decode utilities behind the `sbe` feature.

No trading semantics, adapter behavior, order lifecycle behavior, or persistence
format migration is introduced by this task.

`capnp` may still appear in `Cargo.lock` as a transitive dependency of
`hypersync-client` through the optional blockchain adapter path. That dependency
is separate from the removed first-party `nautilus-serialization` Cap'n Proto
feature and is not a public NTPRO serialization surface.

## Replacement

Use the supported Rust serialization paths instead:

```bash
cargo check -p nautilus-serialization --features arrow
cargo check -p nautilus-serialization --features sbe
cargo bench -p nautilus-serialization --bench serialization_comparison
```

For catalog storage and analytics workflows, use the existing Arrow and Parquet
catalog paths rather than Cap'n Proto wire-format schemas.

## Release note

This is a breaking removal for any downstream consumer that enabled the
`capnp` feature or imported `nautilus_serialization::capnp`. Those users must
migrate to Arrow, JSON, MsgPack, SBE, or a project-local serializer outside
NTPRO.
