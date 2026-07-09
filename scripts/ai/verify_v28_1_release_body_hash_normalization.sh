#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

SEMANTICS_DOC="docs/rust-cutover/release/release_body_hash_semantics.md"
V280_CLOSEOUT="docs/rust-cutover/release/v0_28_0_release_closeout_evidence.md"
V280_MANIFEST="docs/rust-cutover/release/v0_28_0_release_manifest.json"
V280_009_EVIDENCE="docs/rust-cutover/evidence/V280-009.md"
V281_001_EVIDENCE="docs/rust-cutover/evidence/V281-001.md"
PUBLICATION_GUARD="scripts/ai/check_github_release_published.sh"

for path in \
  "$SEMANTICS_DOC" \
  "$V280_CLOSEOUT" \
  "$V280_MANIFEST" \
  "$V280_009_EVIDENCE" \
  "$V281_001_EVIDENCE" \
  "$PUBLICATION_GUARD"; do
  [[ -f "$path" ]] || {
    echo "missing release body hash normalization input: $path" >&2
    exit 1
  }
done

python3 <<'PY'
import hashlib
import json
from pathlib import Path

semantics = Path("docs/rust-cutover/release/release_body_hash_semantics.md").read_text(encoding="utf-8")
closeout = Path("docs/rust-cutover/release/v0_28_0_release_closeout_evidence.md").read_text(encoding="utf-8")
manifest = json.loads(Path("docs/rust-cutover/release/v0_28_0_release_manifest.json").read_text(encoding="utf-8"))
v280_009 = Path("docs/rust-cutover/evidence/V280-009.md").read_text(encoding="utf-8")
v281_001 = Path("docs/rust-cutover/evidence/V281-001.md").read_text(encoding="utf-8")
guard = Path("scripts/ai/check_github_release_published.sh").read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def normalize(value: str) -> str:
    return "\n".join(line.rstrip() for line in value.splitlines()).strip()


def sha256(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


notes = "# Release\n\nBody\n"
trailing_newline_body = "# Release\n\nBody\n\n"
content_drift_body = "# Release\n\nBody changed\n"

require(notes != trailing_newline_body, "raw trailing-newline fixture must differ")
require(sha256(notes) != sha256(trailing_newline_body), "raw trailing-newline hash must differ")
require(normalize(notes) == normalize(trailing_newline_body), "trailing-newline drift must normalize away")
require(normalize(notes) != normalize(content_drift_body), "content drift must not normalize away")

for marker in (
    "release_body_hash_semantics = normalized_sha256",
    "release_body_normalization = line_rstrip_and_outer_strip",
    "accepted_trailing_newline_only_drift = true",
    "accepted_content_drift_beyond_normalization = false",
    "raw_sha256_is_acceptance_rule = false",
):
    require(marker in semantics, f"missing semantics marker: {marker}")

for marker in (
    "release body hash semantics = normalized_sha256",
    "release body normalized sha256 = a0586ed49c0154ab1bae4ceff46dcf582139a1b0324efc379662f96a092ee219",
    "tracked release notes normalized sha256 = a0586ed49c0154ab1bae4ceff46dcf582139a1b0324efc379662f96a092ee219",
    "normalized release body matches tracked release notes = true",
    "release body raw sha256 = fb4ee2773151b5685a6b89fd75da53d4c496116fdd3bf315f532525fc1ce6b00",
    "tracked release notes raw sha256 = fb4ee2773151b5685a6b89fd75da53d4c496116fdd3bf315f532525fc1ce6b00",
    "raw hash equality is diagnostic, not the acceptance rule",
):
    require(marker in closeout, f"missing v28 closeout marker: {marker}")

published = manifest.get("published_release") or {}
require(published.get("release_body_hash_semantics") == "normalized_sha256", "manifest hash semantics mismatch")
require(published.get("release_body_normalization") == "line_rstrip_and_outer_strip", "manifest normalization mismatch")
require(published.get("release_body_normalized_sha256") == "a0586ed49c0154ab1bae4ceff46dcf582139a1b0324efc379662f96a092ee219", "manifest normalized body hash mismatch")
require(published.get("tracked_release_notes_normalized_sha256") == "a0586ed49c0154ab1bae4ceff46dcf582139a1b0324efc379662f96a092ee219", "manifest normalized notes hash mismatch")
require(published.get("release_body_raw_sha256") == "fb4ee2773151b5685a6b89fd75da53d4c496116fdd3bf315f532525fc1ce6b00", "manifest raw body hash mismatch")
require(published.get("tracked_release_notes_raw_sha256") == "fb4ee2773151b5685a6b89fd75da53d4c496116fdd3bf315f532525fc1ce6b00", "manifest raw notes hash mismatch")
require(published.get("release_body_raw_sha256_is_acceptance_rule") is False, "manifest raw acceptance flag mismatch")

for text, label in ((v280_009, "V280-009"), (v281_001, "V281-001")):
    require("release body normalized sha256 = a0586ed49c0154ab1bae4ceff46dcf582139a1b0324efc379662f96a092ee219" in text, f"{label} normalized hash marker missing")
    require("raw hash equality is diagnostic, not the acceptance rule" in text, f"{label} diagnostic raw hash marker missing")

for marker in (
    "release_body_hash_semantics=normalized_sha256",
    "release_body_normalization=line_rstrip_and_outer_strip",
    "release_body_normalized_sha256_matches_tracked_release_notes=true",
    "release_body_raw_sha256_is_acceptance_rule=false",
):
    require(marker in guard, f"publication guard output marker missing: {marker}")

print(
    "v28_1_release_body_hash_normalization=pass "
    "semantics=normalized_sha256 "
    "trailing_newline_only_drift=ignored "
    "content_drift=fail_closed "
    "raw_sha256_acceptance_rule=false"
)
PY
