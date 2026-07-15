# DEXG-006 Concepts And Retired Python API Links

Date: 2026-07-15
Executor: Codex
GitHub issue: #1085
Milestone: post-backend-docs-examples-governance
Status: READY FOR PR

## Goal

Remove the retired Python API URL family from current documentation and bind
the affected concept pages to tracked Rust source authority.

Plain Chinese summary: 本任务清理 9 个 concept 页中的 26 个失效 Python API 链接，
逐项替换为真实 Rust source，并把页面里的 Python snippet 明确限定为上游 lineage。
不会把失效 URL 静默重定向，也不会用文档声明不存在的运行时能力。

## Dependency

DEXG-003 / #1082 and DEXG-005 / #1084 are merged and closed.

## Scope

Included:

- replace all remaining retired Python API links in current docs;
- map order, position, instrument, strategy, cache, portfolio, logging, risk,
  and live config references to tracked Rust source files;
- add Rust-only authority boundaries to all nine affected concept pages;
- remove active Cython/PyO3 product wording from the instrument overview.

Not included:

- changing Rust implementation or public APIs;
- claiming that retained Python snippets are runnable;
- enabling any frozen backend capability.

## Acceptance Criteria

- repository search finds no retired Python API URL;
- every replacement Rust source path exists;
- all nine affected pages carry the exact Rust-only authority warning;
- changed local Markdown links resolve;
- backend freeze and fast smoke checks pass.

## Validation

```bash
! rg -n '/docs/python-api''-latest/' docs
python3 - <<'PY'
import re
from pathlib import Path

for page in Path('docs/concepts').glob('*.md'):
    text = page.read_text(encoding='utf-8')
    for target in re.findall(r'\]\((\.\./\.\./crates/[^)#]+)', text):
        if not (page.parent / target).resolve().exists():
            raise SystemExit(f'missing Rust source link: {page}: {target}')
PY
scripts/ai/check_backend_freeze_baseline.sh
scripts/ai/verify_fast.sh
git diff --check
```
