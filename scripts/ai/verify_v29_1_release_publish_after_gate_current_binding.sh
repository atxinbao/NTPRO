#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

RELEASE_VERSION="${NTPRO_V291_PUBLISH_AFTER_GATE_VERSION:-v0.29.0}"
RELEASE_TAG="${NTPRO_V291_PUBLISH_AFTER_GATE_TAG:-ntpro-rust-only-v0.29.0}"
RELEASE_NAME="${NTPRO_V291_PUBLISH_AFTER_GATE_NAME:-NTPRO Rust-only v0.29.0}"
RELEASE_NOTES="${NTPRO_V291_PUBLISH_AFTER_GATE_NOTES:-docs/rust-cutover/release/v0_29_0_release_notes.md}"
RELEASE_MANIFEST="${NTPRO_V291_PUBLISH_AFTER_GATE_MANIFEST:-docs/rust-cutover/release/v0_29_0_release_manifest.json}"
RELEASE_CLOSEOUT="${NTPRO_V291_PUBLISH_AFTER_GATE_CLOSEOUT:-docs/rust-cutover/release/v0_29_0_release_closeout_evidence.md}"
RELEASE_GATE_RUN_ID="${NTPRO_V291_PUBLISH_AFTER_GATE_RUN_ID:-29091765148}"
RELEASE_TAG_SHA="${NTPRO_V291_PUBLISH_AFTER_GATE_TAG_SHA:-85110d29867763f8d3b6395f4ff8154378b475b9}"
LIVE_CURRENT="${NTPRO_V291_PUBLISH_AFTER_GATE_LIVE_CURRENT:-0}"
REQUIRE_LIVE_CURRENT="${NTPRO_V291_PUBLISH_AFTER_GATE_REQUIRE_LIVE_CURRENT:-0}"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

fail() {
  echo "v29.1 release publish-after-gate current binding failed: $*" >&2
  exit 1
}

run_binding() {
  local output="$1"
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_VERSION="$RELEASE_VERSION" \
    NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_TAG="$RELEASE_TAG" \
    NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_NAME="$RELEASE_NAME" \
    NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_NOTES="$RELEASE_NOTES" \
    NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_MANIFEST="$RELEASE_MANIFEST" \
    NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_CLOSEOUT="$RELEASE_CLOSEOUT" \
    NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_GATE_RUN_ID="$RELEASE_GATE_RUN_ID" \
    NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_TAG_SHA="$RELEASE_TAG_SHA" \
    NTPRO_RELEASE_PUBLISH_AFTER_GATE_LIVE_CURRENT="$LIVE_CURRENT" \
    NTPRO_RELEASE_PUBLISH_AFTER_GATE_REQUIRE_LIVE_CURRENT="$REQUIRE_LIVE_CURRENT" \
    scripts/ai/verify_release.sh release-publish-after-gate | tee "$output"
}

has_v29_binding() {
  local output="$1"
  grep -F "release_publish_after_gate_current_binding=pass release_tag=$RELEASE_TAG release_gate_run_id=$RELEASE_GATE_RUN_ID tag_sha=$RELEASE_TAG_SHA" "$output" >/dev/null &&
    ! grep -F "release_tag=ntpro-rust-only-v0.28.0" "$output" >/dev/null &&
    ! grep -F "release_gate_run_id=28969059200" "$output" >/dev/null
}

run_binding "$tmp_dir/v29-current.out"
has_v29_binding "$tmp_dir/v29-current.out" || fail "v29 current binding marker missing"

if NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_VERSION="v0.28.0" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_TAG="ntpro-rust-only-v0.28.0" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_NAME="NTPRO Rust-only v0.28.0" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_NOTES="docs/rust-cutover/release/v0_28_0_release_notes.md" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_MANIFEST="docs/rust-cutover/release/v0_28_0_release_manifest.json" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_CLOSEOUT="docs/rust-cutover/release/v0_28_0_release_closeout_evidence.md" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_GATE_RUN_ID="28969059200" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_TAG_SHA="41ef23417a4f21226cbc069de8cc31d0fa5e696e" \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_LIVE_CURRENT=0 \
  NTPRO_RELEASE_PUBLISH_AFTER_GATE_REQUIRE_LIVE_CURRENT=0 \
  scripts/ai/verify_release.sh release-publish-after-gate >"$tmp_dir/v28-current.out" 2>&1; then
  if has_v29_binding "$tmp_dir/v28-current.out"; then
    fail "negative self-test unexpectedly accepted v0.28.0 fallback as v29 current binding"
  fi
fi

echo "v29_1_release_publish_after_gate_current_binding status=ok release_tag=$RELEASE_TAG release_gate_run_id=$RELEASE_GATE_RUN_ID tag_sha=$RELEASE_TAG_SHA historical_v28_fallback_rejected=true live_current=$LIVE_CURRENT"
