#!/usr/bin/env bash
set -euo pipefail

# V160-005: v0.16 guarded production HTTP send path.
# Default validation is local/offline only. It proves the guarded send command can
# evaluate ready source artifacts without network, and that manual-online mode is
# blocked unless every owner/env gate is explicitly opened.

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

GATE_ROOT="${NTPRO_V16_GUARDED_SEND_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v16-guarded-send.XXXXXX")}"
REQUEST_BUILDER_ROOT="$GATE_ROOT/request-builder"
OUTPUT_DIR="$GATE_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

NTPRO_V16_SKIP_BUILD=1 \
NTPRO_V16_NAUTILUS_BIN="$NAUTILUS_BIN" \
NTPRO_V16_REQUEST_BUILDER_ROOT="$REQUEST_BUILDER_ROOT" \
  scripts/ai/verify_v16_request_builder.sh >/dev/null

REQUEST_BUILDER="$REQUEST_BUILDER_ROOT/command-output/ready-request-builder.json"
REQUEST_PREVIEW="$REQUEST_BUILDER_ROOT/command-output/production-request-preview.json"
KILL_SWITCH_RUNTIME_GATE="$REQUEST_BUILDER_ROOT/command-output/kill-switch-runtime-gate.json"
MISSING_FLAGS_GUARDED_SEND="$OUTPUT_DIR/missing-flags-guarded-send.json"
READY_OFFLINE_GUARDED_SEND="$OUTPUT_DIR/ready-offline-guarded-send.json"
MANUAL_MISSING_ENV_GUARDED_SEND="$OUTPUT_DIR/manual-missing-env-guarded-send.json"
KILL_SWITCH_ACTIVE_GATE="$OUTPUT_DIR/kill-switch-active-runtime-gate.json"
KILL_SWITCH_ACTIVE_GUARDED_SEND="$OUTPUT_DIR/kill-switch-active-guarded-send.json"

if [[ ! -f "$REQUEST_BUILDER" || ! -f "$REQUEST_PREVIEW" || ! -f "$KILL_SWITCH_RUNTIME_GATE" ]]; then
  echo "request-builder setup did not produce expected inputs" >&2
  exit 1
fi

run_guarded_send() {
  local output="$1"
  shift
  "$NAUTILUS_BIN" live production-mutation-guarded-send \
    --run-id v160-production-mutation-guarded-send \
    --request-builder "$REQUEST_BUILDER" \
    --kill-switch-runtime-gate "$KILL_SWITCH_RUNTIME_GATE" \
    --request-preview "$REQUEST_PREVIEW" \
    --api-key-env NTPRO_V160004_API_KEY \
    --api-secret-env NTPRO_V160004_API_SECRET \
    --timestamp-ms 1718400000000 \
    --recv-window-ms 5000 \
    --max-notional 10.00 \
    --output "$output" \
    "$@"
}

run_guarded_send "$MISSING_FLAGS_GUARDED_SEND" >/dev/null

run_guarded_send "$READY_OFFLINE_GUARDED_SEND" \
  --allow-production-mutation-guarded-send \
  --confirm-owner-approved-guarded-send \
  --confirm-single-limit-gtc \
  --confirm-tiny-notional \
  --confirm-single-shot \
  --confirm-no-retry \
  --confirm-no-secret-persistence \
  --confirm-response-redacted \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-listen-key-lifecycle >/dev/null

run_guarded_send "$MANUAL_MISSING_ENV_GUARDED_SEND" \
  --manual-online \
  --allow-production-mutation-guarded-send \
  --confirm-owner-approved-guarded-send \
  --confirm-single-limit-gtc \
  --confirm-tiny-notional \
  --confirm-single-shot \
  --confirm-no-retry \
  --confirm-no-secret-persistence \
  --confirm-response-redacted \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-listen-key-lifecycle >/dev/null

python3 - "$KILL_SWITCH_RUNTIME_GATE" "$KILL_SWITCH_ACTIVE_GATE" <<'PY'
import json
import sys
from pathlib import Path

gate = json.loads(Path(sys.argv[1]).read_text())
gate["status"] = "blocked_kill_switch_active"
gate["runtime_gate_open"] = False
gate["kill_switch_active"] = True
Path(sys.argv[2]).write_text(json.dumps(gate, indent=2) + "\n")
PY

"$NAUTILUS_BIN" live production-mutation-guarded-send \
  --run-id v160-production-mutation-guarded-send-kill-switch-active \
  --request-builder "$REQUEST_BUILDER" \
  --kill-switch-runtime-gate "$KILL_SWITCH_ACTIVE_GATE" \
  --request-preview "$REQUEST_PREVIEW" \
  --api-key-env NTPRO_V160004_API_KEY \
  --api-secret-env NTPRO_V160004_API_SECRET \
  --timestamp-ms 1718400000000 \
  --recv-window-ms 5000 \
  --max-notional 10.00 \
  --output "$KILL_SWITCH_ACTIVE_GUARDED_SEND" \
  --allow-production-mutation-guarded-send \
  --confirm-owner-approved-guarded-send \
  --confirm-single-limit-gtc \
  --confirm-tiny-notional \
  --confirm-single-shot \
  --confirm-no-retry \
  --confirm-no-secret-persistence \
  --confirm-response-redacted \
  --confirm-dashboard-order-controls-disabled \
  --confirm-no-listen-key-lifecycle >/dev/null

python3 - "$MISSING_FLAGS_GUARDED_SEND" "$READY_OFFLINE_GUARDED_SEND" "$MANUAL_MISSING_ENV_GUARDED_SEND" "$KILL_SWITCH_ACTIVE_GUARDED_SEND" <<'PY'
import json
import sys
from pathlib import Path

missing_flags = json.loads(Path(sys.argv[1]).read_text())
ready_offline = json.loads(Path(sys.argv[2]).read_text())
manual_missing_env = json.loads(Path(sys.argv[3]).read_text())
kill_switch_active = json.loads(Path(sys.argv[4]).read_text())

assert missing_flags["schema_version"] == "ntpro.v160_production_mutation_guarded_send.v1"
assert missing_flags["status"] == "blocked_missing_gate"
assert missing_flags["guarded_send_ready"] is False
assert missing_flags["request_sent"] is False
assert missing_flags["network_attempted"] is False
assert missing_flags["production_orders_submitted"] == 0
assert missing_flags["production_order_mutations_attempted"] == 0
assert "--allow-production-mutation-guarded-send" in missing_flags["missing_cli_flags"]
assert "--confirm-owner-approved-guarded-send" in missing_flags["missing_cli_flags"]

assert ready_offline["schema_version"] == "ntpro.v160_production_mutation_guarded_send.v1"
assert ready_offline["status"] == "ready_guarded_send_path_offline_no_network"
assert ready_offline["manual_online_requested"] is False
assert ready_offline["guarded_send_ready"] is True
assert ready_offline["send_path_evaluated"] is True
assert ready_offline["kill_switch_enforcement_ready"] is True
assert ready_offline["kill_switch_checked_before_send"] is True
assert ready_offline["kill_switch_checked_after_send"] is True
assert ready_offline["pre_send_kill_switch_runtime_gate_open"] is True
assert ready_offline["pre_send_kill_switch_active"] is False
assert ready_offline["post_send_kill_switch_runtime_gate_open"] is True
assert ready_offline["post_send_kill_switch_active"] is False
assert ready_offline["kill_switch_blocked_send"] is False
assert ready_offline["single_shot_send_allowed"] is False
assert ready_offline["request_builder_status"] == "ready_request_object_built_no_send"
assert ready_offline["request_object_built"] is True
assert ready_offline["request_method"] == "POST"
assert ready_offline["request_target"] == "/api/v3/order"
assert ready_offline["order_type"] == "LIMIT"
assert ready_offline["time_in_force"] == "GTC"
assert ready_offline["credential_material"] == "production_live_alpha"
assert ready_offline["api_key_value_recorded"] is False
assert ready_offline["api_secret_value_recorded"] is False
assert ready_offline["api_key_header_value_recorded"] is False
assert ready_offline["signature_recorded"] is False
assert ready_offline["signed_query_recorded"] is False
assert ready_offline["signed_url_recorded"] is False
assert ready_offline["request_body_recorded"] is False
assert ready_offline["raw_request_body_recorded"] is False
assert ready_offline["raw_exchange_response_recorded"] is False
assert ready_offline["response_body_recorded"] is False
assert ready_offline["response_redacted"] is True
assert ready_offline["error_code"] == "not_attempted_offline"
assert ready_offline["request_sent"] is False
assert ready_offline["network_attempted"] is False
assert ready_offline["production_order_submission_allowed"] is False
assert ready_offline["production_order_mutation_allowed"] is False
assert ready_offline["production_order_state_reads_allowed"] is False
assert ready_offline["listen_key_lifecycle_allowed"] is False
assert ready_offline["production_order_submissions_attempted"] == 0
assert ready_offline["production_orders_submitted"] == 0
assert ready_offline["production_order_mutations_attempted"] == 0
assert ready_offline["production_order_state_reads_attempted"] == 0
assert ready_offline["listen_key_lifecycle_attempted"] == 0
assert ready_offline["retry_attempted"] is False
assert ready_offline["cancel_attempted"] is False
assert ready_offline["replace_attempted"] is False
assert ready_offline["amend_attempted"] is False
assert ready_offline["flatten_attempted"] is False
assert ready_offline["dashboard_order_controls_enabled"] is False
assert ready_offline["real_orders_submitted"] is False
assert ready_offline["real_funds"] is False
assert ready_offline["production_trading_enabled"] is False
assert ready_offline["source_artifact_issues"] == []
assert ready_offline["missing_cli_flags"] == []
assert ready_offline["missing_env_vars"] == []

assert manual_missing_env["status"] == "blocked_missing_manual_online_gate"
assert manual_missing_env["manual_online_requested"] is True
assert manual_missing_env["guarded_send_ready"] is False
assert manual_missing_env["kill_switch_enforcement_ready"] is True
assert manual_missing_env["kill_switch_blocked_send"] is False
assert manual_missing_env["single_shot_send_allowed"] is False
assert manual_missing_env["request_sent"] is False
assert manual_missing_env["network_attempted"] is False
assert manual_missing_env["production_order_submission_allowed"] is False
assert manual_missing_env["production_orders_submitted"] == 0
assert manual_missing_env["production_order_mutations_attempted"] == 0
assert "NTPRO_ALLOW_PRODUCTION_MUTATION_HTTP_SEND" in manual_missing_env["missing_env_vars"]
assert "NTPRO_OWNER_APPROVED_PRODUCTION_MUTATION_HTTP_SEND" in manual_missing_env["missing_env_vars"]
assert "NTPRO_CONFIRM_PRODUCTION_MUTATION_SINGLE_SHOT" in manual_missing_env["missing_env_vars"]
assert "NTPRO_ALLOW_PRODUCTION_MUTATION_SIGNING_MATERIAL" in manual_missing_env["missing_env_vars"]

assert kill_switch_active["status"] == "blocked_kill_switch_enforcement"
assert kill_switch_active["guarded_send_ready"] is False
assert kill_switch_active["kill_switch_enforcement_ready"] is False
assert kill_switch_active["kill_switch_checked_before_send"] is True
assert kill_switch_active["kill_switch_checked_after_send"] is True
assert kill_switch_active["pre_send_kill_switch_runtime_gate_open"] is False
assert kill_switch_active["pre_send_kill_switch_active"] is True
assert kill_switch_active["post_send_kill_switch_runtime_gate_open"] is False
assert kill_switch_active["post_send_kill_switch_active"] is True
assert kill_switch_active["kill_switch_blocked_send"] is True
assert kill_switch_active["single_shot_send_allowed"] is False
assert kill_switch_active["request_sent"] is False
assert kill_switch_active["network_attempted"] is False
assert kill_switch_active["production_order_submission_allowed"] is False
assert "kill_switch_runtime_gate_not_open" in kill_switch_active["source_artifact_issues"]
assert "kill_switch_active_before_send" in kill_switch_active["source_artifact_issues"]
PY

if grep -R "ntpro_v160004_production_like_api_key_value\\|ntpro_v160004_production_like_api_secret_value" "$OUTPUT_DIR" "$REQUEST_BUILDER_ROOT/command-output" >/dev/null; then
  echo "guarded send artifacts persisted secret material" >&2
  exit 1
fi

echo "v16_guarded_send_path status=ok root=$GATE_ROOT request_sent=false network_attempted=false production_orders_submitted=0 production_order_mutations_attempted=0 dashboard_order_controls_enabled=false"
