#!/usr/bin/env bash
set -euo pipefail

# V170-008: v0.17 failure/incident integration.
# This verifier stays local/offline. It proves v0.16 failure semantics are
# reflected in v0.17 reconciliation/orphan incident evidence without retry,
# cancel, remediation, or Dashboard controls.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

cargo test -p nautilus-cli \
  production_mutation_reconciliation_classifier_integrates_failure_incident_semantics \
  --lib

rg -n "timeout_readback_required|http_4xx_terminal_evidence|malformed_response_manual_review|readback_mismatch_risk_halt|kill_switch_transition_halt|failure_incident_risk_halt" \
  crates/cli/src/live.rs >/dev/null

echo "verify_v17_failure_incident_integration PASS"
