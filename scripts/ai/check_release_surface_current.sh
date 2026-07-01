#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CURRENT_RELEASE_VERSION="${NTPRO_CURRENT_RELEASE_VERSION:-v0.21.0}"
CURRENT_RELEASE_TAG="${NTPRO_CURRENT_RELEASE_TAG:-ntpro-rust-only-${CURRENT_RELEASE_VERSION}}"
NEXT_PATCH_VERSION="${NTPRO_NEXT_PATCH_VERSION:-v0.21.1}"
NEXT_CAPABILITY_VERSION="${NTPRO_NEXT_CAPABILITY_VERSION:-v0.22.0}"
CURRENT_RELEASE_CAPABILITY="${NTPRO_CURRENT_RELEASE_CAPABILITY:-Unified Read Model Foundation}"

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

reject_stale_current_release_wording() {
  python3 - "$CURRENT_RELEASE_VERSION" "$CURRENT_RELEASE_TAG" "$@" <<'PY'
import re
import sys
from pathlib import Path

current_version = sys.argv[1]
current_tag = sys.argv[2]
files = [Path(p) for p in sys.argv[3:]]

def parse_version(value: str) -> tuple[int, int, int]:
    match = re.search(r"v(\d+)\.(\d+)\.(\d+)", value)
    if not match:
        raise SystemExit(f"invalid release version: {value}")
    return tuple(int(part) for part in match.groups())

current_tuple = parse_version(current_version)
release_value = re.compile(r"(?:ntpro-rust-only-)?v(\d+)\.(\d+)\.(\d+)")
context_line = re.compile(
    r"current|当前|当前正式公开发布点|current public|current source|current published",
    re.IGNORECASE,
)
multiline_context = re.compile(
    r"current public milestone is|current published release line is|current release line is",
    re.IGNORECASE,
)

errors: list[str] = []
for path in files:
    pending_context = 0
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line_has_context = bool(context_line.search(line))
        if multiline_context.search(line):
            pending_context = 4
        should_scan = line_has_context or pending_context > 0
        if should_scan:
            for match in release_value.finditer(line):
                value = tuple(int(part) for part in match.groups())
                release_text = match.group(0)
                if value < current_tuple and release_text != current_tag:
                    errors.append(f"{path}:{number}: stale current release wording -> {line}")
        if pending_context > 0:
            pending_context -= 1

if errors:
    print("\n".join(errors), file=sys.stderr)
    raise SystemExit(1)
PY
}

echo "== release surface current guard =="
echo "current_release_version=$CURRENT_RELEASE_VERSION"
echo "current_release_tag=$CURRENT_RELEASE_TAG"
echo "current_release_capability=$CURRENT_RELEASE_CAPABILITY"
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
  "The next capability track is \`$NEXT_CAPABILITY_VERSION\`" \
  "README next capability track"

require_contains ROADMAP.md \
  "\`$CURRENT_RELEASE_TAG\`, the $CURRENT_RELEASE_CAPABILITY release" \
  "ROADMAP current release and patch track"
require_contains ROADMAP.md \
  "The next patch track is \`$NEXT_PATCH_VERSION\`" \
  "ROADMAP next patch track"
require_contains ROADMAP.md \
  "## Published Capability Track: $CURRENT_RELEASE_VERSION" \
  "ROADMAP published capability track"
require_contains ROADMAP.md \
  "The next capability track is \`$NEXT_CAPABILITY_VERSION\`" \
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
  "\`$(basename "$CURRENT_READINESS_REPORT")\` - released readiness report for the formal" \
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
if ! grep -F -- "Status: PASS" "$CURRENT_READINESS_REPORT" >/dev/null \
  && ! grep -F -- "Status: RELEASED" "$CURRENT_READINESS_REPORT" >/dev/null; then
  echo "expected: readiness report PASS or RELEASED status" >&2
  echo "file: $CURRENT_READINESS_REPORT" >&2
  fail "missing current readiness report release status"
fi

reject_stale_current_release_wording README.md ROADMAP.md docs/versioning.md

echo "release_surface_current_guard=pass"
