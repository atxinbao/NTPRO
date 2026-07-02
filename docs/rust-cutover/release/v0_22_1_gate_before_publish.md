# v0.22.1 Gate-Before-Publish Release Governance

Date: 2026-07-02
Executor: Codex
Task: `V221-004` / GitHub issue `#708`

## Summary

v0.22.1 hardens the Rust-only release publication order. Public GitHub Release
publication is now a separate scripted step that requires successful hosted
release gate evidence for the same tag commit.

Plain Chinese summary: v0.22.1 不再接受“公开 GitHub Release 后再等 tag-triggered
gate 成功”的顺序。可以先准备 draft release；但公开发布必须通过
`scripts/ai/publish_ntpro_release_after_gate.sh` 或
`Rust Cutover Publish Release` workflow，并输入已经成功完成的
`Rust Cutover Release Gate` run id。

## Operational Sequence

1. Merge all v0.22.1 PRs and synchronize `main`.
2. Prepare release notes and any GitHub Release page as draft only.
3. Create and push the immutable `ntpro-rust-only-v0.22.1` tag.
4. Wait for the tag-triggered `Rust Cutover Release Gate` run to complete with
   `conclusion=success`.
5. Publish the public GitHub Release through one of the controlled entrypoints:
   - GitHub Actions: run `Rust Cutover Publish Release` with `tag_name`,
     `release_version`, `release_gate_run_id`, `release_name`, and
     `release_notes_path`.
   - Local operator path: run `scripts/ai/publish_ntpro_release_after_gate.sh`
     with the same environment values and authenticated `gh`.
6. Record the generated publication evidence JSON and the hosted publish
   workflow URL in the v0.22.1 release evidence.

## Enforced Boundary

The publish entrypoint verifies:

- the release gate run exists in the target repository;
- the gate run `status` is `completed`;
- the gate run `conclusion` is `success`;
- the gate run `workflowName` is `Rust Cutover Release Gate`;
- the gate run `headSha` matches the release tag commit;
- the GitHub Release body matches the release notes file;
- if the release is already public, its `publishedAt` timestamp is not earlier
  than the gate run completion timestamp.

If the release does not exist, the script creates the public release only after
the gate checks pass. If the release exists as draft, the script publishes that
draft only after the same checks pass.

## Rollback Plan

If a release is accidentally made public before the hosted gate succeeds, do
not move or rewrite the release tag. Capture the release URL and timestamp,
delete the premature public release object, keep the tag immutable, wait for the
hosted gate to succeed, then recreate or publish the release through the
controlled entrypoint. If the tag commit itself is wrong, stop the v0.22.1
publication and create a superseding patch tag only after a separate release
decision.

## v0.23.0 Dependency Boundary

The `v0.23.0` GitHub issues are already published as `#711-#718`, but the
milestone remains hard-blocked by v0.22.1. No v0.23.0 implementation starts
until V221 issues are closed and the v0.22.1 release evidence is published
through this gate-before-publish sequence.
