#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CURRENT_RELEASE_VERSION="${NTPRO_CURRENT_RELEASE_VERSION:-v0.32.0}"
CURRENT_RELEASE_TAG="${NTPRO_CURRENT_RELEASE_TAG:-ntpro-rust-only-v0.32.0}"
RELEASE_NAME="${NTPRO_CURRENT_RELEASE_NAME:-NTPRO Rust-only v0.32.0}"
RELEASE_URL="${NTPRO_CURRENT_RELEASE_URL:-https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.32.0}"
CURRENT_RELEASE_NOTES="${NTPRO_CURRENT_RELEASE_NOTES:-docs/rust-cutover/release/v0_32_0_release_notes.md}"
REGISTRY="${NTPRO_BACKEND_FREEZE_REGISTRY:-docs/rust-cutover/governance/backend_freeze_registry.json}"
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

[[ "$CURRENT_RELEASE_VERSION" == "v0.32.0" ]] \
  || fail "only the frozen v0.32.0 backend baseline is current"
[[ "$CURRENT_RELEASE_TAG" == "ntpro-rust-only-v0.32.0" ]] || fail "current tag mismatch"
[[ "$RELEASE_NAME" == "NTPRO Rust-only v0.32.0" ]] || fail "current release name mismatch"
[[ "$RELEASE_URL" == "https://github.com/atxinbao/NTPRO/releases/tag/$CURRENT_RELEASE_TAG" ]] \
  || fail "current release URL mismatch"
[[ -f "$CURRENT_RELEASE_NOTES" ]] || fail "missing release notes: $CURRENT_RELEASE_NOTES"
[[ -f "$REGISTRY" ]] || fail "missing backend freeze registry: $REGISTRY"
command -v jq >/dev/null 2>&1 || fail "jq is required"

scripts/ai/check_release_surface_current.sh
scripts/ai/check_backend_freeze_baseline.sh

expected_sha="$(jq -er '.baseline.tag.peeled_commit_sha' "$REGISTRY")"
expected_published_at="$(jq -er '.github_release.published_at' "$REGISTRY")"
expected_gate_run_id="$(jq -er '.hosted_release_gate.run_id' "$REGISTRY")"
expected_gate_status="$(jq -er '.hosted_release_gate.status' "$REGISTRY")"
expected_gate_conclusion="$(jq -er '.hosted_release_gate.conclusion' "$REGISTRY")"
[[ "$expected_gate_status" == "completed" && "$expected_gate_conclusion" == "success" ]] \
  || fail "frozen hosted gate is not successful"

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
[[ "$tag_sha" == "$expected_sha" ]] || fail "tag SHA mismatch: $tag_sha"

release_json="$($GH_BIN release view "$CURRENT_RELEASE_TAG" \
  --json tagName,name,isDraft,isPrerelease,url,body,publishedAt,targetCommitish 2>/dev/null || true)"
if [[ -z "$release_json" ]]; then
  if [[ "$PREPUBLISH_TAG_GATE" == "1" ]]; then
    echo "release_publication_guard=pass mode=prepublish_release_absent tag=$CURRENT_RELEASE_TAG"
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
[[ "$published_at" == "$expected_published_at" ]] || fail "publishedAt mismatch: $published_at"

for marker in \
  "Status: RELEASED" \
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

printf '%s\n' "$body_hash_report"
echo "release_publication_guard=pass"
echo "release_version=$CURRENT_RELEASE_VERSION"
echo "release_tag=$CURRENT_RELEASE_TAG"
echo "release_tag_sha=$tag_sha"
echo "release_gate_run_id=$expected_gate_run_id"
echo "published_at=$published_at"
