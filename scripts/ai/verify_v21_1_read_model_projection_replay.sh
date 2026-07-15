#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

SCOPE_PATH="${NTPRO_V211_REPLAY_SCOPE:-docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json}"
RELEASE_NOTES_PATH="${NTPRO_V211_RELEASE_NOTES:-docs/rust-cutover/release/v0_21_1_release_notes.md}"
EVIDENCE_PATH="${NTPRO_V211_REPLAY_EVIDENCE:-docs/rust-cutover/evidence/V211-003.md}"
PYTHON_BIN="${PYTHON_BIN:-}"

if [ -z "$PYTHON_BIN" ]; then
  if command -v python3 >/dev/null 2>&1; then
    PYTHON_BIN=python3
  elif command -v python >/dev/null 2>&1; then
    PYTHON_BIN=python
  else
    echo "python3 or python is required" >&2
    exit 127
  fi
fi

for path in "$SCOPE_PATH" "$RELEASE_NOTES_PATH" "$EVIDENCE_PATH"; do
  if [ ! -f "$path" ]; then
    echo "missing required V211 read-model replay file: $path" >&2
    exit 1
  fi
done

cargo test -p nautilus-cli --test golden_trace_read_model_projection
scripts/ai/ntpro_governance.sh golden-trace-release-scope \
  --manifest "$SCOPE_PATH" \
  --trace-glob 'tests/golden/*.jsonl'

SCOPE_PATH="$SCOPE_PATH" \
RELEASE_NOTES_PATH="$RELEASE_NOTES_PATH" \
EVIDENCE_PATH="$EVIDENCE_PATH" \
"$PYTHON_BIN" <<'PY'
import json
import os
from pathlib import Path

PROMOTED_CASES = {
    "read_model.account_snapshot.fresh.001",
    "read_model.account_snapshot.stale.001",
    "read_model.order_lifecycle.matched.001",
    "read_model.order_lifecycle.missing_ledger.001",
    "read_model.risk_state.healthy.001",
    "read_model.risk_state.mismatch.001",
    "read_model.dashboard.readonly_complete.001",
    "read_model.dashboard.missing_evidence_degraded.001",
}
V221_PROMOTED_CASES = {
    "read_model.position.long.001",
    "read_model.position.short.001",
    "read_model.position.flat.001",
    "read_model.position.precision_mismatch.001",
    "read_model.position.stale_source.001",
    "read_model.position.account_mismatch.001",
    "read_model.order_lifecycle.unknown_response.001",
    "read_model.order_lifecycle.readback_mismatch.001",
    "read_model.order_lifecycle.duplicate_attempt.001",
    "read_model.fill_execution.reconciled.001",
    "read_model.fill_execution.partial_fill.001",
    "read_model.fill_execution.duplicate_fill.001",
    "read_model.fill_execution.missing_order_linkage.001",
    "read_model.fill_execution.stale_source.001",
    "read_model.fill_execution.ambiguous_source.001",
    "read_model.risk_state.risk_visible.001",
    "read_model.risk_state.manual_review.001",
    "read_model.risk_state.halted.001",
    "read_model.risk_state.stale.001",
    "read_model.dashboard.forbidden_controls_blocked.001",
}
REMAINING_SCHEMA_ONLY_CASES = {
    "read_model.contract.healthy_minimal.001",
    "read_model.contract.fail_closed_missing_lineage_source_freshness.001",
    "read_model.account_snapshot.missing_provenance.001",
    "read_model.account_snapshot.redaction_breach.001",
}
HARNESS = "cargo test -p nautilus-cli --test golden_trace_read_model_projection"
ENTRYPOINT = "crates/cli/tests/golden_trace_read_model_projection.rs::rust_cli_read_model_projection_replays_v211_required_paths"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


scope = json.loads(Path(os.environ["SCOPE_PATH"]).read_text(encoding="utf-8"))
notes = Path(os.environ["RELEASE_NOTES_PATH"]).read_text(encoding="utf-8")
evidence = Path(os.environ["EVIDENCE_PATH"]).read_text(encoding="utf-8")
entries = {case["case_id"]: case for case in scope.get("cases", [])}
read_model_entries = [
    case for case in scope.get("cases", []) if case.get("category") == "read_model"
]
read_model_exec = [
    case for case in read_model_entries if case.get("status") == "executable_replay"
]
read_model_schema_only = [
    case for case in read_model_entries if case.get("status") == "schema_only_scoped"
]

require(len(read_model_entries) == 32, "read_model scope count must remain 32")
require(len(read_model_exec) >= len(PROMOTED_CASES), "V211-003 promoted read_model cases regressed")
require({case["case_id"] for case in read_model_exec}.issuperset(PROMOTED_CASES), "V211 promoted read_model case set mismatch")

for case_id in PROMOTED_CASES:
    entry = entries.get(case_id)
    require(entry is not None, f"missing promoted entry {case_id}")
    require(entry.get("status") == "executable_replay", f"{case_id}: must be executable_replay")
    require(entry.get("evidence_id") == "V211-003", f"{case_id}: evidence_id mismatch")
    require(entry.get("harness") == HARNESS, f"{case_id}: harness mismatch")
    require(entry.get("rust_entrypoint") == ENTRYPOINT, f"{case_id}: rust entrypoint mismatch")
    require(entry.get("release_decision") == "included_in_final_replay_scope", f"{case_id}: release decision mismatch")

for case_id in V221_PROMOTED_CASES:
    entry = entries.get(case_id)
    require(entry is not None, f"missing V221 promoted entry {case_id}")
    require(entry.get("status") == "executable_replay", f"{case_id}: V221 promotion must be executable_replay")
    require(entry.get("evidence_id") == "V221-003", f"{case_id}: V221 evidence_id mismatch")
    require(entry.get("harness") == HARNESS, f"{case_id}: harness mismatch")
    require(entry.get("rust_entrypoint") == ENTRYPOINT, f"{case_id}: rust entrypoint mismatch")
    require(entry.get("release_decision") == "included_in_final_replay_scope", f"{case_id}: release decision mismatch")

schema_only_ids = {case["case_id"] for case in read_model_schema_only}
require(schema_only_ids == REMAINING_SCHEMA_ONLY_CASES, "remaining read_model schema-only set mismatch")
for case_id in REMAINING_SCHEMA_ONLY_CASES:
    entry = entries.get(case_id)
    require(entry is not None, f"missing schema-only entry {case_id}")
    require(entry.get("status") == "schema_only_scoped", f"{case_id}: must remain schema_only_scoped")
    require("V211" in entry.get("follow_up", ""), f"{case_id}: follow-up must name V211 work")
    forbidden = {"evidence_id", "harness", "rust_entrypoint"} & set(entry)
    require(not forbidden, f"{case_id}: schema-only entry must not claim executable fields {sorted(forbidden)}")

for marker in (
    "read_model.account_snapshot.fresh.001",
    "read_model.order_lifecycle.missing_ledger.001",
    "read_model.dashboard.missing_evidence_degraded.001",
    "remaining schema-only",
    HARNESS,
):
    require(marker in notes, f"release notes missing marker {marker}")
    require(marker in evidence, f"evidence missing marker {marker}")

print(
    "v211_read_model_projection_replay status=ok "
    f"v211_promoted_read_model_cases={len(PROMOTED_CASES)} "
    f"v221_promoted_read_model_cases={len(V221_PROMOTED_CASES)} "
    f"remaining_schema_only={len(REMAINING_SCHEMA_ONLY_CASES)} "
    "harness=cargo_test"
)
PY
