#!/usr/bin/env bash
set -euo pipefail

# V130-005: Dashboard trader/ops control boundary gate.
# This script is CI-safe. It does not open network connections, does not read
# credentials, and does not submit, cancel, replace, amend, retry, correct, or
# reconnect production orders.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

echo "== v13 Dashboard boundary: targeted tests =="
cargo test -p nautilus-cli dashboard_trader_ops_boundary --lib
cargo test -p nautilus-cli dashboard_http_server_rejects_v13_production_order_control_routes --lib

echo "== v13 Dashboard boundary: positive contract markers =="
grep -nE \
  "dashboard_surface=local_read_model|trader_role=read_only_status_and_evidence|ops_role=local_supervisor_lifecycle_only|allowed_local_controls=start,stop,pause,resume,reconnect_data,reconnect_execution|dashboard_order_controls_enabled=false|dashboard_credential_entry_enabled=false|production_order_submission_allowed=false|production_order_mutation_allowed=false|production_reconnect_allowed=false|listen_key_lifecycle_allowed=false" \
  docs/rust-cutover/release/v0_13_0_dashboard_control_boundary.md \
  docs/rust-cutover/evidence/V130-005.md >/dev/null

echo "== v13 Dashboard boundary: v0.13 scope linkage =="
grep -nE \
  "Trader/Ops Dashboard control boundary|Dashboard role boundary|Dashboard order controls" \
  docs/rust-cutover/release/v0_13_0_scope_decision.md \
  docs/rust-cutover/tasks/V130-001.md >/dev/null

echo "== v13 Dashboard boundary: forbidden route scan =="
if rg -n \
  'route\("/api/nodes/\{node_id\}/actions/(submit|submit_order|cancel|cancel_order|replace|replace_order|amend|amend_order|retry|retry_order|correct|correct_order|flatten|flatten_position|credential_entry|listen_key)"' \
  crates/cli/src/dashboard.rs >/dev/null; then
  echo "Dashboard exposes a v0.13 forbidden production order-control route" >&2
  exit 1
fi

echo "== v13 Dashboard boundary: forbidden release claims =="
if grep -nE \
  "dashboard_order_controls_enabled=true|dashboard_credential_entry_enabled=true|production_order_submission_allowed=true|production_order_mutation_allowed=true|production_reconnect_allowed=true|listen_key_lifecycle_allowed=true|production trading ready|real funds ready|Dashboard order controls = enabled" \
  docs/rust-cutover/release/v0_13_0_dashboard_control_boundary.md \
  docs/rust-cutover/evidence/V130-005.md \
  docs/rust-cutover/tasks/V130-005.md >/dev/null; then
  echo "v13 Dashboard boundary docs contain an enabled production control claim" >&2
  exit 1
fi

echo "v13_dashboard_control_boundary status=ok dashboard_order_controls_enabled=false production_order_mutation_allowed=false production_reconnect_allowed=false"
