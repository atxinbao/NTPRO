# NDASH-001 - Dashboard MVP scope contract

Milestone: v0.2.0 Dashboard Boundary
Priority: P2
Default role: Control & Scope
Risk: low

## Goal

Lock the first Dashboard MVP scope before implementation starts.

## Scope

The first version may include:

- status viewing;
- alert viewing;
- node lifecycle viewing;
- start, stop, pause, and resume controls.

The first version must not include:

- manual order entry;
- strategy parameter hot reload;
- multi-user permissions;
- complex asset management;
- full trading frontend scope;
- Docker delivery as a requirement.

## Likely files

- `docs/architecture/`
- `docs/rust-cutover/evidence/NDASH-001.md`

## Non-goals

- Do not implement the dashboard.
- Do not add runtime control endpoints.
- Do not add order-entry workflows.

## Dependencies

- `NARCH-005`

## Acceptance criteria

- Dashboard MVP scope contract exists.
- Included and excluded controls are explicit.
- Evidence records that this is contract-only.

## Required commands

```bash
git diff --check
scripts/ai/verify_fast.sh
```

## Evidence required

Create `docs/rust-cutover/evidence/NDASH-001.md`.
