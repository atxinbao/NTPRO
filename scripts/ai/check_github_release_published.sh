#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

REPO="${NTPRO_RELEASE_REPOSITORY:-${GITHUB_REPOSITORY:-atxinbao/NTPRO}}"
CURRENT_RELEASE_VERSION="${NTPRO_CURRENT_RELEASE_VERSION:-v0.33.0}"
CURRENT_RELEASE_TAG="${NTPRO_CURRENT_RELEASE_TAG:-ntpro-rust-only-v0.33.0}"
RELEASE_NAME="${NTPRO_CURRENT_RELEASE_NAME:-NTPRO Rust-only v0.33.0}"
RELEASE_URL="${NTPRO_CURRENT_RELEASE_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.33.0}"
CURRENT_RELEASE_NOTES="${NTPRO_CURRENT_RELEASE_NOTES:-docs/rust-cutover/release/v0_33_0_release_notes.md}"
MANIFEST="${NTPRO_CURRENT_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_33_0_release_manifest.json}"
GH_BIN="${NTPRO_RELEASE_PUBLICATION_GH_BIN:-gh}"
PREPUBLISH_TAG_GATE="${NTPRO_RELEASE_PUBLICATION_PREPUBLISH_TAG_GATE:-0}"

fail() {
  echo "release publication drift: $*" >&2
  exit 1
}

offline_skip() {
  local reason="$1"
  if [[ "${NTPRO_RELEASE_PUBLICATION_ALLOW_OFFLINE:-0}" == "1" ]]; then
    echo "release_publication_guard=offline_skip reason=$reason"
    exit 0
  fi
  fail "$reason"
}

json_field() {
  jq -r --arg key "$2" \
    'if has($key) and .[$key] != null then .[$key] else "" end | if type == "boolean" then tostring else . end' \
    <<<"$1"
}

select_successful_tag_gate() {
  local runs_json="$1"
  local tag_sha="$2"
  local tag_name="$3"
  jq -c --arg sha "$tag_sha" --arg tag "$tag_name" '
    [.[] | select(
      .headSha == $sha
      and .status == "completed"
      and .conclusion == "success"
      and .event == "push"
      and .headBranch == $tag
    )] | sort_by(.databaseId) | last // empty
  ' <<<"$runs_json"
}

run_gate_selection_selftest() {
  local sha="0123456789abcdef0123456789abcdef01234567"
  local tag="ntpro-rust-only-v0.33.0"
  local valid
  valid='[{"databaseId":4,"status":"completed","conclusion":"success","event":"push","headBranch":"ntpro-rust-only-v0.33.0","headSha":"0123456789abcdef0123456789abcdef01234567"}]'
  [[ "$(select_successful_tag_gate "$valid" "$sha" "$tag" | jq -r '.databaseId')" == "4" ]] \
    || fail "gate selection selftest rejected valid tag push"

  local mutation
  for mutation in \
    '.[0].event = "workflow_dispatch"' \
    '.[0].headBranch = "main"' \
    '.[0].headSha = "ffffffffffffffffffffffffffffffffffffffff"'; do
    [[ -z "$(select_successful_tag_gate "$(jq "$mutation" <<<"$valid")" "$sha" "$tag")" ]] \
      || fail "gate selection selftest accepted invalid run: $mutation"
  done
  echo "release_publication_gate_selection_selftest=pass cases=4"
}

[[ "$CURRENT_RELEASE_VERSION" == "v0.33.0" ]] || fail "current maintenance version mismatch"
[[ "$CURRENT_RELEASE_TAG" == "ntpro-rust-only-v0.33.0" ]] || fail "current tag mismatch"
[[ "$RELEASE_NAME" == "NTPRO Rust-only v0.33.0" ]] || fail "current release name mismatch"
[[ "$RELEASE_URL" == "https://github.com/atxinbao/NTPRO/releases/tag/$CURRENT_RELEASE_TAG" ]] \
  || fail "current release URL mismatch"
[[ -f "$CURRENT_RELEASE_NOTES" ]] || fail "missing release notes: $CURRENT_RELEASE_NOTES"
[[ -f "$MANIFEST" ]] || fail "missing release manifest: $MANIFEST"
command -v jq >/dev/null 2>&1 || fail "jq is required"
run_gate_selection_selftest

jq -e \
  --arg version "$CURRENT_RELEASE_VERSION" \
  --arg tag "$CURRENT_RELEASE_TAG" \
  --arg name "$RELEASE_NAME" \
  --arg url "$RELEASE_URL" '
    .product_version == $version
    and .release_scope_name == "backend_maintenance"
    and .planned_release.tag == $tag
    and .planned_release.name == $name
    and .planned_release.github_release_url == $url
    and .publication_governance.gate_before_publish == true
    and .publication_governance.public_release_requires_successful_hosted_gate_for_same_tag_commit == true
    and .publication_governance.publication_evidence_strategy == "source_tree_plus_github_remote"
    and (.boundary_flags | length == 27)
    and (.boundary_flags | all(. == false))
  ' "$MANIFEST" >/dev/null || fail "maintenance release manifest contract mismatch"

scripts/ai/check_release_surface_current.sh
scripts/ai/check_backend_freeze_baseline.sh
scripts/ai/check_backend_maintenance_release.sh

if ! command -v "$GH_BIN" >/dev/null 2>&1; then
  offline_skip "gh_unavailable"
fi
if ! "$GH_BIN" auth status >/dev/null 2>&1; then
  offline_skip "gh_auth_unavailable"
fi
if ! git rev-parse -q --verify "${CURRENT_RELEASE_TAG}^{commit}" >/dev/null; then
  offline_skip "missing_local_git_tag:$CURRENT_RELEASE_TAG"
fi

tag_sha="$(git rev-list -n 1 "$CURRENT_RELEASE_TAG")"
release_json="$("$GH_BIN" release view "$CURRENT_RELEASE_TAG" \
  --json tagName,name,isDraft,isPrerelease,url,body,publishedAt,targetCommitish 2>/dev/null || true)"
if [[ -z "$release_json" ]]; then
  if [[ "$PREPUBLISH_TAG_GATE" == "1" ]]; then
    echo "release_publication_guard=pass mode=prepublish_release_absent tag=$CURRENT_RELEASE_TAG"
    echo "release_tag_sha=$tag_sha"
    exit 0
  fi
  offline_skip "github_release_unavailable"
fi

tag_name="$(json_field "$release_json" tagName)"
name="$(json_field "$release_json" name)"
is_draft="$(json_field "$release_json" isDraft)"
is_prerelease="$(json_field "$release_json" isPrerelease)"
url="$(json_field "$release_json" url)"
published_at="$(json_field "$release_json" publishedAt)"

[[ "$tag_name" == "$CURRENT_RELEASE_TAG" ]] || fail "release tag mismatch: $tag_name"
[[ "$name" == "$RELEASE_NAME" ]] || fail "release name mismatch: $name"
[[ "$is_draft" == "false" ]] || fail "release is draft"
[[ "$is_prerelease" == "false" ]] || fail "release is prerelease"
[[ "$url" == "$RELEASE_URL" ]] || fail "release URL mismatch: $url"
[[ -n "$published_at" ]] || fail "release publishedAt is empty"

for marker in \
  "Status: RELEASE GATE READY" \
  "Tag: \`$CURRENT_RELEASE_TAG\`" \
  "Release name: \`$RELEASE_NAME\`" \
  "Release URL: \`$RELEASE_URL\`"; do
  grep -F -- "$marker" "$CURRENT_RELEASE_NOTES" >/dev/null \
    || fail "release notes missing marker: $marker"
done

body_hash_report="$(printf '%s' "$release_json" \
  | scripts/ai/ntpro_governance.sh release-body-hash --notes "$CURRENT_RELEASE_NOTES")"
grep -F 'release_body_normalized_sha256_matches_tracked_release_notes=true' \
  <<<"$body_hash_report" >/dev/null || fail "release body does not match tracked notes"

runs_json="$("$GH_BIN" run list \
  --repo "$REPO" \
  --workflow release-tag.yml \
  --limit 100 \
  --json databaseId,status,conclusion,event,headBranch,headSha,updatedAt,url)"
gate_json="$(select_successful_tag_gate "$runs_json" "$tag_sha" "$CURRENT_RELEASE_TAG")"
[[ -n "$gate_json" ]] || fail "missing successful hosted release gate for tag commit"
gate_run_id="$(json_field "$gate_json" databaseId)"
gate_completed_at="$(json_field "$gate_json" updatedAt)"
scripts/ai/ntpro_governance.sh timestamp-ge \
  --left "$published_at" \
  --right "$gate_completed_at" >/dev/null \
  || fail "GitHub Release was published before hosted gate success"

printf '%s\n' "$body_hash_report"
echo "release_publication_guard=pass"
echo "release_version=$CURRENT_RELEASE_VERSION"
echo "release_tag=$CURRENT_RELEASE_TAG"
echo "release_tag_sha=$tag_sha"
echo "release_gate_run_id=$gate_run_id"
echo "release_gate_completed_at=$gate_completed_at"
echo "published_at=$published_at"
