#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CURRENT_RELEASE_VERSION="${NTPRO_CURRENT_RELEASE_VERSION:-v0.19.0}"
CURRENT_RELEASE_TAG="${NTPRO_CURRENT_RELEASE_TAG:-ntpro-rust-only-${CURRENT_RELEASE_VERSION}}"
RELEASE_NAME="${NTPRO_CURRENT_RELEASE_NAME:-NTPRO Rust-only ${CURRENT_RELEASE_VERSION}}"
RELEASE_URL="${NTPRO_CURRENT_RELEASE_URL:-https://github.com/atxinbao/NTPRO/releases/tag/${CURRENT_RELEASE_TAG}}"
GH_BIN="${NTPRO_RELEASE_PUBLICATION_GH_BIN:-gh}"

CURRENT_RELEASE_STEM="v${CURRENT_RELEASE_VERSION#v}"
CURRENT_RELEASE_STEM="${CURRENT_RELEASE_STEM//./_}"
CURRENT_RELEASE_NOTES="${NTPRO_CURRENT_RELEASE_NOTES:-docs/rust-cutover/release/${CURRENT_RELEASE_STEM}_release_notes.md}"

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

require_file() {
  local file="$1"
  [[ -f "$file" ]] || fail "missing required file: $file"
}

require_contains_text() {
  local haystack="$1"
  local needle="$2"
  local description="$3"
  if [[ "$haystack" != *"$needle"* ]]; then
    echo "expected: $description" >&2
    echo "needle: $needle" >&2
    fail "missing release body key field"
  fi
}

require_file_contains() {
  local file="$1"
  local needle="$2"
  local description="$3"
  if ! grep -F -- "$needle" "$file" >/dev/null; then
    echo "expected: $description" >&2
    echo "file: $file" >&2
    echo "needle: $needle" >&2
    fail "missing release notes key field"
  fi
}

ensure_origin_main_ref() {
  if git rev-parse -q --verify origin/main^{commit} >/dev/null; then
    return 0
  fi

  if git remote get-url origin >/dev/null 2>&1; then
    git fetch --no-tags --depth=1 origin +refs/heads/main:refs/remotes/origin/main >/dev/null 2>&1 || true
  fi
}

extract_json_field() {
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

echo "== GitHub release publication guard =="
echo "current_release_version=$CURRENT_RELEASE_VERSION"
echo "current_release_tag=$CURRENT_RELEASE_TAG"
echo "release_name=$RELEASE_NAME"
echo "release_url=$RELEASE_URL"
echo "release_notes=$CURRENT_RELEASE_NOTES"

require_file "$CURRENT_RELEASE_NOTES"

if ! command -v "$GH_BIN" >/dev/null 2>&1; then
  offline_skip "gh_unavailable"
fi

if ! "$GH_BIN" auth status >/dev/null 2>&1; then
  offline_skip "gh_auth_unavailable"
fi

if ! git rev-parse -q --verify "${CURRENT_RELEASE_TAG}^{commit}" >/dev/null; then
  fail "missing local git tag: $CURRENT_RELEASE_TAG"
fi

ensure_origin_main_ref

if ! git rev-parse -q --verify origin/main^{commit} >/dev/null; then
  fail "missing local origin/main ref"
fi

tag_sha="$(git rev-list -n 1 "$CURRENT_RELEASE_TAG")"
origin_main_sha="$(git rev-parse origin/main)"

if ! git merge-base --is-ancestor "$tag_sha" "$origin_main_sha"; then
  fail "release tag $CURRENT_RELEASE_TAG is not reachable from origin/main"
fi

release_json="$("$GH_BIN" release view "$CURRENT_RELEASE_TAG" --json tagName,name,isDraft,isPrerelease,url,body,publishedAt,targetCommitish 2>/dev/null)" \
  || offline_skip "github_release_unavailable"

tag_name="$(extract_json_field "$release_json" tagName)"
name="$(extract_json_field "$release_json" name)"
is_draft="$(extract_json_field "$release_json" isDraft)"
is_prerelease="$(extract_json_field "$release_json" isPrerelease)"
url="$(extract_json_field "$release_json" url)"
published_at="$(extract_json_field "$release_json" publishedAt)"
target_commitish="$(extract_json_field "$release_json" targetCommitish)"
body="$(extract_json_field "$release_json" body)"

[[ "$tag_name" == "$CURRENT_RELEASE_TAG" ]] || fail "release tag mismatch: $tag_name"
[[ "$name" == "$RELEASE_NAME" ]] || fail "release name mismatch: $name"
[[ "$is_draft" == "False" || "$is_draft" == "false" ]] || fail "release is draft"
[[ "$is_prerelease" == "False" || "$is_prerelease" == "false" ]] || fail "release is prerelease"
[[ "$url" == "$RELEASE_URL" ]] || fail "release URL mismatch: $url"
[[ -n "$published_at" ]] || fail "release publishedAt is empty"

case "$CURRENT_RELEASE_VERSION" in
  v0.14.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Production Order-State Read-Only + Live Alpha Dry-Run"
      "no-production-mutation"
      "owner-gated production order-state GET proof"
      "live-alpha dry-run"
      "production order submission"
      "production cancel, replace, amend, retry, correction, or flatten"
      "listenKey lifecycle"
      "real funds"
      "production trading"
      "Dashboard order/cancel/replace/amend/retry/reconnect controls"
    )
    ;;
  v0.15.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Guarded Live Alpha Mutation Scope + Execution Dry-Run Harness"
      "production mutation endpoint classification"
      "redacted local order request preview"
      "manual approval lifecycle for preview artifact creation"
      "kill switch runtime gate"
      "local dry-run execution adapter artifact"
      "dry-run mutation golden trace replay"
      "read-only Dashboard mutation preflight panel"
      "production request sending"
      "production order submission"
      "production order mutation"
      "production cancel, replace, amend, retry, correction, or flatten"
      "listenKey lifecycle"
      "real funds"
      "production trading"
      "Dashboard order/cancel/replace/amend/retry/reconnect controls"
    )
    ;;
  v0.16.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Minimum Owner-Approved Production Order Mutation Candidate"
      "owner-approved runtime gates"
      "production signing-material approval evidence"
      "single \`LIMIT\` \`GTC\` request builder"
      "guarded production HTTP send path"
      "production mutation response redaction"
      "post-submit order-state readback proof contract"
      "kill-switch checks around the send boundary"
      "production mutation audit trail"
      "failure-mode and no-retry semantics"
      "read-only Dashboard production mutation evidence panel"
      "strategy-driven production execution"
      "multiple orders"
      "MARKET orders"
      "cancel, replace, amend, retry, correction, flatten, or remediation"
      "Dashboard order controls"
      "listenKey lifecycle"
      "real-funds proof in CI"
      "production trading platform claim"
    )
    ;;
  v0.17.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Production Reconciliation And Orphan Recovery Evidence"
      "capability_expansion_from_v16 = reconciliation_evidence_only"
      "lineage_scope = single_v16_mutation_candidate"
      "local ledger"
      "redacted readback mapper"
      "reconciliation classifier"
      "orphan order detector"
      "restart recovery evidence"
      "failure incident semantics"
      "read-only Dashboard evidence"
      "network readback execution = not included"
      "production order submission = not included"
      "production order mutation = not included"
      "actual cancel send = deferred"
      "automatic cancel = disabled"
      "Dashboard order controls = disabled"
      "Dashboard cancel controls = disabled"
      "retry_attempted = false"
      "cancel_attempted = false"
      "remediation_attempted = false"
      "strategy-driven production execution"
      "multi-account production execution"
      "multi-venue production execution"
      "real-funds proof in CI"
      "general production trading platform claim"
    )
    ;;
  v0.18.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Owner-Approved Cancel Recovery Preview"
      "capability_expansion = preview_gate_approval_only"
      "lineage_scope = single_v16_mutation_candidate"
      "cancel request preview"
      "cancel risk gate"
      "manual owner approval lifecycle"
      "cancel response redaction contract"
      "post-cancel readback contract"
      "incident/audit closeout contract"
      "read-only Dashboard evidence"
      "aggregate v0.18 release gate"
      "actual cancel send = not included"
      "automatic cancel = disabled"
      "automatic remediation = disabled"
      "Dashboard order controls = disabled"
      "Dashboard cancel controls = disabled"
      "retry_attempted = false"
      "cancel_attempted = false"
      "remediation_attempted = false"
      "DELETE /api/v3/order"
      "DELETE /api/v3/openOrders"
      "strategy-driven cancel"
      "cancel all open orders"
      "bulk cancel"
      "multi-account cancel recovery"
      "multi-venue cancel recovery"
      "production trading claim"
    )
    ;;
  v0.19.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Owner-Approved Single-Shot Actual Cancel"
      "actual cancel only owner-approved single-shot"
      "owner approval = required"
      "single order = required"
      "single venue = required"
      "single execution attempt = required"
      "post-cancel readback = required"
      "failure evidence = required"
      "Dashboard cancel button = not included"
      "production order submit lifecycle = not included"
      "automatic cancel = not included"
      "bulk cancel = not included"
      "retry / replace / amend / flatten = not included"
    )
    ;;
  *)
    fail "unsupported release publication guard version: $CURRENT_RELEASE_VERSION"
    ;;
esac

for field in "${required_fields[@]}"; do
  require_file_contains "$CURRENT_RELEASE_NOTES" "$field" "release notes key field"
  require_contains_text "$body" "$field" "GitHub Release body key field"
done

if [[ "${NTPRO_RELEASE_PUBLICATION_STRICT_BODY:-0}" == "1" ]]; then
  normalized_notes="$(python3 - "$CURRENT_RELEASE_NOTES" <<'PY'
from pathlib import Path
import sys

print("\n".join(line.rstrip() for line in Path(sys.argv[1]).read_text(encoding="utf-8").splitlines()).strip())
PY
)"
  normalized_body="$(python3 - <<'PY' "$body"
import sys

print("\n".join(line.rstrip() for line in sys.argv[1].splitlines()).strip())
PY
)"
  [[ "$normalized_body" == "$normalized_notes" ]] || fail "release body does not strictly match release notes"
fi

echo "release_publication_guard=pass"
echo "tag_sha=$tag_sha"
echo "origin_main_sha=$origin_main_sha"
echo "target_commitish=$target_commitish"
echo "published_at=$published_at"
