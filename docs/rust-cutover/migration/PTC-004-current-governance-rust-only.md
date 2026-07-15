# PTC-004 Current Governance Rust-Only Migration

Date: 2026-07-16
Executor: Codex

## Supported Commands

```bash
scripts/ai/ntpro_governance.sh release-surface
scripts/ai/ntpro_governance.sh docs-examples
scripts/ai/ntpro_governance.sh rust-examples
scripts/ai/ntpro_governance.sh backend-freeze
scripts/ai/ntpro_governance.sh release-publish-binding \
  --manifest PATH --closeout PATH --version VERSION --tag TAG \
  --name NAME --gate-run-id RUN_ID --tag-sha SHA
```

Existing shell entrypoints remain supported. They now dispatch structured
validation to Rust and use `jq` only for GitHub JSON extraction or generated
publication evidence.

## Preserved Boundaries

- v0.32.0 remains the frozen backend baseline;
- 27 backend capability flags remain explicit false;
- 20 backend-freeze negative mutations remain mandatory;
- release body comparison remains normalized SHA-256, with raw SHA-256
  reported separately;
- publication remains blocked until a successful hosted gate for the same tag
  commit.

## Deferred Ownership

Historical `verify_v*.sh` and the release-tag historical matrix remain for
PTC-006. Python/wheel CI, `pip-audit`, Make, pre-commit, `pyproject.toml`, and
`uv.lock` remain for PTC-007. PTC-008 owns the final repository-wide zero-Python
guard.
