# PAR-001A Sandbox Async Modify Cache Repair

Date: 2026-07-29
Executor: Codex
GitHub issue: #1181
Risk: high
Classification: separately-scoped-runtime-maintenance
Status: REVIEW_REQUIRED

## Goal

Repair marketable modify processing when sandbox-style order events are
dispatched asynchronously and the cache still contains the pre-update order.

中文摘要：改单事件尚未回写缓存时，撮合引擎不能把旧订单当作本次改单后的有效
状态。历史路径会在父单已成交 `0.600`、再把数量改为 `0.600` 时重复成交，并错误
更新关联单。本任务让改单、成交、关联单和撮合核心统一使用同一份有效订单状态。

## Scope

- apply the validated `OrderUpdated` event to a local effective order before
  dispatch;
- retain the effective order until cache confirms the same update event so
  deferred limit and stop match actions cannot reload stale parameters;
- retain runtime-only liquidity and trailing activation state on that effective
  order without copying unapplied event history into the canonical cache;
- when cache replay contains the pending update but omits a later pending fill,
  merge runtime-only liquidity and trailing activation into the canonical
  cache snapshot without replacing its event history;
- preserve cache-owned position linkage while reconciling a pending snapshot,
  both before and after event histories align;
- use that effective order for immediate marketable fills and core resync;
- derive effective leaves from matching-engine filled quantity while cache
  event application is deferred;
- subtract fills that have not reached canonical cache from reduce-only
  position leaves, including fills emitted by other pending reduce-only orders
  targeting the same position, so aggregate deferred fills cannot over-close
  a position;
- retain reduce-only position consumption by generated trade ID after the
  originating order reaches a terminal state, until the position cache applies
  that exact fill;
- bind deferred reduce-only consumption to the position opening generation so
  a closed-and-reopened netting position cannot inherit prior consumption;
- reap acknowledged, replaced, or orphaned deferred reduce-only consumption
  during normal matching-engine iteration without requiring a later order;
- resize an L1 reduce-only order when its total leaves exceed position leaves,
  including when the visible fill exactly equals the remaining position, so a
  slipped remainder cannot reverse the position;
- isolate netting position lookup and deferred consumption by strategy, and
  cancel an order when other pending fills exhaust its effective position
  budget;
- provide fixed-fee calculation with effective prior filled quantity so its
  charge-once policy remains correct before cache replay;
- cap each async fill sequence to effective leaves after market-liquidity
  consumption so later data cannot overfill a pending order;
- persist quote-to-base conversion in the effective overlay so a trigger-style
  quote order cannot be converted again at a later market price;
- refresh converted pending snapshots after Maker or Taker assignment so
  deferred fills retain valid liquidity attribution;
- refresh both cache and pending state when an immediately marketable limit
  order leaves a resting remainder that must revert from Taker to Maker;
- refresh trigger-style market-order pending snapshots after Taker assignment
  so cache reconciliation cannot restore stale Maker attribution;
- persist Maker attribution for a pre-cached trailing-stop-limit order after
  quote conversion so later fee calculation cannot observe
  `NoLiquiditySide`;
- propagate the latest parent leaves to an OTO child when a marketable modify
  partially fills before cache event application completes;
- retain an OTO protection child at the amended parent quantity when the
  marketable modify fully fills before cache event application completes;
- retain a fully filled amended OTO parent snapshot until `oto_full_trigger`
  validation has activated its protection child;
- resize an inactive OTO protection child before activation can execute it at
  the stale pre-modify quantity;
- preserve an OTO child's prior fills when translating target protection leaves
  into the child's new total quantity;
- use the parent's effective generated fill quantity when deciding whether an
  OTO child can activate after only the parent acceptance reached cache;
- remove pending effective snapshots on terminal rejection so a converted
  rejected order cannot remain reachable through the async overlay;
- revalidate both effective order state and the matching-core price/trigger
  representation when instrument precision changes;
- cover passive async modify and marketable zero-leaves modify with executable
  Rust integration tests;
- preserve PAR-001 contingent-order rejection and cancellation behavior.

## Non-Goals

- no trailing-stop trigger formula or public behavior change;
- no risk-engine, adapter transport, CLI, or public API change;
- no production order submission or mutation authorization;
- no edit to frozen `docs/rust-cutover/release/v0_32_0_*` files.

## Acceptance

- an async passive price modify updates matching-core state before cache event
  application, fills from the new limit when market data arrives in that gap,
  and converges after the event is applied;
- an async stop modify triggers and fills with the new trigger and quantity
  before cache event application;
- a marketable async modify that fills in multiple passes retains Taker
  liquidity attribution until completion and cannot exceed effective leaves
  when refreshed book liquidity is larger than the remainder;
- a quote-quantity stop order converts exactly once, retains the converted base
  quantity across partial fills, and closes at that base quantity;
- a passive quote-quantity limit order remains Maker when it crosses during
  deferred cache application and commission calculation does not panic;
- the resting remainder of an immediately marketable quote-quantity limit order
  reverts to Maker in both cache and pending effective state;
- an OTO child tracks the latest parent leaves after a marketable parent modify
  partially fills;
- a fully filled marketable OTO parent modify retains its protection child at
  the amended parent quantity;
- a downsized OTO parent that fully fills before cache replay activates its
  resized child when `oto_full_trigger` is enabled;
- an instrument size-precision update retains an effective compatible quantity
  and cancels an effective incompatible quantity even while cache is stale;
- an instrument price-increment update cancels an incompatible matching-core
  entry even when cache already contains a newer compatible price;
- trailing activation during the async gap does not duplicate order event
  history and converges after explicit event replay;
- a cache that replays an older fill and the pending update, but not the latest
  fill, retains Taker attribution for subsequent fills despite the divergent
  event histories;
- cache-owned `position_id` remains attached after a pending order update
  reaches the same event-history boundary;
- a partially filled async stop-market order remains Taker after its update and
  fill reach cache and a later market-data action reconciles pending state;
- a reduce-only marketable modify cannot emit aggregate fills above the
  effective remaining position while order and position cache updates lag;
- concurrent reduce-only orders sharing one position cannot each consume the
  full stale position quantity while their generated fills remain deferred;
- canceling an IOC reduce-only order after a partial fill cannot forget that
  deferred position consumption or let a later order over-close the position;
- deferred consumption from a closed netting position cannot reduce the budget
  of a reopened position that reuses the same deterministic position ID;
- acknowledged deferred consumption is reaped during normal iteration even
  when no later reduce-only order references the position;
- an L1 reduce-only market order whose visible fill closes the position cannot
  emit a slipped remainder that reverses the position;
- deferred reduce-only consumption from one strategy cannot reduce another
  strategy's netting-position budget, and a zero-budget order is canceled;
- a pre-cached quote-quantity trailing-stop-limit order retains Maker
  attribution through trigger and fill without a liquidity-side panic;
- an immediately triggered OTO child receives its amended protection quantity
  before any fill;
- an OTO child can activate from effective parent fills when cache contains the
  parent acceptance but not the generated fill;
- a partially filled OTO child retains its prior fills and the full target
  protection leaves when resized;
- a rejected quote-denominated order cannot be filled from a retained pending
  snapshot;
- a charge-once fixed fee is charged on the first deferred fill only;
- reducing a partially filled parent from `1.000` to its filled quantity
  `0.600` emits no duplicate fill;
- the zero-leaves parent and linked OUO child leave the matching core and
  converge to closed cache state;
- standard and high-precision matching-engine suites pass;
- full execution and sandbox tests, Clippy, risk inventory, and current
  governance pass;
- independent review and hosted checks approve the high-risk change.
