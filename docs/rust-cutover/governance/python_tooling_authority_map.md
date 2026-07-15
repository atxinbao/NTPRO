# Python Tooling Closeout Authority Map

Date: 2026-07-15
Executor: Codex
Baseline: `20455c86be25c5b7d083ce0a67d4053a844352d2`
Milestone: `python-tooling-closeout` (#33)
Status: ACTIVE

## Decision

NTPRO product build and backend runtime are Rust-only, but repository tooling
is not yet Python-free. The closeout must replace current validation authority
before deleting Python, then retire historical executables and packaging
surfaces, and only then enable a zero-Python drift guard.

Plain Chinese summary: 初始“只剩 8 个 Python 文件”的盘点只覆盖 tracked `.py`，
没有覆盖 shell heredoc、Make 和 GitHub Actions 中的 Python 执行。修正后的基线是
8 个 Python 文件、263 个含 Python/tooling 执行的 shell 文件、18 个 workflow/action
文件和 695 条执行入口。历史 `docs/rust-cutover/` 继续保留；当前验证能力先迁移到
Rust，历史可执行 gate 再退役，最后删除 Python 工具链并 fail closed。

The machine-readable baseline is `python_tooling_baseline.json`.

## Corrected Baseline

| Surface | Count | Meaning |
| --- | ---: | --- |
| tracked Python files | 8 | standalone Python implementations under `scripts/` |
| tracked shell files | 339 | all tracked `scripts/**/*.sh` and `scripts/**/*.bash` |
| shell files executing Python tooling | 263 | Python, uv, pytest, Ruff, or pip-audit execution |
| versioned `verify_v*.sh` files | 262 | historical release verification executables |
| versioned files executing Python | 236 | historical executable retirement scope |
| shell files with inline Python heredoc | 233 | Python hidden from extension-only inventory |
| workflow/action files with Python or wheel paths | 18 | CI and packaging cleanup scope |
| executable invocation lines | 695 | Python/tooling calls across scripts, CI, and Make |
| packages in `uv.lock` | 55 | helper dependency surface, not product runtime |
| local `.venv` size | 136 MiB | ephemeral local environment, removed last |

Counts describe the captured baseline, not permanent acceptance thresholds.
Every count must reach the PTC-008 contract instead of being maintained.

## Authority Classes

### Replace Before Deletion

- `golden_trace_runner.py`: JSONL validation and optional replay comparison;
- `validate_golden_trace_release_scope.py`: manifest/trace reconciliation and
  executable/schema-only classification;
- `validate_v21_read_model_schema.py`: Draft 2020-12 read-model schema,
  capability boundary, and negative mutation validation.

These are active correctness checks. PTC-002 and PTC-003 must provide tested
Rust equivalents before deleting the Python files or changing callers.

### Current Governance Migration

Current merge and freeze authority includes Rust Cutover Smoke, backend freeze,
docs/examples, release-surface, Rust-only, and release publication checks.
PTC-004 must remove embedded Python from retained current authority while
preserving deterministic positive and fail-closed negative behavior.

### Retire Or Replace Manual Control

- `lease.py` and `dispatch_next.py` mutate legacy AgentFlow/Shrimp local state;
- `close_merged_pr.py` mirrors merged PR state into legacy Shrimp local state;
- `validate_agentflow_roles.py` validates an old task-role registry;
- `inventory_cython.py` regenerates an inventory after Cython reached zero.

None is called by a tracked Make target, workflow, or non-Python control entry.
PTC-005 must either bind a still-required operation to Rust or documented
standard `git`/`gh`/`jq` commands, or record formal retirement. Historical
inventory reports remain retained.

### Retire Historical Executables

The 262 `scripts/ai/verify_v*.sh` files describe successive release lines;
236 still execute Python. v0.32.0 is now the frozen backend baseline, so main
must not carry every historical release implementation as current executable
authority. PTC-006 owns a reviewed deletion manifest and simplified current
smoke/release contract. Immutable tags, GitHub Releases, hosted runs,
`tests/golden`, Rust integration tests, and `docs/rust-cutover/` retain history.

PTC-006 is high risk. It must stop at `REVIEW_REQUIRED`; auto-merge is forbidden.

### Remove Toolchain And Packaging Last

PTC-007 owns selective cleanup of Make targets, pre-commit Python hooks,
Python/wheel workflows and composite actions, obsolete test scripts,
`pyproject.toml`, and `uv.lock`. Rust jobs in mixed workflows must be retained
or moved before Python jobs are removed. The local `.venv` is deleted only
after tracked references reach zero.

PTC-007 is high risk. It must stop at `REVIEW_REQUIRED`; auto-merge is forbidden.

### Retained Historical Evidence

The following are not runtime or tooling residue and must not be text-cleaned:

- `docs/rust-cutover/` release, task, evidence, migration, inventory, and
  governance records;
- published tags and GitHub Releases;
- hosted workflow run references;
- `tests/golden/` fixtures and Rust golden-trace integration tests.

Historical mentions of Python, Cython, PyO3, wheels, or deleted script names are
allowed in these evidence surfaces. Executable references from supported
scripts, Make, workflows, or actions are not allowed after closeout.

## Execution Map

| Issue | Owned surface | Dependency |
| --- | --- | --- |
| `#1096` / PTC-001 | authority map and corrected baseline | none |
| `#1097` / PTC-002 | golden trace and release-scope Rust validators | PTC-001 |
| `#1098` / PTC-003 | v0.21 read-model Rust schema validator | PTC-002 |
| `#1099` / PTC-004 | current governance/smoke Python removal | PTC-002, PTC-003 |
| `#1100` / PTC-005 | control-plane and Cython inventory retirement | PTC-001 |
| `#1101` / PTC-006 | historical executable gate retirement | PTC-004, PTC-005 |
| `#1102` / PTC-007 | Make, CI, action, dependency, and `.venv` cleanup | PTC-006 |
| `#1103` / PTC-008 | strict zero-Python guard and live closeout | all preceding |

## Final Contract

PTC-008 may close milestone #33 only when all conditions hold:

```text
tracked .py/.pyi/.pyx/.pxd/.ipynb files = 0
pyproject.toml = absent
uv.lock = absent
supported script Python/uv/pytest/Ruff/pip execution = 0
supported workflow/action Python or wheel execution = 0
local .venv = absent
current Rust smoke, docs, security, and freeze gates = pass
historical docs/rust-cutover evidence = retained
v0.32.0 frozen release files changed = false
```

No PTC task authorizes backend capability, trading controls, mutation, adapter
send, live exchange access, retry, remediation, or a v0.32.1 release.
