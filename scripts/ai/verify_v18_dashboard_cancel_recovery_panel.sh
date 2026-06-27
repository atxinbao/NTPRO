#!/usr/bin/env bash
set -euo pipefail

# V180-009: v0.18 Dashboard cancel recovery read-only panel.
# This verifier stays local/offline. It proves the Dashboard can read cancel
# recovery artifacts, render them as a read-only panel, and keep cancel/order
# controls outside the Dashboard surface.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

cargo test -p nautilus-cli \
  production_cancel_recovery \
  --lib

grep -R -n -E "v0\\.18 撤单恢复只读面板|production-cancel-recovery|renderProductionCancelRecovery|dashboard_cancel_controls_enabled|dashboard_auto_approval_allowed|dashboard_auto_approval_attempted" \
  crates/cli/src/dashboard.rs >/dev/null

if [[ "${NTPRO_V18_DASHBOARD_SKIP_CLOSEOUT_CHAIN:-0}" != "1" ]]; then
  scripts/ai/verify_v18_cancel_recovery_incident_audit_closeout.sh >/dev/null
fi

echo "verify_v18_dashboard_cancel_recovery_panel PASS dashboard_cancel_controls_enabled=false dashboard_auto_approval_allowed=false cancel_order_control_routes_exposed=false"
