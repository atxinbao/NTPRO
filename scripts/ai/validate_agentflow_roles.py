#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
ROLE_IDS = {
    "control_scope_agent",
    "rust_product_surface_agent",
    "rust_core_runtime_agent",
    "adapter_integration_agent",
    "verification_release_gatekeeper",
}
VALID_STATES = {
    "TODO",
    "READY",
    "LEASED",
    "RUNNING",
    "PR_OPEN",
    "QA_REQUIRED",
    "QA_PASSED",
    "QA_FAILED",
    "NEEDS_CHANGES",
    "BLOCKED",
    "REVIEW_REQUIRED",
    "MERGED",
    "DONE",
    "SCOPE_DECISION_REQUIRED",
    "DEFERRED_BY_SCOPE_DECISION",
    "FAILED",
    "CANCELLED",
    "SUPERSEDED",
}
VALID_RISK = {"low", "medium", "high", "critical"}


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(message)


def read(path: str) -> str:
    full = ROOT / path
    require(full.exists(), f"missing {path}")
    return full.read_text()


def check_required_role_ids(path: str) -> None:
    text = read(path)
    for role_id in ROLE_IDS:
        require(re.search(rf"^\s{{2}}{re.escape(role_id)}:", text, re.MULTILINE), f"{path} missing role {role_id}")


def check_gate_policy() -> None:
    text = read(".agentflow/policies/gates.yaml")
    for token in [
        "owner_cannot_review_own_task: true",
        "blocked_is_not_done: true",
        "qa_passed_is_not_done: true",
        "python_api_removal",
        "pyo3_removal",
        "cython_removal",
        "release_gatekeeper_approved",
    ]:
        require(token in text, f"gates.yaml missing {token}")


def check_task_status() -> None:
    path = ROOT / ".agentflow/state/task_status.json"
    require(path.exists(), "missing .agentflow/state/task_status.json")
    data = json.loads(path.read_text())
    tasks = data.get("tasks")
    require(isinstance(tasks, dict), "task_status.json tasks must be object")

    task_files = sorted((ROOT / "docs/rust-cutover/tasks").glob("*.md"))
    task_ids = {p.stem for p in task_files}
    require(task_ids == set(tasks), "task_status.json must cover every docs/rust-cutover task exactly")

    for task_id, task in tasks.items():
        status = task.get("status")
        owner = task.get("owner_role")
        review = task.get("review_role")
        risk = task.get("risk_level")
        done_requires = task.get("done_requires")
        require(status in VALID_STATES, f"{task_id} invalid status {status}")
        require(owner in ROLE_IDS, f"{task_id} invalid owner_role {owner}")
        require(review in ROLE_IDS, f"{task_id} invalid review_role {review}")
        require(owner != review, f"{task_id} owner_role must not equal review_role")
        require(risk in VALID_RISK, f"{task_id} invalid risk_level {risk}")
        require(isinstance(done_requires, list) and done_requires, f"{task_id} missing done_requires")


def main() -> int:
    check_required_role_ids(".agentflow/roles.yaml")
    check_required_role_ids(".agentflow/policies/path_scope.yaml")
    check_gate_policy()
    check_task_status()
    print("agentflow role protocol validation passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
