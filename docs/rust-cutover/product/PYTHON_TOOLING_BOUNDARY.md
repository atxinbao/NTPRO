# Python Tooling Boundary

Date: 2026-06-05
Executor: Codex
Task ID: NAUDIT-001

## Decision

The root `pyproject.toml` is retained only as local helper-tool configuration.
It is not a Python package manifest for NTPRO.

Allowed root `pyproject.toml` content:

- `dependency-groups` for local helper tools used by retained scripts, docs,
  linting, or tests.
- `[tool.uv]` configuration, including `required-version` and
  `no-build-package` drift checks.
- `[tool.ruff]`, `[tool.mypy]`, `[tool.pytest.*]`, and `[tool.coverage.*]`
  configuration for retained helper scripts and historical test tooling.

Forbidden root Python package metadata:

- `[project]`
- `[project.urls]`
- `[project.optional-dependencies]`
- root Python runtime `dependencies`
- `requires-python`
- Python package classifiers
- PyPI, wheel, sdist, or editable root package publication signals
- upstream NautilusTrader package homepage/repository/docs URLs as current
  NTPRO package metadata

## Product Boundary

NTPRO's supported product surface is Rust-only:

- Rust workspace and crates through Cargo;
- Rust CLI entrypoints;
- Rust examples and Rust cutover documentation.

Python helper tooling may remain for local repository maintenance, but it is
not a supported product runtime, API surface, package install path, or release
artifact.

## Lockfile Boundary

`uv.lock` may retain third-party packages used by local helper tooling. It must
not retain an editable root `nautilus-trader` or `nautilus_trader` package entry
for the repository itself.

## Operational Guard

`scripts/ai/check_rust_only_runtime.sh` now fails if root Python package
metadata is reintroduced in `pyproject.toml` or if `uv.lock` again records this
repository as an editable Python package.

## Migration Note

Users must not run NTPRO as:

- `pip install nautilus_trader`
- `uv build --wheel`
- `import nautilus_trader`

Use Cargo, Rust crates, and the Rust CLI instead.
