# DEXG-007 Docs Build And Link Gate Evidence

Date: 2026-07-15
Executor: Codex
GitHub issue: #1086
Milestone: post-backend-docs-examples-governance
Status: VALIDATION PASSED

## Summary

This evidence covers the deterministic docs/examples governance gate, actual
Rust docs build, and generated-output cleanup.

Plain Chinese summary: 本任务把 Rust docs build、内部链接、authority、examples、
assets 和 backend freeze 组合为同一个可重复 gate，并把网络外链检查拆为非阻断的
periodic target。

## Files

- `Makefile`
- `.github/workflows/rust-cutover-smoke.yml`
- `scripts/ai/check_docs_examples_governance.sh`
- `docs/rust-cutover/tasks/DEXG-007.md`
- `docs/rust-cutover/evidence/DEXG-007.md`

## Validation

The first `make docs` attempt exposed a stale Makefile assumption: the active
Cargo binary is not a rustup proxy, and no nightly toolchain is installed, so
`cargo +nightly doc` could not start. The supported workspace toolchain is Rust
1.95.0 and regular crate docs do not require unstable index-page flags.
DEXG-007 therefore routes `docs-rust` through stable `cargo doc`; the separate
`docsrs-check` nightly contract is unchanged.

The corrected `make docs` run completed in 3 minutes 34 seconds and generated
the workspace Rust documentation under `target/doc`. Before cleanup, `target/`
used 2.3 GiB and `target/doc/` used 549 MiB. The post-build governance gate
reported:

```text
docs_examples_governance=pass markdown_files=105 local_links=293 image_links=20 integration_pages=15 python_fences_classified=203 concept_pages=9 tutorial_assets=20
rust_examples_integrity=pass required_paths=14 toml_files=7 readme_paths=7
backend_freeze_baseline=pass tag=ntpro-rust-only-v0.32.0 commit=2b955cb8a989827e3351c08c3d82d9578253e1f6 boundaries=27 source_hashes=4
backend_freeze_negative_selftest=pass cases=20
```

Final commands:

```bash
bash -n scripts/ai/check_docs_examples_governance.sh
scripts/ai/check_docs_examples_governance.sh
make docs-check-links
make docs
cargo clean
scripts/ai/verify_fast.sh
git diff --check
```

The generated `target/` output is removed after validation and is not part of
the source-controlled result.

## Behavior Impact

None. Documentation build and governance validation only.
