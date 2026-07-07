#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

MANIFEST_PATH="${NTPRO_V261_STALE_V260_MANIFEST:-docs/rust-cutover/release/v0_26_0_release_manifest.json}"

fail() {
  echo "v26.1 stale V260 evidence cleanup failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

targets=(
  docs/rust-cutover/evidence/V260-008.md
  docs/rust-cutover/evidence/V260-009.md
  docs/rust-cutover/evidence/V260-010.md
  docs/rust-cutover/evidence/V260-011.md
  docs/rust-cutover/evidence/V260-012.md
  docs/rust-cutover/evidence/V260-013.md
  docs/rust-cutover/release/v0_26_0_readiness_report.md
  docs/rust-cutover/release/v0_26_0_release_closeout_evidence.md
  docs/rust-cutover/release/v0_26_0_release_notes.md
)

for path in "$MANIFEST_PATH" verification.md "${targets[@]}"; do
  require_file "$path"
done

MANIFEST_PATH="$MANIFEST_PATH" \
python3 <<'PY'
import copy
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))

target_paths = [
    Path("docs/rust-cutover/evidence/V260-008.md"),
    Path("docs/rust-cutover/evidence/V260-009.md"),
    Path("docs/rust-cutover/evidence/V260-010.md"),
    Path("docs/rust-cutover/evidence/V260-011.md"),
    Path("docs/rust-cutover/evidence/V260-012.md"),
    Path("docs/rust-cutover/evidence/V260-013.md"),
    Path("docs/rust-cutover/release/v0_26_0_readiness_report.md"),
    Path("docs/rust-cutover/release/v0_26_0_release_closeout_evidence.md"),
    Path("docs/rust-cutover/release/v0_26_0_release_notes.md"),
]

stale_patterns = (
    "current_issue_state=OPEN",
    "final_scope_issues=9",
    "HOSTED GATE REQUIRED",
    "Pending final release validation",
    "EXPECTED REMOTE BODY DRIFT",
    "BLOCKED by transient",
    "hosted release gate remains required",
    "release-publish reruns",
    "LOCAL CORRECTIVE VALIDATION PARTIAL",
    "NTPRO_RELEASE_PUBLICATION_ALLOW_OFFLINE=1 scripts/ai/verify_release.sh v26-release-gates",
)

required_final_markers = {
    "docs/rust-cutover/evidence/V260-008.md": (
        "Status: FINAL RELEASE VALIDATION RECORDED",
        "hosted release gate = https://github.com/atxinbao/NTPRO/actions/runs/28853960135",
        "publish workflow = https://github.com/atxinbao/NTPRO/actions/runs/28858791493",
        "GitHub Release published at = 2026-07-07T05:29:16Z",
        "scripts/ai/verify_release.sh v26-release-gates = PASS, current_issue_state=CLOSED, final_scope_issues=14, corrective_scope=5, negative_selftest=1",
    ),
    "docs/rust-cutover/evidence/V260-012.md": (
        "Status: FINAL CORRECTIVE VALIDATION RECORDED",
        "hosted Rust Cutover Release Gate run `28853960135` completed with conclusion `success`",
        "release-publish.yml run `28858791493` completed with conclusion `success`",
        "scripts/ai/verify_release.sh v26-release-gates = PASS, current_issue_state=CLOSED, final_scope_issues=14, corrective_scope=5, negative_selftest=1",
    ),
    "docs/rust-cutover/evidence/V260-013.md": (
        "Final release validation:",
        "hosted release-publish.yml run `28858791493` completed with conclusion `success`",
        "final strict publication guard passes for `ntpro-rust-only-v0.26.0`",
        "public GitHub Release body matches `docs/rust-cutover/release/v0_26_0_release_notes.md`",
    ),
}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def v260_verification_slice(text: str) -> str:
    start = text.find("# V260-010 Verification")
    require(start >= 0, "verification.md missing V260-010 section")
    end = text.find("# V260-007 Verification", start)
    require(end > start, "verification.md missing V260-007 boundary")
    return text[start:end]


def validate_texts(texts: dict[str, str], verification_text: str, candidate_manifest: dict) -> None:
    for name, text in texts.items():
        for stale in stale_patterns:
            require(stale not in text, f"{name} contains stale pre-publication wording: {stale}")

    verification_window = v260_verification_slice(verification_text)
    for stale in stale_patterns:
        require(stale not in verification_window, f"verification.md V260 window contains stale wording: {stale}")
    require("final_scope_issues=14" in verification_window, "verification.md V260 window missing final scope 14")
    require("corrective_scope=5" in verification_window, "verification.md V260 window missing corrective scope 5")

    for name, markers in required_final_markers.items():
        text = texts[name]
        for marker in markers:
            require(marker in text, f"{name} missing final marker: {marker}")

    scope = candidate_manifest.get("release_scope") or {}
    require(scope.get("final_release_scope_issue_count") == 14, "manifest final scope issue count must be 14")
    require(scope.get("final_release_scope_evidence_count") == 14, "manifest final scope evidence count must be 14")
    require(scope.get("corrective_issue_count") == 5, "manifest corrective issue count must be 5")

    inputs = candidate_manifest.get("release_inputs") or {}
    require(
        inputs.get("v261_stale_v260_evidence_cleanup_path") == "scripts/ai/verify_v26_1_stale_v260_evidence_cleanup.sh",
        "manifest release input missing stale cleanup script",
    )
    commands = {
        gate.get("command")
        for gate in candidate_manifest.get("release_gates", [])
        if gate.get("required") is True
    }
    require(
        "scripts/ai/verify_v26_1_stale_v260_evidence_cleanup.sh" in commands,
        "manifest release gates missing stale cleanup script",
    )

    requirements = candidate_manifest.get("post_publication_requirements") or {}
    require(requirements.get("hosted_release_gate_success_required") is True, "manifest hosted gate requirement missing")
    require(requirements.get("strict_release_body_match_required") is True, "manifest strict body requirement missing")
    require(requirements.get("publication_after_hosted_gate_required") is True, "manifest post-gate publication requirement missing")


texts = {str(path): path.read_text(encoding="utf-8") for path in target_paths}
verification = Path("verification.md").read_text(encoding="utf-8")
validate_texts(texts, verification, manifest)

if os.environ.get("NTPRO_V261_STALE_V260_SELFTEST", "1") == "1":
    stale_open = dict(texts)
    stale_open["docs/rust-cutover/evidence/V260-008.md"] += "\nscripts/ai/verify_release.sh v26-release-gates = PASS, current_issue_state=OPEN, final_scope_issues=9, negative_selftest=1\n"
    try:
        validate_texts(stale_open, verification, manifest)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: stale V260-008 open/scope-9 wording")

    stale_pending = dict(texts)
    stale_pending["docs/rust-cutover/evidence/V260-012.md"] += "\nPending final release validation:\n"
    try:
        validate_texts(stale_pending, verification, manifest)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: stale V260-012 pending validation wording")

    stale_verification = verification.replace(
        "# V260-010 Verification\n",
        "# V260-010 Verification\nscripts/ai/verify_release.sh v26-release-gates = PASS, current_issue_state=CLOSED, final_scope_issues=9, negative_selftest=1\n",
        1,
    )
    try:
        validate_texts(texts, stale_verification, manifest)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: stale verification scope-9 wording")

    missing_gate = copy.deepcopy(manifest)
    missing_gate["release_gates"] = [
        gate
        for gate in missing_gate.get("release_gates", [])
        if gate.get("command") != "scripts/ai/verify_v26_1_stale_v260_evidence_cleanup.sh"
    ]
    try:
        validate_texts(texts, verification, missing_gate)
    except SystemExit:
        pass
    else:
        raise SystemExit("negative self-test unexpectedly passed: missing stale cleanup release gate")
PY

echo "v26_1_stale_v260_evidence_cleanup status=ok final_scope_issues=14 corrective_scope=5 gate_run=28853960135 publish_run=28858791493 negative_selftest=${NTPRO_V261_STALE_V260_SELFTEST:-1}"
