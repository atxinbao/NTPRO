#!/usr/bin/env bash
set -euo pipefail

# V141-001: validate owner-run production order-state read-only evidence.
#
# Default mode is offline and validates a synthetic, redacted proof package so
# PR/release gates can exercise the artifact contract without production
# credentials or network. Owner-run mode is enabled by setting:
#
#   NTPRO_V141_ORDER_STATE_EVIDENCE_ROOT=/path/to/proof-root
#
# That directory must contain a v0.14 production order-state read-only proof JSON
# artifact for `GET /api/v3/openOrders`. A `GET /api/v3/order` proof artifact is
# optional. The validator accepts successful owner reads and stable classified
# owner-read failures, but it never treats a failure as exchange-truth data.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

PROOF_ROOT="${NTPRO_V141_ORDER_STATE_EVIDENCE_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v141-order-state-owner-evidence.XXXXXX")}"
MANIFEST_PATH="${NTPRO_V141_ORDER_STATE_EVIDENCE_MANIFEST:-$PROOF_ROOT/owner-order-state-evidence-manifest.json}"
MODE="owner_run"

if [[ -z "${NTPRO_V141_ORDER_STATE_EVIDENCE_ROOT:-}" ]]; then
  MODE="offline_fixture"
  mkdir -p "$PROOF_ROOT"
  python3 - "$PROOF_ROOT/open-orders.json" <<'PY'
import json
import sys
from pathlib import Path

artifact = {
    "schema_version": "ntpro.v140_production_order_state_readonly_proof.v1",
    "status": "online_order_state_read_ok",
    "endpoint": "open_orders",
    "endpoint_class": "production_order_state_read_only",
    "http_base_url": "https://api.binance.com",
    "method": "GET",
    "path": "/api/v3/openOrders",
    "request_url_redacted": "https://api.binance.com/api/v3/openOrders",
    "query_shape": "symbol,timestamp,recvWindow,signature(redacted)",
    "symbol": "BTCUSDT",
    "order_id_provided": False,
    "orig_client_order_id_provided": False,
    "requires_api_key": True,
    "requires_signature": True,
    "endpoint_read_allowed": True,
    "offline_contract_ready": False,
    "read_allowed": False,
    "contract_ready": False,
    "online_read_allowed": True,
    "mutation_allowed": False,
    "owner_gate_required": True,
    "manual_gate_required": True,
    "missing_cli_flags": [],
    "missing_env_vars": [],
    "manual_online_requested": True,
    "online_execution_supported": True,
    "network_attempted": True,
    "response_status_code": 200,
    "response_shape": "binance_open_orders_v1",
    "response_shape_validated": True,
    "endpoint_shape_validated": True,
    "order_entries_observed": 0,
    "non_empty_order_state_observed": False,
    "order_lifecycle_readiness": False,
    "response_shape_summary": {
        "status": "accepted",
        "endpoint": "open_orders",
        "root_is_array": True,
        "root_is_object": False,
        "order_entry_count": 0,
        "symbol_present": True,
        "symbol_is_string": True,
        "order_id_present": True,
        "status_present": True,
        "status_is_string": True,
        "raw_order_response_recorded": False,
        "raw_order_list_recorded": False,
        "shape_validated": True,
        "endpoint_shape_validated": True,
        "order_entries_observed": 0,
        "non_empty_order_state_observed": False,
        "order_lifecycle_readiness": False,
        "rejection_reason": "none",
    },
    "latency_ms": 1,
    "error_code": "none",
    "env_credentials_only": True,
    "api_key_env": "BINANCE_PRODUCTION_READONLY_API_KEY",
    "api_secret_env": "BINANCE_PRODUCTION_READONLY_API_SECRET",
    "api_key_present": True,
    "api_secret_present": True,
    "api_key_value_recorded": False,
    "api_secret_value_recorded": False,
    "signature_recorded": False,
    "signed_query_recorded": False,
    "signed_url_recorded": False,
    "order_state_read_attempted": True,
    "production_order_state_reads_attempted": 1,
    "production_order_submission_attempted": False,
    "production_order_mutation_attempted": False,
    "cancel_replace_amend_attempted": False,
    "listen_key_lifecycle_attempted": False,
    "dashboard_order_controls_enabled": False,
    "automatic_remediation_attempted": False,
    "real_orders_submitted": False,
    "production_trading_enabled": False,
    "order_state_values_are_exchange_truth": True,
    "shadow_values_are_exchange_truth": False,
    "portfolio_values_are_exchange_truth": False,
    "values_are_exchange_truth": True,
    "secrets_redacted": True,
    "diagnostic": "synthetic_v141_validator_fixture_not_owner_run",
}

Path(sys.argv[1]).write_text(json.dumps(artifact, indent=2, sort_keys=True) + "\n")
PY
fi

python3 - "$PROOF_ROOT" "$MANIFEST_PATH" "$MODE" <<'PY'
import json
import os
import sys
from pathlib import Path

proof_root = Path(sys.argv[1])
manifest_path = Path(sys.argv[2])
mode = sys.argv[3]

schema_version = "ntpro.v141_order_state_owner_evidence_manifest.v1"
proof_schema_version = "ntpro.v140_production_order_state_readonly_proof.v1"
stable_errors = {
    "http_status_not_success",
    "timeout",
    "connect_error",
    "decode_error",
    "request_error",
    "body_error",
    "unknown_http_error",
    "response_shape_invalid",
}
forbidden_true_fields = [
    "production_order_submission_attempted",
    "production_order_mutation_attempted",
    "cancel_replace_amend_attempted",
    "listen_key_lifecycle_attempted",
    "dashboard_order_controls_enabled",
    "automatic_remediation_attempted",
    "real_orders_submitted",
    "production_trading_enabled",
]
redaction_false_fields = [
    "api_key_value_recorded",
    "api_secret_value_recorded",
    "signature_recorded",
    "signed_query_recorded",
    "signed_url_recorded",
]


def fail(message, evidence=None):
    payload = {"error": message}
    if evidence is not None:
        payload["evidence"] = evidence
    raise SystemExit(json.dumps(payload, indent=2, sort_keys=True))


def require(condition, message, evidence=None):
    if not condition:
        fail(message, evidence)


def load_json_files(root):
    require(root.exists(), "evidence root does not exist", str(root))
    require(root.is_dir(), "evidence root must be a directory", str(root))
    files = []
    for path in sorted(root.rglob("*.json")):
        if path.resolve() == manifest_path.resolve():
            continue
        try:
            data = json.loads(path.read_text())
        except json.JSONDecodeError:
            continue
        if data.get("schema_version") == proof_schema_version:
            files.append((path, data))
    return files


def require_false(report, field, path):
    require(report.get(field) is False, f"{field} must be false", str(path))


def validate_shape_summary(report, path):
    summary = report.get("response_shape_summary")
    require(isinstance(summary, dict), "response_shape_summary must be an object", str(path))
    endpoint = report.get("endpoint")
    require(summary.get("endpoint") == endpoint, "shape summary endpoint mismatch", str(path))
    require(
        summary.get("raw_order_list_recorded") is False,
        "raw order list must not be recorded",
        str(path),
    )
    require(
        summary.get("raw_order_response_recorded") is False,
        "raw order response must not be recorded",
        str(path),
    )
    if report.get("status") == "online_order_state_read_ok":
        require(summary.get("shape_validated") is True, "success needs validated shape", str(path))
        require(summary.get("endpoint_shape_validated") is True, "success needs validated endpoint shape", str(path))
        require(
            report.get("endpoint_shape_validated") is True,
            "success needs top-level endpoint_shape_validated=true",
            str(path),
        )
        require(summary.get("status") == "accepted", "success needs accepted summary", str(path))
        entries = summary.get("order_entries_observed")
        require(isinstance(entries, int), "summary order_entries_observed must be an integer", str(path))
        require(
            report.get("order_entries_observed") == entries,
            "top-level order_entries_observed must match summary",
            str(path),
        )
        if entries == 0:
            require(
                summary.get("non_empty_order_state_observed") is False,
                "empty order-state response must not claim non-empty observation",
                str(path),
            )
            require(
                summary.get("order_lifecycle_readiness") is False,
                "empty order-state response must not claim lifecycle readiness",
                str(path),
            )
        else:
            require(
                summary.get("non_empty_order_state_observed") is True,
                "non-empty order-state response must mark non-empty observation",
                str(path),
            )
            require(
                summary.get("order_lifecycle_readiness") is True,
                "non-empty order-state response should mark lifecycle readiness",
                str(path),
            )
        require(
            report.get("non_empty_order_state_observed") == summary.get("non_empty_order_state_observed"),
            "top-level non_empty_order_state_observed must match summary",
            str(path),
        )
        require(
            report.get("order_lifecycle_readiness") == summary.get("order_lifecycle_readiness"),
            "top-level order_lifecycle_readiness must match summary",
            str(path),
        )
    return summary


def validate_owner_artifact(path, report):
    endpoint = report.get("endpoint")
    expected_path = {
        "open_orders": "/api/v3/openOrders",
        "order": "/api/v3/order",
    }.get(endpoint)
    require(expected_path is not None, "unsupported order-state endpoint", str(path))
    require(report.get("endpoint_class") == "production_order_state_read_only", "endpoint class mismatch", str(path))
    require(report.get("method") == "GET", "method must be GET", str(path))
    require(report.get("path") == expected_path, "path mismatch", str(path))
    require(report.get("requires_api_key") is True, "API key requirement must be true", str(path))
    require(report.get("requires_signature") is True, "signature requirement must be true", str(path))
    require(report.get("endpoint_read_allowed") is True, "read allow marker missing", str(path))
    require(report.get("manual_online_requested") is True, "manual online marker missing", str(path))
    require(report.get("online_execution_supported") is True, "online support marker missing", str(path))
    require(report.get("online_read_allowed") is True, "online read marker missing", str(path))
    require(report.get("network_attempted") is True, "owner evidence must record network_attempted=true", str(path))
    require(report.get("order_state_read_attempted") is True, "order-state read marker missing", str(path))
    require(report.get("production_order_state_reads_attempted") == 1, "order-state read counter must be one", str(path))
    require(report.get("env_credentials_only") is True, "credentials must be env-only", str(path))
    require(report.get("secrets_redacted") is True, "secrets redaction marker missing", str(path))
    require(report.get("mutation_allowed") is False, "mutation_allowed must be false", str(path))
    require(report.get("read_allowed") is False, "online artifact must not claim offline read_allowed", str(path))
    require(report.get("contract_ready") is False, "online artifact must not claim offline contract_ready", str(path))
    require(report.get("request_url_redacted"), "redacted request URL is required", str(path))
    require("signature=" not in str(report.get("request_url_redacted")), "redacted request URL must not contain signature", str(path))

    for field in forbidden_true_fields:
        require_false(report, field, path)
    for field in redaction_false_fields:
        require_false(report, field, path)

    if endpoint == "order":
        require(
            report.get("order_id_provided") is True or report.get("orig_client_order_id_provided") is True,
            "single order evidence needs orderId or origClientOrderId",
            str(path),
        )

    status = report.get("status")
    if status == "online_order_state_read_ok":
        require(report.get("error_code") == "none", "success must use error_code=none", str(path))
        require(report.get("response_shape_validated") is True, "success must validate response shape", str(path))
        require(
            report.get("order_state_values_are_exchange_truth") is True,
            "success must mark only order-state exchange truth",
            str(path),
        )
        require(
            report.get("shadow_values_are_exchange_truth") is False,
            "success must not mark shadow exchange truth",
            str(path),
        )
        require(
            report.get("portfolio_values_are_exchange_truth") is False,
            "success must not mark portfolio exchange truth",
            str(path),
        )
        require(report.get("values_are_exchange_truth") is True, "success must mark exchange truth", str(path))
        exchange_truth = True
    elif status == "online_order_state_read_failed":
        require(report.get("error_code") in stable_errors, "failure must use a stable error code", str(path))
        require(report.get("response_shape_validated") is False, "failure must not validate response shape", str(path))
        require(report.get("endpoint_shape_validated") is False, "failure must not validate endpoint shape", str(path))
        require(report.get("order_entries_observed") == 0, "failure must not observe order entries", str(path))
        require(report.get("non_empty_order_state_observed") is False, "failure must not mark non-empty order state", str(path))
        require(report.get("order_lifecycle_readiness") is False, "failure must not mark order lifecycle readiness", str(path))
        require(
            report.get("order_state_values_are_exchange_truth") is False,
            "failure must not mark order-state exchange truth",
            str(path),
        )
        require(
            report.get("shadow_values_are_exchange_truth") is False,
            "failure must not mark shadow exchange truth",
            str(path),
        )
        require(
            report.get("portfolio_values_are_exchange_truth") is False,
            "failure must not mark portfolio exchange truth",
            str(path),
        )
        require(report.get("values_are_exchange_truth") is False, "failure must not mark exchange truth", str(path))
        exchange_truth = False
    else:
        fail("unsupported owner evidence status", {"path": str(path), "status": status})

    validate_shape_summary(report, path)
    return {
        "path": str(path.relative_to(proof_root)),
        "endpoint": endpoint,
        "status": status,
        "response_shape_validated": report.get("response_shape_validated"),
        "values_are_exchange_truth": exchange_truth,
        "response_status_code": report.get("response_status_code"),
        "error_code": report.get("error_code"),
    }


def assert_no_secret_leaks(root):
    secret_values = [
        "ntpro_v140001_script_synthetic_api_key_value",
        "ntpro_v140001_script_synthetic_api_secret_value",
        "ntpro_v141_script_synthetic_api_key_value",
        "ntpro_v141_script_synthetic_api_secret_value",
        os.environ.get("BINANCE_PRODUCTION_READONLY_API_KEY", ""),
        os.environ.get("BINANCE_PRODUCTION_READONLY_API_SECRET", ""),
    ]
    secret_values = [value for value in secret_values if value]
    leaked_files = []
    for path in sorted(root.rglob("*")):
        if not path.is_file():
            continue
        text = path.read_text(errors="ignore")
        for secret in secret_values:
            if secret in text:
                leaked_files.append(str(path.relative_to(root)))
                break
    require(not leaked_files, "secret value leaked into evidence package", leaked_files)


json_files = load_json_files(proof_root)
require(json_files, "no v0.14 order-state proof artifacts found", str(proof_root))

validated = [validate_owner_artifact(path, data) for path, data in json_files]
open_orders = [item for item in validated if item["endpoint"] == "open_orders"]
single_orders = [item for item in validated if item["endpoint"] == "order"]
require(open_orders, "open_orders owner evidence is required", validated)
assert_no_secret_leaks(proof_root)

status = "offline_fixture_contract_ok" if mode == "offline_fixture" else "owner_run_order_state_evidence_ok"
if mode == "owner_run" and any(item["status"] == "online_order_state_read_failed" for item in validated):
    status = "owner_run_classified_failure"

manifest = {
    "schema_version": schema_version,
    "status": status,
    "evidence_mode": mode,
    "proof_root": str(proof_root),
    "open_orders_evidence_required": True,
    "open_orders_evidence_count": len(open_orders),
    "single_order_evidence_optional": True,
    "single_order_evidence_count": len(single_orders),
    "validated_artifacts": validated,
    "production_order_submission_attempted": False,
    "production_order_mutation_attempted": False,
    "cancel_replace_amend_attempted": False,
    "listen_key_lifecycle_attempted": False,
    "dashboard_order_controls_enabled": False,
    "automatic_remediation_attempted": False,
    "real_orders_submitted": False,
    "production_trading_enabled": False,
    "secrets_redacted": True,
}
manifest_path.parent.mkdir(parents=True, exist_ok=True)
manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
print(
    "v141_order_state_owner_evidence "
    f"status={status} mode={mode} root={proof_root} "
    f"manifest={manifest_path} open_orders={len(open_orders)} "
    f"single_order={len(single_orders)} production_order_mutation_attempted=false"
)
PY
