#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

BOUNDARY_PATH="${NTPRO_V231_EVIDENCE_BOUNDARY:-docs/rust-cutover/release/v0_23_0_evidence_replay_only_boundary.md}"
MANIFEST_PATH="${NTPRO_V231_EVIDENCE_MANIFEST:-docs/rust-cutover/release/v0_23_0_release_manifest.json}"

fail() {
  echo "v23.1 evidence replay only boundary failed: $*" >&2
  exit 1
}

require_file() {
  local path="$1"
  [[ -f "$path" ]] || fail "missing required file: $path"
}

require_contains() {
  local path="$1"
  local marker="$2"
  if ! grep -F -- "$marker" "$path" >/dev/null; then
    fail "missing marker in $path: $marker"
  fi
}

release_surface_files=(
  README.md
  ROADMAP.md
  docs/rust-cutover/release/README.md
  docs/rust-cutover/release/product_live_trading_roadmap.md
  docs/rust-cutover/release/v0_23_0_release_notes.md
  docs/rust-cutover/release/v0_23_0_readiness_report.md
  docs/rust-cutover/release/v0_23_0_release_manifest.json
  docs/rust-cutover/release/v0_23_0_release_closeout_evidence.md
  docs/rust-cutover/release/v0_23_0_evidence_replay_only_boundary.md
)

for path in "${release_surface_files[@]}"; do
  require_file "$path"
done
require_file "$BOUNDARY_PATH"
require_file "$MANIFEST_PATH"

for marker in \
  "v0.23.0 capability class = evidence / replay / readonly observability only" \
  "production multi-node runtime implementation = not included" \
  "runtime integrated multi-node execution = not included" \
  "v0.24.0 capability = future contract and gated implementation only" \
  "v0.24.0 runtime capability inherited from v0.23.0 = false"; do
  require_contains "$BOUNDARY_PATH" "$marker"
done

for marker in \
  "Capability class: patch closeout / evidence / replay / readonly observability only" \
  "not a production multi-node runtime implementation" \
  "v0.24.0 entry: future contract and gated implementation only"; do
  require_contains README.md "$marker"
done

for marker in \
  "capability class patch closeout / evidence / replay / readonly observability only" \
  "production multi-node runtime implementation not included" \
  "v0.24.0 future contract and gated implementation only"; do
  require_contains ROADMAP.md "$marker"
done

for marker in \
  "Evidence/replay-only claim language." \
  "production multi-node runtime = not included" \
  "runtime integrated multi-node execution = not included" \
  "v0.24.0 capability = future contract and gated implementation only" \
  "no v0.24.0 runtime capability is inherited automatically from v0.23.0"; do
  require_contains docs/rust-cutover/release/product_live_trading_roadmap.md "$marker"
done

for marker in \
  "capability class = evidence / replay / readonly observability only" \
  "production multi-node runtime implementation = not included" \
  "v0.24.0 entry = future contract and gated implementation only" \
  "scripts/ai/verify_release.sh v23.1-evidence-replay-only-boundary"; do
  require_contains docs/rust-cutover/release/v0_23_0_release_notes.md "$marker"
done

for marker in \
  "v23.1 evidence replay only boundary = required" \
  "capability_class = evidence / replay / readonly observability only" \
  "production_multi_node_runtime_implementation = false" \
  "v0.24.0_runtime_capability_inherited_from_v0.23.0 = false"; do
  require_contains docs/rust-cutover/release/v0_23_0_readiness_report.md "$marker"
done

MANIFEST_PATH="$MANIFEST_PATH" python3 <<'PY'
import json
import os
from pathlib import Path

manifest = json.loads(Path(os.environ["MANIFEST_PATH"]).read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


require(
    manifest.get("capability_class") == "evidence_replay_readonly_observability_only",
    "manifest capability_class mismatch",
)

runtime_claims = manifest.get("runtime_claims") or {}
for key in (
    "production_multi_node_runtime",
    "runtime_integrated_multi_node_execution",
    "runtime_implementation_complete",
    "product_grade_terminal_ready",
    "v24_inherits_runtime_capability_from_v23",
):
    require(runtime_claims.get(key) is False, f"runtime claim must be false: {key}")

next_tracks = manifest.get("next_tracks") or {}
require(
    next_tracks.get("capability_entry") == "future_contract_and_gated_implementation_only",
    "next v0.24.0 capability entry mismatch",
)
require(next_tracks.get("inherits_runtime_capability") is False, "v0.24.0 runtime inheritance must be false")

commands = {
    gate.get("command")
    for gate in manifest.get("release_gates", [])
    if gate.get("required") is True
}
require(
    "scripts/ai/verify_release.sh v23.1-evidence-replay-only-boundary" in commands,
    "missing v23.1 evidence replay only boundary gate command",
)
PY

SCAN_FILES_JSON="$(printf '%s\n' "${release_surface_files[@]}" | python3 -c 'import json,sys; print(json.dumps([line.strip() for line in sys.stdin if line.strip()]))')"
SCAN_FILES_JSON="$SCAN_FILES_JSON" python3 <<'PY'
import json
import os
from pathlib import Path

files = [Path(item) for item in json.loads(os.environ["SCAN_FILES_JSON"])]
forbidden = (
    "production multi-node runtime = included",
    "runtime integrated multi-node execution = true",
    "runtime implementation = complete",
    "product-grade terminal = ready",
    "product-grade live trading terminal = included",
    "v0.24.0 inherits runtime capability from v0.23.0",
    "v0.23.0 implements production multi-node runtime",
    "v0.23.0 is a product-grade live trading terminal",
)


def validate(paths: list[Path]) -> None:
    for path in paths:
        for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            lowered = line.lower()
            if "forbidden" in lowered or "negative self-test" in lowered:
                continue
            for phrase in forbidden:
                if phrase in lowered:
                    raise AssertionError(f"forbidden release overclaim in {path}:{lineno}: {phrase}")


validate(files)

if os.environ.get("NTPRO_V231_EVIDENCE_SELFTEST", "1") == "1":
    from tempfile import NamedTemporaryFile

    with NamedTemporaryFile("w", encoding="utf-8", delete=False) as tmp:
        tmp.write("v0.23.0 implements production multi-node runtime\n")
        tmp_path = Path(tmp.name)
    try:
        try:
            validate([tmp_path])
        except AssertionError:
            pass
        else:
            raise AssertionError("negative boundary overclaim self-test unexpectedly passed")
    finally:
        tmp_path.unlink(missing_ok=True)
PY

echo "v23_1_evidence_replay_only_boundary status=ok files=${#release_surface_files[@]} negative_selftest=${NTPRO_V231_EVIDENCE_SELFTEST:-1}"
