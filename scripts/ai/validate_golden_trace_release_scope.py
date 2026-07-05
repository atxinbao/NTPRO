#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from collections import Counter
from pathlib import Path
from typing import Any

VALID_STATUSES = {"executable_replay", "validator_executable_replay", "schema_only_scoped"}
EXECUTABLE_DECISION = "included_in_final_replay_scope"
VALIDATOR_EXECUTABLE_DECISION = "validator_executable_scope_recorded"
SCHEMA_ONLY_DECISION = "schema_only_scope_recorded"


def load_json(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise SystemExit(f"{path}: invalid JSON: {exc}") from exc
    if not isinstance(data, dict):
        raise SystemExit(f"{path}: manifest must be a JSON object")
    return data


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    with path.open(encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            try:
                row = json.loads(line)
            except json.JSONDecodeError as exc:
                raise SystemExit(f"{path}:{line_number}: invalid JSON: {exc}") from exc
            if not isinstance(row, dict):
                raise SystemExit(f"{path}:{line_number}: row must be a JSON object")
            rows.append(row)
    return rows


def collect_trace_cases(trace_glob: str) -> dict[str, dict[str, str]]:
    cases: dict[str, dict[str, str]] = {}
    traces = sorted(Path().glob(trace_glob))
    if not traces:
        raise SystemExit(f"no golden trace files found for TRACE_GLOB={trace_glob}")

    for trace in traces:
        for row in load_jsonl(trace):
            case_id = require_string(row, "case_id", str(trace))
            category = require_string(row, "category", case_id)
            if case_id in cases:
                raise SystemExit(f"duplicate case_id in tests/golden: {case_id}")
            cases[case_id] = {
                "trace": trace.as_posix(),
                "category": category,
            }
    return cases


def validate_manifest(manifest: dict[str, Any], trace_cases: dict[str, dict[str, str]]) -> Counter[str]:
    errors: list[str] = []
    if manifest.get("schema_version") != "golden-trace-release-scope-v1":
        errors.append("manifest.schema_version must be golden-trace-release-scope-v1")
    if not require_non_empty(manifest.get("owner_signoff")):
        errors.append("manifest.owner_signoff must record the current owner signoff state")

    entries = manifest.get("cases")
    if not isinstance(entries, list) or not entries:
        errors.append("manifest.cases must be a non-empty array")
        entries = []

    manifest_cases: dict[str, dict[str, Any]] = {}
    status_counts: Counter[str] = Counter()
    for index, entry in enumerate(entries):
        path = f"manifest.cases[{index}]"
        if not isinstance(entry, dict):
            errors.append(f"{path}: entry must be an object")
            continue
        case_id = entry.get("case_id")
        if not require_non_empty(case_id):
            errors.append(f"{path}.case_id must be a non-empty string")
            continue
        case_id = str(case_id)
        if case_id in manifest_cases:
            errors.append(f"{path}.case_id duplicates {case_id}")
            continue
        manifest_cases[case_id] = entry

        status = entry.get("status")
        if status not in VALID_STATUSES:
            errors.append(f"{case_id}.status must be one of {sorted(VALID_STATUSES)}")
            continue
        status_counts[str(status)] += 1

        actual = trace_cases.get(case_id)
        if actual is None:
            errors.append(f"{case_id}: manifest entry has no matching tests/golden case")
            continue
        if entry.get("trace") != actual["trace"]:
            errors.append(f"{case_id}.trace expected {actual['trace']} got {entry.get('trace')}")
        if entry.get("category") != actual["category"]:
            errors.append(
                f"{case_id}.category expected {actual['category']} got {entry.get('category')}"
            )

        if status == "executable_replay":
            validate_executable(entry, case_id, errors)
        elif status == "validator_executable_replay":
            validate_validator_executable(entry, case_id, errors)
        elif status == "schema_only_scoped":
            validate_schema_only(entry, case_id, errors)

    missing = sorted(set(trace_cases) - set(manifest_cases))
    extra = sorted(set(manifest_cases) - set(trace_cases))
    if missing:
        errors.append("manifest missing trace cases: " + ", ".join(missing))
    if extra:
        errors.append("manifest has extra trace cases: " + ", ".join(extra))

    if errors:
        raise SystemExit("golden trace release scope validation failed:\n" + "\n".join(errors))
    return status_counts


def validate_executable(entry: dict[str, Any], case_id: str, errors: list[str]) -> None:
    for key in ("evidence_id", "harness", "rust_entrypoint"):
        if not require_non_empty(entry.get(key)):
            errors.append(f"{case_id}.{key} is required for executable_replay")
    if entry.get("release_decision") != EXECUTABLE_DECISION:
        errors.append(f"{case_id}.release_decision must be {EXECUTABLE_DECISION}")


def validate_validator_executable(entry: dict[str, Any], case_id: str, errors: list[str]) -> None:
    for key in ("evidence_id", "harness", "validator_entrypoint"):
        if not require_non_empty(entry.get(key)):
            errors.append(f"{case_id}.{key} is required for validator_executable_replay")
    if "rust_entrypoint" in entry:
        errors.append(
            f"{case_id}: validator_executable_replay must not claim rust_entrypoint"
        )
    if entry.get("release_decision") != VALIDATOR_EXECUTABLE_DECISION:
        errors.append(f"{case_id}.release_decision must be {VALIDATOR_EXECUTABLE_DECISION}")
    if entry.get("runtime_adapter_integration") is not False:
        errors.append(
            f"{case_id}.runtime_adapter_integration must be false for validator_executable_replay"
        )


def validate_schema_only(entry: dict[str, Any], case_id: str, errors: list[str]) -> None:
    for key in ("scope_owner", "reason", "follow_up"):
        if not require_non_empty(entry.get(key)):
            errors.append(f"{case_id}.{key} is required for schema_only_scoped")
    if entry.get("release_decision") != SCHEMA_ONLY_DECISION:
        errors.append(f"{case_id}.release_decision must be {SCHEMA_ONLY_DECISION}")
    forbidden = [key for key in ("evidence_id", "harness", "rust_entrypoint") if key in entry]
    if forbidden:
        errors.append(f"{case_id}: schema_only_scoped must not claim executable fields {forbidden}")


def require_string(row: dict[str, Any], key: str, path: str) -> str:
    value = row.get(key)
    if not isinstance(value, str) or not value:
        raise SystemExit(f"{path}.{key}: must be a non-empty string")
    return value


def require_non_empty(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Validate the final release golden trace replay/scope manifest."
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        default=Path("docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json"),
    )
    parser.add_argument("--trace-glob", default="tests/golden/*.jsonl")
    args = parser.parse_args()

    trace_cases = collect_trace_cases(args.trace_glob)
    status_counts = validate_manifest(load_json(args.manifest), trace_cases)
    total = sum(status_counts.values())
    print(
        "golden trace release scope ok: "
        f"{total} cases, "
        f"{status_counts['executable_replay']} executable replay, "
        f"{status_counts['validator_executable_replay']} validator executable replay, "
        f"{status_counts['schema_only_scoped']} schema-only scoped"
    )


if __name__ == "__main__":
    main()
