#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


DEFAULT_REPO = "atxinbao/NTPRO"
DEFAULT_SHRIMP_TASKS = Path("/Users/mac/.codex/shrimp-data/NTPRO/tasks.json")


def now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="milliseconds").replace("+00:00", "Z")


def run(cmd: list[str], cwd: Path, *, json_output: bool = False) -> Any:
    result = subprocess.run(cmd, cwd=cwd, text=True, capture_output=True)
    if result.returncode != 0:
        raise SystemExit(
            json.dumps(
                {
                    "status": "failed",
                    "command": cmd,
                    "returncode": result.returncode,
                    "stdout": result.stdout,
                    "stderr": result.stderr,
                },
                indent=2,
            )
        )
    if json_output:
        return json.loads(result.stdout)
    return result.stdout.strip()


def load_tasks(path: Path) -> tuple[Any, list[dict[str, Any]]]:
    data = json.loads(path.read_text())
    if isinstance(data, list):
        return data, data
    tasks = data.get("tasks")
    if not isinstance(tasks, list):
        raise SystemExit(f"Shrimp tasks file has no task list: {path}")
    return data, tasks


def task_title(task: dict[str, Any]) -> str:
    return str(task.get("name") or task.get("title") or "")


def find_task(tasks: list[dict[str, Any]], task_id: str) -> dict[str, Any]:
    for task in tasks:
        if task_title(task).startswith(f"{task_id} "):
            return task
    raise SystemExit(f"task not found in Shrimp data: {task_id}")


def backup_tasks(path: Path) -> Path:
    backup_dir = path.parent / "memory"
    backup_dir.mkdir(parents=True, exist_ok=True)
    backup_path = backup_dir / f"tasks_before_close_pr_{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}.json"
    shutil.copy2(path, backup_path)
    return backup_path


def check_required_check(pr: dict[str, Any], required_check: str) -> tuple[bool, str]:
    checks = pr.get("statusCheckRollup") or []
    for check in checks:
        if check.get("name") == required_check:
            if check.get("status") == "COMPLETED" and check.get("conclusion") == "SUCCESS":
                return True, "required check passed"
            return False, f"{required_check} is {check.get('status')} / {check.get('conclusion')}"
    return False, f"{required_check} not found"


def ensure_clean_worktree(workspace: Path) -> None:
    status = run(["git", "status", "--porcelain"], workspace)
    if status:
        raise SystemExit(
            json.dumps(
                {
                    "status": "blocked",
                    "reason": "worktree_not_clean",
                    "details": status.splitlines(),
                },
                indent=2,
            )
        )


def main() -> int:
    parser = argparse.ArgumentParser(description="Close a merged NTPRO PR in local control state.")
    parser.add_argument("--repo", default=DEFAULT_REPO)
    parser.add_argument("--pr", required=True, type=int)
    parser.add_argument("--task-id", required=True)
    parser.add_argument("--required-check", default="smoke")
    parser.add_argument("--workspace", type=Path, default=Path.cwd())
    parser.add_argument("--shrimp-tasks", type=Path, default=DEFAULT_SHRIMP_TASKS)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    workspace = args.workspace.resolve()
    pr = run(
        [
            "gh",
            "pr",
            "view",
            str(args.pr),
            "-R",
            args.repo,
            "--json",
            "state,mergedAt,mergeCommit,statusCheckRollup,url",
        ],
        workspace,
        json_output=True,
    )

    if pr.get("state") != "MERGED":
        print(
            json.dumps(
                {
                    "status": "waiting",
                    "reason": "pr_not_merged",
                    "pr": args.pr,
                    "state": pr.get("state"),
                    "url": pr.get("url"),
                },
                indent=2,
            )
        )
        return 0

    check_ok, check_message = check_required_check(pr, args.required_check)
    if not check_ok:
        print(
            json.dumps(
                {
                    "status": "waiting",
                    "reason": "required_check_not_green",
                    "details": check_message,
                    "pr": args.pr,
                    "url": pr.get("url"),
                },
                indent=2,
            )
        )
        return 0

    data, tasks = load_tasks(args.shrimp_tasks)
    task = find_task(tasks, args.task_id)
    if task.get("status") == "completed":
        print(
            json.dumps(
                {
                    "status": "already_closed",
                    "dry_run": args.dry_run,
                    "task_id": args.task_id,
                    "pr": args.pr,
                    "url": pr.get("url"),
                    "shrimp_tasks": str(args.shrimp_tasks),
                },
                indent=2,
            )
        )
        return 0

    if not args.dry_run:
        ensure_clean_worktree(workspace)
        run(["git", "fetch", "--prune", "origin"], workspace)
        run(["git", "switch", "main"], workspace)
        run(["git", "pull", "--ff-only", "origin", "main"], workspace)
        backup_path = backup_tasks(args.shrimp_tasks)
        task["status"] = "completed"
        task["updatedAt"] = now()
        note = (
            f"{args.task_id} completed by close_merged_pr.py after PR #{args.pr} merged; "
            f"{args.required_check} passed."
        )
        task["notes"] = "\n\n".join([n for n in [task.get("notes"), note] if n])
        args.shrimp_tasks.write_text(json.dumps(data, indent=2) + "\n")
    else:
        backup_path = None

    print(
        json.dumps(
            {
                "status": "closed",
                "dry_run": args.dry_run,
                "task_id": args.task_id,
                "pr": args.pr,
                "url": pr.get("url"),
                "merged_at": pr.get("mergedAt"),
                "merge_commit": (pr.get("mergeCommit") or {}).get("oid"),
                "required_check": args.required_check,
                "shrimp_tasks": str(args.shrimp_tasks),
                "backup": str(backup_path) if backup_path else None,
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
