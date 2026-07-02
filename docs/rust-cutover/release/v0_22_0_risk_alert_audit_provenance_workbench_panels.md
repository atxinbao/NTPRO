# v0.22.0 Risk Alerts Audit And Provenance Workbench Panels

Date: 2026-07-02
Executor: Codex
Task: `V220-004` / GitHub issue `#687`

## Summary

This document records the v0.22 Trader Terminal risk, alerts, audit, and
provenance drill-down panel surface. The panels are read-only projections over
the existing v0.21.1 canonical Unified Read Model runtime bridge. They expose
risk priority, alert severity, audit evidence completeness, release provenance,
artifact digest, artifact sha, source references, and lineage diagnostics
without adding automatic actions or operation controls.

Plain Chinese summary: 本任务只增加 Trader Terminal workbench 的风控、告警、
审计和 provenance drill-down 只读面板。数据来自已有 v0.21.1 canonical read
model runtime；risk priority、alert severity、audit evidence missing 和
provenance mismatch 会显示 degraded、stale 或 fail_closed。`audit_closed` 只能在
required evidence/components 和 provenance digest 完整时成立。本任务不新增任何自动
修复、自动撤单、自动平仓、retry、risk repair、alert action、audit action 或
provenance repair 入口。

## Runtime Contract

```text
surface = Trader Terminal risk, alerts, audit, and provenance panels
source artifact = v0_21/unified_read_model_snapshot.json
source snapshot fields = read_model_runtime.risk / read_model_runtime.lifecycle_status
fallback without artifact = degraded shell with unknown panel values
risk priority = halted > mismatch > stale > manual_review > risk_visible > healthy
alert severity = critical for missing/schema/redaction/forbidden-control, warning for stale source, info when clean
audit_closed rule = required evidence/components/provenance digest must be complete
provenance mismatch = fail_closed/error diagnostic
automatic risk/alert/audit/provenance actions = false or absent
```

## Panel Layout

```text
risk panel = status, priority state, risk state, freshness, critical evidence, risk visible, manual review, halted, mismatch, diagnostics, lineage, source provenance, redaction
alerts panel = severity, missing evidence, stale source, schema mismatch, redaction breach, forbidden control request, summary
audit/provenance panel = lifecycle summary, audit freshness, audit state, audit closed, evidence completeness, component completeness, missing evidence, release provenance, artifact digest, artifact sha, provenance mismatch, diagnostics, lineage, source provenance, redaction
artifact drawer = artifact path, snapshot, source, redaction, blocking reasons, release provenance, artifact digest
```

## Boundary

The panels are display-only. They do not authorize risk repair, alert action,
audit repair, provenance repair, order entry, order mutation, fill repair,
reconciliation repair, execution algorithms, or product-grade live trading
readiness.

```text
risk action controls = not included
alert action controls = not included
audit action controls = not included
provenance repair controls = not included
automatic remediation = not included
automatic cancel / retry / flatten = not included
manual operation entry contract = reserved for V220-005
runtime degradation and boundary tests = reserved for V220-006
release gates = reserved for V220-007
```

## Validation Surface

The implementation is validated through local Rust dashboard tests and a JS
syntax smoke. The tests assert risk priority ordering, alert severity
classification, audit evidence missing fail-closed behavior, provenance
mismatch fail-closed behavior, healthy audit closure only with complete
evidence, and absent/false automatic risk/alert/audit/provenance action
controls.
