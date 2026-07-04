#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

FORMAL_TAG="${NTPRO_V231_STALE_FORMAL_TAG:-ntpro-rust-only-v0.23.0}"
CANDIDATE_TAG="${NTPRO_V231_STALE_CANDIDATE_TAG:-ntpro-rust-only-v0.23.0-candidate}"
FIXTURE_PATH="${NTPRO_V231_STALE_FIXTURE:-tests/golden/v230/dashboard_observability_snapshot.json}"
SMOKE_PATH="${NTPRO_V231_STALE_SMOKE:-scripts/ai/verify_v23_dashboard_observability_smoke.sh}"
STRICT_PATH="${NTPRO_V231_STALE_STRICT:-scripts/ai/verify_v23_strict_provenance.sh}"
RELEASE_NOTES_PATH="${NTPRO_V231_STALE_RELEASE_NOTES:-docs/rust-cutover/release/v0_23_0_release_notes.md}"
READINESS_PATH="${NTPRO_V231_STALE_READINESS:-docs/rust-cutover/release/v0_23_0_readiness_report.md}"
DASHBOARD_SURFACE_PATH="${NTPRO_V231_STALE_DASHBOARD_SURFACE:-docs/rust-cutover/release/v0_23_0_dashboard_observability_surface.md}"
V230_EVIDENCE_PATH="${NTPRO_V231_STALE_V230_EVIDENCE:-docs/rust-cutover/evidence/V230-006.md}"
V231_TASK_PATH="${NTPRO_V231_STALE_TASK:-docs/rust-cutover/tasks/V231-002.md}"
V231_EVIDENCE_PATH="${NTPRO_V231_STALE_EVIDENCE:-docs/rust-cutover/evidence/V231-002.md}"

fail() {
  echo "v23.1 stale provenance cleanup failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

require_contains() {
  local path="$1"
  local marker="$2"
  if ! grep -F -- "$marker" "$path" >/dev/null; then
    fail "missing marker in $path: $marker"
  fi
}

require_not_contains() {
  local path="$1"
  local marker="$2"
  if grep -F -- "$marker" "$path" >/dev/null; then
    fail "stale marker in $path: $marker"
  fi
}

for path in \
  "$FIXTURE_PATH" \
  "$SMOKE_PATH" \
  "$STRICT_PATH" \
  "$RELEASE_NOTES_PATH" \
  "$READINESS_PATH" \
  "$DASHBOARD_SURFACE_PATH" \
  "$V230_EVIDENCE_PATH" \
  "$V231_TASK_PATH" \
  "$V231_EVIDENCE_PATH"; do
  require_file "$path"
done

scan_files=(
  "$FIXTURE_PATH"
  "$RELEASE_NOTES_PATH"
  "$READINESS_PATH"
  "$DASHBOARD_SURFACE_PATH"
  "$V230_EVIDENCE_PATH"
)

stale_markers=(
  "$CANDIDATE_TAG"
  "#718 V230-007 = stays open until tag, hosted gate, public release, and publication evidence are recorded"
  "public release publication = pending"
  "tag gate run = pending"
  "tag gate result = pending"
  "RELEASE GATE CORRECTIVE FIX IN PROGRESS"
  "corrective fix in progress"
)

for path in "${scan_files[@]}"; do
  for marker in "${stale_markers[@]}"; do
    require_not_contains "$path" "$marker"
  done
done

for marker in \
  "$FORMAL_TAG" \
  "release_provenance = $FORMAL_TAG" \
  "release_publication = recorded_by_v0_23_0_release_closeout"; do
  require_contains "$DASHBOARD_SURFACE_PATH" "$marker"
done

for marker in \
  "$FORMAL_TAG" \
  "release_provenance = $FORMAL_TAG" \
  "release_publication = recorded_by_v0_23_0_release_closeout"; do
  require_contains "$V230_EVIDENCE_PATH" "$marker"
done

require_contains "$SMOKE_PATH" "expectedReleaseTag = \"$FORMAL_TAG\""
require_contains "$SMOKE_PATH" "staleReleaseTags = [\"$CANDIDATE_TAG\"]"
require_contains "$STRICT_PATH" "#718 V230-007 = closed after tag, hosted gate, public release, and publication evidence were recorded"
require_contains "$STRICT_PATH" "release notes contain stale marker"
require_contains "$STRICT_PATH" "readiness report contains stale marker"

FORMAL_TAG="$FORMAL_TAG" \
CANDIDATE_TAG="$CANDIDATE_TAG" \
FIXTURE_PATH="$FIXTURE_PATH" \
python3 <<'PY'
import json
import os
from pathlib import Path

formal = os.environ["FORMAL_TAG"]
candidate = os.environ["CANDIDATE_TAG"]
path = Path(os.environ["FIXTURE_PATH"])
data = json.loads(path.read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


raw = json.dumps(data, sort_keys=True)
require(candidate not in raw, f"fixture still contains stale candidate tag: {candidate}")
runtimes = data.get("read_model_runtime") or []
require(len(runtimes) == 2, f"expected two Dashboard observability rows, got {len(runtimes)}")
for index, runtime in enumerate(runtimes):
    provenance = runtime.get("audit_release_provenance") or {}
    require(provenance.get("availability") == "available", f"row {index} release provenance unavailable")
    require(provenance.get("value") == formal, f"row {index} release provenance is not {formal}: {provenance!r}")
PY

NTPRO_V23_DASHBOARD_OBSERVABILITY_SNAPSHOT="$FIXTURE_PATH" "$SMOKE_PATH" >/tmp/ntpro-v231-stale-dashboard-smoke.log

if [[ "${NTPRO_V231_STALE_SELFTEST:-1}" == "1" ]]; then
  tmp_fixture="$(mktemp "${TMPDIR:-/tmp}/ntpro-v231-stale-fixture.XXXXXX.json")"
  cp "$FIXTURE_PATH" "$tmp_fixture"
  FORMAL_TAG="$FORMAL_TAG" CANDIDATE_TAG="$CANDIDATE_TAG" TMP_FIXTURE="$tmp_fixture" python3 <<'PY'
import os
from pathlib import Path

path = Path(os.environ["TMP_FIXTURE"])
path.write_text(path.read_text(encoding="utf-8").replace(os.environ["FORMAL_TAG"], os.environ["CANDIDATE_TAG"]), encoding="utf-8")
PY
  if NTPRO_V231_STALE_SELFTEST=0 NTPRO_V231_STALE_FIXTURE="$tmp_fixture" "$0" >/tmp/ntpro-v231-stale-negative.log 2>&1; then
    rm -f "$tmp_fixture"
    fail "negative candidate provenance self-test unexpectedly passed"
  fi
  rm -f "$tmp_fixture"
fi

echo "v23_1_stale_provenance_cleanup status=ok release_tag=$FORMAL_TAG fixture=$FIXTURE_PATH stale_candidate_selftest=${NTPRO_V231_STALE_SELFTEST:-1}"
