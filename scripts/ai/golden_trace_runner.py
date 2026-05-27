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
    errors = []
    for idx, row in enumerate(rows):
        for key in ["case_id", "input", "expected"]:
            if key not in row:
                errors.append(f"row {idx}: missing {key}")
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
