#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


DEFAULT_SHRIMP_TASKS = Path("/Users/mac/.codex/shrimp-data/NTPRO/tasks.json")
ROOT = Path(__file__).resolve().parents[2]
RISK_ORDER = {"low": 0, "medium": 1, "high": 2, "critical": 3}
AUTO_DISPATCH_FORBIDDEN = ("RREM-", "RREL-", "NREM-", "NREL-", "NGATE-")


def now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="milliseconds").replace("+00:00", "Z")


def run(cmd: list[str], cwd: Path) -> str:
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
    return result.stdout.strip()


def load_json(path: Path) -> Any:
    return json.loads(path.read_text())


def write_json(path: Path, data: Any) -> None:
    path.write_text(json.dumps(data, indent=2) + "\n")


def load_shrimp(path: Path) -> tuple[Any, list[dict[str, Any]]]:
    data = load_json(path)
    if isinstance(data, list):
        return data, data
    tasks = data.get("tasks")
    if not isinstance(tasks, list):
        raise SystemExit(f"Shrimp tasks file has no task list: {path}")
    return data, tasks


def task_title(task: dict[str, Any]) -> str:
    return str(task.get("name") or task.get("title") or "")


def short_task_id(task: dict[str, Any]) -> str:
    match = re.match(r"^([A-Z]+-\d+)\b", task_title(task))
    if not match:
        raise SystemExit(f"cannot parse task id from Shrimp task title: {task_title(task)}")
    return match.group(1)


def slugify(value: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    return re.sub(r"-+", "-", slug)


def completed_shrimp_ids(tasks: list[dict[str, Any]]) -> set[str]:
    return {str(task.get("id")) for task in tasks if task.get("status") == "completed"}


def dependencies_complete(task: dict[str, Any], completed_ids: set[str]) -> bool:
    deps = task.get("dependencies") or []
    for dep in deps:
        dep_id = dep.get("taskId") if isinstance(dep, dict) else dep
        if dep_id and str(dep_id) not in completed_ids:
            return False
    return True


def first_eligible_shrimp_task(tasks: list[dict[str, Any]]) -> dict[str, Any] | None:
    completed_ids = completed_shrimp_ids(tasks)
    for task in tasks:
        if task.get("status") == "pending" and dependencies_complete(task, completed_ids):
            return task
    return None


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


def backup_tasks(path: Path, task_id: str) -> Path:
    backup_dir = path.parent / "memory"
    backup_dir.mkdir(parents=True, exist_ok=True)
    backup_path = backup_dir / (
        f"tasks_before_dispatch_{task_id}_{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}.json"
    )
    shutil.copy2(path, backup_path)
    return backup_path


def update_agentflow_state(path: Path, task_id: str, tasks: list[dict[str, Any]]) -> None:
    data = load_json(path)
    agent_tasks = data.get("tasks")
    if not isinstance(agent_tasks, dict):
        raise SystemExit("agentflow task_status.json has no tasks object")

    shrimp_by_short_id = {}
    for task in tasks:
        try:
            shrimp_by_short_id[short_task_id(task)] = task
        except SystemExit:
            continue

    timestamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    for existing_id, existing in agent_tasks.items():
        if existing.get("status") == "RUNNING":
            shrimp_task = shrimp_by_short_id.get(existing_id)
            if shrimp_task and shrimp_task.get("status") == "completed":
                existing["status"] = "DONE"
                existing["updated_at"] = timestamp

    target = agent_tasks.get(task_id)
    if not target:
        raise SystemExit(f"{task_id} missing from .agentflow/state/task_status.json")
    target["status"] = "RUNNING"
    target["updated_at"] = timestamp
    write_json(path, data)


def claim_lease(workspace: Path, task_id: str, branch: str, agent_id: str) -> None:
    source = f"docs/rust-cutover/tasks/{task_id}.md"
    evidence = f"docs/rust-cutover/evidence/{task_id}.md"
    lease = f".agentflow/leases/{task_id}.json"
    run(
        [
            "python3",
            "scripts/ai/lease.py",
            "claim",
            task_id,
            "--branch",
            branch,
            "--agent-id",
            agent_id,
            "--path",
            source,
            "--path",
            evidence,
            "--path",
            ".agentflow/state/task_status.json",
            "--path",
            lease,
        ],
        workspace,
    )


def risk_allowed(task_id: str, risk: str, max_risk: str) -> tuple[bool, str]:
    if task_id.startswith(AUTO_DISPATCH_FORBIDDEN):
        return False, "task prefix requires manual gate"
    if RISK_ORDER.get(risk, 99) > RISK_ORDER[max_risk]:
        return False, f"risk {risk} exceeds max auto-dispatch risk {max_risk}"
    return True, "risk allowed"


def main() -> int:
    parser = argparse.ArgumentParser(description="Dispatch the next eligible NTPRO task.")
    parser.add_argument("--workspace", type=Path, default=Path.cwd())
    parser.add_argument("--shrimp-tasks", type=Path, default=DEFAULT_SHRIMP_TASKS)
    parser.add_argument("--agent-id", default="Codex")
    parser.add_argument("--branch-prefix", default="ai")
    parser.add_argument("--max-risk", choices=sorted(RISK_ORDER, key=RISK_ORDER.get), default="medium")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    workspace = args.workspace.resolve()
    state_path = workspace / ".agentflow/state/task_status.json"
    _, shrimp_tasks = load_shrimp(args.shrimp_tasks)

    in_progress = [task for task in shrimp_tasks if task.get("status") == "in_progress"]
    if in_progress:
        print(
            json.dumps(
                {
                    "status": "waiting",
                    "reason": "shrimp_task_in_progress",
                    "tasks": [task_title(task) for task in in_progress],
                },
                indent=2,
            )
        )
        return 0

    next_task = first_eligible_shrimp_task(shrimp_tasks)
    if not next_task:
        print(json.dumps({"status": "waiting", "reason": "no_eligible_task"}, indent=2))
        return 0

    task_id = short_task_id(next_task)
    agentflow = load_json(state_path)
    task_meta = agentflow.get("tasks", {}).get(task_id)
    if not task_meta:
        raise SystemExit(f"{task_id} missing from .agentflow/state/task_status.json")

    risk = task_meta.get("risk_level")
    allowed, reason = risk_allowed(task_id, str(risk), args.max_risk)
    if not allowed:
        print(
            json.dumps(
                {
                    "status": "manual_gate_required",
                    "task_id": task_id,
                    "risk": risk,
                    "reason": reason,
                },
                indent=2,
            )
        )
        return 0

    branch = f"{args.branch_prefix}/{task_id}-{slugify(task_meta.get('title', task_id))}"

    if args.dry_run:
        print(
            json.dumps(
                {
                    "status": "dry_run",
                    "task_id": task_id,
                    "title": task_meta.get("title"),
                    "risk": risk,
                    "branch": branch,
                    "reason": reason,
                },
                indent=2,
            )
        )
        return 0

    ensure_clean_worktree(workspace)
    run(["git", "fetch", "--prune", "origin"], workspace)
    if run(["git", "branch", "--list", branch], workspace):
        raise SystemExit(json.dumps({"status": "blocked", "reason": "branch_exists", "branch": branch}, indent=2))
    lease_path = workspace / f".agentflow/leases/{task_id}.json"
    if lease_path.exists():
        raise SystemExit(
            json.dumps(
                {
                    "status": "blocked",
                    "reason": "lease_exists",
                    "lease": str(lease_path.relative_to(workspace)),
                },
                indent=2,
            )
        )

    run(["git", "switch", "-c", branch, "origin/main"], workspace)
    update_agentflow_state(state_path, task_id, shrimp_tasks)
    claim_lease(workspace, task_id, branch, args.agent_id)

    data, shrimp_tasks_for_write = load_shrimp(args.shrimp_tasks)
    target = next(task for task in shrimp_tasks_for_write if str(task.get("id")) == str(next_task.get("id")))
    backup_path = backup_tasks(args.shrimp_tasks, task_id)
    target["status"] = "in_progress"
    target["updatedAt"] = now()
    note = f"{task_id} dispatched by dispatch_next.py on branch {branch}."
    target["notes"] = "\n\n".join([n for n in [target.get("notes"), note] if n])
    write_json(args.shrimp_tasks, data)

    print(
        json.dumps(
            {
                "status": "dispatched",
                "task_id": task_id,
                "title": task_meta.get("title"),
                "risk": risk,
                "branch": branch,
                "lease": f".agentflow/leases/{task_id}.json",
                "shrimp_tasks": str(args.shrimp_tasks),
                "backup": str(backup_path),
            },
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
