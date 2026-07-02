# v0.22.0 Gated Manual Operation Entry Contract

Date: 2026-07-02
Executor: Codex
Task: `V220-005` / GitHub issue `#688`

## Summary

This document records the v0.22 Trader Terminal gated manual operation entry
contract. The workbench can display an operation intent preview and gate
references, but the entry remains disabled unless a future execution-control
version explicitly supplies owner approval, risk gate, and audit gate evidence.
Even with all references present in the read model, v0.22 keeps the entry as a
disabled preview and does not implement order-control execution.

Plain Chinese summary: v0.22 只定义人工操作入口合同，不开放真实操作按钮。Workbench
可以展示 intent preview、owner approval reference、risk decision reference 和
audit evidence reference；缺任何门禁、read model stale 或 provenance mismatch
时入口必须 disabled/blocked。即使门禁引用齐全，v0.22 仍然只是 disabled preview，
不是执行算法或订单控制版本。

## Runtime Contract

```text
surface = Trader Terminal gated manual operation entry contract
source artifact = v0_21/unified_read_model_snapshot.json
source snapshot field = read_model_runtime.components.operation_entry
intent preview = display-only
owner approval reference = required before any future real operation
risk decision reference = required before any future real operation
audit evidence reference = required before any future real operation
default entry state = disabled/blocked
ungated attempt = fail_closed
future execution-control target = v0.24 or later, not v0.22
```

## Blocked States

```text
missing owner approval = blocked
missing risk gate = blocked
missing audit gate = blocked
stale read model = blocked
provenance mismatch = fail_closed
ungated submit/cancel/retry/replace/amend/flatten attempt = fail_closed
```

## Workbench Layout

```text
operation entry panel = status, freshness, intent preview, owner approval ref, risk decision ref, audit evidence ref
blocked state rows = missing approval, missing risk gate, missing audit gate, stale read model, provenance mismatch
attempt rows = ungated attempt, attempt status, fail-closed marker
boundary rows = entry enabled, submit, cancel, retry, replace, amend, flatten, automatic action
```

## Boundary

The contract is display-only and does not authorize real operation controls.

```text
manual operation entry button = not included
submit order route/button = not included
cancel order route/button = not included
retry order route/button = not included
replace order route/button = not included
amend order route/button = not included
flatten position route/button = not included
automatic execution = not included
execution algorithm = not included
product-grade live trading terminal claim = false
runtime degradation and boundary tests = reserved for V220-006
release gates = reserved for V220-007
```

## Validation Surface

The implementation is validated through local Rust dashboard tests, a JS syntax
smoke, format/diff checks, `scripts/ai/verify_fast.sh`, and workspace clippy.
The tests assert missing owner/risk/audit gates block the entry, ready gate
references still render only a disabled preview, stale read-model state blocks
entry, provenance mismatch fails closed, and any ungated operation attempt
fails closed.
