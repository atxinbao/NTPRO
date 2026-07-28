# Contributing to NTPRO

NTPRO accepts focused contributions that preserve the current Rust-only product
surface and the frozen v0.32.0 backend baseline.

> [!NOTE]
>
> **Integrations:**
> New integrations are a major undertaking for the project and therefore require additional discussion and approval before opening any PRs.
> Please see the
> [v0.33.0+ intake policy](../docs/rust-cutover/governance/v0_33_plus_intake_policy.md)
> for the current approval process and the
> [adapter policy](../docs/governance/adapter_policy.md) for adapter tiers,
> community listings, and support boundaries.

## Steps

To contribute, follow these steps:

1. Open a GitHub issue and agree on its scope before implementation. Post-freeze
   work must declare whether it touches the backend baseline or requests a
   separately scoped capability.

2. Create one branch from the latest `main` for that issue:

   ```bash
   git fetch origin
   git switch -c <branch-name> origin/main
   ```

3. Install the pinned Rust toolchain, workspace dependencies, and development
   tools. [prek](https://github.com/j178/prek) runs the repository hooks:

   ```bash
   rustup toolchain install 1.95.0
   cargo install cargo-binstall --locked
   make install-deps
   make install-tools
   prek install
   ```

   Tool versions are controlled by `rust-toolchain.toml`,
   `Cargo.toml` under `[workspace.metadata.tools]`, and `tools.toml`. The
   repository does not use Python, uv, or `pyproject.toml` as development
   tooling authority.

4. Keep the change within the issue scope, add tests for behavior changes, and
   run the applicable local checks:

   ```bash
   scripts/ai/verify_fast.sh
   scripts/ai/verify_full.sh
   make docs-check-links
   scripts/ai/check_zero_python_closeout.sh
   scripts/ai/verify_release.sh backend-freeze-baseline
   ```

   `verify_fast.sh` is only a fast formatting and toolchain smoke. Use targeted
   tests and `verify_full.sh` when the change affects code behavior. Documentation
   changes must pass `make docs-check-links`. Post-freeze governance changes must
   also pass the zero-Python and backend-freeze gates shown above.

5. Open the pull request against `main`. Include the task ID, a plain Chinese
   summary, goal, changed files, commands and results, behavior and public API
   impact, migration-note status, rollback plan, and `Closes #<issue>`.

6. Wait for hosted checks and the required independent review before merging.
   Work above medium risk stops at `REVIEW_REQUIRED`. See the
   [task execution protocol](../docs/rust-cutover/TASK_EXECUTION.md) for the
   complete risk and review rules.

7. Read and accept the repository [Contributor License Agreement](CLA.md).

## Tips

- Read `AGENTS.md` and the task file under `docs/rust-cutover/tasks/` before
  changing code.
- Use the [Rust cutover contract](../docs/rust-cutover/CONTRACT.md) and
  [definition of done](../docs/rust-cutover/DEFINITION_OF_DONE.md) as the product
  boundary.
- Use sentence case for Markdown headings below H1. Repository hooks enforce
  the current Rust and documentation conventions.
- Keep PRs small and focused for easier review.
- Reference the relevant GitHub issue(s) in your PR comment.
