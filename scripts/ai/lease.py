#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

LEASE_DIR = Path(".agentflow/leases")


def now() -> str:
    return datetime.now(timezone.utc).isoformat()


def load(path: Path):
    return json.loads(path.read_text()) if path.exists() else None


def claim(args):
    LEASE_DIR.mkdir(parents=True, exist_ok=True)
    lease_path = LEASE_DIR / f"{args.task_id}.json"
    if lease_path.exists() and not args.force:
        print(f"lease already exists: {lease_path}", file=sys.stderr)
        sys.exit(2)
    data = {
        "task_id": args.task_id,
        "agent_id": args.agent_id or os.environ.get("USER", "unknown-agent"),
        "branch": args.branch,
        "touched_paths": args.path,
        "claimed_at": now(),
        "status": "LEASED",
    }
    lease_path.write_text(json.dumps(data, indent=2) + "\n")
    print(f"claimed {lease_path}")


def release(args):
    lease_path = LEASE_DIR / f"{args.task_id}.json"
    data = load(lease_path)
    if not data:
        print(f"lease not found: {lease_path}", file=sys.stderr)
        sys.exit(1)
    data["status"] = args.status
    data["released_at"] = now()
    lease_path.write_text(json.dumps(data, indent=2) + "\n")
    print(f"released {lease_path} as {args.status}")


def list_leases(_args):
    for p in sorted(LEASE_DIR.glob("*.json")):
        print(p.read_text().strip())


def main():
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(required=True)
    c = sub.add_parser("claim")
    c.add_argument("task_id")
    c.add_argument("--branch", required=True)
    c.add_argument("--agent-id")
    c.add_argument("--path", action="append", default=[])
    c.add_argument("--force", action="store_true")
    c.set_defaults(func=claim)
    r = sub.add_parser("release")
    r.add_argument("task_id")
    r.add_argument("--status", default="PR_READY", choices=["PR_READY", "DONE", "BLOCKED", "ABANDONED"])
    r.set_defaults(func=release)
    l = sub.add_parser("list")
    l.set_defaults(func=list_leases)
    args = parser.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
