# PTC-008 Zero-Python Closeout Migration

Date: 2026-07-17
Executor: Codex

## Current Authority

Repository development and validation are governed by Rust, Cargo, shell, jq,
and pinned native tools. `ntpro-governance zero-python-closeout` is the final
repository drift guard. It checks the Git tracked tree and local generated
artifacts, while historical `docs/rust-cutover/` records remain audit evidence.

Supported entry points:

```text
scripts/ai/check_zero_python_closeout.sh
scripts/ai/verify_release.sh current-governance
.github/workflows/rust-cutover-smoke.yml
```

## Fail-Closed Behavior

The guard rejects:

- tracked Python source, type stubs, Cython, notebooks, and bytecode;
- Python manifests/locks at any tracked path, plus local `.venv`, `venv`,
  `__pycache__`, and `.pyc`;
- executable Python, uv, pytest, Ruff, and pip commands in supported scripts;
- Python setup and wheel build/upload action or workflow surfaces.

Historical prose is not interpreted as executable tooling. Required PTC task,
evidence, baseline, retirement manifest, and inventory files must remain
tracked, so a text-clean zero result cannot be achieved by deleting history.

## Boundary

This governance closeout is not a backend patch or capability release.
v0.32.0 remains the frozen Backend Production Closeout baseline, and no submit,
mutation, adapter send, live exchange, retry, remediation, or trading-control
capability is added or inherited.
