#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CURRENT_RELEASE_VERSION="${NTPRO_CURRENT_RELEASE_VERSION:-v0.33.0}"
CURRENT_RELEASE_TAG="${NTPRO_CURRENT_RELEASE_TAG:-ntpro-rust-only-${CURRENT_RELEASE_VERSION}}"
POST_BASELINE_GOVERNANCE_TRACK="${NTPRO_POST_BASELINE_GOVERNANCE_TRACK:-backend-maintenance}"
NEXT_CAPABILITY_FAMILY="${NTPRO_NEXT_CAPABILITY_VERSION:-v0.34.0+}"
CURRENT_RELEASE_CAPABILITY="${NTPRO_CURRENT_RELEASE_CAPABILITY:-v0.33.0 Backend Maintenance}"

CURRENT_RELEASE_STEM="v${CURRENT_RELEASE_VERSION#v}"
CURRENT_RELEASE_STEM="${CURRENT_RELEASE_STEM//./_}"
CURRENT_RELEASE_NOTES="docs/rust-cutover/release/${CURRENT_RELEASE_STEM}_release_notes.md"
CURRENT_READINESS_REPORT="docs/rust-cutover/release/${CURRENT_RELEASE_STEM}_readiness_report.md"

echo "== release surface current guard =="
echo "current_release_version=$CURRENT_RELEASE_VERSION"
echo "current_release_tag=$CURRENT_RELEASE_TAG"
echo "current_release_capability=$CURRENT_RELEASE_CAPABILITY"
echo "backend_patch_scheduled=false"
echo "post_baseline_governance_track=$POST_BASELINE_GOVERNANCE_TRACK"
echo "next_capability_family=$NEXT_CAPABILITY_FAMILY"

args=(
  release-surface
  --current-version "$CURRENT_RELEASE_VERSION"
  --current-tag "$CURRENT_RELEASE_TAG"
  --governance-track "$POST_BASELINE_GOVERNANCE_TRACK"
  --next-capability "$NEXT_CAPABILITY_FAMILY"
  --current-capability "$CURRENT_RELEASE_CAPABILITY"
)
if [[ "${NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG:-0}" == "1" ]]; then
  args+=(--allow-missing-tag)
fi
scripts/ai/ntpro_governance.sh "${args[@]}"
