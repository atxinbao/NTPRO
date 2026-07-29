# PAR-001A Sandbox Async Modify Cache Repair Evidence

Date: 2026-07-29
Executor: Codex
GitHub issue: #1181
Status: LOCAL VALIDATION PASSED / REVIEW REQUIRED

## Baseline Reproduction

The sandbox-style fixture records order events without applying them to cache.
The parent limit order has quantity `1.000` and an already applied fill of
`0.600`; modifying quantity to `0.600` reproduced this invalid five-event
sequence:

```text
1. OrderUpdated(parent quantity=0.600)
2. OrderFilled(parent quantity=0.600)        # duplicate fill
3. OrderUpdated(child quantity=0.400)        # stale leaves
4. OrderCanceled(child)
5. OrderCanceled(parent)
```

The expected sequence is only parent update, child cancel, and parent cancel.

## Repair Boundary

```text
modify validation source = effective pending snapshot, otherwise cache snapshot
post-update decision source = locally applied effective OrderUpdated snapshot
filled quantity source = matching-engine cached_filled_qty
core resync source = effective local order
runtime-only state source = effective local order until event-history alignment
cache event application may remain deferred = supported
public API changed = false
production trading capability changed = false
v0.32.0 frozen files changed = false
```

## Executable Evidence

`test_async_modify_uses_effective_order_before_cache_application` proves a
passive price update changes the matching-core entry while cache still exposes
the old price, then receives market data and fills at the effective new limit
before converging after the emitted events are applied.

`test_async_marketable_modify_to_filled_quantity_does_not_duplicate_fill`
replays the partial-fill and zero-leaves case through the real Rust matching
engine with deferred cache application. It asserts exactly three events, no
additional `OrderFilled`, removal of both linked orders from the core, and
closed cache state after event application.

`test_async_stop_modify_uses_effective_order_for_deferred_trigger` changes both
trigger and quantity, leaves cache intentionally stale, then moves market data
through the new trigger. The emitted fill must use the new `0.500` quantity and
the new market price.

`test_async_marketable_modify_retains_taker_side_across_partial_fills` makes a
passive limit marketable, consumes only `0.400`, refreshes market liquidity
to `1.000` before cache catch-up, and proves the second fill is capped to the
effective `0.600` leaves while both fills retain `LiquiditySide::Taker`.

`test_async_trailing_activation_does_not_copy_pending_event_history` activates
a trailing stop while its modify event is still pending. Canonical cache
history remains unchanged before explicit replay; after replay, quantity and
runtime activation state converge without a duplicate event.

`test_async_quote_quantity_conversion_is_persisted_across_partial_fills`
triggers a quote-denominated stop market order at `1502.00`, then moves the
market to `2000.00` before cache catch-up. It proves there is exactly one
quote-to-base update to `0.999` and fills close at `0.400 + 0.599`, rather than
reconverting the original quote notional.

`test_async_quote_quantity_passive_limit_retains_maker_side` submits a passive
quote-denominated limit order while cache event application is deferred, then
crosses the market. It proves the effective snapshot retains Maker attribution
and the fee model does not panic on an unset liquidity side.

`test_async_quote_quantity_marketable_remainder_retains_maker_side` submits an
immediately marketable quote-denominated limit order against only `0.400`
liquidity. It proves the first fill is Taker and the resting `0.600` remainder
is retained as Maker in the effective pending snapshot before its later fill.

`test_async_oto_child_tracks_marketable_parent_modify` increases an OTO parent
to `2.000` and makes it marketable against `0.400` liquidity. It proves the
activated child is resized to the effective parent leaves of `1.600` even
though the fill occurs inside the modify call.

`test_async_oto_child_survives_fully_filled_parent_modify` makes the same
marketable parent modify fill the complete `2.000` while cache application is
deferred. It proves the child is updated to `2.000`, remains in the matching
core, and receives no cancellation event, so the resulting position is not
left without its OTO protection order.

`test_async_oto_full_trigger_uses_amended_parent_until_child_activation`
starts with a `2.000` OTO parent, enables full-fill child triggering, then
downsizes and fully fills the parent at `1.000` before cache replay. It proves
the effective parent snapshot remains available long enough to resize and
activate the `1.000` protection child.

`test_async_oto_child_is_resized_before_immediate_execution` makes the amended
parent fill `0.400` while its stop child is immediately triggerable. It proves
the child update to `1.600` precedes its `1.600` fill, rather than allowing the
old `1.000` protection size to execute first.

`test_async_oto_child_resize_preserves_prior_fills` gives an OTO child an
existing `0.200` fill before a parent update targets `1.600` protection leaves.
It proves the child total becomes `1.800`, preserving both the prior fill and
the full `1.600` remaining protection.

`test_async_runtime_state_survives_divergent_cache_history` replays only an
older Maker fill and the pending update into cache while a newer Taker fill
remains deferred. It proves reconciliation keeps the canonical cached event
history but merges the pending Taker runtime state, so the final `0.600` fill
is still attributed as Taker.

`test_async_stop_fill_reconciliation_preserves_taker_side` partially fills a
modified stop-market order as Taker, explicitly replays the update and fill
into cache, then invokes another market-data reconciliation. It proves the
pending snapshot was refreshed at trigger-time fill and cannot overwrite the
canonical Taker side with stale Maker attribution.

`test_async_reduce_only_modify_caps_aggregate_fills_to_position` opens a
`0.600` long position, makes a `1.000` reduce-only sell marketable against
`0.400`, then refreshes `1.000` liquidity before cache replay. It proves the
two fills are capped to `0.400 + 0.200` and the effective order is resized to
`0.600`, rather than over-closing by `0.400`.

`test_async_reduce_only_orders_share_effective_position_budget` places two
reduce-only sell orders against the same `0.600` long position while generated
fills remain absent from canonical cache. It proves pending consumption is
aggregated across both orders, their total fill cannot exceed `0.600`, and the
second order is canceled rather than left open with no effective position
budget.

`test_l1_reduce_only_slipped_remainder_cannot_reverse_position` submits a
`1.000` L1 reduce-only market sell against a `0.400` long position and exactly
`0.400` visible bid liquidity. It proves the order is resized to `0.400` and
emits only that fill, with no slipped `0.600` remainder that reverses the
position.

`test_async_reduce_only_consumption_does_not_cross_position_generation`
applies a deferred close fill, reopens the flat canonical position at `0.500`
with the same netting position ID, and submits a new reduce-only order. It
proves the previous generation is discarded by opening order and timestamp
identity and the new position retains its full `0.500` budget.

`test_iterate_reaps_acknowledged_reduce_only_consumption` applies a deferred
partial close to canonical position state, runs normal engine iteration, then
removes historical trade IDs from the position snapshot. It proves the
acknowledged record was already reaped and cannot be revived to reduce the
remaining `0.400` position budget.

`test_async_reduce_only_position_budget_isolated_by_strategy` opens separate
`0.600` netting positions for two strategies on the same instrument. It proves
each strategy can consume its own position and one strategy's deferred fill is
not subtracted from the other's budget.

`test_async_quote_quantity_trailing_limit_retains_maker_side` pre-caches a
quote-denominated trailing-stop-limit order before trigger-time conversion. It
proves the converted pending and cached runtime snapshots retain Maker
attribution, so the later fill completes without an unset-liquidity fee panic.

`test_rejected_quote_order_cannot_fill_from_pending_overlay` converts a
quote-denominated limit order and then rejects its unsupported time in force.
It proves a direct later fill attempt emits no event because terminal rejection
removed the pending effective snapshot.

`test_async_marketable_modify_charges_fixed_fee_once` runs the same two-pass
deferred fill shape with a charge-once fixed fee. It proves commissions are
`1.00 + 0.00 USD`, rather than charging the fixed amount twice while canonical
order history still reports zero fills.

`test_async_modify_effective_quantity_survives_compatible_instrument_update`
and
`test_async_modify_effective_quantity_is_canceled_when_instrument_incompatible`
exercise both sides of instrument revalidation while cache intentionally
retains the pre-modify quantity. The effective compatible order remains in the
core, while the effective incompatible order is canceled.

`test_async_reconciliation_preserves_cache_position_id` applies a pending
price update, assigns a position link only to canonical cache, then replays the
update and invokes matching-engine reconciliation. It proves the aligned
pending snapshot cannot clear the cache-owned `position_id`.

`test_update_instrument_cancels_incompatible_core_after_cache_price_update`
applies a newer tick-compatible price to cache while the matching core still
contains the previous tick-incompatible price. It proves an instrument update
validates both representations and cancels the order instead of allowing the
stale core entry to remain matchable.

## Validation

```text
cargo test -p nautilus-execution --test matching_engine
result = PASS (201 passed, 1 ignored)

cargo test -p nautilus-execution --features high-precision \
  --test matching_engine
result = PASS (201 passed, 1 ignored)

cargo test -p nautilus-sandbox
result = PASS (39 passed across unit and integration suites)

cargo clippy -p nautilus-execution -p nautilus-sandbox \
  --all-targets --all-features -- -D warnings
result = PASS

scripts/ai/check_backend_runtime_risk_inventory.sh
result = PASS (29,109 signals in 1,215 files)

cargo test -p nautilus-execution
result = PASS

cargo test -p nautilus-execution --features high-precision
result = PASS

scripts/ai/verify_release.sh current-governance backend-freeze-baseline
result = PASS

scripts/ai/verify_fast.sh
result = PASS

git diff --check
result = PASS
```

The final assertions add one hundred thirty-two test-owned inventory signals
relative to the pre-task baseline: one hundred twenty-three `unwrap` calls,
six `expect` calls, and three panics.
Production-owned signal counts remain unchanged.

## Review Correction

The first independent review proved that updating matching-core state alone was
insufficient: a market-data action in the async gap called `fill_limit_order`
or `trigger_stop_order`, which reloaded the pre-update cache order. The engine
now retains the locally applied effective order by client order ID. Modify,
fill, trigger, cancel, core resync, and iterate paths use it until cache history
contains the exact update event; terminal paths and reset remove it eagerly.
Limit and stop regressions both exercise market data before cache catch-up.

The second independent review found two remaining runtime-state gaps. First,
the pending snapshot was retained before a marketable modify set Taker
liquidity, so a later partial fill could fall back to Maker attribution.
Second, trailing activation copied a pending snapshot into canonical cache
before its update event was explicitly replayed. The engine now refreshes the
pending snapshot after Taker assignment and keeps trailing runtime state in
pending memory until cache event histories align. Dedicated partial-fill and
trailing-history regressions cover both corrections.

The third independent review found three remaining boundaries. A refreshed
book could overfill a partially filled pending order, trigger-style quote
quantity conversion was not persisted in the pending overlay, and instrument
updates validated stale cached parameters. The engine now caps fills against
effective leaves after liquidity consumption, persists the converted order,
and validates the effective order snapshot. The full suite additionally proves
that pre-existing trade-consumption behavior remains intact.

The fourth independent review reproduced two additional trading-semantic
failures. A converted passive order retained its pre-acceptance
`NoLiquiditySide` snapshot and panicked during fee calculation when later
crossed. A marketable OTO parent modify returned after its immediate partial
fill without propagating the new leaves to the child. Pending snapshots now
refresh after liquidity assignment, and the immediate-fill modify branch
updates contingent quantities before returning. Both reviewer reproductions
are retained as executable regressions.

The fifth independent review reproduced a partial replay boundary: cache had
applied an older Maker fill and the pending update, but not a newer Taker fill.
Because event histories diverged, reconciliation discarded the pending
runtime state and the next fill reverted to Maker. Reconciliation now keeps
the canonical cached event history while merging non-empty liquidity and
activated trailing-stop runtime state, then persists that merged snapshot.
The partial replay sequence is retained as an executable regression.

The sixth independent review reproduced two accounting boundaries. Deferred
reduce-only fills reread an unchanged position and could aggregate beyond its
remaining quantity, while the charge-once fixed fee reread an order with zero
applied fills and charged every partial fill. Position leaves now subtract
generated fills absent from canonical cache, reduce-only resizing is applied
to the local effective order, and fee calculation receives effective prior
filled quantity. Both reviewer reproductions are permanent regressions.

The seventh independent review found two remaining state-divergence
boundaries. A fully filled marketable OTO parent modify could cancel its
protection child when cache application was deferred, and an instrument update
could retain a tick-incompatible matching-core price when cache already held a
newer compatible price. Full OTO fills now target the amended parent quantity
instead of zero leaves, and instrument updates validate both the effective
order and its matching-core representation. Both reviewer reproductions are
permanent regressions.

The eighth independent review reproduced a trigger-style market-order
reconciliation boundary. A partial Taker fill updated canonical cache, but the
pending snapshot still held Maker and overwrote the newer fill-derived side on
the next market-data action. `fill_market_order` now refreshes an existing
pending snapshot immediately after assigning Taker. The exact replay and
reconciliation sequence is retained as an executable regression.

The ninth independent review reproduced two aggregate-state boundaries. Two
reduce-only orders targeting the same position each consumed the full stale
position quantity because effective leaves considered only the current order.
A pre-cached quote-denominated trailing-stop-limit order also retained
`NoLiquiditySide` after trigger conversion and panicked in fee calculation.
Effective position leaves now aggregate unapplied fills from every matching
reduce-only order, and trailing-stop processing persists Maker attribution in
both pending and pre-cached runtime snapshots. Both reproductions are permanent
standard and high-precision regressions.

The tenth independent review found four terminal and ownership boundaries.
An order with no remaining effective reduce-only budget returned without
canceling; netting consumption crossed strategy ownership; an immediately
triggerable OTO child could execute before its amended protection size was
applied; and a converted rejected order retained a reachable pending snapshot.
The engine now cancels zero-budget orders, resolves netting positions and
pending consumption by strategy, resizes inactive OTO children before
activation, and removes pending overlays during terminal rejection. Three new
regressions plus the strengthened shared-budget assertion cover all four
findings in both precision modes.

The eleventh independent review found two remaining liquidity and protection
boundaries. An immediately marketable quote-denominated limit order could leave
its pending remainder marked Taker after the cached order reverted to Maker,
and resizing a partially filled OTO child treated target leaves as its new total
quantity. The engine now synchronizes Maker attribution across cache and pending
state, and adds the child's prior fills when deriving its new total quantity.
Both reviewer reproductions are permanent standard and high-precision
regressions.

The twelfth independent review found two terminal-state and contingent
activation boundaries. Canceling an IOC reduce-only order after a partial fill
removed the order overlay and forgot its deferred position consumption, so a
later reduce-only order could over-close the position. Separately, replaying
only an OTO parent's acceptance left the cached parent in `Accepted`, so its
child did not activate even though the matching engine had already generated a
parent fill. The engine now tracks pending reduce-only consumption by generated
trade ID until the position applies that trade, and OTO validation uses the
effective parent fill quantity. Both exact reproductions pass in standard and
high-precision modes.

The thirteenth independent review found that aligned reconciliation replaced
the entire canonical order with its pending snapshot. If the execution engine
had assigned an OTO protection child a cache-owned `position_id` after that
snapshot was created, replacement silently cleared the linkage. Reconciliation
now carries a canonical position link into an unaligned pending view and always
merges pending-owned liquidity and trailing activation fields into canonical
state after histories align. The exact aligned-history reproduction passes in
standard and high-precision modes.

The fourteenth independent review found two reduce-only lifecycle boundaries.
An L1 market-style order whose visible fill exactly matched the remaining
position retained a larger order quantity, allowing the later slipped-fill
path to reverse the position. Separately, deferred consumption from a closed
netting position survived until the same deterministic position ID reopened;
because reopening clears old trade IDs, that applied fill reduced the new
position's budget. The engine now resizes from total order leaves before the L1
slip path and binds pending consumption to the position's opening order and
timestamp. Both exact reproductions pass in standard and high-precision modes.

The fifteenth independent review found an OTO full-trigger lifecycle gap and a
retention gap. A downsized parent that fully filled before cache replay removed
its effective snapshot before child validation, so the stale larger cached
quantity classified it as partial and left protection inactive. Acknowledged
reduce-only records were also only reaped by a later order for the same
position. Terminal parent overlays now survive through contingent processing,
and normal iteration reconciles pending consumption against canonical
positions. Both exact reproductions pass in standard and high-precision modes.

The sixteenth independent review inspected the complete branch diff after all
fifteenth-round corrections and found no actionable regression. The reviewer
independently reran the standard and high-precision matching-engine suites;
both completed with 201 passed and 1 ignored.

## Review State

This is a high-risk trading-semantic repair. It stops at `REVIEW_REQUIRED`;
hosted checks and a reviewer independent from the implementation must approve
the PR before manual merge.

## Rollback

Reverting the implementation restores stale-cache reads during async modify.
Rollback must also remove the PAR-001A regressions, the matching-core
divergence regression, and restore the prior risk inventory summary.
