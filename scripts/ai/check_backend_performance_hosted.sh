#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CONTRACT="${NTPRO_BACKEND_PERFORMANCE_HOSTED_CONTRACT:-docs/rust-cutover/governance/backend_performance_hosted_contract.json}"

scripts/ai/ntpro_governance.sh backend-benchmark-contract \
  --contract "$CONTRACT"
