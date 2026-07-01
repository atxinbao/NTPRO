#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

DASHBOARD_PATH="crates/cli/src/dashboard.rs"
DASHBOARD_DOC="docs/rust-cutover/release/v0_21_0_trader_terminal_readonly_dashboard.md"
RELEASE_NOTES_PATH="docs/rust-cutover/release/v0_21_1_release_notes.md"
EVIDENCE_PATH="docs/rust-cutover/evidence/V211-005.md"

for path in "$DASHBOARD_PATH" "$DASHBOARD_DOC" "$RELEASE_NOTES_PATH" "$EVIDENCE_PATH"; do
  if [ ! -f "$path" ]; then
    echo "missing V211-005 required file: $path" >&2
    exit 1
  fi
done

cargo test -p nautilus-cli trader_terminal_read_model -- --nocapture

for marker in \
  "TRADER_TERMINAL_READ_MODEL_ARTIFACT_RELATIVE_PATH" \
  "v0_21/unified_read_model_snapshot.json" \
  "read_model_runtime" \
  "renderReadModelRuntime" \
  "canonical_unified_read_model_artifact_missing" \
  "schema_mismatch" \
  "component_unavailable"; do
  if ! grep -Fq "$marker" "$DASHBOARD_PATH"; then
    echo "dashboard bridge missing marker: $marker" >&2
    exit 1
  fi
done

for marker in \
  "V211-005" \
  "v21.1-trader-terminal-read-model-bridge" \
  "v0_21/unified_read_model_snapshot.json" \
  "missing_artifact" \
  "schema_mismatch" \
  "stale_artifact" \
  "component_unavailable" \
  "dashboard_order_controls_enabled = false"; do
  if ! grep -Fq "$marker" "$DASHBOARD_DOC" "$RELEASE_NOTES_PATH" "$EVIDENCE_PATH"; then
    echo "V211-005 docs/evidence missing marker: $marker" >&2
    exit 1
  fi
done

echo "v211_trader_terminal_read_model_bridge status=ok cargo_filter=trader_terminal_read_model canonical_artifact=v0_21/unified_read_model_snapshot.json"
