# Verification

Date: 2026-06-26
Executor: Codex

## Result

Local verification passed for V171/V180 changes.

## Commands

```text
cargo fmt --check -p nautilus-cli
bash -n scripts/ai/verify_v171_release_hardening.sh scripts/ai/verify_v18_cancel_recovery_gates.sh scripts/ai/verify_v18_release_gates.sh scripts/ai/verify_release.sh
ruby -e 'require "yaml"; YAML.load_file(".github/workflows/release-tag.yml"); YAML.load_file(".github/workflows/rust-cutover-smoke.yml")'
git diff --check
scripts/ai/verify_fast.sh
scripts/ai/verify_release.sh v17-release-gates
scripts/ai/verify_release.sh v171-release-hardening
scripts/ai/verify_release.sh v18-release-gates
```

`scripts/ai/verify_release.sh v17-release-gates` now includes gate-level
assertions that v0.17 source references carry `sha256`, `bytes`,
`source_command`, `source_commit`, and `source_release_tag` in addition to the
legacy `fnv1a64` hash.

`scripts/ai/verify_release.sh v18-release-gates` now also asserts the v0.18
release docs state that actual single-shot cancel remains a v0.19+ scope
decision.

## Test Coverage

```text
production_mutation_local_order_ledger_links_single_candidate_chain = PASS
production_reconciliation_orphan_artifacts_populate_readonly_dashboard_panel = PASS
production_reconciliation_orphan_missing_artifacts_degrade_panel = PASS
production_reconciliation_orphan_schema_provenance_and_stale_diagnostics_degrade_panel = PASS
production_cancel_recovery_artifacts_populate_readonly_dashboard_panel = PASS
production_cancel_recovery_missing_artifacts_degrade_panel = PASS
production_cancel_recovery_boundary_violation_degrades_panel = PASS
```

## Risk

No production network access, order submission, order mutation, actual cancel
send, automatic cancel, automatic remediation, or Dashboard cancel controls were
added. GitHub issue closure remains pending until PR review/merge and live
GitHub state audit.
