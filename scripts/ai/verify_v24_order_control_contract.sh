#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

CONTRACT_PATH="${NTPRO_V24_ORDER_CONTROL_CONTRACT:-docs/rust-cutover/release/v0_24_0_order_control_contract.md}"
TASK_PATH="${NTPRO_V24_ORDER_CONTROL_TASK:-docs/rust-cutover/tasks/V240-001.md}"
EVIDENCE_PATH="${NTPRO_V24_ORDER_CONTROL_EVIDENCE:-docs/rust-cutover/evidence/V240-001.md}"

fail() {
  echo "v24 order-control contract failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

contains() {
  local path="$1"
  local marker="$2"
  grep -F -- "$marker" "$path" >/dev/null
}

validate_contract_file() {
  local path="$1"
  local marker

  for marker in \
    "schema_version = ntpro.v240_order_control_contract.v1" \
    "contract_id = ntpro.v240_execution_order_control_contract.v1" \
    "contract_status = contract_only_no_runtime_adapter_call" \
    "start_gate_dependency = scripts/ai/verify_release.sh v24-intake-gate" \
    "limit_order_boundary = preview_contract_only" \
    "market_order_boundary = preview_contract_only" \
    "rate_limit_gate = required_before_any_runtime_operation" \
    "throttle_gate = required_before_any_runtime_operation" \
    "order_slicing_gate = required_before_child_order_preview" \
    "cancel_replace_amend_gate = preview_contract_only" \
    "retry_policy = default_no_retry" \
    "owner_approval_required = true" \
    "policy_approval_required = true" \
    "risk_gate_required = true" \
    "audit_gate_required = true" \
    "account_key_required = true" \
    "strategy_key_required = true" \
    "venue_node_key_required = true" \
    "isolation_scope_key_required = true" \
    "missing_account_key = fail_closed" \
    "missing_strategy_key = fail_closed" \
    "missing_venue_node_key = fail_closed" \
    "missing_isolation_scope_key = fail_closed" \
    "cross_account_operation = fail_closed" \
    "cross_strategy_operation = fail_closed" \
    "cross_venue_operation = fail_closed" \
    "cross_node_operation = fail_closed" \
    "dashboard_workbench_boundary = read_only_preview" \
    "live_order_control_button_enabled = false" \
    "new_submit_capability = false" \
    "production_order_mutation_allowed = false" \
    "execution_adapter_call_allowed = false" \
    "dashboard_operation_controls_enabled = false" \
    "product_grade_trading_terminal_claim = false" \
    "V240-002 = order intent and execution policy model" \
    "V240-003 = rate-limit and throttle gate preview" \
    "V240-004 = order slicing preview foundation" \
    "V240-005 = cancel replace amend preview contract" \
    "V240-006 = retry no-retry policy ledger" \
    "V240-007 = readback and audit evidence" \
    "V240-008 = Dashboard Workbench read-only order-control preview"; do
    contains "$path" "$marker" || return 1
  done

  for marker in \
    "new_submit_capability = true" \
    "production_order_mutation_allowed = true" \
    "execution_adapter_call_allowed = true" \
    "dashboard_operation_controls_enabled = true" \
    "live_order_control_button_enabled = true" \
    "product_grade_trading_terminal_claim = true" \
    "missing_account_key = allow" \
    "missing_strategy_key = allow" \
    "missing_venue_node_key = allow" \
    "missing_isolation_scope_key = allow" \
    "retry_policy = implicit_retry_allowed" \
    "dashboard_workbench_boundary = live_order_controls"; do
    if contains "$path" "$marker"; then
      return 1
    fi
  done
}

for path in "$CONTRACT_PATH" "$TASK_PATH" "$EVIDENCE_PATH" docs/rust-cutover/release/v0_24_0_intake_gate.md; do
  require_file "$path"
done

for marker in \
  "Task: \`V240-001\` / GitHub issue \`#744\`" \
  "Status: LOCAL VALIDATION PASSED" \
  "scripts/ai/verify_release.sh v24-order-control-contract"; do
  contains "$TASK_PATH" "$marker" || fail "missing marker in $TASK_PATH: $marker"
  contains "$EVIDENCE_PATH" "$marker" || fail "missing marker in $EVIDENCE_PATH: $marker"
done

validate_contract_file "$CONTRACT_PATH" || fail "contract markers failed validation: $CONTRACT_PATH"

tmp_contract="$(mktemp)"
cp "$CONTRACT_PATH" "$tmp_contract"
python3 - "$tmp_contract" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = text.replace("new_submit_capability = false", "new_submit_capability = true", 1)
path.write_text(text, encoding="utf-8")
PY

if validate_contract_file "$tmp_contract"; then
  rm -f "$tmp_contract"
  fail "negative selftest failed: new_submit_capability=true was accepted"
fi
rm -f "$tmp_contract"

scripts/ai/verify_release.sh v24-intake-gate

echo "v24_order_control_contract=pass"
echo "contract_id=ntpro.v240_execution_order_control_contract.v1"
echo "limit_order_boundary=preview_contract_only"
echo "market_order_boundary=preview_contract_only"
echo "missing_identity_keys=fail_closed"
echo "new_submit_capability=false"
echo "production_order_mutation_allowed=false"
echo "dashboard_operation_controls_enabled=false"
echo "execution_adapter_call_allowed=false"
echo "negative_selftest=1"
