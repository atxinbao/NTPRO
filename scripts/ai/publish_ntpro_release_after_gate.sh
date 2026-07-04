#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

REPO="${NTPRO_RELEASE_REPOSITORY:-${GITHUB_REPOSITORY:-atxinbao/NTPRO}}"
RELEASE_VERSION="${NTPRO_RELEASE_VERSION:-${NTPRO_CURRENT_RELEASE_VERSION:-v0.23.0}}"
TAG_NAME="${NTPRO_RELEASE_TAG:-${NTPRO_CURRENT_RELEASE_TAG:-ntpro-rust-only-${RELEASE_VERSION}}}"
RELEASE_NAME="${NTPRO_RELEASE_NAME:-${NTPRO_CURRENT_RELEASE_NAME:-NTPRO Rust-only ${RELEASE_VERSION}}}"
GH_BIN="${NTPRO_RELEASE_PUBLICATION_GH_BIN:-gh}"
GATE_RUN_ID="${NTPRO_RELEASE_GATE_RUN_ID:-}"
GATE_WORKFLOW_NAME="${NTPRO_RELEASE_GATE_WORKFLOW_NAME:-Rust Cutover Release Gate}"
DRY_RUN="${NTPRO_RELEASE_PUBLICATION_DRY_RUN:-0}"
TEST_MODE="${NTPRO_RELEASE_PUBLICATION_TEST_MODE:-0}"

release_stem="v${RELEASE_VERSION#v}"
release_stem="${release_stem//./_}"
RELEASE_NOTES="${NTPRO_RELEASE_NOTES:-${NTPRO_CURRENT_RELEASE_NOTES:-docs/rust-cutover/release/${release_stem}_release_notes.md}}"
EVIDENCE_PATH="${NTPRO_RELEASE_PUBLICATION_EVIDENCE_PATH:-release-publication-evidence/${TAG_NAME}.json}"

fail() {
  echo "release publication after gate failed: $*" >&2
  exit 1
}

require_value() {
  local value="$1"
  local name="$2"
  [[ -n "$value" ]] || fail "$name is required"
}

json_field() {
  python3 - "$1" "$2" <<'PY'
import json
import sys

payload = json.loads(sys.argv[1])
value = payload.get(sys.argv[2])
if value is None:
    value = ""
print(value)
PY
}

json_bool() {
  local value="$1"
  case "$value" in
    true|True) echo "true" ;;
    false|False) echo "false" ;;
    *) echo "$value" ;;
  esac
}

timestamp_ge() {
  python3 - "$1" "$2" <<'PY'
from datetime import datetime, timezone
import sys

def parse(value: str) -> datetime:
    if not value:
        raise SystemExit(2)
    value = value.replace("Z", "+00:00")
    parsed = datetime.fromisoformat(value)
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)

published = parse(sys.argv[1])
gate_completed = parse(sys.argv[2])
raise SystemExit(0 if published >= gate_completed else 1)
PY
}

normalize_file() {
  python3 - "$1" <<'PY'
from pathlib import Path
import sys

print("\n".join(line.rstrip() for line in Path(sys.argv[1]).read_text(encoding="utf-8").splitlines()).strip())
PY
}

normalize_text() {
  python3 - "$1" <<'PY'
import sys

print("\n".join(line.rstrip() for line in sys.argv[1].splitlines()).strip())
PY
}

require_body_matches_notes() {
  local body="$1"
  local expected_body
  local actual_body

  expected_body="$(normalize_file "$RELEASE_NOTES")"
  actual_body="$(normalize_text "$body")"
  [[ "$actual_body" == "$expected_body" ]] || fail "release body does not match release notes: $RELEASE_NOTES"
}

write_evidence() {
  local status="$1"
  local gate_url="$2"
  local gate_completed_at="$3"
  local release_url="$4"
  local published_at="$5"
  local tag_sha="$6"

  mkdir -p "$(dirname "$EVIDENCE_PATH")"
  python3 - "$EVIDENCE_PATH" \
    "$status" \
    "$REPO" \
    "$TAG_NAME" \
    "$RELEASE_VERSION" \
    "$RELEASE_NAME" \
    "$RELEASE_NOTES" \
    "$GATE_RUN_ID" \
    "$gate_url" \
    "$gate_completed_at" \
    "$release_url" \
    "$published_at" \
    "$tag_sha" <<'PY'
import json
from pathlib import Path
import sys

(
    evidence_path,
    status,
    repository,
    tag_name,
    release_version,
    release_name,
    release_notes,
    gate_run_id,
    gate_url,
    gate_completed_at,
    release_url,
    published_at,
    tag_sha,
) = sys.argv[1:]

payload = {
    "status": status,
    "repository": repository,
    "tag_name": tag_name,
    "release_version": release_version,
    "release_name": release_name,
    "release_notes": release_notes,
    "release_gate_run_id": gate_run_id,
    "release_gate_url": gate_url,
    "release_gate_completed_at": gate_completed_at,
    "release_url": release_url,
    "published_at": published_at,
    "tag_sha": tag_sha,
}

Path(evidence_path).write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
}

emit_evidence_policy() {
  echo "publication_evidence_strategy=source_tree_plus_github_remote"
  echo "local_evidence_path_is_generated_artifact=true"
  echo "local_evidence_path_required_in_source_tree=false"
  echo "remote_reconstruction_required=true"
}

require_value "$TAG_NAME" "NTPRO_RELEASE_TAG"
require_value "$GATE_RUN_ID" "NTPRO_RELEASE_GATE_RUN_ID"

[[ -f "$RELEASE_NOTES" ]] || fail "missing release notes: $RELEASE_NOTES"
command -v "$GH_BIN" >/dev/null 2>&1 || fail "gh command not found: $GH_BIN"
"$GH_BIN" auth status >/dev/null 2>&1 || fail "gh auth is required"

if [[ "$TEST_MODE" == "1" && -n "${NTPRO_RELEASE_PUBLICATION_TEST_TAG_SHA:-}" ]]; then
  tag_sha="$NTPRO_RELEASE_PUBLICATION_TEST_TAG_SHA"
else
  git rev-parse -q --verify "${TAG_NAME}^{commit}" >/dev/null || fail "missing local release tag: $TAG_NAME"
  tag_sha="$(git rev-list -n 1 "$TAG_NAME")"
fi

run_json="$("$GH_BIN" run view "$GATE_RUN_ID" --repo "$REPO" --json status,conclusion,workflowName,headSha,url,updatedAt)"
run_status="$(json_field "$run_json" status)"
run_conclusion="$(json_field "$run_json" conclusion)"
run_workflow_name="$(json_field "$run_json" workflowName)"
run_head_sha="$(json_field "$run_json" headSha)"
run_url="$(json_field "$run_json" url)"
run_completed_at="$(json_field "$run_json" updatedAt)"

[[ "$run_status" == "completed" ]] || fail "release gate run is not completed: $run_status"
[[ "$run_conclusion" == "success" ]] || fail "release gate run did not succeed: $run_conclusion"
[[ "$run_workflow_name" == "$GATE_WORKFLOW_NAME" ]] || fail "release gate workflow mismatch: $run_workflow_name"
[[ "$run_head_sha" == "$tag_sha" ]] || fail "release gate run headSha $run_head_sha does not match tag commit $tag_sha"
[[ -n "$run_completed_at" ]] || fail "release gate run updatedAt is empty"

echo "release_gate_run_id=$GATE_RUN_ID"
echo "release_gate_url=$run_url"
echo "release_gate_completed_at=$run_completed_at"
echo "release_tag=$TAG_NAME"
echo "release_tag_sha=$tag_sha"

release_json="$("$GH_BIN" release view "$TAG_NAME" --repo "$REPO" --json isDraft,url,body,publishedAt 2>/dev/null)" || release_json=""

if [[ -n "$release_json" ]]; then
  is_draft="$(json_bool "$(json_field "$release_json" isDraft)")"
  release_url="$(json_field "$release_json" url)"
  published_at="$(json_field "$release_json" publishedAt)"
  release_body="$(json_field "$release_json" body)"

  require_body_matches_notes "$release_body"

  if [[ "$is_draft" == "false" ]]; then
    [[ -n "$published_at" ]] || fail "published release has empty publishedAt"
    if ! timestamp_ge "$published_at" "$run_completed_at"; then
      fail "release was published before hosted gate success: published_at=$published_at gate_completed_at=$run_completed_at"
    fi
    write_evidence "already_published_after_gate" "$run_url" "$run_completed_at" "$release_url" "$published_at" "$tag_sha"
    echo "release_publication_after_gate=pass status=already_published_after_gate"
    echo "release_url=$release_url"
    echo "published_at=$published_at"
    echo "evidence_path=$EVIDENCE_PATH"
    emit_evidence_policy
    exit 0
  fi

  if [[ "$is_draft" != "true" ]]; then
    fail "unexpected release draft state: $is_draft"
  fi

  if [[ "$DRY_RUN" == "1" ]]; then
    write_evidence "dry_run_publish_draft_after_gate" "$run_url" "$run_completed_at" "$release_url" "" "$tag_sha"
    echo "release_publication_after_gate=pass status=dry_run_publish_draft_after_gate"
    echo "release_url=$release_url"
    echo "evidence_path=$EVIDENCE_PATH"
    emit_evidence_policy
    exit 0
  fi

  "$GH_BIN" release edit "$TAG_NAME" --repo "$REPO" --draft=false >/dev/null
else
  if [[ "$DRY_RUN" == "1" ]]; then
    write_evidence "dry_run_create_public_release_after_gate" "$run_url" "$run_completed_at" "" "" "$tag_sha"
    echo "release_publication_after_gate=pass status=dry_run_create_public_release_after_gate"
    echo "evidence_path=$EVIDENCE_PATH"
    emit_evidence_policy
    exit 0
  fi

  "$GH_BIN" release create "$TAG_NAME" \
    --repo "$REPO" \
    --verify-tag \
    --title "$RELEASE_NAME" \
    --notes-file "$RELEASE_NOTES" >/dev/null
fi

published_json="$("$GH_BIN" release view "$TAG_NAME" --repo "$REPO" --json isDraft,url,body,publishedAt)"
published_is_draft="$(json_bool "$(json_field "$published_json" isDraft)")"
published_url="$(json_field "$published_json" url)"
published_at="$(json_field "$published_json" publishedAt)"
published_body="$(json_field "$published_json" body)"

[[ "$published_is_draft" == "false" ]] || fail "release is still draft after publish"
[[ -n "$published_at" ]] || fail "publishedAt is empty after publish"
timestamp_ge "$published_at" "$run_completed_at" || fail "release publishedAt is earlier than gate completion"
require_body_matches_notes "$published_body"

write_evidence "published_after_gate" "$run_url" "$run_completed_at" "$published_url" "$published_at" "$tag_sha"
echo "release_publication_after_gate=pass status=published_after_gate"
echo "release_url=$published_url"
echo "published_at=$published_at"
echo "evidence_path=$EVIDENCE_PATH"
emit_evidence_policy
