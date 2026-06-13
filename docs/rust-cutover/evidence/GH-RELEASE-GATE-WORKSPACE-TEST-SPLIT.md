# GH Release Gate Workspace Test Split Evidence

Date: 2026-06-13
Executor: Codex

## Task

Release gate optimization after `full-rust-tests-workspace` remained the last
long-running black-box stage in GitHub Actions.

## Goal

Preserve the release gate's full Rust test coverage while reducing wall-clock
time and making failures easier to locate. The previous single workspace test
stage is split into smaller package partitions:

- `full-rust-tests-workspace-core`
- `full-rust-tests-workspace-runtime`
- `full-rust-tests-workspace-adapters-a`
- `full-rust-tests-workspace-adapters-b`

## Root Cause

After the prior release gate split, hosted run `27450423485` showed that the
golden trace stages, log-global tests, live-node serial tests, dashboard smoke,
supervisor smoke, clippy, docs, and Rust-only gates could complete as separate
stages. The only remaining long-running black box was
`full-rust-tests-workspace`, which still ran all workspace packages in one
GitHub Actions matrix entry.

## Change Summary

`scripts/ai/verify_full.sh` now supports four workspace test partitions:

- `rust-tests-workspace-core`
- `rust-tests-workspace-runtime`
- `rust-tests-workspace-adapters-a`
- `rust-tests-workspace-adapters-b`

The original aggregate stage remains available:

- `scripts/ai/verify_full.sh rust-tests-workspace`

`.github/workflows/release-tag.yml` now runs the four partitions independently
instead of a single `full-rust-tests-workspace` matrix entry.

`rust-cutover-smoke.yml` now treats release-verification workflow/script-only
changes as a separate light PR class. These PRs skip duplicate workspace
`cargo check`, workspace clippy, product-path warning clippy, CLI tests, and
supervisor smoke, then run targeted shell/YAML checks for the release
verification scripts instead.

## Files Changed

- `.github/workflows/release-tag.yml`
- `.github/workflows/rust-cutover-smoke.yml`
- `scripts/ai/verify_full.sh`
- `docs/rust-cutover/evidence/GH-RELEASE-GATE-WORKSPACE-TEST-SPLIT.md`

## Commands Run

```bash
scripts/ai/verify_fast.sh
```

Result: passed.

Output summary:

```text
== verify_fast complete: fast smoke only; release work still requires stronger verification ==
```

```bash
bash -n scripts/ai/verify_full.sh scripts/ai/verify_release.sh scripts/ai/v03_dashboard_smoke.sh
```

Result: passed.

```bash
ruby -e 'require "yaml"; YAML.load_file(".github/workflows/release-tag.yml"); puts "release-tag yaml ok"'
```

Result: passed.

Output summary:

```text
release-tag yaml ok
```

```bash
ruby -e 'require "yaml"; %w[.github/workflows/release-tag.yml .github/workflows/rust-cutover-smoke.yml].each { |p| YAML.load_file(p); puts "#{p} yaml ok" }'
```

Result: passed.

Output summary:

```text
.github/workflows/release-tag.yml yaml ok
.github/workflows/rust-cutover-smoke.yml yaml ok
```

```bash
{ git diff --name-only main...HEAD; git diff --name-only; } | sort -u | tee /tmp/ntpro-pr274-changed-files.txt
if grep -Ev '^(README\.md|CHANGELOG.*|MIGRATION.*|docs/|release/|\.agentflow/|\.github/(ISSUE_TEMPLATE/|pull_request_template\.md|workflows/(release-tag|rust-cutover-smoke)\.yml)|scripts/ai/(verify_full|verify_release|v03_dashboard_smoke)\.sh|\.gitignore$)' /tmp/ntpro-pr274-changed-files.txt >/tmp/ntpro-pr274-heavy-files.txt; then
  echo heavy_rust=true
  cat /tmp/ntpro-pr274-heavy-files.txt
else
  echo heavy_rust=false
  if grep -E '^(\.github/workflows/(release-tag|rust-cutover-smoke)\.yml|scripts/ai/(verify_full|verify_release|v03_dashboard_smoke)\.sh|docs/rust-cutover/evidence/GH-RELEASE-)' /tmp/ntpro-pr274-changed-files.txt >/dev/null; then
    echo release_verify=true
  else
    echo release_verify=false
  fi
fi
```

Result: passed.

Output summary:

```text
heavy_rust=false
release_verify=true
```

```bash
python3 - <<'PY'
import json, re, subprocess, pathlib
packages = sorted(json.loads(subprocess.check_output(['cargo','metadata','--no-deps','--format-version','1']))['packages'], key=lambda p: p['name'])
workspace = {p['name'] for p in packages}
text = pathlib.Path('scripts/ai/verify_full.sh').read_text()
selected = set()
for name in ['core', 'runtime', 'adapters_a', 'adapters_b']:
    m = re.search(rf'run_rust_workspace_{name}_tests\(\) \{{\n(.*?)\n\}}', text, re.S)
    if not m:
        raise SystemExit(f'missing function {name}')
    for pkg in re.findall(r'\bnautilus-[a-z0-9-]+\b', m.group(1)):
        selected.add(pkg)
missing = sorted(workspace - selected)
extra = sorted(selected - workspace)
print(f'workspace packages={len(workspace)} selected={len(selected)}')
if missing or extra:
    print('missing:', missing)
    print('extra:', extra)
    raise SystemExit(1)
print('workspace partition coverage ok')
PY
```

Result: passed.

Output summary:

```text
workspace packages=41 selected=41
workspace partition coverage ok
```

```bash
scripts/ai/verify_full.sh rust-tests-workspace-adapters-b
```

Result: passed.

Output summary:

```text
== verify_full: rust tests workspace partition adapters-b ==
packages=nautilus-deribit nautilus-dydx nautilus-hyperliquid nautilus-interactive-brokers nautilus-kraken nautilus-tardis nautilus-okx nautilus-polymarket
features=arrow,high-precision
== verify_full complete ==
```

```bash
git diff --check
```

Result: passed.

## Behavior Impact

No runtime behavior changed. This only changes release verification stage
granularity and PR smoke classification for release verification script/workflow
changes.

## Public API Impact

None.

## Migration Note Status

Not required. This is a CI/release verification workflow change.

## Rollback Plan

Revert this PR. The release gate will return to the previous single
`full-rust-tests-workspace` matrix stage.
