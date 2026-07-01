#!/usr/bin/env python3
from __future__ import annotations

import argparse
import copy
import json
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator


REQUIRED_BOUNDARY_FLAGS = {
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_approval_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "dashboard_retry_controls_enabled",
    "retry_replace_amend_flatten_allowed",
    "product_grade_trading_terminal_claim",
}
DASHBOARD_BOUNDARY_FLAGS = {
    "dashboard_submit_controls_enabled",
    "dashboard_replace_controls_enabled",
    "dashboard_amend_controls_enabled",
    "dashboard_flatten_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "trader_terminal_live_trading_claim",
}


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise SystemExit(f"{path}: invalid JSON: {exc}") from exc
    if not isinstance(value, dict):
        raise SystemExit(f"{path}: root must be a JSON object")
    return value


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        try:
            row = json.loads(line)
        except json.JSONDecodeError as exc:
            raise SystemExit(f"{path}:{line_number}: invalid JSON: {exc}") from exc
        if not isinstance(row, dict):
            raise SystemExit(f"{path}:{line_number}: row must be an object")
        rows.append(row)
    return rows


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def snapshot_from_row(row: dict[str, Any], path: Path) -> dict[str, Any] | None:
    events = row.get("input", {}).get("events", [])
    if not isinstance(events, list) or not events:
        raise SystemExit(f"{path}:{row.get('case_id')}: input.events must be a non-empty array")
    snapshot = events[0].get("payload", {}).get("snapshot")
    if snapshot is None:
        return None
    if not isinstance(snapshot, dict):
        raise SystemExit(f"{path}:{row.get('case_id')}: payload.snapshot must be an object")
    return snapshot


def collect_read_model_snapshots(trace_glob: str) -> dict[str, tuple[Path, dict[str, Any]]]:
    snapshots: dict[str, tuple[Path, dict[str, Any]]] = {}
    for path in sorted(Path().glob(trace_glob)):
        for row in load_jsonl(path):
            if row.get("category") != "read_model":
                continue
            case_id = row.get("case_id")
            if not isinstance(case_id, str) or not case_id:
                raise SystemExit(f"{path}: read_model row has invalid case_id")
            snapshot = snapshot_from_row(row, path)
            if snapshot is None:
                raise SystemExit(f"{path}:{case_id}: read_model row must carry input payload.snapshot")
            if case_id in snapshots:
                raise SystemExit(f"duplicate read_model case_id {case_id}")
            snapshots[case_id] = (path, snapshot)
    if not snapshots:
        raise SystemExit(f"no read_model snapshots found for {trace_glob}")
    return snapshots


def format_errors(errors: list[Any]) -> str:
    parts: list[str] = []
    for error in sorted(errors, key=lambda item: list(item.path)):
        location = ".".join(str(part) for part in error.path) or "<root>"
        parts.append(f"{location}: {error.message}")
    return "\n".join(parts)


def validate_schema_strategy(schema: dict[str, Any]) -> None:
    require(schema.get("additionalProperties") is False, "snapshot schema must fail closed on undeclared top-level fields")
    defs = schema.get("$defs", {})
    for name in (
        "source_provenance",
        "snapshot_redaction",
        "component_redaction",
        "capability_boundary",
        "lineage",
        "freshness",
        "component",
        "component_data",
    ):
        value = defs.get(name)
        require(isinstance(value, dict), f"schema missing $defs.{name}")
        require(value.get("additionalProperties") is False, f"$defs.{name} must set additionalProperties=false")

    boundary = defs["capability_boundary"]
    boundary_props = set(boundary.get("properties", {}))
    missing_core = REQUIRED_BOUNDARY_FLAGS - boundary_props
    missing_dashboard = DASHBOARD_BOUNDARY_FLAGS - boundary_props
    require(not missing_core, f"capability boundary missing core flags: {sorted(missing_core)}")
    require(not missing_dashboard, f"capability boundary missing dashboard/order-ticket flags: {sorted(missing_dashboard)}")
    for flag in REQUIRED_BOUNDARY_FLAGS | DASHBOARD_BOUNDARY_FLAGS:
        require(boundary["properties"][flag].get("const") is False, f"{flag} must be constrained to false")

    source = defs["source_provenance"]
    require(source.get("allOf"), "source_provenance must include exchange truth / adapter runtime constraints")


def validate_all_snapshots(
    validator: Draft202012Validator,
    snapshots: dict[str, tuple[Path, dict[str, Any]]],
) -> None:
    errors: list[str] = []
    for case_id, (path, snapshot) in snapshots.items():
        snapshot_errors = list(validator.iter_errors(snapshot))
        if snapshot_errors:
            errors.append(f"{path}:{case_id}\n{format_errors(snapshot_errors)}")
    if errors:
        raise SystemExit("read_model JSON Schema validation failed:\n" + "\n\n".join(errors))


def expect_invalid(
    validator: Draft202012Validator,
    label: str,
    snapshot: dict[str, Any],
) -> None:
    errors = list(validator.iter_errors(snapshot))
    if not errors:
        raise SystemExit(f"negative schema mutation unexpectedly passed: {label}")


def run_negative_mutations(
    validator: Draft202012Validator,
    snapshots: dict[str, tuple[Path, dict[str, Any]]],
) -> None:
    account = copy.deepcopy(snapshots["read_model.account_snapshot.fresh.001"][1])
    account["snapshot_kind"] = "unified_snapshot"
    account["health_status"] = "healthy"
    account["blocking_reasons"] = []
    expect_invalid(validator, "partial component snapshot masquerades as unified healthy", account)

    undeclared_root = copy.deepcopy(snapshots["read_model.contract.healthy_minimal.001"][1])
    undeclared_root["raw_exchange_response"] = {"leak": True}
    expect_invalid(validator, "undeclared top-level raw exchange response", undeclared_root)

    sensitive_data = copy.deepcopy(snapshots["read_model.account_snapshot.fresh.001"][1])
    sensitive_data["components"]["account"]["data"]["api_secret"] = "not-allowed"
    expect_invalid(validator, "sensitive component data field", sensitive_data)

    unauthorized_boundary = copy.deepcopy(snapshots["read_model.dashboard.readonly_complete.001"][1])
    unauthorized_boundary["capability_boundary"]["dashboard_force_submit_enabled"] = False
    expect_invalid(validator, "undeclared dashboard boundary flag", unauthorized_boundary)

    missing_dashboard_flag = copy.deepcopy(snapshots["read_model.dashboard.readonly_complete.001"][1])
    del missing_dashboard_flag["capability_boundary"]["dashboard_submit_controls_enabled"]
    expect_invalid(validator, "dashboard submit flag omitted", missing_dashboard_flag)

    exchange_truth_fixture = copy.deepcopy(snapshots["read_model.contract.healthy_minimal.001"][1])
    exchange_truth_fixture["source_provenance"]["source_type"] = "fixture"
    exchange_truth_fixture["source_provenance"]["exchange_truth"] = True
    exchange_truth_fixture["source_provenance"]["adapter_runtime_integrated"] = False
    expect_invalid(validator, "fixture claims exchange truth", exchange_truth_fixture)

    adapter_runtime_fixture = copy.deepcopy(snapshots["read_model.contract.healthy_minimal.001"][1])
    adapter_runtime_fixture["source_provenance"]["source_type"] = "fixture"
    adapter_runtime_fixture["source_provenance"]["adapter_runtime_integrated"] = True
    expect_invalid(validator, "fixture claims adapter runtime integration", adapter_runtime_fixture)

    redaction_extra = copy.deepcopy(snapshots["read_model.contract.healthy_minimal.001"][1])
    redaction_extra["redaction"]["signed_url"] = "not-allowed"
    expect_invalid(validator, "redaction object undeclared signed URL", redaction_extra)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Validate v0.21/v0.21.1 read_model snapshots against the unified JSON Schema."
    )
    parser.add_argument(
        "--schema",
        type=Path,
        default=Path("docs/rust-cutover/release/v0_21_0_unified_read_model_schema.json"),
    )
    parser.add_argument("--trace-glob", default="tests/golden/**/*.jsonl")
    args = parser.parse_args()

    schema = load_json(args.schema)
    Draft202012Validator.check_schema(schema)
    validate_schema_strategy(schema)

    validator = Draft202012Validator(schema)
    snapshots = collect_read_model_snapshots(args.trace_glob)
    validate_all_snapshots(validator, snapshots)
    run_negative_mutations(validator, snapshots)
    print(
        "v211_read_model_schema_boundary status=ok "
        f"validated_read_model_snapshots={len(snapshots)} "
        "negative_mutations=8 additional_properties=false boundary_flags=strict"
    )


if __name__ == "__main__":
    main()
