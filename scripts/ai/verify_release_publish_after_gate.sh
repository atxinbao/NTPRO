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

if [[ "$1" == "release" && "$2" == "view" ]]; then
  case "$mode" in
    missing)
      exit 1
      ;;
    draft)
      cat <<JSON
{"isDraft":true,"url":"https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.22.1","publishedAt":"","body":$body}
JSON
      ;;
    published_after)
      cat <<JSON
{"isDraft":false,"url":"https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.22.1","publishedAt":"2026-07-02T10:05:00Z","body":$body}
JSON
      ;;
    published_before)
      cat <<JSON
{"isDraft":false,"url":"https://github.com/atxinbao/NTPRO/releases/tag/ntpro-rust-only-v0.22.1","publishedAt":"2026-07-02T09:55:00Z","body":$body}
JSON
      ;;
    *)
      echo "unknown fake release mode: $mode" >&2
      exit 2
      ;;
  esac
  exit 0
fi

if [[ "$1" == "release" && ( "$2" == "edit" || "$2" == "create" ) ]]; then
  echo "$*" >> "${NTPRO_FAKE_GH_CALL_LOG:?}"
  exit 0
fi

echo "unexpected fake gh invocation: $*" >&2
exit 2
EOF
chmod +x "$fake_gh"

run_publish_script() {
  NTPRO_RELEASE_PUBLICATION_GH_BIN="$fake_gh" \
    NTPRO_RELEASE_PUBLICATION_TEST_MODE=1 \
    NTPRO_RELEASE_PUBLICATION_TEST_TAG_SHA="$tag_sha" \
    NTPRO_RELEASE_PUBLICATION_DRY_RUN=1 \
    NTPRO_RELEASE_GATE_RUN_ID=1 \
    NTPRO_RELEASE_VERSION=v0.22.1 \
    NTPRO_RELEASE_TAG=ntpro-rust-only-v0.22.1 \
    NTPRO_RELEASE_NAME="NTPRO Rust-only v0.22.1" \
    NTPRO_RELEASE_NOTES="$notes_file" \
    NTPRO_RELEASE_PUBLICATION_EVIDENCE_PATH="$evidence_path" \
    NTPRO_FAKE_RELEASE_NOTES="$notes_file" \
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

echo "== verify release publish after gate: failed gate blocks publication =="
if NTPRO_FAKE_RELEASE_MODE=draft NTPRO_FAKE_GATE_CONCLUSION=failure \
  run_publish_script scripts/ai/publish_ntpro_release_after_gate.sh >"$tmp_dir/failed-gate.out" 2>&1; then
  echo "expected failed gate to block publication" >&2
  exit 1
fi
grep -F "release gate run did not succeed: failure" "$tmp_dir/failed-gate.out" >/dev/null

echo "== verify release publish after gate: public before gate is rejected =="
if NTPRO_FAKE_RELEASE_MODE=published_before \
  run_publish_script scripts/ai/publish_ntpro_release_after_gate.sh >"$tmp_dir/published-before.out" 2>&1; then
  echo "expected pre-gate publication to fail" >&2
  exit 1
fi
grep -F "release was published before hosted gate success" "$tmp_dir/published-before.out" >/dev/null

echo "release_publish_after_gate=pass"
