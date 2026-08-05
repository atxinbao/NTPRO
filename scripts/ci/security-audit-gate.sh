#!/usr/bin/env bash
set -euo pipefail

# Resolve the event's changed files, then delegate path classification to the
# shared CI classifier. Unknown or incomplete event state fails closed by
# forcing both workflow and dependency audits.
#
# Required env vars:
#   EVENT_NAME       - github.event_name
#   PR_BASE_REF      - github.event.pull_request.base.ref (PR only)
#   PR_HEAD_SHA      - github.event.pull_request.head.sha (PR only)
#   PR_MERGE_SHA     - github.sha synthetic merge commit (PR only, optional)
#   PUSH_BEFORE_SHA  - github.event.before (push only)
#   PUSH_AFTER_SHA   - github.event.after  (push only)
#
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

changed_files="$(mktemp)"
trap 'rm -f "$changed_files"' EXIT

force_full() {
  echo "security audit classification forced full: $1"
  : >"$changed_files"
  NTPRO_CI_FORCE_FULL_SECURITY=1 \
    scripts/ci/classify-ci-changes.sh "$changed_files"
  exit 0
}

case "$EVENT_NAME" in
  schedule | workflow_dispatch)
    force_full "event=${EVENT_NAME}"
    ;;
  push)
    base="$PUSH_BEFORE_SHA"
    head="$PUSH_AFTER_SHA"
    if [[ -z "$base" || "$base" =~ ^0+$ ]]; then
      force_full "new branch push has no base"
    fi
    if ! git cat-file -e "${base}^{commit}" 2> /dev/null; then
      force_full "push base SHA ${base} is unavailable"
    fi
    if [[ -z "$head" ]] || ! git cat-file -e "${head}^{commit}" 2> /dev/null; then
      force_full "push head SHA ${head:-<empty>} is unavailable"
    fi
    ;;
  pull_request)
    head="$PR_HEAD_SHA"
    if [[ -z "$head" ]] || ! git cat-file -e "${head}^{commit}" 2> /dev/null; then
      force_full "pull request head SHA ${head:-<empty>} is unavailable"
    fi

    if [[ -n "${PR_MERGE_SHA:-}" ]] \
      && git cat-file -e "${PR_MERGE_SHA}^{commit}" 2> /dev/null; then
      read -r merge_commit base merge_head extra \
        < <(git rev-list --parents -n 1 "$PR_MERGE_SHA")
      if [[ "$merge_commit" != "$PR_MERGE_SHA" \
        || -z "${base:-}" \
        || "$merge_head" != "$head" \
        || -n "${extra:-}" ]]; then
        force_full "pull request merge commit parent binding is invalid"
      fi
    else
      # Local callers and older events may not provide the synthetic merge
      # commit. Preserve the full-history merge-base fallback and force the
      # complete audit when its graph is unavailable.
      if ! base="$(git merge-base "origin/${PR_BASE_REF}" "$head" 2> /dev/null)"; then
        force_full "cannot compute merge-base against origin/${PR_BASE_REF}"
      fi
      if [[ -z "$base" ]]; then
        force_full "merge-base against origin/${PR_BASE_REF} is empty"
      fi
    fi
    ;;
  *)
    force_full "unknown event=${EVENT_NAME}"
    ;;
esac

if ! git diff --no-renames --name-only "$base" "$head" >"$changed_files"; then
  force_full "git diff failed for ${base}..${head}"
fi
cat "$changed_files"
scripts/ci/classify-ci-changes.sh "$changed_files"
