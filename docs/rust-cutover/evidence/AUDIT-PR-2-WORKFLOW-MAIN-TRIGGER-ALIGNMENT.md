# AUDIT-PR-2 - Workflow Main Trigger Alignment Evidence

Date: 2026-06-06
Executor: Codex
Branch: `codex/workflow-main-trigger-alignment`

## Task

审计修复项 PR-2：把仍指向旧默认分支 `master` 的 CI/workflow
触发口径对齐到 NTPRO 当前默认分支 `main`。

## Goal

- 让主干检查类 workflow 能响应 `main`。
- 不把旧 Python wheel / PyPI / GitHub release 发布路径重新接到
  `main`。
- 不让 NTPRO `main` 自动向上游 `nautilus_docs` 派发文档发布事件。
- 不让 NTPRO `main` push 隐式发布 Docker/GHCR 镜像。

## Files Changed

- `.github/workflows/build.yml`
- `.github/workflows/build-v2.yml`
- `.github/workflows/security-audit.yml`
- `.github/workflows/build-docs.yml`
- `.github/workflows/docker.yml`

## Change Summary

已清理：

- `build.yml` push trigger 从 `master` 改为 `main`。
- `build.yml` 的 `cargo-deny` 和 `cargo-vet` gate 从 `master` 改为
  `main`。
- `build-v2.yml` 增加 `main` push trigger。
- `security-audit.yml` push trigger 从 `master` 改为 `main`。
- `build-docs.yml` push trigger 从 `master` 改为 `main`。
- `docker.yml` push trigger 从 `master` 改为 `main`。

有意保留/限制：

- `build.yml` 中旧 wheel、PyPI、sdist、GitHub release 发布 job 仍保留
  `refs/heads/master` 条件，目的是避免把上游旧发布路径恢复到
  NTPRO `main`。这些路径不是当前 Rust-only v0.2 发布入口。
- `build.yml` 的旧 wheel-oriented build matrix 明确排除 `main`，避免
  `main` push 重新触发混合 Python/wheel 时代的大型构建。
- `build-docs.yml` 增加 `github.repository ==
  'nautechsystems/nautilus_trader'` 限制。NTPRO `main` 不会向上游
  `nautilus_docs` 发 dispatch。
- `docker.yml` 的生产镜像发布 job 只允许 `nightly`，`main` 不会自动推
  GHCR 镜像。`test-docker` 测试构建路径保持不变。

## Commands Run

```bash
ruby -e 'require "psych"; ARGV.each { |f| Psych.load_file(f); puts "OK #{f}" }' \
  .github/workflows/build.yml \
  .github/workflows/build-docs.yml \
  .github/workflows/docker.yml \
  .github/workflows/security-audit.yml \
  .github/workflows/build-v2.yml
```

Result: passed. All touched workflow YAML files parsed successfully.

```bash
scripts/ai/verify_fast.sh
```

Result: passed. Toolchain smoke and `cargo fmt --check` passed. The script
correctly reported that workspace cargo check and clippy are not part of its
default fast-smoke mode.

```bash
git diff --check
```

Result: passed.

```bash
rg -n "branches: \[main|branches:|refs/heads/main|refs/heads/master|github.repository == 'nautechsystems/nautilus_trader'|github.ref == 'refs/heads/nightly'" \
  .github/workflows/build.yml \
  .github/workflows/build-docs.yml \
  .github/workflows/docker.yml \
  .github/workflows/security-audit.yml \
  .github/workflows/build-v2.yml
```

Result: passed for classification. The remaining `refs/heads/master` matches
are old release/publish jobs intentionally left disconnected from `main`.

## Behavior Impact

No Rust runtime behavior changed.

GitHub workflow behavior changes:

- Main branch pushes now match the check-oriented workflow triggers.
- `main` can run supply-chain gates where intended.
- Old wheel/PyPI/GitHub release publishing remains disabled for `main`.
- Docker image publishing remains non-main until a dedicated Rust-only Docker
  release workflow is designed.
- Upstream docs dispatch remains disabled in NTPRO.

## Public API Impact

None. This is CI/workflow configuration only.

## Migration Note

No user migration note is required. Release operators should continue using the
manual Rust-only tag/release process until a dedicated v0.2 release workflow is
approved.

## Rollback Plan

Revert this PR to restore the previous workflow trigger state. If reverting,
also re-check that NTPRO `main` still has an active PR smoke gate, because
`main` would no longer be covered by these aligned push triggers.
