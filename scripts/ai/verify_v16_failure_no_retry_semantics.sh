#!/usr/bin/env bash
set -euo pipefail

# V160-010: v0.16 failure-mode and no-retry semantics.
# This verifier is local/offline. It derives failure semantics evidence from the
# redacted audit trail and proves every supported failure mode writes evidence
# and stops without retry, cancel, replace, amend, correction, flatten,
# Dashboard order controls, listenKey lifecycle, or strategy continuation.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V16_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V16_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V16_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

GATE_ROOT="${NTPRO_V16_FAILURE_SEMANTICS_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v16-failure-semantics.XXXXXX")}"
AUDIT_TRAIL_ROOT="$GATE_ROOT/audit-trail"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

NTPRO_V16_SKIP_BUILD=1 \
NTPRO_V16_NAUTILUS_BIN="$NAUTILUS_BIN" \
NTPRO_V16_AUDIT_TRAIL_ROOT="$AUDIT_TRAIL_ROOT" \
  scripts/ai/verify_v16_mutation_audit_trail.sh >/dev/null

AUDIT_TRAIL="$AUDIT_TRAIL_ROOT/command-output/ready-redacted-audit-trail.json"
MISSING_FLAGS_FAILURE="$OUTPUT_DIR/missing-flags-failure-semantics.json"

if [[ ! -f "$AUDIT_TRAIL" ]]; then
  echo "failure semantics setup did not produce expected audit trail input" >&2
  exit 1
fi

run_failure_semantics() {
  local mode="$1"
  local output="$2"
  shift 2
  "$NAUTILUS_BIN" live production-mutation-failure-semantics \
    --run-id "v160-production-mutation-failure-semantics-$mode" \
    --audit-trail "$AUDIT_TRAIL" \
    --failure-mode "$mode" \
    --output "$output" \
    "$@"
}

run_failure_semantics "timeout" "$MISSING_FLAGS_FAILURE" >/dev/null

declare -a MODES=(
  "timeout"
  "http-4xx"
  "http-5xx"
  "malformed-response"
  "readback-mismatch"
  "kill-switch-transition"
)

for mode in "${MODES[@]}"; do
  run_failure_semantics "$mode" "$OUTPUT_DIR/ready-$mode.json" \
    --allow-production-mutation-failure-semantics \
    --confirm-evidence-only-failure-handling \
    --confirm-no-retry \
    --confirm-no-automatic-cancel-replace-amend \
    --confirm-no-correction-or-flatten \
    --confirm-dashboard-order-controls-disabled \
    --confirm-no-strategy-continuation \
    --confirm-no-listen-key-lifecycle >/dev/null
done

python3 - "$MISSING_FLAGS_FAILURE" "$OUTPUT_DIR" <<'PY'
import json
import sys
from pathlib import Path

missing_flags = json.loads(Path(sys.argv[1]).read_text())
output_dir = Path(sys.argv[2])

assert missing_flags["schema_version"] == "ntpro.v160_production_mutation_failure_semantics.v1"
assert missing_flags["status"] == "blocked_missing_gate"
assert missing_flags["failure_semantics_ready"] is False
assert missing_flags["failure_state"] == "blocked_missing_gate"
assert missing_flags["retry_attempted"] is False
assert missing_flags["remediation_attempted"] is False
assert missing_flags["strategy_continuation_allowed"] is False
assert "--allow-production-mutation-failure-semantics" in missing_flags["missing_cli_flags"]
assert "--confirm-no-retry" in missing_flags["missing_cli_flags"]

expected_states = {
    "timeout": "timeout_write_evidence_and_stop",
    "http-4xx": "http_4xx_write_evidence_and_stop",
    "http-5xx": "http_5xx_write_evidence_and_stop",
    "malformed-response": "malformed_response_write_evidence_and_stop",
    "readback-mismatch": "readback_mismatch_write_evidence_and_stop",
    "kill-switch-transition": "kill_switch_transition_write_evidence_and_stop",
}

for mode, expected_state in expected_states.items():
    artifact = json.loads((output_dir / f"ready-{mode}.json").read_text())
    assert artifact["schema_version"] == "ntpro.v160_production_mutation_failure_semantics.v1"
    assert artifact["status"] == "ready_failure_semantics_evidence"
    assert artifact["failure_semantics_ready"] is True
    assert artifact["failure_mode"] == mode
    assert artifact["failure_state"] == expected_state
    assert artifact["terminal_action"] == "write_evidence_and_stop"
    assert artifact["evidence_written"] is True
    assert artifact["stop_after_evidence"] is True
    assert artifact["strategy_continuation_allowed"] is False
    assert artifact["source_audit_trail_status"] == "ready_redacted_audit_trail"
    assert artifact["source_audit_trail_ready"] is True
    assert artifact["source_failure_state"] == "none_recorded"
    assert artifact["source_artifact_issues"] == []
    assert artifact["missing_cli_flags"] == []
    for field in [
        "retry_allowed",
        "retry_attempted",
        "cancel_attempted",
        "replace_attempted",
        "amend_attempted",
        "correction_attempted",
        "flatten_attempted",
        "remediation_attempted",
        "automatic_remediation_allowed",
        "dashboard_order_controls_enabled",
        "production_order_mutation_allowed",
        "production_order_state_reads_allowed",
        "listen_key_lifecycle_allowed",
    ]:
        assert artifact[field] is False, (mode, field)
    assert artifact["retry_attempts"] == 0
    assert artifact["max_retry_attempts"] == 0
    assert artifact["production_order_mutations_attempted"] == 0
    assert artifact["production_order_state_reads_attempted"] == 0
    assert artifact["listen_key_lifecycle_attempted"] == 0
    assert artifact["evidence_only_failure_handling_confirmed"] is True
    assert artifact["no_retry_confirmed"] is True
    assert artifact["no_automatic_cancel_replace_amend_confirmed"] is True
    assert artifact["no_correction_or_flatten_confirmed"] is True
    assert artifact["dashboard_controls_disabled_confirmed"] is True
    assert artifact["no_strategy_continuation_confirmed"] is True
    assert artifact["no_listen_key_lifecycle_confirmed"] is True
PY

if grep -R "ntpro_v160005_production_like_api_key_value\\|ntpro_v160005_production_like_api_secret_value\\|ntpro_v160007_api_key_value\\|ntpro_v160007_api_secret_value\\|X-MBX-APIKEY\\|signature=" "$OUTPUT_DIR" >/dev/null; then
  echo "failure semantics artifacts persisted forbidden secret or signed material" >&2
  exit 1
fi

echo "v16_failure_no_retry_semantics status=ok root=$GATE_ROOT modes=${MODES[*]} retry_attempted=false remediation_attempted=false strategy_continuation_allowed=false dashboard_order_controls_enabled=false"
