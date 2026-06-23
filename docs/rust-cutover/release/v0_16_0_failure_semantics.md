# v0.16.0 Failure And No-Retry Semantics Contract

Date: 2026-06-23
Executor: Codex
Scope: v0.16.0 minimum owner-approved production order mutation candidate
Task: V160-010

## Purpose

V160-010 defines what NTPRO does after a scoped v0.16 production mutation
candidate fails or becomes ambiguous: write redacted evidence and stop.

Plain Chinese summary: 这不是自动补救系统。大白话：失败后只留证据，不自动重试、
撤单、改单、补单或继续策略。

## Supported Failure Modes

```text
timeout
http-4xx
http-5xx
malformed-response
readback-mismatch
kill-switch-transition
```

## Required Semantics

Every supported failure mode must produce:

```text
terminal_action = write_evidence_and_stop
evidence_written = true
stop_after_evidence = true
strategy_continuation_allowed = false
retry_allowed = false
retry_attempted = false
retry_attempts = 0
max_retry_attempts = 0
cancel_attempted = false
replace_attempted = false
amend_attempted = false
correction_attempted = false
flatten_attempted = false
remediation_attempted = false
automatic_remediation_allowed = false
dashboard_order_controls_enabled = false
listen_key_lifecycle_allowed = false
```

## Non-Goals

V160-010 does not implement:

```text
automatic retry
automatic cancel/replace/amend
automatic correction or flatten
automatic remediation
Dashboard order controls
listenKey lifecycle
strategy continuation after failure
new production send behavior
```
