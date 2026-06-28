#!/usr/bin/env bash
set -euo pipefail

# V190-008: v0.19 Dashboard actual-cancel audit read-only view.
# This verifier stays local/offline. It proves the Dashboard can read actual
# cancel audit artifacts, distinguish outcome states, diagnose evidence
# problems, and keep cancel/approval/retry/bulk controls outside the Dashboard
# surface.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

cargo test -p nautilus-cli \
  production_actual_cancel_audit \
  --lib

grep -R -n -E "v0\\.19 真实撤单审计只读视图|production-actual-cancel-audit|renderProductionActualCancelAudit|production_actual_cancel_audit_(ready|recovered|degraded|failed|unknown)|production_actual_cancel_audit_boundary_violation|production_actual_cancel_audit_unknown_readback" \
  crates/cli/src/dashboard.rs >/dev/null

python3 - <<'PY'
from pathlib import Path

source = Path("crates/cli/src/dashboard.rs").read_text()
needle = "function renderProductionActualCancelAudit"
start = source.index(needle)
end = source.find("\nfunction ", start + len(needle))
renderer = source[start:] if end == -1 else source[start:end]
for forbidden in [
    "<button",
    "data-dashboard-action",
    "fetch(",
    "/api/control/cancel",
    "/api/control/order",
    "cancel button",
    "approve button",
    "retry button",
    "bulk action",
]:
    if forbidden in renderer:
        raise SystemExit(f"renderer contains forbidden marker: {forbidden}")
PY

echo "verify_v19_dashboard_actual_cancel_audit_view PASS states=ready,recovered,degraded,failed,unknown dashboard_cancel_controls_enabled=false dashboard_order_controls_enabled=false retry_button=false bulk_action=false"
