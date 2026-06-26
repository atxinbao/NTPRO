#!/usr/bin/env bash
set -euo pipefail

# V171-007：v0.17 Dashboard reconciliation/orphan panel diagnostics。
# 该 verifier 保持本地离线，只证明 Dashboard snapshot 能读取 v0.17 对账/孤儿单
# artifacts，并把缺失、schema、provenance、stale 诊断显示为只读信息。

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

cargo test -p nautilus-cli \
  production_reconciliation_orphan \
  --lib

grep -R -n -E "v0\.17 对账与孤儿单风险|production-reconciliation-orphan|renderProductionReconciliationOrphan" \
  crates/cli/src/dashboard.rs >/dev/null

echo "verify_v17_dashboard_reconciliation_panel PASS"
