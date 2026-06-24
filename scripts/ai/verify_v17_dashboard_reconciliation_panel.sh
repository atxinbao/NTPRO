#!/usr/bin/env bash
set -euo pipefail

# V170-007: v0.17 Dashboard reconciliation/orphan panel.
# This verifier stays local/offline. It proves the Dashboard snapshot reads
# v0.17 reconciliation/orphan fixtures and the renderer remains read-only.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

cargo test -p nautilus-cli \
  production_reconciliation_orphan_artifacts_populate_readonly_dashboard_panel \
  --lib

grep -R -n -E "v0\.17 对账与孤儿单风险|production-reconciliation-orphan|renderProductionReconciliationOrphan" \
  crates/cli/src/dashboard.rs >/dev/null

echo "verify_v17_dashboard_reconciliation_panel PASS"
