# Trace And Performance Expansion Plan

Date: 2026-06-04
Executor: Codex
Task ID: NTRACE-001

## Purpose

This plan defines how NTPRO v0.2.0 should expand trace and performance
evidence after the formal Rust-only v0.1.0 release.

It does not implement new runners, modify trading semantics, weaken existing
golden trace gates, or make performance smoke a release blocker by accident.

## Current Baseline

The existing golden trace gate is documented in:

- `docs/rust-cutover/golden_trace/SCHEMA.md`
- `docs/rust-cutover/golden_trace/GATE_EVIDENCE.md`
- `docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json`
- `docs/rust-cutover/release/final_release_verification.md`

Current inventory:

| Area | Current state |
| --- | --- |
| Trace schema | `golden-trace-v1` is enforced for every JSONL row. |
| Trace files | 8 JSONL files under `tests/golden/`. |
| Trace rows | 18 total rows. |
| Executable replay | Backtest, live/sandbox lifecycle, cache/message-bus, OKX adapter payload, and scoped backtest/live parity have Rust replay evidence. |
| Schema-only scope | Market data and order lifecycle seed rows are explicitly scoped in the release replay manifest. |
| Release mode | `REQUIRE_GOLDEN_REPLAY=1 scripts/ai/run_golden_traces.sh` validates the release replay scope manifest when no external replay command is configured. |

Current command:

```bash
scripts/ai/run_golden_traces.sh
```

Release-mode command:

```bash
REQUIRE_GOLDEN_REPLAY=1 scripts/ai/run_golden_traces.sh
```

## Expansion Principles

1. Keep trace rows deterministic.
2. Add schema rows before executable replay only when the missing replay hook is
   documented.
3. Promote schema-only rows to executable replay as soon as a Rust replay hook
   exists.
4. Do not use performance duration fields as trading-semantic trace evidence.
5. Do not hide price, quantity, side, order state, position, balance, risk
   decision, or adapter payload differences behind tolerances.
6. Keep performance smoke separate from golden trace pass/fail until a later
   release gate explicitly promotes it.

## Required v0.2.0 Trace Expansion

These items should be planned as v0.2.0 verification work before broader
operator or dashboard work depends on runtime claims.

| Priority | Area | Required evidence | Current blocker | Owner class |
| --- | --- | --- | --- | --- |
| P1 | Backtest trace expansion | Add at least one executable replay beyond the current single quote scope, preferably including strategy signal, order submit intent, and result artifact metadata. | Current backtest trace proves a minimal quote replay but not strategy-to-order flow. | Verification + Rust Core Runtime |
| P1 | Live and sandbox lifecycle trace expansion | Add lifecycle replay for configured start, adapter registration, connection state, one scoped event cycle, and shutdown artifact. | Current live/sandbox trace covers scoped lifecycle but not the RHARD-004 sandbox CLI demo artifact flow. | Verification + Rust Product Surface |
| P1 | Data source trace expansion | Promote `market_data` schema-only rows for quote, trade, bar, order book delta, instrument status, and catalog ordering into executable Rust replay where feasible. | `market_data` rows are currently schema-only scoped. | Verification + Rust Core Runtime |
| P1 | Execution order lifecycle trace expansion | Promote submit accept, submit reject, modify accept, cancel accept, triggered fill, and partial-to-filled rows into executable Rust replay. | `order_lifecycle` rows are currently schema-only scoped. | Verification + Rust Core Runtime |
| P1 | Risk rejection trace expansion | Add deterministic risk accept/reject rows for notional, rate limit, trading-state gate, and invalid order shape. | No executable `risk` golden trace replay exists yet. | Verification + Rust Core Runtime |
| P1 | Adapter payload trace expansion | Add at least one fixture-backed payload trace for each supported adapter family that is claimed as v0.2.0 supported. | Only OKX has an executable adapter payload golden trace. | Verification + Adapter Integration |

## Deferred v0.2.x Trace Expansion

These are important but should not block the first v0.2.0 planning pass unless
the product scope expands to claim the behavior.

| Area | Deferred evidence | Reason |
| --- | --- | --- |
| Position lifecycle | Open, increase, reduce, close, flip, hedge/netting traces. | Needs deterministic position setup and accounting expectations. |
| Portfolio/PnL | Account balance, margin, realized PnL, unrealized PnL, equity, and snapshot traces. | Rust integration tests exist, but release-level trace replay still needs deterministic fixture output. |
| Persistence/event store | Replay from persisted event source into cache/message-bus or result artifact. | Requires stable artifact contract and event-store boundary decision. |
| Multi-account/multi-venue | Aggregation and routing traces across accounts or venues. | Should wait for adapter support and account-state source decisions. |
| Adapter execution reports | Venue order reports, fill reports, and reconciliation traces. | Requires fixture strategy per supported adapter and no live API dependency. |

## Future Evidence

These areas are future work and should remain out of the v0.2.0 release gate
unless explicitly promoted by a later owner-approved task.

| Area | Future direction |
| --- | --- |
| Operator dashboard trace | Dashboard should consume stable telemetry/control contracts, not internal trace files directly. |
| Long-running live soak | Requires environment, data retention, log redaction, and manual gate policy. |
| Cross-platform binary performance | Depends on a future binary artifact release task. |
| Full adapter matrix replay | Requires per-adapter fixture ownership and explicit supported/deferred decisions. |

## Performance Smoke Scope

Performance smoke for v0.2.0 should answer a narrow question:

```text
Did this change introduce an obvious local performance regression in a small,
repeatable Rust smoke path?
```

It should not answer:

```text
Is NTPRO production-performance certified?
```

### Initial Smoke Candidates

Use a small subset of existing Rust benches or smoke commands. Candidate areas:

| Area | Candidate source | Why |
| --- | --- | --- |
| CLI startup | `cargo run -q -p nautilus-cli -- --help` | User-facing command startup stays visible. |
| Backtest minimal path | RHARD-006 dry-run command | Covers documented user path without full runtime semantics. |
| Sandbox demo | RHARD-004 sandbox run command | Covers local artifact emission and simulated event flow. |
| Live init smoke | `cargo run -q -p nautilus-live --no-default-features --features node --example live-init-smoke` | Covers lifecycle init/shutdown without real orders. |
| Network benches | `crates/network/benches/*` | Existing focused transport/rate-limit benches. |
| Model precision benches | `crates/model/benches/*` | Existing numeric performance coverage. |

### Non-Blocking Default

Performance smoke is non-blocking by default for v0.2.0.

The first implementation should record:

- command;
- platform;
- Rust toolchain;
- build profile;
- elapsed time or criterion summary;
- whether the result is informational or blocking.

It should not fail release by default. A later task may promote a metric to a
release blocker only after it defines:

- stable command;
- stable machine/profile assumptions;
- baseline source point;
- allowed variance;
- failure remediation path;
- owner approval.

## Required vs Deferred vs Future Summary

| Bucket | Meaning | Examples |
| --- | --- | --- |
| Required | Needed to strengthen v0.2.0 claims about supported Rust product paths. | Market data replay, order lifecycle replay, risk rejection replay, adapter payload expansion. |
| Deferred | Important but not required unless v0.2.0 claims the behavior. | Portfolio/PnL, persistence replay, multi-account/multi-venue traces. |
| Future | Belongs after architecture/control/observability scope is stable. | Dashboard trace consumption, long-running live soak, full binary performance matrix. |

## Follow-Up Task Shape

Every future trace expansion task should declare:

- trace category;
- fixture file path;
- expected replay harness;
- exact Cargo command;
- expected behavior;
- whether the trace starts as schema-only or executable replay;
- whether release mode must include it;
- rollback plan.

Every future performance smoke task should declare:

- command;
- platform;
- toolchain;
- profile;
- baseline source point;
- whether result is informational or blocking.

## Acceptance Boundary

NTRACE-001 is complete when this plan exists and evidence records local
validation. It does not by itself create new trace fixtures, performance
baselines, release blockers, or runtime behavior.
