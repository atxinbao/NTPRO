# MVP-001A - 修复 event-listener 安全公告并恢复 main security audit

Date: 2026-08-02
Executor: Codex
GitHub issue: #1199
Risk: medium
Owner role: Rust Product Surface Agent
Review role: Verification & Release Gatekeeper
Status: REVIEW_REQUIRED

## 目标

修复 PR #1198 合并后 `main` 的 security-audit run #446：将受
`RUSTSEC-2026-0221` 影响的 `event-listener 5.4.1` 升级到 `5.4.2`，并同步
供应链审计配置。

## 范围

- 更新 `Cargo.lock` 中的 `event-listener` 补丁版本；
- 更新 `.supply-chain/config.toml` 中对应的 cargo-vet exemption；
- 记录本地和 hosted 安全检查证据。

## 非目标

- 修改 Rust runtime 或产品行为；
- 修改公共 API 或交易语义；
- 修改 `docs/rust-cutover/release/v0_32_0_*`；
- 新增 submit、mutation、adapter send、live exchange、retry、remediation 或交易控件能力。

## 验收

```bash
cargo audit
cargo deny --all-features check advisories licenses sources bans
cargo vet --locked
cargo test -p nautilus-cli mvp --lib -j 2 --locked
cargo clippy -p nautilus-cli --all-targets --all-features --locked -- -D warnings
scripts/ai/check_rust_only_runtime.sh
scripts/ai/check_backend_freeze_baseline.sh
git diff --check
```

完成实现与本地验证后进入 `REVIEW_REQUIRED`，由 Verification & Release
Gatekeeper 审查后合并。
