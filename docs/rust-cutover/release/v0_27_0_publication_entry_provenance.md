# NTPRO v0.27.0 Publication Entry Provenance

Date: 2026-07-08
Executor: Codex
Task: `V271-002` / GitHub issue `#888`
Milestone: `v0.27.1`
Status: PUBLICATION ENTRY PROVENANCE RECORDED

## Summary

This document records the actual publication entry for the public
`ntpro-rust-only-v0.27.0` GitHub Release. It is release-governance evidence
only: it does not change runtime behavior, adapter behavior, Dashboard/Admin
behavior, public API behavior, or trading semantics.

Plain Chinese summary: `v0.27.0` 的公开 Release 不是由 hosted
`release-publish.yml` workflow 发布，而是由本地
`scripts/ai/publish_ntpro_release_after_gate.sh` 在 hosted release gate 成功后发布。
该本地入口使用已认证的 `gh` 用户 `atxinbao`，强制校验 gate run、tag commit、release
notes body 和发布时间顺序。这个路径是有边界的发布治理入口，不是 runtime 能力扩展。

## Publication Entry

```text
release tag = ntpro-rust-only-v0.27.0
release name = NTPRO Rust-only v0.27.0
release URL = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.27.0
publication path = local_publish_script_after_hosted_gate
publication entrypoint = scripts/ai/publish_ntpro_release_after_gate.sh
publication entrypoint mode = local Codex shell with authenticated gh
publication actor source = GitHub Release API author
GitHub Release API author login = atxinbao
GitHub Release API author type = User
GitHub Release API author id = 254527493
GitHub Release API release id = 350764398
GitHub Release API node id = RE_kwDOSox1D84U6D1u
hosted release-publish workflow used for v0.27.0 = false
hosted release-publish workflow required for this already-published v0.27.0 = false
bounded non-workflow publication path = true
owner-approved authenticated publication path = true
```

## Command Evidence

```text
NTPRO_RELEASE_GATE_RUN_ID=28921344889 \
NTPRO_RELEASE_VERSION=v0.27.0 \
NTPRO_RELEASE_TAG=ntpro-rust-only-v0.27.0 \
NTPRO_RELEASE_NAME="NTPRO Rust-only v0.27.0" \
NTPRO_RELEASE_NOTES=docs/rust-cutover/release/v0_27_0_release_notes.md \
NTPRO_RELEASE_PUBLICATION_DRY_RUN=0 \
scripts/ai/publish_ntpro_release_after_gate.sh
```

Expected and observed script proof:

```text
release_gate_run_id = 28921344889
release_gate_url = https://github.com/atxinbao/NTPRO/actions/runs/28921344889
release_gate_completed_at = 2026-07-08T07:29:57Z
release_tag = ntpro-rust-only-v0.27.0
release_tag_sha = 67db6d2c4d2f5b922f9e52e7d20588cb41f972f3
release_publication_after_gate = pass
publication status = published_after_gate
release_url = https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.27.0
published_at = 2026-07-08T07:31:08Z
updated_at = 2026-07-08T07:31:08Z
evidence_path = release-publication-evidence/ntpro-rust-only-v0.27.0.json
publication_evidence_strategy = source_tree_plus_github_remote
local_evidence_path_is_generated_artifact = true
local_evidence_path_required_in_source_tree = false
remote_reconstruction_required = true
```

## Hosted Workflow Check

```text
hosted publish workflow file = .github/workflows/release-publish.yml
hosted publish workflow name = Rust Cutover Publish Release
hosted publish workflow entrypoint = workflow_dispatch
hosted publish workflow command = scripts/ai/publish_ntpro_release_after_gate.sh
v0.27.0 matching hosted publish run = none
publication window checked = 2026-07-08T07:20:00Z..2026-07-08T07:40:00Z
publication window release-publish runs = []
latest observed release-publish.yml run before v0.27.0 publication = 28902924185
latest observed release-publish.yml run head SHA = bc90355158a7897c7ca78ed31e638d6cf8120da1
latest observed release-publish.yml run created_at = 2026-07-07T22:25:20Z
latest observed release-publish.yml run conclusion = success
latest observed release-publish.yml run is v0.27.0 publication = false
```

Interpretation: v0.27.0 publication did not use the hosted
`release-publish.yml` workflow. The public Release was published by the same
guarded script from a local authenticated shell after hosted release gate
success.

## Gate-Before-Publication Ordering

```text
hosted release gate run = 28921344889
hosted release gate workflow = Rust Cutover Release Gate
hosted release gate URL = https://github.com/atxinbao/NTPRO/actions/runs/28921344889
hosted release gate status = completed
hosted release gate conclusion = success
hosted release gate head SHA = 67db6d2c4d2f5b922f9e52e7d20588cb41f972f3
hosted release gate jobs = 82/82 success
hosted release gate created_at = 2026-07-08T06:04:46Z
hosted release gate completed_at = 2026-07-08T07:29:57Z
GitHub Release API created_at = 2026-07-08T06:03:41Z
GitHub Release API published_at = 2026-07-08T07:31:08Z
GitHub Release API updated_at = 2026-07-08T07:31:08Z
public publication after hosted gate success = true
created_at is public publication proof = false
published_at is public publication proof = true
```

## Release Body and Source Match

```text
release notes source = docs/rust-cutover/release/v0_27_0_release_notes.md
release body sha256 = 91184074bab30a50f69147697aecf19d91977d615ad313eef96fbcb2c470138b
tracked release notes sha256 = 91184074bab30a50f69147697aecf19d91977d615ad313eef96fbcb2c470138b
release body matches tracked release notes = true
release body normalized line count = 88
strict release body match required = true
```

## Bounded Non-Workflow Policy

```text
non_workflow_publication_entry_allowed_for_v0_27_0_closeout = true
reason = already-published release was created by guarded local script after hosted gate success
hosted workflow publication available = true
hosted workflow publication retroactive rerun required = false
future v0.27.1 publication entry must be explicit = true
future v0.27.1 gates may require hosted workflow or tracked owner-approved non-workflow evidence = true
generated publication evidence authoritative = false
source_tree_plus_github_remote remains authoritative = true
```

## Boundary Statement

```text
new_submit_capability = false
production_order_submission_allowed = false
production_order_mutation_allowed = false
execution_adapter_call_allowed = false
adapter_send_allowed = false
live_exchange_request_allowed = false
retry_scheduler_enabled = false
automatic_remediation_allowed = false
automatic_operation_action_allowed = false
dashboard_operation_controls_enabled = false
dashboard_trading_controls_enabled = false
admin_workbench_operation_controls_enabled = false
admin_workbench_trading_controls_enabled = false
trader_terminal_order_ticket_enabled = false
manual_operation_submit_allowed = false
product_grade_trading_terminal_claim = false
```

## Reconstruction Commands

```text
gh api repos/atxinbao/NTPRO/releases/tags/ntpro-rust-only-v0.27.0 --jq '{id,node_id,tag_name,name,draft,prerelease,created_at,published_at,updated_at,target_commitish,html_url,author:{login:.author.login,type:.author.type,id:.author.id}}'
gh run view 28921344889 --repo atxinbao/NTPRO --json status,conclusion,url,headSha,createdAt,updatedAt,workflowName,jobs
gh run list --repo atxinbao/NTPRO --workflow release-publish.yml --limit 20 --json databaseId,status,conclusion,workflowName,displayTitle,event,headBranch,headSha,createdAt,updatedAt,url
gh run list --repo atxinbao/NTPRO --limit 30 --json databaseId,status,conclusion,workflowName,displayTitle,event,headBranch,headSha,createdAt,updatedAt,url --jq '[.[] | select(.createdAt >= "2026-07-08T07:20:00Z" and .createdAt <= "2026-07-08T07:40:00Z")]'
python3 -m json.tool release-publication-evidence/ntpro-rust-only-v0.27.0.json
NTPRO_CURRENT_RELEASE_VERSION=v0.27.0 NTPRO_CURRENT_RELEASE_TAG=ntpro-rust-only-v0.27.0 NTPRO_CURRENT_RELEASE_NAME="NTPRO Rust-only v0.27.0" NTPRO_RELEASE_PUBLICATION_STRICT_BODY=1 scripts/ai/check_github_release_published.sh
```

## Evidence Sources

```text
GitHub issue #888 body
GitHub Release API for ntpro-rust-only-v0.27.0
GitHub Actions run 28921344889
GitHub Actions release-publish.yml run list
scripts/ai/publish_ntpro_release_after_gate.sh
.github/workflows/release-publish.yml
docs/rust-cutover/release/v0_27_0_release_notes.md
docs/rust-cutover/release/v0_27_0_release_closeout_evidence.md
release-publication-evidence/ntpro-rust-only-v0.27.0.json
```

## Next Step

After this evidence is merged through issue `#888`, proceed to `#889`
`V271-003 stale V270 evidence cleanup` on its own branch and PR. No `v0.28.0`
implementation starts until all V271 issues are closed and `v0.27.1` release
evidence is published.
