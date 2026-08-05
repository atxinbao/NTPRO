#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

MANIFEST="${NTPRO_V33_RELEASE_MANIFEST:-docs/rust-cutover/release/v0_33_0_release_manifest.json}"
REPO="${NTPRO_RELEASE_REPOSITORY:-${GITHUB_REPOSITORY:-atxinbao/NTPRO}}"
ALLOW_OPEN_CLOSEOUT="${NTPRO_V33_ALLOW_OPEN_CLOSEOUT:-${NTPRO_RELEASE_SURFACE_ALLOW_MISSING_TAG:-0}}"
ALLOW_OFFLINE="${NTPRO_RELEASE_PUBLICATION_ALLOW_OFFLINE:-0}"
ALLOW_POST_RELEASE_HEAD="${NTPRO_V33_ALLOW_POST_RELEASE_HEAD:-0}"
POST_RELEASE_HEAD_SHA="${NTPRO_V33_POST_RELEASE_HEAD_SHA:-}"
TAG="ntpro-rust-only-v0.33.0"

fail() {
  echo "current backend maintenance release validation failed: $*" >&2
  exit 1
}

release_compare_payload_valid() {
  local payload="$1"
  local tag_sha="$2"
  local head_sha="$3"

  jq -e \
    --arg tag_sha "$tag_sha" \
    --arg head_sha "$head_sha" '
      (.status == "ahead" or .status == "identical")
      and .base_commit.sha == $tag_sha
      and .merge_base_commit.sha == $tag_sha
      and .behind_by == 0
      and .ahead_by >= 0
      and (
        (
          .status == "ahead"
          and (.commits | length) > 0
          and .commits[-1].sha == $head_sha
        )
        or (
          .status == "identical"
          and $head_sha == $tag_sha
          and (.commits | length) == 0
        )
      )
    ' <<<"$payload" >/dev/null
}

release_remote_ancestry_valid() {
  local tag_sha="$1"
  local head_sha="$2"
  local payload

  command -v gh >/dev/null 2>&1 || return 1
  gh auth status >/dev/null 2>&1 || return 1
  payload="$(gh api "repos/$REPO/compare/${tag_sha}...${head_sha}")" \
    || return 1
  release_compare_payload_valid "$payload" "$tag_sha" "$head_sha"
}

release_head_binding_valid() {
  local tag_sha="$1"
  local checkout_sha="$2"
  local allow_post_release_head="$3"
  local post_release_head_sha="${4:-}"

  [[ "$allow_post_release_head" == "0" || "$allow_post_release_head" == "1" ]] \
    || return 1
  if [[ -n "$post_release_head_sha" ]]; then
    [[ "$allow_post_release_head" == "1" ]] || return 1
    git rev-parse -q --verify "${post_release_head_sha}^{commit}" >/dev/null \
      || return 1
    git merge-base --is-ancestor "$post_release_head_sha" "$checkout_sha" \
      || return 1
    git merge-base --is-ancestor "$tag_sha" "$post_release_head_sha" \
      && return 0
    release_remote_ancestry_valid "$tag_sha" "$post_release_head_sha"
    return $?
  fi

  [[ "$tag_sha" == "$checkout_sha" ]] && return 0
  [[ "$allow_post_release_head" == "1" ]] || return 1
  git merge-base --is-ancestor "$tag_sha" "$checkout_sha"
}

release_tag_missing_allowed() {
  local allow_open_closeout="$1"
  local allow_post_release_head="$2"

  [[ "$allow_open_closeout" == "1" && "$allow_post_release_head" != "1" ]]
}

run_release_head_binding_selftest() {
  local tag_sha="$1"
  local earlier_sha="2b955cb8a989827e3351c08c3d82d9578253e1f6"

  release_head_binding_valid "$tag_sha" "$tag_sha" "0" \
    || fail "release head binding selftest rejected the tag commit"
  if release_head_binding_valid "$tag_sha" "$earlier_sha" "0"; then
    fail "release head binding selftest accepted a default mismatch"
  fi
  if release_head_binding_valid "$tag_sha" "$earlier_sha" "1"; then
    fail "release head binding selftest accepted a non-descendant commit"
  fi
  echo "v33_release_head_binding_selftest=pass cases=3"
}

report_release_head_binding_failure() {
  local tag_sha="$1"
  local checkout_sha="$2"
  local post_release_head_sha="$3"
  local tag_to_checkout="fail"
  local tag_to_pr_head="not_checked"
  local pr_head_to_checkout="not_checked"

  git merge-base --is-ancestor "$tag_sha" "$checkout_sha" \
    && tag_to_checkout="pass"
  if [[ -n "$post_release_head_sha" ]]; then
    git merge-base --is-ancestor "$tag_sha" "$post_release_head_sha" \
      && tag_to_pr_head="pass" || tag_to_pr_head="fail"
    git merge-base --is-ancestor "$post_release_head_sha" "$checkout_sha" \
      && pr_head_to_checkout="pass" || pr_head_to_checkout="fail"
  fi

  echo "v33_release_head_binding=fail tag_sha=$tag_sha checkout_sha=$checkout_sha post_release_head_sha=${post_release_head_sha:-unset}" >&2
  echo "v33_release_head_ancestry tag_to_checkout=$tag_to_checkout tag_to_pr_head=$tag_to_pr_head pr_head_to_checkout=$pr_head_to_checkout" >&2
}

run_missing_tag_policy_selftest() {
  release_tag_missing_allowed "1" "0" \
    || fail "missing tag policy selftest rejected pre-tag closeout"
  if release_tag_missing_allowed "1" "1"; then
    fail "missing tag policy selftest accepted post-release mode"
  fi
  if release_tag_missing_allowed "0" "0"; then
    fail "missing tag policy selftest accepted strict mode"
  fi
  echo "v33_missing_tag_policy_selftest=pass cases=3"
}

run_release_compare_payload_selftest() {
  local tag_sha="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
  local head_sha="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
  local valid_payload
  local diverged_payload
  local wrong_head_payload

  valid_payload="$(jq -n --arg tag "$tag_sha" --arg head "$head_sha" '{
    status: "ahead",
    ahead_by: 2,
    behind_by: 0,
    base_commit: {sha: $tag},
    merge_base_commit: {sha: $tag},
    commits: [{sha: $head}]
  }')"
  diverged_payload="$(jq '.status = "diverged" | .behind_by = 1' <<<"$valid_payload")"
  wrong_head_payload="$(jq '.commits[-1].sha = "cccccccccccccccccccccccccccccccccccccccc"' <<<"$valid_payload")"

  release_compare_payload_valid "$valid_payload" "$tag_sha" "$head_sha" \
    || fail "release compare payload selftest rejected an ahead commit"
  if release_compare_payload_valid "$diverged_payload" "$tag_sha" "$head_sha"; then
    fail "release compare payload selftest accepted diverged history"
  fi
  if release_compare_payload_valid "$wrong_head_payload" "$tag_sha" "$head_sha"; then
    fail "release compare payload selftest accepted the wrong head"
  fi
  echo "v33_release_compare_payload_selftest=pass cases=3"
}

validate_manifest() {
  local manifest="$1"
  jq -e '
    .schema_version == "ntpro.v330_backend_maintenance_release_manifest.v1"
    and .task_id == "BPO-007"
    and .product_version == "v0.33.0"
    and .release_status == "release_gate_ready"
    and .release_scope_name == "backend_maintenance"
    and .base_release.version == "v0.32.0"
    and .base_release.tag == "ntpro-rust-only-v0.32.0"
    and .base_release.peeled_commit_sha == "2b955cb8a989827e3351c08c3d82d9578253e1f6"
    and .planned_release.tag == "ntpro-rust-only-v0.33.0"
    and .planned_release.name == "NTPRO Rust-only v0.33.0"
    and .planned_release.draft == false
    and .planned_release.prerelease == false
    and .release_scope.milestone_number == 35
    and .release_scope.milestone_title == "v0.33.0-backend-maintenance"
    and .release_scope.exact_issue_numbers == [1120,1121,1122,1123,1124,1125,1126,1141]
    and .release_scope.exact_pr_numbers == [1134,1135,1136,1137,1138,1139,1140,1142]
    and .release_scope.issue_count == 8
    and .release_scope.pr_count == 8
    and .release_scope.registered_corrective_scope_exception_count == 1
    and .release_scope.registered_corrective_scope_exception_issue_numbers == [1141]
    and .release_scope.maintenance_only == true
    and .release_scope.runtime_behavior_changed_by_release_gate == false
    and .release_scope.trading_behavior_changed_by_release_gate == false
    and (.boundary_flags | length == 27)
    and (.boundary_flags | all(. == false))
    and .publication_governance.gate_before_publish == true
    and .publication_governance.release_gate_workflow_name == "Rust Cutover Release Gate"
    and .publication_governance.publish_workflow_name == "Rust Cutover Publish Release"
    and .publication_governance.public_release_requires_successful_hosted_gate_for_same_tag_commit == true
    and .publication_governance.release_gate_tag_push_event_required == true
    and .publication_governance.release_gate_ref_must_equal_release_tag == true
    and .publication_governance.release_gate_success_before_publication_required == true
    and .publication_governance.publication_evidence_strategy == "source_tree_plus_github_remote"
    and .publication_governance.local_generated_evidence_required_in_source_tree == false
    and .publication_governance.remote_reconstruction_required == true
    and .publication_governance.generated_publication_evidence_sole_proof_allowed == false
    and .rollback.tag_rewrite_allowed == false
    and .rollback.release_candidate_rejection_before_publication == true
    and .next_tracks.capability == "v0.34.0+"
    and .next_tracks.capability_entry == "separately_scoped_only"
  ' "$manifest" >/dev/null
}

[[ -f "$MANIFEST" ]] || fail "missing manifest: $MANIFEST"
command -v jq >/dev/null 2>&1 || fail "jq is required"
[[ "$ALLOW_POST_RELEASE_HEAD" == "0" || "$ALLOW_POST_RELEASE_HEAD" == "1" ]] \
  || fail "NTPRO_V33_ALLOW_POST_RELEASE_HEAD must be 0 or 1"
if [[ -n "$POST_RELEASE_HEAD_SHA" ]]; then
  [[ "$ALLOW_POST_RELEASE_HEAD" == "1" ]] \
    || fail "NTPRO_V33_POST_RELEASE_HEAD_SHA requires post-release mode"
  [[ "${GITHUB_EVENT_NAME:-}" == "pull_request" ]] \
    || fail "NTPRO_V33_POST_RELEASE_HEAD_SHA is only valid for pull_request checks"
  [[ "$POST_RELEASE_HEAD_SHA" =~ ^[0-9a-f]{40}$ ]] \
    || fail "NTPRO_V33_POST_RELEASE_HEAD_SHA must be a full commit SHA"
fi
validate_manifest "$MANIFEST" || fail "manifest structure or boundary mismatch"

while IFS= read -r path; do
  [[ -f "$path" ]] || fail "missing registered release input: $path"
done < <(jq -r '.release_inputs[]' "$MANIFEST")

NOTES="$(jq -r '.release_inputs.release_notes' "$MANIFEST")"
READINESS="$(jq -r '.release_inputs.readiness_report' "$MANIFEST")"
CLOSEOUT="$(jq -r '.release_inputs.release_closeout_evidence' "$MANIFEST")"

for marker in \
  "Status: RELEASE GATE READY" \
  "Tag: \`ntpro-rust-only-v0.33.0\`" \
  "Release name: \`NTPRO Rust-only v0.33.0\`" \
  "maintenance-only" \
  "source_tree_plus_github_remote"; do
  grep -F -- "$marker" "$NOTES" >/dev/null || fail "release notes missing marker: $marker"
done
grep -F "Status: RELEASE GATE READY" "$READINESS" >/dev/null \
  || fail "readiness status mismatch"
grep -F "milestone v0.33.0-backend-maintenance must close after release publication = true" \
  "$CLOSEOUT" >/dev/null || fail "closeout milestone ordering is missing"

scripts/ai/check_backend_freeze_baseline.sh
scripts/ai/check_release_surface_current.sh

run_missing_tag_policy_selftest
run_release_compare_payload_selftest

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
jq '.boundary_flags.production_order_submission_allowed = true' \
  "$MANIFEST" >"$tmp_dir/enabled.json"
if validate_manifest "$tmp_dir/enabled.json"; then
  fail "negative selftest accepted production submission"
fi
jq 'del(.boundary_flags.live_exchange_request_allowed)' \
  "$MANIFEST" >"$tmp_dir/missing.json"
if validate_manifest "$tmp_dir/missing.json"; then
  fail "negative selftest accepted a missing boundary"
fi

if git rev-parse -q --verify "${TAG}^{commit}" >/dev/null; then
  tag_sha="$(git rev-list -n 1 "$TAG")"
  head_sha="$(git rev-parse HEAD)"
  run_release_head_binding_selftest "$tag_sha"
  if ! release_head_binding_valid \
    "$tag_sha" \
    "$head_sha" \
    "$ALLOW_POST_RELEASE_HEAD" \
    "$POST_RELEASE_HEAD_SHA"; then
    report_release_head_binding_failure \
      "$tag_sha" \
      "$head_sha" \
      "$POST_RELEASE_HEAD_SHA"
    fail "release tag must equal or be an ancestor of the checked-out commit"
  fi
elif ! release_tag_missing_allowed "$ALLOW_OPEN_CLOSEOUT" "$ALLOW_POST_RELEASE_HEAD"; then
  fail "missing release tag: $TAG"
fi

if ! command -v gh >/dev/null 2>&1 || ! gh auth status >/dev/null 2>&1; then
  if [[ "$ALLOW_OFFLINE" == "1" ]]; then
    echo "v33_live_state=offline_skip"
  else
    fail "authenticated gh is required for live issue/PR reconstruction"
  fi
else
  issues_json="$(gh api "repos/$REPO/issues?milestone=35&state=all&per_page=100")"
  jq -e '
    [.[] | select(.pull_request == null) | .number] | sort
    == [1120,1121,1122,1123,1124,1125,1126,1141]
  ' <<<"$issues_json" >/dev/null || fail "milestone exact issue set mismatch"
  jq -e '
    [.[] | select(.pull_request == null and .number != 1141) | .state]
    | all(. == "closed")
  ' <<<"$issues_json" >/dev/null || fail "a BPO dependency issue is open"
  if [[ "$ALLOW_OPEN_CLOSEOUT" != "1" ]]; then
    jq -e '
      [.[] | select(.pull_request == null and .number == 1141) | .state]
      == ["closed"]
    ' <<<"$issues_json" >/dev/null || fail "BPO-008 corrective issue is not closed"
  fi

  closeout_pr="$(jq -r '.release_scope.exact_pr_numbers[7]' "$MANIFEST")"
  while IFS= read -r pr; do
    pr_json="$(gh api "repos/$REPO/pulls/$pr")"
    state="$(jq -r '.state' <<<"$pr_json")"
    merged_at="$(jq -r '.merged_at // empty' <<<"$pr_json")"
    if [[ "$pr" == "$closeout_pr" && "$ALLOW_OPEN_CLOSEOUT" == "1" ]]; then
      [[ "$state" == "open" || -n "$merged_at" ]] \
        || fail "closeout PR has invalid state: state=$state merged_at=$merged_at"
    else
      [[ -n "$merged_at" ]] || fail "release PR #$pr is not merged"
    fi
  done < <(jq -r '.release_scope.exact_pr_numbers[]' "$MANIFEST")
  echo "v33_live_state=pass exact_issues=8 exact_prs=8"
fi

echo "v33_maintenance_release=pass boundaries=27 negative_cases=2"
