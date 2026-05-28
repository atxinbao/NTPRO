#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shlex
import subprocess
import tempfile
from pathlib import Path
from typing import Any

SCHEMA_VERSION = "golden-trace-v1"
REQUIRED_ROW_FIELDS = (
    "schema_version",
    "case_id",
    "category",
    "description",
    "input",
    "expected",
    "tolerances",
)
REQUIRED_EVENT_FIELDS = ("event_type", "ts_event", "payload")
VALID_CATEGORIES = {
    "market_data",
    "order_lifecycle",
    "risk",
    "execution",
    "position",
    "portfolio_pnl",
    "cache_msgbus",
    "backtest_live",
    "adapter_payload",
}


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    rows = []
    with path.open(encoding="utf-8") as f:
        for lineno, line in enumerate(f, 1):
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            try:
                rows.append(json.loads(line))
            except json.JSONDecodeError as e:
                raise SystemExit(f"{path}:{lineno}: invalid JSON: {e}") from e
    return rows


def is_timestamp(value: Any) -> bool:
    if isinstance(value, int) and not isinstance(value, bool):
        return True
    if isinstance(value, str):
        return value.isdecimal()
    return False


def validate_event(event: Any, path: str, errors: list[str]) -> None:
    if not isinstance(event, dict):
        errors.append(f"{path}: event must be an object")
        return

    for key in REQUIRED_EVENT_FIELDS:
        if key not in event:
            errors.append(f"{path}: missing event field {key}")

    event_type = event.get("event_type")
    if not isinstance(event_type, str) or not event_type:
        errors.append(f"{path}.event_type: must be a non-empty string")

    for key in ("ts_event", "ts_init"):
        if key in event and not is_timestamp(event[key]):
            errors.append(
                f"{path}.{key}: must be an integer or decimal string nanosecond timestamp"
            )

    payload = event.get("payload")
    if not isinstance(payload, dict):
        errors.append(f"{path}.payload: must be an object")


def validate_event_section(section: Any, path: str, errors: list[str]) -> None:
    if not isinstance(section, dict):
        errors.append(f"{path}: must be an object")
        return

    events = section.get("events")
    if not isinstance(events, list):
        errors.append(f"{path}.events: must be an array")
        return

    for idx, event in enumerate(events):
        validate_event(event, f"{path}.events[{idx}]", errors)


def validate_rows(rows: list[dict[str, Any]]) -> list[str]:
    errors: list[str] = []
    seen_case_ids: set[str] = set()

    if not rows:
        errors.append("trace must contain at least one row")
        return errors

    for idx, row in enumerate(rows):
        path = f"row {idx}"
        if not isinstance(row, dict):
            errors.append(f"{path}: row must be an object")
            continue

        for key in REQUIRED_ROW_FIELDS:
            if key not in row:
                errors.append(f"{path}: missing {key}")

        if row.get("schema_version") != SCHEMA_VERSION:
            errors.append(f"{path}.schema_version: expected {SCHEMA_VERSION}")

        case_id = row.get("case_id")
        if not isinstance(case_id, str) or not case_id:
            errors.append(f"{path}.case_id: must be a non-empty string")
        elif case_id in seen_case_ids:
            errors.append(f"{path}.case_id: duplicate case_id {case_id}")
        else:
            seen_case_ids.add(case_id)

        category = row.get("category")
        if category not in VALID_CATEGORIES:
            valid = ", ".join(sorted(VALID_CATEGORIES))
            errors.append(f"{path}.category: expected one of {valid}")

        description = row.get("description")
        if not isinstance(description, str) or not description:
            errors.append(f"{path}.description: must be a non-empty string")

        validate_event_section(row.get("input"), f"{path}.input", errors)
        validate_event_section(row.get("expected"), f"{path}.expected", errors)

        tolerances = row.get("tolerances")
        if not isinstance(tolerances, dict):
            errors.append(f"{path}.tolerances: must be an object")

    return errors


def normalize(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def run_replay(trace: Path, command_template: str, rows: list[dict[str, Any]]) -> None:
    with tempfile.TemporaryDirectory() as tmp:
        actual_path = Path(tmp) / "actual.jsonl"
        command = command_template.replace("{trace}", shlex.quote(str(trace))).replace("{actual}", shlex.quote(str(actual_path)))
        if "{trace}" not in command_template and "{actual}" not in command_template:
            command = f"{command} {shlex.quote(str(trace))}"
        result = subprocess.run(command, shell=True, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        if result.returncode != 0:
            raise SystemExit(f"replay command failed ({result.returncode})\nSTDOUT:\n{result.stdout}\nSTDERR:\n{result.stderr}")
        if actual_path.exists():
            actual_rows = load_jsonl(actual_path)
        else:
            actual_rows = [json.loads(line) for line in result.stdout.splitlines() if line.strip()]

    expected = {str(row["case_id"]): row["expected"] for row in rows}
    actual = {}
    for row in actual_rows:
        if "case_id" not in row:
            raise SystemExit("replay output row missing case_id")
        actual[str(row["case_id"])] = row.get("actual", row.get("output", row.get("expected")))

    errors = []
    for case_id, exp in expected.items():
        if case_id not in actual:
            errors.append(f"case {case_id}: missing actual output")
            continue
        if normalize(exp) != normalize(actual[case_id]):
            errors.append(f"case {case_id}: expected {normalize(exp)} got {normalize(actual[case_id])}")
    extra = sorted(set(actual) - set(expected))
    if extra:
        errors.append("unexpected actual cases: " + ", ".join(extra))
    if errors:
        raise SystemExit("golden trace replay mismatch:\n" + "\n".join(errors))
    print(f"replay ok: {trace} ({len(rows)} cases)")


def main() -> None:
    parser = argparse.ArgumentParser(description="Golden trace validator/replay wrapper")
    parser.add_argument("trace", type=Path)
    parser.add_argument("--mode", choices=["validate-only", "replay"], default="validate-only")
    parser.add_argument("--replay-command", default=os.environ.get("GOLDEN_TRACE_REPLAY_COMMAND", ""), help="Command that replays a trace. May use {trace} and optionally write JSONL to {actual}.")
    args = parser.parse_args()

    rows = load_jsonl(args.trace)
    errors = validate_rows(rows)
    if errors:
        raise SystemExit("\n".join(errors))

    if args.mode == "validate-only":
        print(f"valid trace: {args.trace} ({len(rows)} rows)")
        return

    if not args.replay_command:
        raise SystemExit("replay mode requires --replay-command or GOLDEN_TRACE_REPLAY_COMMAND")
    run_replay(args.trace, args.replay_command, rows)


if __name__ == "__main__":
    main()
