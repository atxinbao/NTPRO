# v0.10.0 Offline and Manual Release Gates

Date: 2026-06-19
Executor: Codex
Task: V100-010

## Plain Chinese Summary

这份文档说明 v0.10.0 的发布门禁怎么跑。默认门禁只做离线检查：
确认缺少人工批准时会 fail-closed，确认 JSON schema、签名/凭据脱敏、
reconciliation fixture 和 Dashboard 只读展示都没问题。真实 Binance testnet
小额提交再撤单证明被单独留在人工门禁里，不会被 CI 或 tag release 自动执行。

## Default Offline Gate

The default offline gate is:

```bash
scripts/ai/verify_v10_offline_release_gates.sh
```

It runs:

- `scripts/ai/verify_v10_offline_fail_closed.sh`;
- `scripts/ai/verify_v10_order_preflight.sh`;
- `scripts/ai/verify_v10_signed_order_request.sh`;
- `scripts/ai/verify_v10_order_test_preflight.sh`;
- `scripts/ai/verify_v10_execution_artifact_contract.sh`;
- `scripts/ai/verify_v10_reconciliation_fixture.sh`;
- `cargo test -p nautilus-cli testnet_order_proof_artifacts_populate_dashboard_read_only_fields --lib`.

This gate must keep:

```text
network_attempted=false
real_orders_submitted=false
production_orders_submitted=0
dashboard_order_controls=false
```

## Manual Order-Proof Gate

The separate manual preflight gate is:

```bash
scripts/ai/verify_v10_manual_order_proof_gate.sh
```

Default mode only proves the manual online order gate is closed:

```text
manual_online=false
network_attempted=false
real_orders_submitted=false
```

If an owner-approved V100-006 run later creates a real Binance testnet artifact
package, the same script can validate that package with:

```bash
NTPRO_V10_MANUAL_ONLINE=1 \
NTPRO_ALLOW_BINANCE_TESTNET_ORDER=1 \
NTPRO_OWNER_APPROVED_BINANCE_TESTNET_ORDER=1 \
NTPRO_CONFIRM_TESTNET_TINY_NOTIONAL=1 \
NTPRO_CONFIRM_TESTNET_CANCEL_AFTER_SUBMIT=1 \
NTPRO_V10_MANUAL_ORDER_PROOF_DIR=... \
scripts/ai/verify_v10_manual_order_proof_gate.sh
```

The validator checks the expected artifact package shape and boundary counters.
It does not submit, cancel, retry, or reconcile orders by itself.

## Release Wiring

`scripts/ai/verify_release.sh` exposes:

```text
v10-offline-release-gates
v10-manual-order-proof-preflight
```

The tag release workflow includes both stages:

```text
release-v10-offline-release-gates
release-v10-manual-order-proof-preflight
```

PR smoke runs the v0.10 gates when v0.10 scripts, v0.10 release docs, the
relevant live/Dashboard CLI files, or the v0.10 strategy config change.

## Boundary

These gates do not:

- complete V100-006;
- claim real Binance testnet submit/cancel proof;
- connect to production Binance;
- handle real funds;
- add Dashboard order controls;
- create a release tag;
- publish a GitHub Release.

## Rollback

Revert the V100-010 PR. This removes the v0.10 gate scripts, release gate
stages, PR smoke classifier updates, task document, release document, and
evidence file.
