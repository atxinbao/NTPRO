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
