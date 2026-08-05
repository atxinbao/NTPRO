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
    ;;
  pull_request)
    # The PR event payload freezes base.sha at PR creation time, so intervening
    # pushes to the base branch make that SHA stale. Diff against the
    # merge-base with the current base-branch tip so the gate reflects only
    # the PR's own changes.
    head="$PR_HEAD_SHA"
    if ! base="$(git merge-base "origin/${PR_BASE_REF}" "$head" 2> /dev/null)"; then
      force_full "cannot compute merge-base against origin/${PR_BASE_REF}"
    fi
    if [[ -z "$base" ]]; then
      force_full "merge-base against origin/${PR_BASE_REF} is empty"
    fi
    ;;
  *)
    force_full "unknown event=${EVENT_NAME}"
    ;;
esac

git diff --name-only "$base" "$head" | tee "$changed_files"
scripts/ci/classify-ci-changes.sh "$changed_files"
