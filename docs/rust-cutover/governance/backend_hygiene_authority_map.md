# Post-Freeze Backend Hygiene Authority Map

Date: 2026-07-19
Executor: Codex
Baseline commit: `0a265368ab`
Milestone: `post-freeze-backend-hygiene` (#34)
Classification: `baseline-preserving-governance`
Status: ACTIVE

## Decision

Repository hygiene after the v0.32.0 backend freeze must distinguish immutable
release facts, retained source authority, reproducible local output, user-owned
local state, and separately scoped runtime work. A path is not removable merely
because it is large, historical, ignored, or not part of the product binary.

Plain Chinese summary: 本文件是 v0.32.0 后端冻结后的仓库清理总表。冻结发布证据和
Rust cutover 审计链继续保留；缓存和 Finder 元数据可以清理；本地分析结果及大型测试
数据必须先确认所有权；`crates/**` 的重构和性能优化属于独立的 v0.33.0 maintenance
路线，不能伪装成普通文件清理。

## Captured Source Baseline

The following counts are descriptive evidence captured from Git tracked state
at BFH-001. They are not permanent quotas.

| Surface | Tracked files | Tracked bytes | Authority |
| --- | ---: | ---: | --- |
| repository total | 5,253 | n/a | mixed; use the classes below |
| `crates/` | 2,700 | 43,298,673 | Rust product/runtime source; separately scoped only |
| `docs/` | 2,044 | 11,143,868 | current docs plus retained audit evidence |
| `docs/rust-cutover/` | 1,929 | included above | retained audit and governance system |
| `tests/` | 339 | 82,984,239 | retained tests and fixtures pending BFH-006 ownership audit |
| `scripts/` | 54 | 156,201 | supported Rust-only tooling; scoped BFH-003/BFH-005 cleanup |
| `.github/` | 26 | 77,556 | active issue, PR, CI, and release control surface |
| `.pre-commit-hooks/` | 18 | 127,579 | active Rust-only developer checks |
| `examples/` | 14 | 16,742 | retained canonical Rust examples |
| `assets/` | 7 | 713,671 | retained referenced product/documentation media |
| `schema/` | 4 | 28,477 | retained data and contract schemas |
| `.supply-chain/` | 3 | 199,453 | retained dependency audit authority |
| `.cargo/`, `.config/`, `.docker/`, `configs/` | 6 | 6,517 | retained build, service, and runtime configuration |

Additional retained facts:

- `docs/rust-cutover/release/` contains 420 tracked release files;
- `docs/rust-cutover/release/v0_32_0_*` contains 22 frozen baseline files;
- `docs/rust-cutover/tasks/` contains 619 tracked task records;
- `docs/rust-cutover/evidence/` contains 775 tracked evidence records;
- `tests/golden/` contains 59 tracked golden fixtures;
- tracked Python, Cython, stub, and notebook source count is zero.

## Authority Classes

### Immutable Frozen Baseline

- the tag, release, hosted gate, milestone, and exact issue set registered by
  `backend_freeze_registry.json`;
- all files matching `docs/rust-cutover/release/v0_32_0_*`;
- the 27 explicit-false capability boundary fields and their registered source
  hashes.

Routine cleanup must not edit, delete, move, or regenerate these surfaces.
Clarification belongs in governance errata. A proven baseline defect requires a
separate `backend-freeze-exception`.

### Retained Audit And Control Authority

- `docs/rust-cutover/`, including task, evidence, release, governance, golden
  trace, inventory, migration, product, and scope records;
- `.github/` issue templates, PR template, workflows, actions, and CODEOWNERS;
- `scripts/ai/`, current Rust-only checks, `AGENTS.md`, and cutover contracts;
- `.supply-chain/`, `deny.toml`, `osv-scanner.toml`, and security policy;
- Git history, published tags, GitHub Releases, hosted runs, issues, PRs, and
  milestones.

These are not caches. BFH tasks may add scoped governance or repair a proven
current-route defect, but directory-wide historical cleanup is forbidden.

### Retained Product, Test, And Input Authority

- `crates/`, `Cargo.toml`, `Cargo.lock`, and supported Rust profiles/features;
- `tests/`, `tests/golden/`, fixture manifests, and tracked deterministic test
  data;
- `examples/`, `schema/`, `configs/`, `.cargo/`, `.config/`, `.docker/`, and
  referenced `assets/`;
- supported top-level product, contributor, security, release, and roadmap
  documents.

BFH-006 may classify fixture ownership and propose a deterministic replacement,
but no fixture is deleted by size alone. Runtime source optimization is outside
this milestone and enters only through the approved v0.33.0 maintenance intake.

### Tracked Cleanup Candidates

- unreachable `.gitignore` exceptions and retired configuration/profile names;
- supported Make cleanup targets whose behavior is redundant or destructive;
- current contributor instructions that reference deleted manifests, scripts,
  or documentation paths;
- a generated-output rule or guard that is stale relative to current ownership.

BFH-003, BFH-004, and BFH-005 own these surfaces. Every removal requires a
repository-wide reachability check and task-specific validation.

BFH-003 outcome: 28 explicit `.gitignore` exceptions for absent files and the
retired `scripts/control/*.sh` exception were removed. The unused
`ci-pr-wheel` Cargo profile was removed, and the sole active comment naming the
deleted `scripts/ci/plan.sh` helper was corrected. All retained Make, workflow,
action, pre-commit, and script execution references resolve to tracked paths.

BFH-004 outcome: `CONTRIBUTING.md` now routes contributors through `main`, the
pinned Rust toolchain, Cargo-owned tool manifests, Rust-only validation,
post-freeze governance gates, the local CLA, and the current task execution
protocol. References to Python, uv, `pyproject.toml`, retired developer-guide
setup pages, the upstream `develop` branch, and an absent CLA Assistant workflow
were removed from the current contributor route.

BFH-005 outcome: the redundant `clean-build-artifacts`, `clean-caches`, and
`clean-builds` targets and the unbounded `git clean -fxd` route were removed.
`make clean` now uses a documented build-output allowlist. Generated audit and
analysis output requires `FORCE=1`, both cleanup levels have dry-run targets,
and user-owned local state and test data remain outside every deletion set.

BFH-006 outcome: `backend_fixture_inventory.json` records all 17 test fixtures
at or above 1 MiB, totaling 324,882,650 bytes. Three ignored market-data caches
have active Rust consumers, checksums, metadata, and deterministic download or
local-cache rules. Fourteen tracked files totaling 76,020,960 bytes have no
active Rust consumer and are retained in quarantine because no
fidelity-preserving replacement is yet proven. Size alone does not authorize
their deletion.

### Reproducible Local Output

The following paths may be removed locally when present and must remain
untracked:

- `/target/`, `/target-v2/`, `/build/`, `/dist/`, `.coverage*`, and
  `.benchmarks*`;
- `/release-publication-evidence/` and `/graphify-out/`;
- `__pycache__/`, `*.pyc`, `.pytest_cache/`, `.ruff_cache/`, and retired local
  Python environments;
- `.DS_Store`.

At BFH-001 capture time, `target/`, `target-v2/`,
`release-publication-evidence/`, `graphify-out/`, `.venv/`, and Python caches
are absent. Eight ignored `.DS_Store` files remain and are owned by BFH-002.

BFH-002 outcome: the eight `.DS_Store` files (84,000 bytes total) were removed
with a path-scoped `find` command. A repository-wide recount returned zero.

### User-Owned Or Conditionally Removable Local State

- untracked `project.html` (52 KiB), a local NTPRO review/roadmap HTML report;
- ignored `.agentflow/` (1.0 MiB), `.codex/` (37 MiB), and
  `.understand-anything/` (14 MiB);
- ignored `tests/test_data/large/` (237 MiB) and `tests/test_data/local/` when
  present;
- local service containers, volumes, credentials, databases, logs, or operator
  output.

These paths are not release authority, but they may contain user work,
development caches, or expensive test inputs. BFH-002 may remove only confirmed
filesystem noise. BFH-005 must keep destructive cleanup explicit and preserve
declared exclusions. BFH-006 owns large fixture disposition.

BFH-002 outcome: `project.html` is a local Codex review/roadmap report generated
on 2026-07-15. It contains no credential-like text, is not current source or
release authority, and is retained under the local `/project.html` rule in
`.git/info/exclude` so it cannot enter an unrelated PR. `.agentflow/`, `.codex/`,
`.understand-anything/`, and `tests/test_data/large/` remain intact.

BFH-006 fixture policy: a quarantined tracked fixture may be removed only
through a dedicated owner issue after deterministic replacement or
externalization, provenance, content hash, consumer migration, targeted Rust
tests, offline reproducibility, and backend-freeze evidence are all present.
Live network responses are not acceptable fixture replacements.

### Separately Scoped Runtime Work

Refactoring, dependency/feature changes, panic/error hardening, and measured
runtime optimization under `crates/**` are not hygiene deletion. They are
blocked until BFH-007 closes and must follow the approved
`v0.33.0-backend-maintenance` intake, independent review, targeted tests, and
benchmark or correctness evidence.

No such work inherits backend go-live, submit, mutation, adapter send, live
exchange, retry, remediation, recovery, or trading-control authority from
v0.32.0.

## Execution Map

| Issue | Owned surface | Dependency |
| --- | --- | --- |
| `#1112` / BFH-001 | authority map and captured inventory | none |
| `#1113` / BFH-002 | `.DS_Store`, `project.html`, local analysis ownership | BFH-001 |
| `#1114` / BFH-003 | stale ignore, Cargo profile, and tooling references | BFH-001 |
| `#1115` / BFH-004 | current Rust-only contributor routes | BFH-001 |
| `#1116` / BFH-005 | build artifact and local-state cleanup semantics | BFH-003 |
| `#1117` / BFH-006 | tracked and ignored fixture ownership and size | BFH-001 |
| `#1118` / BFH-007 | hygiene drift guard and milestone closeout | BFH-002 through BFH-006 |

Phase 2 issues `#1120-#1126` are all blocked by BFH-007 (`#1118`).

## Reproduction Commands

```bash
git ls-files | wc -l
git ls-files 'crates/**' | wc -l
git ls-files 'docs/**' | wc -l
git ls-files 'docs/rust-cutover/**' | wc -l
git ls-files 'tests/**' | wc -l
git ls-files 'scripts/**' | wc -l
git ls-files '.github/**' | wc -l
git ls-files 'examples/**' | wc -l
git ls-files '*.py' '*.pyi' '*.pyx' '*.pxd' '*.ipynb'
git status --short --ignored
```

## Guardrails

1. One issue, one branch, and one PR for every tracked cleanup.
2. A later issue must cite this map and remain within its owned surface.
3. Large, historical, ignored, or untracked does not mean automatically
   removable.
4. Cleanup uses the BFH-005 path allowlists; `make distclean FORCE=1` never
   grants authority to delete arbitrary untracked or ignored files.
5. Deletion requires positive ownership, reachability, replacement, and
   validation evidence.
6. Frozen v0.32.0 release files and registered boundaries remain unchanged.
