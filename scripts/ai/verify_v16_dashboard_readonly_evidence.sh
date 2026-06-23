#!/usr/bin/env bash
set -euo pipefail

# V160-011: v0.16 Dashboard read-only production mutation evidence panel.
# This verifier is local/offline. It proves the Dashboard snapshot exposes
# v0.16 production mutation evidence as read-only status only, with no order
# controls or credential input surface.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

cargo test -p nautilus-cli production_mutation_v16_evidence --lib

grep -nE \
  "production_mutation_evidence|renderProductionMutationEvidence|v0\\.16 Production Mutation Evidence" \
  crates/cli/src/dashboard.rs >/dev/null

renderer_body="$(awk '
  /^function renderProductionMutationEvidence\(items\) \{/ { in_renderer = 1 }
  in_renderer { print }
  in_renderer && /^}$/ { exit }
' crates/cli/src/dashboard.rs)"

if [[ -z "$renderer_body" ]]; then
  echo "missing v0.16 production mutation evidence renderer" >&2
  exit 1
fi

if printf '%s\n' "$renderer_body" | grep -E "(<button|data-dashboard-action|fetch\\(|credential)" >/dev/null; then
  echo "v0.16 production mutation evidence renderer contains control or credential surface" >&2
  exit 1
fi

echo "v16_dashboard_readonly_evidence status=ok panel=production_mutation_evidence renderer=read_only controls=false credential_input=false"
