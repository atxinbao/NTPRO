#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

SCHEMA_PATH="${NTPRO_V211_SCHEMA_BOUNDARY_SCHEMA:-docs/rust-cutover/release/v0_21_0_unified_read_model_schema.json}"
TRACE_GLOB="${NTPRO_V211_SCHEMA_BOUNDARY_TRACE_GLOB:-tests/golden/**/*.jsonl}"
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

if [ ! -f "$SCHEMA_PATH" ]; then
  echo "missing V211 schema boundary schema: $SCHEMA_PATH" >&2
  exit 1
fi

scripts/ai/ntpro_governance.sh golden-trace tests/golden/read_model_contract_schema.jsonl --mode validate-only
scripts/ai/ntpro_governance.sh golden-trace tests/golden/v211/read_model_health_status_semantics_schema.jsonl --mode validate-only

if "$PYTHON_BIN" - <<'PY' >/dev/null 2>&1
import jsonschema
PY
then
  "$PYTHON_BIN" scripts/ai/validate_v21_read_model_schema.py \
    --schema "$SCHEMA_PATH" \
    --trace-glob "$TRACE_GLOB"
elif command -v uv >/dev/null 2>&1; then
  uv run --group test python scripts/ai/validate_v21_read_model_schema.py \
    --schema "$SCHEMA_PATH" \
    --trace-glob "$TRACE_GLOB"
else
  echo "jsonschema is required; install project test dependencies or provide uv" >&2
  exit 127
fi
