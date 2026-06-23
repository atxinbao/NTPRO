#!/usr/bin/env bash
set -euo pipefail

# V160-008: v0.16 kill-switch enforcement around the guarded send path.
# This verifier stays offline and proves the guarded send artifact records
# pre-send and post-send kill-switch checks, and blocks when the kill switch is
# active.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

GATE_ROOT="${NTPRO_V16_KILL_SWITCH_AROUND_SEND_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v16-kill-switch-around-send.XXXXXX")}"

NTPRO_V16_GUARDED_SEND_ROOT="$GATE_ROOT" \
  scripts/ai/verify_v16_guarded_send_path.sh >/dev/null

READY_OFFLINE="$GATE_ROOT/command-output/ready-offline-guarded-send.json"
KILL_SWITCH_ACTIVE="$GATE_ROOT/command-output/kill-switch-active-guarded-send.json"

python3 - "$READY_OFFLINE" "$KILL_SWITCH_ACTIVE" <<'PY'
import json
import sys
from pathlib import Path

ready = json.loads(Path(sys.argv[1]).read_text())
active = json.loads(Path(sys.argv[2]).read_text())

assert ready["kill_switch_enforcement_ready"] is True
assert ready["kill_switch_checked_before_send"] is True
assert ready["kill_switch_checked_after_send"] is True
assert ready["pre_send_kill_switch_snapshot_source"] == ready["post_send_kill_switch_snapshot_source"]
assert ready["pre_send_kill_switch_snapshot_hash"] == ready["post_send_kill_switch_snapshot_hash"]
assert ready["pre_send_kill_switch_checked_at"]
assert ready["post_send_kill_switch_checked_at"]
assert ready["pre_send_kill_switch_runtime_gate_open"] is True
assert ready["pre_send_kill_switch_active"] is False
assert ready["post_send_kill_switch_runtime_gate_open"] is True
assert ready["post_send_kill_switch_active"] is False
assert ready["post_send_kill_switch_clean"] is True
assert ready["kill_switch_blocked_send"] is False
assert ready["post_send_progression_blocked"] is False
assert ready["manual_review_required"] is False
assert ready["new_orders_blocked"] is False
assert ready["request_sent"] is False
assert ready["network_attempted"] is False

assert active["status"] == "blocked_kill_switch_enforcement"
assert active["kill_switch_enforcement_ready"] is False
assert active["pre_send_kill_switch_snapshot_source"] == active["post_send_kill_switch_snapshot_source"]
assert active["pre_send_kill_switch_snapshot_hash"] == active["post_send_kill_switch_snapshot_hash"]
assert active["pre_send_kill_switch_checked_at"]
assert active["post_send_kill_switch_checked_at"]
assert active["pre_send_kill_switch_active"] is True
assert active["post_send_kill_switch_active"] is True
assert active["post_send_kill_switch_clean"] is False
assert active["kill_switch_blocked_send"] is True
assert active["post_send_progression_blocked"] is True
assert active["manual_review_required"] is True
assert active["new_orders_blocked"] is True
assert active["request_sent"] is False
assert active["network_attempted"] is False
PY

echo "v16_kill_switch_around_send status=ok root=$GATE_ROOT kill_switch_checked_before_send=true kill_switch_checked_after_send=true post_send_second_read_evidence=true active_kill_switch_blocks_send=true post_send_progression_blocked=true request_sent=false network_attempted=false"
