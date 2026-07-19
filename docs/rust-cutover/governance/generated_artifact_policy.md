# Backend Freeze Generated Artifact Policy

Date: 2026-07-15
Executor: Codex
Baseline: `ntpro-rust-only-v0.32.0`
Status: ACTIVE

## Decision

Post-baseline verification may generate local files, but those files do not
become release authority and must not pollute the Git worktree. NTPRO classifies
evidence by durability and authority rather than by filename alone.

Plain Chinese summary: 本地运行发布检查、strict provenance 或项目分析时可以生成文件，
但这些文件不是唯一发布证明，也不应该出现在待提交列表。正式审计仍依赖 tracked source
evidence 与 GitHub remote 事实共同重建；本地生成物可以安全重建，不进入版本库。

## Classification

### Tracked Source Evidence

These files are reviewed and committed because they define or record durable
contracts:

- `docs/rust-cutover/release/` release manifests, notes, readiness reports, and
  source-controlled closeout contracts;
- `docs/rust-cutover/evidence/` task validation summaries;
- `docs/rust-cutover/governance/` freeze registry, policy, errata, and intake
  rules;
- verification scripts under `scripts/ai/`.

Tracked source evidence must be changed through one issue, one branch, and one
PR. The frozen `docs/rust-cutover/release/v0_32_0_*` package is not routine
cleanup scope.

### Remote-Reconstructable Evidence

These GitHub facts are authoritative only when bound to tracked source:

- annotated release tag and peeled commit;
- published GitHub Release and release body;
- hosted workflow run, head SHA, status, and conclusion;
- milestone, exact issue set, PRs, and closeout state.

The audit strategy remains `source_tree_plus_github_remote`. Remote facts alone
do not authorize new capability, and local generated files alone do not prove
publication.

### Ephemeral Local Output

The following outputs are reproducible working artifacts and are not tracked:

- `/release-publication-evidence/` generated publication snapshots;
- `/graphify-out/` generated project analysis graphs and reports;
- `/target/ntpro-v*/` strict-provenance and release-gate manifests, already
  covered by the repository-wide `*target/` ignore rule;
- temporary `ntpro-*` directories created under the operating-system temp
  directory and removed by their owning checks.

`.codex/` and `.agentflow/` remain governed by their existing repository ignore
rules. This policy does not reclassify them as release evidence.

## Hygiene Rules

1. Do not commit ephemeral local output, even when it contains successful gate
   results.
2. Do not delete tracked historical evidence during cleanup.
3. Verification may overwrite its own ephemeral output when the format and
   path are already defined by the owning script.
4. A clean checkout must remain free of untracked generated output after
   verification; ignored output is acceptable and reproducible.
5. Before commit, use `git status --short` and `git check-ignore -v` to confirm
   that generated paths are ignored and task files are not.
6. If a generated output becomes required for durable review, promote only its
   stable contract or summarized evidence through a dedicated issue and PR.

## Cleanup Interface

The supported cleanup commands are path allowlists, not Git ownership
decisions:

```text
make clean-dry-run          list reproducible build/report output
make clean                  remove only that build/report output
make clean-generated-dry-run
                            list release-publication-evidence and graphify-out
make clean-generated FORCE=1
                            remove only those generated directories
make distclean-dry-run      list the combined cleanup set and protected paths
make distclean FORCE=1      remove the combined allowlist
```

`distclean` never runs `git clean` and does not delete arbitrary untracked or
ignored files. `.codex/`, `.agentflow/`, `.understand-anything/`,
`project.html`, `tests/test_data/large/`, and `tests/test_data/local/` are
outside every cleanup allowlist. Cargo registry and Git caches are also outside
scope.

## Verification Contract

The following paths must resolve to ignore rules without requiring the files to
exist:

```text
release-publication-evidence/ntpro-rust-only-v0.32.0.json
graphify-out/GRAPH_REPORT.md
target/ntpro-v320/v0_32_0_strict_release_manifest.json
```

No file under these paths may be tracked at BFG-004 closeout. Running the
publication guard, backend-freeze guard, strict-provenance gate, and fast smoke
must not add undeclared paths to `git status --short`.
