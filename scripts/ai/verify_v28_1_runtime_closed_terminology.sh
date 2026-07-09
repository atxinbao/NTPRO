#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

TERMINOLOGY_DOC="docs/rust-cutover/release/v0_28_1_runtime_closed_terminology.md"
CONTRACT_DOC="docs/rust-cutover/release/v0_28_0_backend_closure_boundary_contract.md"
MATRIX_PATH="docs/rust-cutover/release/v0_28_0_backend_closure_readiness_matrix.json"
READINESS_REPORT="docs/rust-cutover/release/v0_28_0_readiness_report.md"
RELEASE_README="docs/rust-cutover/release/README.md"
TASK_PATH="docs/rust-cutover/tasks/V281-005.md"
EVIDENCE_PATH="docs/rust-cutover/evidence/V281-005.md"
V280_001_EVIDENCE="docs/rust-cutover/evidence/V280-001.md"
V280_009_EVIDENCE="docs/rust-cutover/evidence/V280-009.md"

for path in \
  "$TERMINOLOGY_DOC" \
  "$CONTRACT_DOC" \
  "$MATRIX_PATH" \
  "$READINESS_REPORT" \
  "$RELEASE_README" \
  "$TASK_PATH" \
  "$EVIDENCE_PATH" \
  "$V280_001_EVIDENCE" \
  "$V280_009_EVIDENCE"; do
  [[ -f "$path" ]] || {
    echo "v28.1 runtime-closed terminology failed: missing required file: $path" >&2
    exit 1
  }
done

NTPRO_V280_BACKEND_CLOSURE_SELFTEST=1 scripts/ai/verify_v28_backend_closure_boundary_contract.sh

python3 <<'PY'
from __future__ import annotations

import copy
import json
from pathlib import Path
from typing import Any

TERMINOLOGY_DOC = Path("docs/rust-cutover/release/v0_28_1_runtime_closed_terminology.md")
CONTRACT_DOC = Path("docs/rust-cutover/release/v0_28_0_backend_closure_boundary_contract.md")
MATRIX_PATH = Path("docs/rust-cutover/release/v0_28_0_backend_closure_readiness_matrix.json")
READINESS_REPORT = Path("docs/rust-cutover/release/v0_28_0_readiness_report.md")
RELEASE_README = Path("docs/rust-cutover/release/README.md")
TASK_PATH = Path("docs/rust-cutover/tasks/V281-005.md")
EVIDENCE_PATH = Path("docs/rust-cutover/evidence/V281-005.md")
V280_001_EVIDENCE = Path("docs/rust-cutover/evidence/V280-001.md")
V280_009_EVIDENCE = Path("docs/rust-cutover/evidence/V280-009.md")

REQUIRED_FALSE_CLAIM_FLAGS = (
    "backend_service_runtime_claim_allowed",
    "live_external_integration_claim_allowed",
    "production_execution_runtime_claim_allowed",
    "product_ready_claim_allowed",
)
FORBIDDEN_POSITIVE_CLAIM_PHRASES = (
    "live external idp",
    "live deployment execution",
    "live adapter send",
    "production trading runtime",
    "product-ready live trading",
    "backend service runtime integration",
)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def require_markers(path: Path, markers: tuple[str, ...]) -> None:
    text = path.read_text(encoding="utf-8")
    for marker in markers:
        require(marker in text, f"missing marker in {path}: {marker}")


def validate_matrix(candidate: dict[str, Any]) -> None:
    terminology = candidate.get("terminology")
    require(isinstance(terminology, dict), "matrix terminology must be an object")
    require(
        terminology.get("runtime_closed_meaning") == "deterministic_artifact_replay_closure",
        "runtime_closed_meaning mismatch",
    )
    require(
        terminology.get("runtime_closed_label") == "runtime-closed (artifact replay)",
        "runtime_closed_label mismatch",
    )
    does_not_mean = terminology.get("runtime_closed_does_not_mean")
    require(isinstance(does_not_mean, list), "runtime_closed_does_not_mean must be a list")
    for value in (
        "backend_service_runtime",
        "live_external_integration",
        "production_execution_runtime",
        "product_ready_live_trading_runtime",
    ):
        require(value in does_not_mean, f"runtime_closed_does_not_mean missing {value}")
    for flag in REQUIRED_FALSE_CLAIM_FLAGS:
        require(terminology.get(flag) is False, f"terminology flag must be false: {flag}")

    modules = candidate.get("module_readiness")
    require(isinstance(modules, list), "module_readiness must be a list")
    runtime_closed = [item for item in modules if item.get("classification") == "runtime-closed"]
    require(len(runtime_closed) == 10, f"runtime-closed module count mismatch: {len(runtime_closed)}")
    for item in runtime_closed:
        module_id = item.get("module_id")
        require(item.get("closure_mode") == "deterministic_artifact_replay", f"closure_mode mismatch: {module_id}")
        for flag in REQUIRED_FALSE_CLAIM_FLAGS:
            require(item.get(flag) is False, f"{module_id} flag must be false: {flag}")
        claim = item.get("claim")
        require(isinstance(claim, str) and claim, f"claim missing: {module_id}")
        lowered = claim.lower()
        for phrase in FORBIDDEN_POSITIVE_CLAIM_PHRASES:
            require(phrase not in lowered, f"forbidden positive claim in {module_id}: {phrase}")


require_markers(
    TERMINOLOGY_DOC,
    (
        "deterministic_artifact_replay_closure = source-controlled artifacts plus local deterministic replay and release-gate validation",
        "backend_service_runtime = running backend service/process/API integration that owns live operational state",
        "live_external_integration = real external provider, IdP/SSO, deployment platform, adapter, exchange, or network dependency",
        "production_execution_runtime = production environment where submit, mutation, adapter send, exchange request, or trading controls can affect live state",
        "runtime_closed_terminology = deterministic_artifact_replay_closure_only",
        "closure_mode = deterministic_artifact_replay",
        "backend_service_runtime_claim_allowed = false",
        "live_external_integration_claim_allowed = false",
        "production_execution_runtime_claim_allowed = false",
        "product_ready_claim_allowed = false",
        "live external IdP",
        "live deployment execution",
        "live adapter send",
        "production trading runtime",
    ),
)
require_markers(
    CONTRACT_DOC,
    (
        "runtime_closed_terminology = deterministic_artifact_replay_closure_only",
        "deterministic_artifact_replay_closure = source-controlled artifacts plus deterministic local replay and release-gate validation",
        "backend_service_runtime_claim = false",
        "live_external_integration_claim = false",
        "runtime-closed (artifact replay)",
        "runtime-closed live-service/product-ready positive claim => fail_closed_boundary_violation",
    ),
)
require_markers(
    READINESS_REPORT,
    (
        "runtime_closed_terminology = deterministic_artifact_replay_closure_only",
        "runtime_closed_label = runtime-closed (artifact replay)",
        "backend_service_runtime_claim_allowed = false",
        "live_external_integration_claim_allowed = false",
        "production_execution_runtime_claim_allowed = false",
        "product_ready_claim_allowed = false",
    ),
)
require_markers(
    RELEASE_README,
    (
        "v0_28_1_runtime_closed_terminology.md",
        "runtime-closed (artifact replay)",
        "../evidence/V281-005.md",
    ),
)
for path in (TASK_PATH, EVIDENCE_PATH):
    require_markers(
        path,
        (
            "Task: `V281-005`" if path == EVIDENCE_PATH else "GitHub issue: `#923`",
            "runtime-closed",
            "deterministic_artifact_replay",
            "production trading runtime",
        ),
    )
for path in (V280_001_EVIDENCE, V280_009_EVIDENCE):
    require_markers(
        path,
        (
            "runtime_closed_terminology = deterministic_artifact_replay_closure_only",
            "runtime_closed_label = runtime-closed (artifact replay)",
        ),
    )
for path in list(Path("docs/rust-cutover/release").glob("v0_28_0_*.md")) + list(
    Path("docs/rust-cutover/evidence").glob("V280-00[2-8].md")
):
    text = path.read_text(encoding="utf-8")
    if "matrix classification = runtime-closed" in text:
        require("closure_mode = deterministic_artifact_replay" in text, f"missing closure_mode marker in {path}")
        require("runtime_closed_label = runtime-closed (artifact replay)" in text, f"missing runtime_closed_label marker in {path}")

matrix = json.loads(MATRIX_PATH.read_text(encoding="utf-8"))
validate_matrix(matrix)

for phrase in FORBIDDEN_POSITIVE_CLAIM_PHRASES:
    mutated = copy.deepcopy(matrix)
    for item in mutated["module_readiness"]:
        if item.get("classification") == "runtime-closed":
            item["claim"] = f"{phrase} is complete"
            break
    try:
        validate_matrix(mutated)
    except SystemExit:
        pass
    else:
        raise SystemExit(f"negative self-test unexpectedly allowed phrase: {phrase}")

missing_mode = copy.deepcopy(matrix)
for item in missing_mode["module_readiness"]:
    if item.get("classification") == "runtime-closed":
        item.pop("closure_mode", None)
        break
try:
    validate_matrix(missing_mode)
except SystemExit:
    pass
else:
    raise SystemExit("negative self-test unexpectedly allowed missing closure_mode")

print(
    "v28_1_runtime_closed_terminology=pass "
    "runtime_closed_meaning=deterministic_artifact_replay_closure "
    "runtime_closed_modules=10 "
    "forbidden_positive_claim_selftest=pass"
)
PY
