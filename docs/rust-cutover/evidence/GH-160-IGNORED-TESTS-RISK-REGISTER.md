# GH-160 - Ignored tests risk register evidence

Date: 2026-06-05
Executor: Codex

## Task

- GitHub issue: <https://github.com/atxinbao/NTPRO/issues/160>
- Branch: `codex/audit-ignored-tests-register`
- Owner role: Verification & Release Gatekeeper
- Review role: Control & Scope Agent
- Risk level: Medium

## Plain Chinese summary

这次没有删除任何 ignored tests，也没有改交易语义。只是把当前仓库里真正的
`#[ignore]` Rust 测试集中登记成风险台账。

台账按 High / Medium / Low 分类，重点标出了 common cache、risk engine、
execution matching、live stress、persistence/catalog、Postgres cache 和
adapter reconnect/API-key 相关项。高影响项都写了 owner role、影响范围、
当前状态和下一步建议。

## Goal

Create an ignored tests risk register so ignored tests are no longer invisible
release or product-path risk.

## Files changed

- `docs/rust-cutover/verification/ignored_tests_risk_register.md`
- `docs/rust-cutover/verification/README.md`
- `docs/rust-cutover/evidence/GH-160-IGNORED-TESTS-RISK-REGISTER.md`

## Scan results

Commands:

```bash
rg -n '^\s*#\[ignore(?:\s*=\s*"[^"]*")?\]' crates tests --glob '*.rs' -S
rg -n '^\s*#\[ignore(?:\s*=\s*"[^"]*")?\]' crates tests --glob '*.rs' -S | wc -l
rg -n '^\s*#\[ignore(?:\s*=\s*"[^"]*")?\]' crates tests --glob '*.rs' -S | cut -d: -f1 | sort | uniq -c
rg -n '^\s*#\[ignore(?:\s*=\s*"[^"]*")?\]' crates tests --glob '*.rs' -S | sed 's/:.*//' | xargs -n1 dirname | sort | uniq -c
```

Result summary:

- 30 active ignored Rust test attributes were found.
- High impact product-path groups:
  - `crates/common/src/cache/tests.rs`
  - `crates/execution/tests/matching_engine.rs`
  - `crates/risk/tests/risk_engine.rs`
  - `crates/adapters/dydx/tests/websocket.rs`
- Medium impact groups:
  - `crates/live/tests/stress.rs`
  - `crates/persistence/tests/test_catalog.rs`
  - `crates/infrastructure/tests/test_cache_postgres.rs`
  - selected adapter/plugin slow or platform-specific tests
- Low/manual groups:
  - live API-key tests
  - one-time dataset curation tests
  - user-fetched fixture tests

## Behavior impact

No runtime behavior changed. No tests were deleted, enabled, disabled, or
rewritten.

## Public API impact

No public API change.

## Migration note status

No migration note required. This is a QA/verification risk register.

## Validation

Required validation for this docs-only register:

```bash
git diff --check
scripts/ai/verify_fast.sh
rg -n '^\s*#\[ignore(?:\s*=\s*"[^"]*")?\]' crates tests --glob '*.rs' -S | wc -l
rg -n '^\s*#\[ignore(?:\s*=\s*"[^"]*")?\]' crates tests --glob '*.rs' -S | cut -d: -f1 | sort | uniq -c
```

Results:

- `git diff --check` passed.
- `scripts/ai/verify_fast.sh` passed.
- The precise ignored-test scan still reports 30 active ignored Rust test
  attributes.
- Per-file count remains:
  - `crates/adapters/betfair/src/loader.rs`: 2
  - `crates/adapters/bitmex/tests/http.rs`: 1
  - `crates/adapters/blockchain/src/data/client.rs`: 1
  - `crates/adapters/blockchain/src/data/core.rs`: 1
  - `crates/adapters/bybit/tests/http.rs`: 4
  - `crates/adapters/dydx/tests/websocket.rs`: 1
  - `crates/adapters/hyperliquid/tests/exec_client.rs`: 1
  - `crates/adapters/tardis/src/csv/load.rs`: 1
  - `crates/common/src/cache/tests.rs`: 2
  - `crates/execution/tests/matching_engine.rs`: 3
  - `crates/infrastructure/tests/test_cache_postgres.rs`: 2
  - `crates/live/tests/stress.rs`: 2
  - `crates/persistence/tests/test_catalog.rs`: 1
  - `crates/plugin/tests/load_example_cdylib.rs`: 1
  - `crates/risk/tests/risk_engine.rs`: 6
  - `crates/testkit/src/itch/parse.rs`: 1

## Remaining risks

- The register does not fix ignored tests.
- High impact ignored tests remain open until repaired or formally scoped.
- Some adapter/API-key tests need fixtures or mocks before they can become
  release evidence.

## Rollback plan

Revert the PR. No source behavior, runtime state, or test execution behavior is
affected.
