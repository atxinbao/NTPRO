#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

BASE="${NTPRO_S3_CLOSEOUT_MANIFEST:-docs/product/s3_live_closeout_manifest.json}"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/ntpro-s3-closeout.XXXXXX")"
trap 'find "$TMP_DIR" -depth -delete 2>/dev/null || true' EXIT

expect_failure() {
  local name="$1"
  local filter="$2"
  local fixture="$TMP_DIR/$name.json"
  jq "$filter" "$BASE" >"$fixture"
  if NTPRO_S3_CLOSEOUT_MANIFEST="$fixture" \
    NTPRO_S3_CLOSEOUT_MODE=source \
    NTPRO_S3_CLOSEOUT_NEGATIVE_SELFTEST=0 \
    scripts/ai/check_s3_live_closeout.sh >/dev/null 2>&1; then
    echo "s3 live closeout negative selftest unexpectedly passed: $name" >&2
    exit 1
  fi
}

expect_failure missing_issue '.scope.exact_issue_numbers |= .[:-1]'
expect_failure extra_issue '.scope.exact_issue_numbers += [9999]'
expect_failure wrong_closeout_pr '.closeout.pull_request_number = 9999'
expect_failure enabled_boundary '.boundaries.automatic_retry_allowed = true'
expect_failure missing_requirement '.requirements[0].status = "missing"'
expect_failure renamed_requirement '.requirements[0].id = "renamed_requirement"'
expect_failure renamed_boundary '.boundaries |= with_entries(if .key == "automatic_retry_allowed" then .key = "renamed_boundary" else . end)'
expect_failure rewritten_roadmap_token '.documents.roadmap_token = "S3"'
expect_failure rewritten_product_claim '.product_claim = "complete"'
expect_failure wrong_milestone '.milestone.number = 9999'
expect_failure missing_symbol '.requirements[0].evidence[0].symbol = "missing_s3_closeout_symbol"'

echo "s3_live_closeout_negative_selftest=pass cases=11"
