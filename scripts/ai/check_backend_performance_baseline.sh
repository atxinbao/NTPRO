#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

BASELINE="${NTPRO_BACKEND_PERFORMANCE_BASELINE:-docs/rust-cutover/governance/backend_performance_baseline.json}"
RUN_NEGATIVE_SELFTEST="${NTPRO_BACKEND_PERFORMANCE_NEGATIVE_SELFTEST:-1}"

case "$RUN_NEGATIVE_SELFTEST" in
  0) negative_selftest=false ;;
  1) negative_selftest=true ;;
  *)
    echo "backend performance baseline drift: NTPRO_BACKEND_PERFORMANCE_NEGATIVE_SELFTEST must be 0 or 1" >&2
    exit 1
    ;;
esac

scripts/ai/ntpro_governance.sh backend-performance-baseline \
  --baseline "$BASELINE" \
  --negative-selftest "$negative_selftest"
