#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

REGISTRY="${NTPRO_BACKEND_FREEZE_REGISTRY:-docs/rust-cutover/governance/backend_freeze_registry.json}"
RELEASE_MANIFEST="${NTPRO_BACKEND_FREEZE_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_32_0_release_manifest.json}"
POLICY="${NTPRO_BACKEND_FREEZE_POLICY:-docs/rust-cutover/governance/backend_freeze_policy.md}"
README_PATH="${NTPRO_BACKEND_FREEZE_README:-README.md}"
ROADMAP_PATH="${NTPRO_BACKEND_FREEZE_ROADMAP:-docs/product/roadmap.md}"
VERSIONING_PATH="${NTPRO_BACKEND_FREEZE_VERSIONING:-docs/versioning.md}"
RUN_NEGATIVE_SELFTEST="${NTPRO_BACKEND_FREEZE_NEGATIVE_SELFTEST:-1}"

case "$RUN_NEGATIVE_SELFTEST" in
  0) negative_selftest=false ;;
  1) negative_selftest=true ;;
  *)
    echo "backend freeze baseline drift: NTPRO_BACKEND_FREEZE_NEGATIVE_SELFTEST must be 0 or 1" >&2
    exit 1
    ;;
esac

scripts/ai/ntpro_governance.sh backend-freeze \
  --registry "$REGISTRY" \
  --release-manifest "$RELEASE_MANIFEST" \
  --policy "$POLICY" \
  --readme "$README_PATH" \
  --roadmap "$ROADMAP_PATH" \
  --versioning "$VERSIONING_PATH" \
  --negative-selftest "$negative_selftest"
