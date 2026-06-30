#!/usr/bin/env bash
set -euo pipefail

# V200-012: v0.20 aggregate release gates.
# 聚合 owner-approved production order lifecycle foundation 证据链，并保持默认本地/offline。
# 该 gate 不打开生产网络，不允许隐式 retry、自动撤单、自动补救或 Dashboard 写操作控件。

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh
export NTPRO_SOURCE_COMMIT="${NTPRO_SOURCE_COMMIT:-$(git rev-parse HEAD)}"
export NTPRO_SOURCE_RELEASE_TAG="${NTPRO_SOURCE_RELEASE_TAG:-unreleased-v20-local-gate}"

if [[ "${NTPRO_V20_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V20_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --release --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V20_NAUTILUS_BIN:-$ROOT_DIR/target/release/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi
if [[ "$NAUTILUS_BIN" != "$ROOT_DIR/target/release/nautilus" && "$NAUTILUS_BIN" != */target/release/nautilus ]]; then
  if [[ "${NTPRO_V20_ALLOW_LOCAL_SMOKE_ONLY:-0}" != "1" ]]; then
    echo "v20 release gate requires release binary evidence: $NAUTILUS_BIN" >&2
    echo "use scripts/ai/verify_release.sh v20-release-gates or pass target/release/nautilus" >&2
    echo "set NTPRO_V20_ALLOW_LOCAL_SMOKE_ONLY=1 only for explicit local smoke runs" >&2
    exit 1
  fi
  echo "local smoke only: non-release nautilus binary: $NAUTILUS_BIN"
  echo "local smoke only: this run is not release binary evidence"
else
  echo "release binary: $NAUTILUS_BIN"
fi

export NTPRO_V20_SKIP_BUILD=1
export NTPRO_V20_NAUTILUS_BIN="$NAUTILUS_BIN"

unset NTPRO_ALLOW_PRODUCTION_LIVE_ALPHA_MUTATION
unset NTPRO_ALLOW_PRODUCTION_ORDER_SUBMISSION
unset NTPRO_ALLOW_PRODUCTION_ORDER_MUTATION
unset NTPRO_ALLOW_PRODUCTION_MUTATION_SIGNING_MATERIAL
unset NTPRO_ALLOW_PRODUCTION_MUTATION_HTTP_SEND
unset NTPRO_ALLOW_PRODUCTION_ORDER_STATE_READ
unset NTPRO_ALLOW_DASHBOARD_ORDER_CONTROLS
unset NTPRO_ALLOW_DASHBOARD_APPROVAL_CONTROLS
unset NTPRO_ALLOW_DASHBOARD_CANCEL_CONTROLS
unset BINANCE_API_KEY
unset BINANCE_API_SECRET
unset BINANCE_PRODUCTION_API_KEY
unset BINANCE_PRODUCTION_API_SECRET

GATE_ROOT="${NTPRO_V20_RELEASE_GATE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v20-release-gates.XXXXXX")}"
LOG_DIR="$GATE_ROOT/logs"
mkdir -p "$LOG_DIR"

run_gate() {
  local name="$1"
  shift
  local log="$LOG_DIR/$name.log"
  echo "== v20 release gates: $name =="
  "$@" 2>&1 | tee "$log"
}

run_gate pre-submit-risk-gate \
  cargo test -p nautilus-risk --test v20_pre_submit_gate -- --nocapture
run_gate owner-approval-lifecycle \
  cargo test -p nautilus-risk --test v20_owner_approval -- --nocapture
run_gate signing-material-gate \
  cargo test -p nautilus-risk --test v20_signing_material_gate -- --nocapture
run_gate submit-request-builder \
  cargo test -p nautilus-risk --test v20_submit_request_builder -- --nocapture
run_gate guarded-submit-candidate \
  cargo test -p nautilus-risk --test v20_submit_candidate -- --nocapture
run_gate order-lifecycle-golden-traces \
  scripts/ai/verify_v20_order_lifecycle_golden_traces.sh
run_gate golden-trace-release-scope \
  python3 scripts/ai/validate_golden_trace_release_scope.py \
    --manifest docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json \
    --trace-glob 'tests/golden/*.jsonl'

python3 - "$GATE_ROOT" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
violations: list[str] = []

required_docs = [
    "docs/rust-cutover/scope/v0_20_0_owner_approved_production_order_lifecycle_foundation.md",
    "docs/rust-cutover/release/v0_20_0_order_lifecycle_safety_contract.md",
    "docs/rust-cutover/release/v0_20_0_pre_submit_risk_gate.md",
    "docs/rust-cutover/release/v0_20_0_owner_approval_lifecycle.md",
    "docs/rust-cutover/release/v0_20_0_signing_material_env_gate.md",
    "docs/rust-cutover/release/v0_20_0_single_shot_submit_request_builder.md",
    "docs/rust-cutover/release/v0_20_0_guarded_single_shot_submit_candidate.md",
    "docs/rust-cutover/release/v0_20_0_submit_response_redaction.md",
    "docs/rust-cutover/release/v0_20_0_submit_readback_reconciliation.md",
    "docs/rust-cutover/release/v0_20_0_failure_no_retry_evidence.md",
    "docs/rust-cutover/release/v0_20_0_dashboard_order_lifecycle_audit.md",
    "docs/rust-cutover/release/v0_20_0_order_lifecycle_golden_traces.md",
    "docs/rust-cutover/release/v0_20_0_release_notes.md",
    "docs/rust-cutover/release/v0_20_0_readiness_report.md",
    "docs/rust-cutover/release/v0_20_0_release_manifest.json",
]
required_docs.extend(f"docs/rust-cutover/evidence/V200-{index:03d}.md" for index in range(0, 12))

for doc in required_docs:
    if not Path(doc).is_file():
        violations.append(f"missing v0.20 release gate document: {doc}")

rows = []
trace_path = Path("tests/golden/production_order_lifecycle_schema.jsonl")
if trace_path.is_file():
    for raw in trace_path.read_text(encoding="utf-8").splitlines():
        if raw.strip():
            rows.append(json.loads(raw))
else:
    violations.append("missing production order lifecycle golden trace")

required_scenarios = {
    "pre_submit_blocked_missing_approval",
    "accepted_readback_matched_audit_closed",
    "venue_rejected_failure_no_retry",
    "unknown_response_failure_no_retry",
    "readback_mismatch_failure_no_retry",
    "readback_missing_failure_no_retry",
}
seen = set()
false_fields = {
    "credential_plaintext_recorded",
    "raw_response_recorded",
    "raw_readback_body_recorded",
    "credential_material_recorded",
    "signature_material_recorded",
    "token_value_recorded",
    "signed_query_recorded",
    "signed_url_recorded",
    "retry_attempted",
    "duplicate_submit_attempted",
    "second_submit_attempted",
    "replace_attempted",
    "amend_attempted",
    "flatten_attempted",
    "automatic_cancel_attempted",
    "automatic_remediation_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_approval_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "dashboard_retry_controls_enabled",
    "network_replay_required",
    "live_broker_required",
}
for row in rows:
    for section in ("input", "expected"):
        payload = row[section]["events"][0]["payload"]
        runtime_provenance = payload.get("runtime_release_provenance") or {}
        if runtime_provenance.get("release_tag") != "ntpro-rust-only-v0.20.0":
            violations.append(f"{payload.get('scenario')}: {section} runtime release tag is not v20")
        if runtime_provenance.get("release_gate") != "v20-release-gates":
            violations.append(f"{payload.get('scenario')}: {section} runtime release gate is not v20")
        if runtime_provenance.get("strict_provenance") is not True:
            violations.append(f"{payload.get('scenario')}: {section} strict provenance is not true")
    payload = row["expected"]["events"][0]["payload"]
    scenario = payload.get("scenario")
    seen.add(scenario)
    refs = payload.get("refs") or {}
    for key in ("candidate_ref", "response_ref", "readback_ref", "failure_ref", "audit_ref", "dashboard_ref", "provenance_ref"):
        if not refs.get(key):
            violations.append(f"{scenario}: missing {key}")
    if payload.get("no_implicit_retry") is not True:
        violations.append(f"{scenario}: no_implicit_retry not true")
    if payload.get("retry_allowed") is not False:
        violations.append(f"{scenario}: retry_allowed not false")
    if payload.get("retry_attempts") != 0:
        violations.append(f"{scenario}: retry_attempts not zero")
    for key in false_fields:
        if payload.get(key) is not False:
            violations.append(f"{scenario}: {key} is not false")

missing = required_scenarios - seen
if missing:
    violations.append("missing v20 golden trace scenarios: " + ", ".join(sorted(missing)))

manifest_path = Path("docs/rust-cutover/release/v0_20_0_release_manifest.json")
if manifest_path.is_file():
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("schema_version") != "ntpro.v200_release_manifest.v1":
        violations.append("v20 release manifest schema mismatch")
    if manifest.get("task_id") != "V200-012":
        violations.append("v20 release manifest task_id mismatch")
    gates = {gate.get("command") for gate in manifest.get("release_gates", []) if gate.get("required") is True}
    for command in (
        "scripts/ai/verify_release.sh v20-release-gates",
        "scripts/ai/verify_release.sh v20-strict-provenance",
        "scripts/ai/verify_release_strict.sh v20",
    ):
        if command not in gates:
            violations.append(f"v20 release manifest missing gate: {command}")
    boundary = manifest.get("boundary_flags") or {}
    for key in (
        "implicit_retry_allowed",
        "automatic_cancel_allowed",
        "automatic_remediation_allowed",
        "dashboard_order_controls_enabled",
        "dashboard_approval_controls_enabled",
        "dashboard_cancel_controls_enabled",
        "strategy_driven_production_execution_allowed",
        "general_production_trading_platform_claim",
    ):
        if boundary.get(key) is not False:
            violations.append(f"v20 release manifest boundary flag must be false: {key}")

notes = Path("docs/rust-cutover/release/v0_20_0_release_notes.md")
readiness = Path("docs/rust-cutover/release/v0_20_0_readiness_report.md")
combined = ""
if notes.is_file():
    combined += notes.read_text(encoding="utf-8")
if readiness.is_file():
    combined += "\n" + readiness.read_text(encoding="utf-8")
for marker in (
    "Owner-Approved Production Order Lifecycle Foundation",
    "owner approval = required",
    "single submit attempt = required",
    "post-submit readback = required",
    "failure/no-retry evidence = required",
    "Dashboard order controls = not included",
    "implicit retry = not included",
    "strategy-driven production execution = not included",
    "general production trading platform claim = not included",
):
    if marker not in combined:
        violations.append(f"missing v0.20 release boundary marker: {marker}")

regression_markers = {
    "crates/risk/src/v20_pre_submit_gate.rs": (
        "v200_pre_submit_notional_mismatch",
        "computed_notional",
        "notional_consistency_required",
    ),
    "crates/risk/src/v20_submit_request_builder.rs": (
        "v200_submit_request_notional_mismatch",
        "candidate_notional_matches_risk",
    ),
    "crates/risk/tests/v20_pre_submit_gate.rs": (
        "denies_underreported_notional_before_max_notional_bypass",
        "denies_overreported_notional_mismatch",
        "allows_exact_boundary_precision_notional",
    ),
    "crates/risk/tests/v20_submit_request_builder.rs": (
        "rejects_candidate_notional_mismatch_after_risk_match",
        "rejects_allow_evidence_without_notional_consistency",
    ),
    "docs/rust-cutover/release/v0_20_0_pre_submit_risk_gate.md": (
        "v200_pre_submit_notional_mismatch",
        "quantity * price == notional",
    ),
    "docs/rust-cutover/release/v0_20_0_single_shot_submit_request_builder.md": (
        "v200_submit_request_notional_mismatch",
        "notional_consistent = true",
    ),
    "crates/risk/src/v20_submit_response_redaction.rs": (
        "SubmitEvidenceSource",
        "v200_submit_response_source_claim_mismatch",
        "source_provenance_id",
    ),
    "crates/risk/src/v20_submit_readback_reconciliation.rs": (
        "v200_submit_readback_source_claim_mismatch",
        "readback_source_claim_consistent",
    ),
    "crates/cli/src/dashboard.rs": (
        "foundation_only_manual_structured",
        "foundation_boundary_status",
        "foundation_only_no_adapter_runtime",
        "adapter_runtime_claim_mismatch",
        "production_order_lifecycle_audit_source_mismatch",
        "production_order_lifecycle_audit_foundation_boundary",
        "Source class",
        "Foundation boundary",
    ),
    "docs/rust-cutover/release/v0_20_0_submit_response_redaction.md": (
        "manual_structured must not claim exchange truth",
        "v200_submit_response_source_claim_mismatch",
    ),
    "docs/rust-cutover/release/v0_20_0_submit_readback_reconciliation.md": (
        "manual_structured must not claim exchange truth",
        "v200_submit_readback_source_claim_mismatch",
    ),
    "docs/rust-cutover/release/v0_20_0_dashboard_order_lifecycle_audit.md": (
        "foundation_only_manual_structured",
        "foundation_only_no_adapter_runtime",
        "adapter runtime claim mismatch",
        "not trader terminal readiness",
    ),
}
for path, markers in regression_markers.items():
    file_path = Path(path)
    if not file_path.is_file():
        violations.append(f"missing v20 notional consistency artifact: {path}")
        continue
    text = file_path.read_text(encoding="utf-8")
    for marker in markers:
        if marker not in text:
            violations.append(f"missing v20 notional consistency marker in {path}: {marker}")

if violations:
    print("v20 release gate validation failed:", file=sys.stderr)
    for violation in violations:
        print(violation, file=sys.stderr)
    raise SystemExit(1)

print(f"v20_release_gate_artifact_scan checked_docs={len(required_docs)} trace_cases={len(rows)} root={root}")
PY

echo "v20_release_gates status=ok root=$GATE_ROOT owner_approved_production_order_lifecycle_foundation=true pre_submit_gate_required=true owner_approval_required=true signing_material_gate_required=true single_submit_attempt_required=true response_redaction_required=true post_submit_readback_required=true failure_no_retry_required=true dashboard_audit_read_only=true foundation_boundary_status=foundation_only_no_adapter_runtime golden_traces_checked=true implicit_retry_allowed=false automatic_cancel_allowed=false automatic_remediation_allowed=false dashboard_order_controls_enabled=false strategy_driven_production_execution_allowed=false general_production_trading_platform_claim=false"
