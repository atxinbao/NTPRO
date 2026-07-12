#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

notes_file="$tmp_dir/release-notes.md"
fake_gh="$tmp_dir/gh"
evidence_path="$tmp_dir/evidence/publication.json"
tag_sha="0123456789abcdef0123456789abcdef01234567"

CURRENT_RELEASE_VERSION="${NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_VERSION:-v0.30.0}"
CURRENT_RELEASE_TAG="${NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_TAG:-ntpro-rust-only-v0.30.0}"
CURRENT_RELEASE_NAME="${NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_NAME:-NTPRO Rust-only v0.30.0}"
CURRENT_RELEASE_NOTES="${NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_NOTES:-docs/rust-cutover/release/v0_30_0_release_notes.md}"
CURRENT_RELEASE_MANIFEST="${NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_MANIFEST:-docs/rust-cutover/release/v0_30_0_release_manifest.json}"
CURRENT_RELEASE_CLOSEOUT="${NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_CLOSEOUT:-docs/rust-cutover/release/v0_30_0_release_closeout_evidence.md}"
CURRENT_RELEASE_GATE_RUN_ID="${NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_GATE_RUN_ID:-29139384219}"
CURRENT_RELEASE_TAG_SHA="${NTPRO_RELEASE_PUBLISH_AFTER_GATE_CURRENT_TAG_SHA:-0f0949156401fa6e6016c0160697e7090a6da788}"
REQUIRE_LIVE_CURRENT="${NTPRO_RELEASE_PUBLISH_AFTER_GATE_REQUIRE_LIVE_CURRENT:-${NTPRO_RELEASE_GATE:-0}}"
LIVE_CURRENT_MODE="${NTPRO_RELEASE_PUBLISH_AFTER_GATE_LIVE_CURRENT:-auto}"
LIVE_CURRENT_TIMEOUT="${NTPRO_RELEASE_PUBLISH_AFTER_GATE_LIVE_TIMEOUT:-90s}"

run_with_optional_timeout() {
  if command -v timeout >/dev/null 2>&1; then
    timeout "$LIVE_CURRENT_TIMEOUT" "$@"
  elif command -v gtimeout >/dev/null 2>&1; then
    gtimeout "$LIVE_CURRENT_TIMEOUT" "$@"
  else
    "$@"
  fi
}

cat >"$notes_file" <<'EOF'
# NTPRO Rust-only v0.22.1

Status: RELEASED
Tag: `ntpro-rust-only-v0.22.1`
Release name: `NTPRO Rust-only v0.22.1`
Release URL: `https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.22.1`

Gate-before-publish release governance.
EOF

cat >"$fake_gh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

mode="${NTPRO_FAKE_RELEASE_MODE:-draft}"
notes_file="${NTPRO_FAKE_RELEASE_NOTES:?}"
body="$(python3 - "$notes_file" <<'PY'
from pathlib import Path
import json
import sys

print(json.dumps(Path(sys.argv[1]).read_text(encoding="utf-8")))
PY
)"
stale_body="$(python3 - <<'PY'
import json

print(json.dumps("# stale release body\n"))
PY
)"
state_file="${NTPRO_FAKE_RELEASE_STATE:?}"

release_payload() {
  local release_mode="$1"
  local release_body="$2"
  local draft="$3"
  local published_at="$4"
  local updated_at="$5"
  local assets="${6:-[]}"
  cat <<JSON
{"tag_name":"ntpro-rust-only-v0.22.1","name":"NTPRO Rust-only v0.22.1","draft":$draft,"prerelease":false,"html_url":"https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.22.1","published_at":"$published_at","updated_at":"$updated_at","body":$release_body,"mode":"$release_mode","assets":$assets}
JSON
}

if [[ "$1" == "auth" && "$2" == "status" ]]; then
  exit 0
fi

if [[ "$1" == "run" && "$2" == "view" ]]; then
  if [[ "${NTPRO_FAKE_GATE_CONCLUSION:-success}" != "success" ]]; then
    cat <<JSON
{"status":"completed","conclusion":"failure","workflowName":"Rust Cutover Release Gate","headSha":"0123456789abcdef0123456789abcdef01234567","url":"https://github.com/atxinbao/NTPRO/actions/runs/1","updatedAt":"2026-07-02T10:00:00Z"}
JSON
    exit 0
  fi
  cat <<JSON
{"status":"completed","conclusion":"success","workflowName":"Rust Cutover Release Gate","headSha":"0123456789abcdef0123456789abcdef01234567","url":"https://github.com/atxinbao/NTPRO/actions/runs/1","updatedAt":"2026-07-02T10:00:00Z"}
JSON
  exit 0
fi

if [[ "$1" == "api" ]]; then
  current_state="$(cat "$state_file" 2>/dev/null || true)"
  if grep -F "release create" "${NTPRO_FAKE_GH_CALL_LOG:?}" >/dev/null 2>&1; then
    release_payload "$mode" "$body" false "2026-07-02T10:05:00Z" "2026-07-02T10:05:00Z"
    exit 0
  fi
  if [[ "$current_state" == "create" ]]; then
    release_payload "$mode" "$body" false "2026-07-02T10:05:00Z" "2026-07-02T10:05:00Z"
    exit 0
  fi
  case "$mode" in
    missing)
      exit 1
      ;;
    draft)
      if [[ "$current_state" == "edit" ]]; then
        release_payload "$mode" "$body" false "2026-07-02T10:05:00Z" "2026-07-02T10:05:00Z"
      else
        release_payload "$mode" "$body" true "" "2026-07-02T09:50:00Z"
      fi
      ;;
    published_after)
      release_payload "$mode" "$body" false "2026-07-02T10:05:00Z" "2026-07-02T10:05:00Z"
      ;;
    published_before)
      release_payload "$mode" "$body" false "2026-07-02T09:55:00Z" "2026-07-02T09:55:00Z"
      ;;
    published_before_with_assets)
      release_payload "$mode" "$body" false "2026-07-02T09:55:00Z" "2026-07-02T09:55:00Z" '[{"name":"artifact.tar.gz"}]'
      ;;
    published_stale_before)
      if [[ "$current_state" == "edit" ]]; then
        release_payload "$mode" "$body" false "2026-07-02T09:55:00Z" "2026-07-02T10:06:00Z"
      else
        release_payload "$mode" "$stale_body" false "2026-07-02T09:55:00Z" "2026-07-02T09:55:00Z"
      fi
      ;;
    *)
      echo "unknown fake release mode: $mode" >&2
      exit 2
      ;;
  esac
  exit 0
fi

if [[ "$1" == "release" && ( "$2" == "edit" || "$2" == "create" || "$2" == "delete" ) ]]; then
  echo "$*" >> "${NTPRO_FAKE_GH_CALL_LOG:?}"
  echo "$2" > "$state_file"
  exit 0
fi

echo "unexpected fake gh invocation: $*" >&2
exit 2
EOF
chmod +x "$fake_gh"

run_publish_script() {
  : > "$tmp_dir/gh-calls.log"
  : > "$tmp_dir/release-state"
  NTPRO_RELEASE_PUBLICATION_GH_BIN="$fake_gh" \
    NTPRO_RELEASE_PUBLICATION_TEST_MODE=1 \
    NTPRO_RELEASE_PUBLICATION_TEST_TAG_SHA="$tag_sha" \
    NTPRO_RELEASE_PUBLICATION_DRY_RUN="${NTPRO_RELEASE_PUBLICATION_DRY_RUN:-1}" \
    NTPRO_RELEASE_GATE_RUN_ID=1 \
    NTPRO_RELEASE_VERSION=v0.22.1 \
    NTPRO_RELEASE_TAG=ntpro-rust-only-v0.22.1 \
    NTPRO_RELEASE_NAME="NTPRO Rust-only v0.22.1" \
    NTPRO_RELEASE_NOTES="$notes_file" \
    NTPRO_RELEASE_PUBLICATION_EVIDENCE_PATH="$evidence_path" \
    NTPRO_FAKE_RELEASE_NOTES="$notes_file" \
    NTPRO_FAKE_RELEASE_STATE="$tmp_dir/release-state" \
    NTPRO_FAKE_GH_CALL_LOG="$tmp_dir/gh-calls.log" \
    NTPRO_FAKE_RELEASE_MODE="${NTPRO_FAKE_RELEASE_MODE:-draft}" \
    NTPRO_FAKE_GATE_CONCLUSION="${NTPRO_FAKE_GATE_CONCLUSION:-success}" \
    "$@"
}

echo "== verify release publish after gate: current release binding =="
CURRENT_RELEASE_VERSION="$CURRENT_RELEASE_VERSION" \
CURRENT_RELEASE_TAG="$CURRENT_RELEASE_TAG" \
CURRENT_RELEASE_NAME="$CURRENT_RELEASE_NAME" \
CURRENT_RELEASE_MANIFEST="$CURRENT_RELEASE_MANIFEST" \
CURRENT_RELEASE_CLOSEOUT="$CURRENT_RELEASE_CLOSEOUT" \
CURRENT_RELEASE_GATE_RUN_ID="$CURRENT_RELEASE_GATE_RUN_ID" \
CURRENT_RELEASE_TAG_SHA="$CURRENT_RELEASE_TAG_SHA" \
python3 <<'PY'
from __future__ import annotations

import copy
import json
import os
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def fail(message: str) -> None:
    raise SystemExit(f"current release publish-after-gate binding failed: {message}")


def require(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def parse_ts(value: str) -> datetime:
    require(isinstance(value, str) and value, "timestamp is empty")
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def canonical_release_sections(manifest: dict[str, Any]) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    planned = manifest.get("planned_release") or {}
    published = manifest.get("published_release") or {}
    closeout = manifest.get("post_publication_closeout") or {}
    if published:
        return planned, published, closeout

    release_closeout = manifest.get("post_release_closeout") or {}
    github_release = release_closeout.get("github_release") or {}
    tag = release_closeout.get("tag") or {}
    hosted_gate = release_closeout.get("hosted_release_gate") or {}
    publication_evidence = release_closeout.get("publication_evidence") or {}

    published = {
        "tag": github_release.get("tag"),
        "name": github_release.get("name"),
        "github_release_url": github_release.get("url"),
        "target_commitish": github_release.get("target_commitish"),
        "draft": github_release.get("draft"),
        "prerelease": github_release.get("prerelease"),
        "tag_sha": tag.get("peeled_commit"),
        "published_at": github_release.get("published_at"),
    }
    closeout = {
        "release_gate_run_id": hosted_gate.get("run_id"),
        "release_gate_status": hosted_gate.get("status"),
        "release_gate_conclusion": hosted_gate.get("conclusion"),
        "release_gate_workflow_name": hosted_gate.get("workflow"),
        "release_gate_head_sha": hosted_gate.get("head_sha"),
        "release_gate_completed_at": hosted_gate.get("completed_at"),
        "published_after_hosted_gate": publication_evidence.get("status") == "published_after_gate",
        "generated_evidence_is_sole_proof": publication_evidence.get("generated_publication_evidence_sole_proof_allowed"),
        "source_controlled_closeout_evidence": bool(release_closeout.get("closeout_evidence_path")),
        "current_release_publish_after_gate_binding": publication_evidence.get("current_release_publish_after_gate_binding"),
    }
    return planned, published, closeout


def remove_closeout(candidate: dict[str, Any]) -> None:
    candidate.pop("post_publication_closeout", None)
    candidate.pop("post_release_closeout", None)


def set_published_at(candidate: dict[str, Any], value: str) -> None:
    if "published_release" in candidate:
        candidate["published_release"]["published_at"] = value
        return
    candidate["post_release_closeout"]["github_release"]["published_at"] = value


def set_gate_head_sha(candidate: dict[str, Any], value: str) -> None:
    if "post_release_closeout" in candidate:
        candidate["post_release_closeout"]["hosted_release_gate"]["head_sha"] = value
        return
    candidate["post_publication_closeout"]["release_gate_head_sha"] = value


def validate(manifest: dict[str, Any], closeout_text: str) -> dict[str, Any]:
    expected_version = os.environ["CURRENT_RELEASE_VERSION"]
    expected_tag = os.environ["CURRENT_RELEASE_TAG"]
    expected_name = os.environ["CURRENT_RELEASE_NAME"]
    expected_gate_run_id = int(os.environ["CURRENT_RELEASE_GATE_RUN_ID"])
    expected_tag_sha = os.environ["CURRENT_RELEASE_TAG_SHA"]

    require(manifest.get("product_version") == expected_version, "product version mismatch")

    planned, published, closeout = canonical_release_sections(manifest)
    governance = manifest.get("publication_governance") or {}
    requirements = manifest.get("post_publication_requirements") or {}

    require(planned.get("tag") == expected_tag, "planned tag mismatch")
    require(planned.get("name") == expected_name, "planned release name mismatch")
    require(published.get("tag") == expected_tag, "published tag mismatch")
    require(published.get("name") == expected_name, "published release name mismatch")
    require(published.get("draft") is False, "published release must not be draft")
    require(published.get("prerelease") is False, "published release must not be prerelease")
    require(isinstance(published.get("tag_sha"), str) and published["tag_sha"], "published tag_sha missing")
    require(published.get("tag_sha") == expected_tag_sha, "published tag sha mismatch")
    require(isinstance(published.get("published_at"), str) and published["published_at"], "published_at missing")

    require(governance.get("publication_evidence_strategy") == "source_tree_plus_github_remote", "publication evidence strategy mismatch")
    require(governance.get("local_generated_evidence_required_in_source_tree") is False, "generated evidence must not be required in source tree")
    require(governance.get("remote_reconstruction_required") is True, "remote reconstruction requirement missing")
    require(governance.get("release_gate_success_before_publication_required") is True, "gate-before-publication requirement missing")
    require(
        governance.get("public_release_requires_successful_hosted_gate_for_same_tag_commit") is True,
        "same-tag hosted gate requirement missing",
    )
    require(governance.get("current_release_publish_after_gate_binding_required") is True, "current release binding requirement missing")
    require(governance.get("historical_fixture_only_current_release_proof_allowed") is False, "fixture-only proof must not be allowed")

    require(closeout.get("release_gate_run_id") == expected_gate_run_id, "current release gate run id mismatch")
    require(closeout.get("release_gate_status") == "completed", "release gate status mismatch")
    require(closeout.get("release_gate_conclusion") == "success", "release gate conclusion mismatch")
    require(closeout.get("release_gate_workflow_name") == "Rust Cutover Release Gate", "release gate workflow mismatch")
    require(closeout.get("release_gate_head_sha") == published.get("tag_sha"), "gate head sha must match published tag sha")
    require(closeout.get("published_after_hosted_gate") is True, "published_after_hosted_gate must be true")
    require(closeout.get("generated_evidence_is_sole_proof") is False, "generated evidence must not be sole proof")
    require(closeout.get("source_controlled_closeout_evidence") is True, "source-controlled closeout evidence missing")
    require(closeout.get("current_release_publish_after_gate_binding") == "pass", "current release binding status missing")

    gate_completed = parse_ts(closeout.get("release_gate_completed_at", ""))
    published_at = parse_ts(published.get("published_at", ""))
    require(published_at >= gate_completed, "published_at is before hosted gate completion")

    require(requirements.get("publication_after_hosted_gate_required") is True, "post-publication ordering requirement missing")
    require(requirements.get("github_release_published_required") is True, "published release requirement missing")
    require(requirements.get("source_controlled_closeout_evidence_required") is True, "source closeout requirement missing")

    for marker in (
        "release publication after gate = pass",
        "release publish after gate current-release binding = pass",
        f"release_gate_run_id = {expected_gate_run_id}",
        "published_at is public publication proof = true",
        "historical fixture-only current-release proof allowed = false",
    ):
        require(marker in closeout_text, f"closeout marker missing: {marker}")

    return {
        "release_tag": expected_tag,
        "release_gate_run_id": expected_gate_run_id,
        "tag_sha": published.get("tag_sha"),
        "release_gate_completed_at": closeout.get("release_gate_completed_at"),
        "published_at": published.get("published_at"),
    }


manifest_path = Path(os.environ["CURRENT_RELEASE_MANIFEST"])
closeout_path = Path(os.environ["CURRENT_RELEASE_CLOSEOUT"])
require(manifest_path.is_file(), f"missing current release manifest: {manifest_path}")
require(closeout_path.is_file(), f"missing current release closeout: {closeout_path}")

manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
closeout_text = closeout_path.read_text(encoding="utf-8")
result = validate(manifest, closeout_text)

for label, mutator in (
    ("missing current closeout proof", remove_closeout),
    (
        "stale publication timestamp",
        lambda candidate: set_published_at(candidate, "2026-07-08T20:00:00Z"),
    ),
    (
        "gate sha mismatch",
        lambda candidate: set_gate_head_sha(candidate, "deadbeef"),
    ),
    (
        "fixture-only proof",
        lambda candidate: candidate["publication_governance"].update(
            {"historical_fixture_only_current_release_proof_allowed": True}
        ),
    ),
):
    mutated = copy.deepcopy(manifest)
    mutator(mutated)
    try:
        validate(mutated, closeout_text)
    except SystemExit:
        pass
    else:
        fail(f"negative self-test unexpectedly allowed {label}")

print(
    "release_publish_after_gate_current_binding=pass "
    f"release_tag={result['release_tag']} "
    f"release_gate_run_id={result['release_gate_run_id']} "
    f"tag_sha={result['tag_sha']} "
    "historical_fixture_only_current_release_proof_allowed=false "
    "negative_selftest=1"
)
PY

live_skip_reason=""
if [[ "$LIVE_CURRENT_MODE" == "0" || "$LIVE_CURRENT_MODE" == "false" ]]; then
  live_skip_reason="live_current_disabled"
elif [[ "${GITHUB_ACTIONS:-}" == "true" && "$REQUIRE_LIVE_CURRENT" != "1" && "$LIVE_CURRENT_MODE" == "auto" ]]; then
  live_skip_reason="github_actions_auto_live_skip"
elif ! command -v gh >/dev/null 2>&1; then
  live_skip_reason="gh_unavailable"
elif ! run_with_optional_timeout gh auth status >/dev/null 2>&1; then
  live_skip_reason="gh_auth_unavailable"
elif ! git rev-parse -q --verify "${CURRENT_RELEASE_TAG}^{commit}" >/dev/null; then
  live_skip_reason="local_tag_unavailable"
fi

if [[ -z "$live_skip_reason" ]]; then
  NTPRO_RELEASE_VERSION="$CURRENT_RELEASE_VERSION" \
    NTPRO_RELEASE_TAG="$CURRENT_RELEASE_TAG" \
    NTPRO_RELEASE_NAME="$CURRENT_RELEASE_NAME" \
    NTPRO_RELEASE_NOTES="$CURRENT_RELEASE_NOTES" \
    NTPRO_RELEASE_PUBLICATION_DRY_RUN=1 \
    NTPRO_RELEASE_GATE_RUN_ID="$CURRENT_RELEASE_GATE_RUN_ID" \
  NTPRO_RELEASE_PUBLICATION_EVIDENCE_PATH="$tmp_dir/current-release-publication.json" \
  run_with_optional_timeout scripts/ai/publish_ntpro_release_after_gate.sh | tee "$tmp_dir/current-release.out"
  grep -F "release_publication_after_gate=pass status=already_published_after_gate" "$tmp_dir/current-release.out" >/dev/null
  grep -F "release_tag_sha=$CURRENT_RELEASE_TAG_SHA" "$tmp_dir/current-release.out" >/dev/null
  echo "release_publish_after_gate_current_live=pass release_tag=$CURRENT_RELEASE_TAG release_gate_run_id=$CURRENT_RELEASE_GATE_RUN_ID"
elif [[ "$REQUIRE_LIVE_CURRENT" == "1" ]]; then
  echo "current release live publish-after-gate proof is required but unavailable: $live_skip_reason" >&2
  exit 1
else
  echo "release_publish_after_gate_current_live=skipped reason=$live_skip_reason"
fi

echo "== verify release publish after gate: draft dry-run passes =="
NTPRO_FAKE_RELEASE_MODE=draft run_publish_script scripts/ai/publish_ntpro_release_after_gate.sh \
  | tee "$tmp_dir/draft.out"
grep -F "release_publication_after_gate=pass status=dry_run_publish_draft_after_gate" "$tmp_dir/draft.out" >/dev/null
python3 - "$evidence_path" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
assert payload["status"] == "dry_run_publish_draft_after_gate"
assert payload["release_gate_completed_at"] == "2026-07-02T10:00:00Z"
PY

echo "== verify release publish after gate: existing public after gate passes =="
NTPRO_FAKE_RELEASE_MODE=published_after run_publish_script scripts/ai/publish_ntpro_release_after_gate.sh \
  | tee "$tmp_dir/published-after.out"
grep -F "release_publication_after_gate=pass status=already_published_after_gate" "$tmp_dir/published-after.out" >/dev/null

echo "== verify release publish after gate: existing stale public release is recreated after gate =="
NTPRO_RELEASE_PUBLICATION_DRY_RUN=0 NTPRO_FAKE_RELEASE_MODE=published_stale_before \
  run_publish_script scripts/ai/publish_ntpro_release_after_gate.sh \
  | tee "$tmp_dir/published-stale.out"
grep -F "release_publication_after_gate=pass status=recreated_public_release_after_gate" "$tmp_dir/published-stale.out" >/dev/null
grep -F "release delete" "$tmp_dir/gh-calls.log" >/dev/null
grep -F "release create" "$tmp_dir/gh-calls.log" >/dev/null

echo "== verify release publish after gate: failed gate blocks publication =="
if NTPRO_FAKE_RELEASE_MODE=draft NTPRO_FAKE_GATE_CONCLUSION=failure \
  run_publish_script scripts/ai/publish_ntpro_release_after_gate.sh >"$tmp_dir/failed-gate.out" 2>&1; then
  echo "expected failed gate to block publication" >&2
  exit 1
fi
grep -F "release gate run did not succeed: failure" "$tmp_dir/failed-gate.out" >/dev/null

echo "== verify release publish after gate: public before gate dry-run plans recreation =="
NTPRO_FAKE_RELEASE_MODE=published_before \
  run_publish_script scripts/ai/publish_ntpro_release_after_gate.sh \
  | tee "$tmp_dir/published-before.out"
grep -F "release_publication_after_gate=pass status=dry_run_recreate_public_release_after_gate" "$tmp_dir/published-before.out" >/dev/null

echo "== verify release publish after gate: public before gate with assets is rejected =="
if NTPRO_FAKE_RELEASE_MODE=published_before_with_assets \
  run_publish_script scripts/ai/publish_ntpro_release_after_gate.sh >"$tmp_dir/published-before-assets.out" 2>&1; then
  echo "expected pre-gate publication with assets to fail" >&2
  exit 1
fi
grep -F "cannot recreate public release with assets: assets=1" "$tmp_dir/published-before-assets.out" >/dev/null

echo "release_publish_after_gate=pass"
