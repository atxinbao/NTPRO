# v0.18.0 Release Notes - Owner-Approved Cancel Recovery Preview

Date: 2026-06-26
Executor: Codex
Status: DRAFT_NOT_PUBLISHED

## Summary

v0.18.0 adds preview-only cancel recovery evidence contracts, local gates, and
a read-only Dashboard panel.

Plain Chinese summary: v0.18.0 只是撤单恢复预览，不会真实发送撤单，也不会在 Dashboard 增加撤单按钮。

## What Changed

- Added local cancel recovery preview artifact generation and validation.
- Added v0.18 release gate stage `v18-release-gates`.
- Added a read-only Dashboard cancel recovery preview panel.
- Added readiness and evidence docs for V180-001 through V180-011.

## What Did Not Change

```text
Owner-Approved Cancel Recovery Preview
actual_cancel_send_allowed=false
cancel_attempted=false
automatic_cancel_allowed=false
dashboard_cancel_controls_enabled=false
network_attempted=false
production_order_mutations_attempted=0
manual_owner_approval_required=true
owner_approved=false
```

## Publication Status

This document does not publish a tag or GitHub Release.

## Next-Version Scope Note

Actual single-shot cancel remains a v0.19+ scope decision.
