#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CURRENT_RELEASE_VERSION="${NTPRO_CURRENT_RELEASE_VERSION:-v0.11.0}"
CURRENT_RELEASE_TAG="${NTPRO_CURRENT_RELEASE_TAG:-ntpro-rust-only-${CURRENT_RELEASE_VERSION}}"
NEXT_PATCH_VERSION="${NTPRO_NEXT_PATCH_VERSION:-v0.11.1}"
NEXT_CAPABILITY_VERSION="${NTPRO_NEXT_CAPABILITY_VERSION:-v0.12.0}"

CURRENT_MINOR_LINE="${CURRENT_RELEASE_VERSION%.*}.x"
CURRENT_RELEASE_STEM="v${CURRENT_RELEASE_VERSION#v}"
CURRENT_RELEASE_STEM="${CURRENT_RELEASE_STEM//./_}"
CURRENT_RELEASE_NOTES="docs/rust-cutover/release/${CURRENT_RELEASE_STEM}_release_notes.md"
CURRENT_READINESS_REPORT="docs/rust-cutover/release/${CURRENT_RELEASE_STEM}_readiness_report.md"

fail() {
  echo "release surface drift: $*" >&2
  exit 1
}

require_file() {
  local file="$1"
  [[ -f "$file" ]] || fail "missing required file: $file"
}

require_contains() {
  local file="$1"
  local needle="$2"
  local description="$3"
  if ! grep -F -- "$needle" "$file" >/dev/null; then
    echo "expected: $description" >&2
    echo "file: $file" >&2
    echo "needle: $needle" >&2
    fail "missing current release surface wording"
  fi
}

reject_current_old_release_wording() {
  local file="$1"
  if grep -Ein \
    'current (public |source |published |formal |release |milestone )*(release|source|tag|milestone).*ntpro-rust-only-v0\.[0-9]\.' \
    "$file" >/tmp/ntpro-release-surface-old-current.txt; then
    cat /tmp/ntpro-release-surface-old-current.txt >&2
    fail "current-release wording points to a pre-v0.10 tag in $file"
  fi
}

echo "== release surface current guard =="
echo "current_release_version=$CURRENT_RELEASE_VERSION"
echo "current_release_tag=$CURRENT_RELEASE_TAG"
echo "next_patch_version=$NEXT_PATCH_VERSION"
echo "next_capability_version=$NEXT_CAPABILITY_VERSION"

require_file README.md
require_file ROADMAP.md
require_file docs/versioning.md
require_file docs/rust-cutover/release/README.md
require_file "$CURRENT_RELEASE_NOTES"
require_file "$CURRENT_READINESS_REPORT"

if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  if ! git rev-parse -q --verify "${CURRENT_RELEASE_TAG}^{commit}" >/dev/null; then
    if [[ "${NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG:-0}" == "1" ]]; then
      echo "release_surface_current_guard=pre_tag_mode missing_tag=$CURRENT_RELEASE_TAG"
    else
      fail "missing local git tag: $CURRENT_RELEASE_TAG"
    fi
  fi
fi

require_contains README.md \
  "Current source tag: $CURRENT_RELEASE_TAG" \
  "README current source tag"
require_contains README.md \
  "https://github.com/atxinbao/NTPRO/releases/tag/$CURRENT_RELEASE_TAG" \
  "README current GitHub Release URL"
require_contains README.md \
  "The next patch track is \`$NEXT_PATCH_VERSION\`" \
  "README next patch track"
require_contains README.md \
  "The next capability track after the ${CURRENT_MINOR_LINE} line is \`$NEXT_CAPABILITY_VERSION\`" \
  "README next capability track"

require_contains ROADMAP.md \
  "\`$CURRENT_RELEASE_TAG\`, the next patch track is \`$NEXT_PATCH_VERSION\`" \
  "ROADMAP current release and patch track"
require_contains ROADMAP.md \
  "## Published Capability Track: $CURRENT_RELEASE_VERSION" \
  "ROADMAP published capability track"
require_contains ROADMAP.md \
  "\`$NEXT_CAPABILITY_VERSION\` is the next capability track" \
  "ROADMAP next capability track"

require_contains docs/versioning.md \
  "\`$CURRENT_RELEASE_VERSION\` 是当前正式公开发布点" \
  "versioning current release statement"
require_contains docs/versioning.md \
  "$CURRENT_RELEASE_TAG" \
  "versioning current release tag"
require_contains docs/versioning.md \
  "$NEXT_PATCH_VERSION" \
  "versioning next patch track"
require_contains docs/versioning.md \
  "$NEXT_CAPABILITY_VERSION" \
  "versioning next capability track"

require_contains docs/rust-cutover/release/README.md \
  "\`$(basename "$CURRENT_READINESS_REPORT")\` - released readiness report for the $CURRENT_RELEASE_VERSION" \
  "release index current readiness report"
require_contains docs/rust-cutover/release/README.md \
  "\`$(basename "$CURRENT_RELEASE_NOTES")\` - release notes for the formal" \
  "release index current release notes"
require_contains docs/rust-cutover/release/README.md \
  "\`$CURRENT_RELEASE_TAG\` GitHub Release" \
  "release index current tag"

require_contains "$CURRENT_RELEASE_NOTES" \
  "Status: RELEASED" \
  "release notes released status"
require_contains "$CURRENT_RELEASE_NOTES" \
  "Tag: \`$CURRENT_RELEASE_TAG\`" \
  "release notes tag"
require_contains "$CURRENT_RELEASE_NOTES" \
  "Release name: \`NTPRO Rust-only $CURRENT_RELEASE_VERSION\`" \
  "release notes release name"

require_contains "$CURRENT_READINESS_REPORT" \
  "Milestone: \`$CURRENT_RELEASE_TAG\`" \
  "readiness report milestone"
require_contains "$CURRENT_READINESS_REPORT" \
  "Status: PASS" \
  "readiness report PASS status"

reject_current_old_release_wording README.md
reject_current_old_release_wording ROADMAP.md
reject_current_old_release_wording docs/versioning.md

echo "release_surface_current_guard=pass"
