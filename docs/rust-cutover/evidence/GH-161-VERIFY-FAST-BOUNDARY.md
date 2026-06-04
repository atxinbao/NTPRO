# GH-161 - verify_fast boundary evidence

Date: 2026-06-05
Executor: Codex

## Task

- GitHub issue: <https://github.com/atxinbao/NTPRO/issues/161>
- Branch: `codex/audit-verify-fast-boundary`
- Owner role: Verification & Release Gatekeeper
- Review role: Control & Scope Agent
- Risk level: Low

## Plain Chinese summary

这次没有把 `verify_fast.sh` 变重，也没有改变 CI。只是把脚本输出和公开文档
说清楚：`verify_fast.sh` 默认只是 fast smoke，主要检查 Rust 工具链和
`cargo fmt --check`。

它默认不跑 workspace `cargo check`，不跑 clippy，不跑 golden trace，也不是
release gate。发布前仍然要用 `verify_release.sh` 这类更强验证。

## Goal

Prevent developers and release work from mistaking `scripts/ai/verify_fast.sh`
for complete workspace or release verification.

## Files changed

- `scripts/ai/verify_fast.sh`
- `README.md`
- `docs/getting_started/installation.md`
- `docs/developer_guide/releases.md`
- `docs/rust-cutover/verification/README.md`
- `docs/rust-cutover/evidence/GH-161-VERIFY-FAST-BOUNDARY.md`

## Behavior impact

`verify_fast.sh` still runs the same default checks:

- pinned Rust toolchain output;
- `cargo fmt --check`;
- optional workspace cargo check when `VERIFY_FAST_CARGO_CHECK=1`;
- optional clippy when `VERIFY_FAST_CLIPPY=1`.

The behavior change is output wording only. The script now explicitly says it
is a fast smoke and not a full workspace check, clippy gate, release gate, or
golden trace gate.

## Public API impact

No Rust public API change. No CLI product command change.

## Migration note status

No migration note required. This is verification documentation and script
wording cleanup.

## Validation

### Fast verification

Command:

```bash
scripts/ai/verify_fast.sh
```

Result summary:

```text
== verify_fast: scope ==
fast smoke only: toolchain + cargo fmt by default
not a full workspace check, clippy gate, release gate, or golden trace gate
for release evidence use scripts/ai/verify_release.sh
== verify_fast: rust fmt ==
== verify_fast: cargo check skipped by fast-smoke default; set VERIFY_FAST_CARGO_CHECK=1 for workspace cargo check ==
== verify_fast: clippy skipped by fast-smoke default; set VERIFY_FAST_CLIPPY=1 to run clippy in fast mode ==
== verify_fast complete: fast smoke only; release work still requires stronger verification ==
```

The command passed.

## Remaining risks

- This does not make `verify_fast.sh` stronger.
- Release work still depends on running stronger gates such as
  `scripts/ai/verify_release.sh`.

## Rollback plan

Revert the PR. The script returns to the previous shorter wording. No runtime,
release artifact, or source behavior is affected.
