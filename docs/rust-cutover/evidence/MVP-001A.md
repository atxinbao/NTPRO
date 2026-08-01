# MVP-001A - event-listener 安全公告修复证据

Date: 2026-08-02
Executor: Codex
GitHub issue: #1199
Status: REVIEW_REQUIRED

## 失败事实

- 失败 workflow：`security-audit` run #446 / run `30630021962`；
- 失败 job：`osv-scanner`；
- 公告：`RUSTSEC-2026-0221`；
- 受影响依赖：`event-listener 5.4.1`；
- OSV 指定修复版本：`5.4.2`；
- 同一 run 的 `cargo-audit`、`cargo-deny`、`cargo-vet` 与 `zizmor` 均通过。

## 依赖范围

`event-listener` 是传递依赖，当前经 `async-lock/redis` 和 `sqlx-core` 进入
workspace。任务不修改 Rust 源码、公共 API 或交易能力边界。

## 修复与验证

- `Cargo.lock` 已将 `event-listener 5.4.1` 精确升级到 `5.4.2`；
- 新版本移除了 `event-listener` 对 `concurrent-queue` 的直接依赖；
- `.supply-chain/config.toml` 已将对应 exemption 更新到 `5.4.2`，并明确记录
  这是等待独立源码审计的安全补丁豁免，不冒充人工审计；
- 未修改 Rust 源码、公共 API、运行时行为或交易能力边界。

```text
cargo audit
PASS: 0 vulnerabilities; 2 repository-allowed warnings

cargo deny --all-features check advisories licenses sources bans
PASS: advisories ok, bans ok, licenses ok, sources ok

cargo vet --locked
PASS: 169 fully audited, 29 partially audited, 690 exempted

osv-scanner v2.3.5 --config=osv-scanner.toml --lockfile=Cargo.lock
PASS: scanned 895 packages; no unfiltered vulnerability reported

cargo test -p nautilus-cli mvp --lib -j 2 --locked
PASS: 6 passed, 0 failed

cargo clippy -p nautilus-cli --all-targets --all-features --locked -- -D warnings
PASS

scripts/ai/check_rust_only_runtime.sh
PASS

scripts/ai/check_backend_freeze_baseline.sh
PASS: baseline plus 20 negative cases

scripts/ai/verify_fast.sh
PASS

git diff --check
PASS
```

OSV 等价本地扫描使用官方 release `v2.3.5` 的 macOS arm64 二进制，并先按
官方 `osv-scanner_SHA256SUMS` 校验。Docker Desktop 当时未运行，因此没有使用
action 容器；扫描器版本、配置和锁文件参数与 hosted job 一致。

所有 Cargo 验证均通过 `scripts/ai/toolchain_env.sh` 使用项目固定的 Rust 1.95.0。
冻结基线脚本首次直接调用继承系统 Rust 1.87，因工具链版本不满足项目要求退出；
使用项目固定工具链重跑后通过，该环境错误不属于代码失败。

## 审查要求

本任务风险为 medium。PR 由 Verification & Release Gatekeeper 审查，合并后再
确认 `main` 的 security-audit 恢复绿色并关闭 issue #1199。
