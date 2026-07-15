#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

scripts/ai/ntpro_governance.sh docs-examples
scripts/ai/check_rust_examples.sh
scripts/ai/check_backend_freeze_baseline.sh
