# PTC-003 Rust Read-Model Schema Migration

Date: 2026-07-15
Executor: Codex

## Replacement

The supported v0.21 read-model schema command is:

```bash
scripts/ai/ntpro_governance.sh read-model-schema \
  --schema docs/rust-cutover/release/v0_21_0_unified_read_model_schema.json \
  --trace-glob 'tests/golden/**/*.jsonl'
```

The implementation uses `jsonschema` 0.47.0 with explicit Draft 2020-12
validation and no HTTP or file resolver features. Official library reference:
https://docs.rs/jsonschema/0.47.0/jsonschema/draft202012/index.html

## Scope Correction

The retired Python validator treated every later `category=read_model` row as
a v0.21 unified snapshot. Its default recursive glob now fails on v0.25+
event-only rows and would also apply the v0.21 schema to v0.23+ snapshot
contracts.

The Rust validator derives the target contract from
`properties.contract_version.const` in the schema. It validates all 36 matching
v0.21 snapshots, skips snapshots owned by other contract versions, and skips
read-model events that do not claim a snapshot. A present non-object snapshot
still fails closed.

## Retired File

- `scripts/ai/validate_v21_read_model_schema.py`.

The v0.21.1 shell gate no longer probes Python, imports `jsonschema`, or falls
back to `uv`. Direct dependency declarations remain untouched until PTC-007
removes the repository Python environment as one reviewed dependency change.

This migration changes repository validation tooling only. It does not change
runtime, trading semantics, public API, backend capability, or the v0.32.0
baseline.
