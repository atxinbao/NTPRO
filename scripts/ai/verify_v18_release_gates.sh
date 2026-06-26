#!/usr/bin/env bash
set -euo pipefail

# V180-010: v0.18 aggregate release gates.
# Aggregates preview-only cancel recovery artifacts and Dashboard read-only
# diagnostics. This gate must not send cancel requests, open network access,
# mutate production orders, or enable Dashboard cancel controls.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

GATE_ROOT="${NTPRO_V18_RELEASE_GATE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v18-release-gates.XXXXXX")}"
LOG_DIR="$GATE_ROOT/logs"
mkdir -p "$LOG_DIR"

run_gate() {
  local name="$1"
  shift
  local log="$LOG_DIR/$name.log"
  echo "== v18 release gates: $name =="
  "$@" 2>&1 | tee "$log"
}

run_gate cancel-recovery-artifacts scripts/ai/verify_v18_cancel_recovery_gates.sh
run_gate dashboard-cancel-recovery cargo test -p nautilus-cli production_cancel_recovery --lib

if grep -RE \
  "actual_cancel_send_allowed=true|cancel_attempted=true|automatic_cancel_allowed=true|dashboard_cancel_controls_enabled=true|network_attempted=true|request_sent=true|production_order_mutations_attempted=[1-9][0-9]*|api_key_value_recorded=true|api_secret_value_recorded=true|signed_query_recorded=true|signed_url_recorded=true|raw_exchange_response_recorded=true" \
  "$LOG_DIR" >/tmp/ntpro-v18-release-forbidden.txt; then
  echo "v18 release gate observed forbidden cancel recovery mutation evidence:" >&2
  cat /tmp/ntpro-v18-release-forbidden.txt >&2
  exit 1
fi

RELEASE_DOCS=(
  docs/rust-cutover/release/v0_18_0_cancel_recovery_artifact_contracts.md
  docs/rust-cutover/release/v0_18_0_readiness_report.md
  docs/rust-cutover/release/v0_18_0_release_notes.md
)

required_release_markers=(
  "Owner-Approved Cancel Recovery Preview"
  "actual_cancel_send_allowed=false"
  "cancel_attempted=false"
  "automatic_cancel_allowed=false"
  "dashboard_cancel_controls_enabled=false"
  "network_attempted=false"
  "production_order_mutations_attempted=0"
  "manual_owner_approval_required=true"
  "owner_approved=false"
  "Actual single-shot cancel remains a v0.19+ scope decision."
)

for doc in "${RELEASE_DOCS[@]}"; do
  if [[ ! -f "$doc" ]]; then
    echo "missing v0.18 release document: $doc" >&2
    exit 1
  fi
done

for marker in "${required_release_markers[@]}"; do
  if ! grep -RFi "$marker" "${RELEASE_DOCS[@]}" >/dev/null; then
    echo "missing v0.18 release boundary marker: $marker" >&2
    exit 1
  fi
done

for evidence in \
  docs/rust-cutover/evidence/V180-001.md \
  docs/rust-cutover/evidence/V180-002.md \
  docs/rust-cutover/evidence/V180-003.md \
  docs/rust-cutover/evidence/V180-004.md \
  docs/rust-cutover/evidence/V180-005.md \
  docs/rust-cutover/evidence/V180-006.md \
  docs/rust-cutover/evidence/V180-007.md \
  docs/rust-cutover/evidence/V180-008.md \
  docs/rust-cutover/evidence/V180-009.md \
  docs/rust-cutover/evidence/V180-010.md \
  docs/rust-cutover/evidence/V180-011.md; do
  if [[ ! -f "$evidence" ]]; then
    echo "missing v0.18 evidence file: $evidence" >&2
    exit 1
  fi
done

echo "v18_release_gates status=ok root=$GATE_ROOT actual_cancel_send_allowed=false cancel_attempted=false automatic_cancel_allowed=false dashboard_cancel_controls_enabled=false network_attempted=false production_order_mutations_attempted=0 manual_owner_approval_required=true owner_approved=false"
