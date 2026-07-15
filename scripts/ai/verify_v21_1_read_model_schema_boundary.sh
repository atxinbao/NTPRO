#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

SCHEMA_PATH="${NTPRO_V211_SCHEMA_BOUNDARY_SCHEMA:-docs/rust-cutover/release/v0_21_0_unified_read_model_schema.json}"
TRACE_GLOB="${NTPRO_V211_SCHEMA_BOUNDARY_TRACE_GLOB:-tests/golden/**/*.jsonl}"

if [ ! -f "$SCHEMA_PATH" ]; then
  echo "missing V211 schema boundary schema: $SCHEMA_PATH" >&2
  exit 1
fi

scripts/ai/ntpro_governance.sh golden-trace tests/golden/read_model_contract_schema.jsonl --mode validate-only
scripts/ai/ntpro_governance.sh golden-trace tests/golden/v211/read_model_health_status_semantics_schema.jsonl --mode validate-only

scripts/ai/ntpro_governance.sh read-model-schema \
  --schema "$SCHEMA_PATH" \
  --trace-glob "$TRACE_GLOB"
