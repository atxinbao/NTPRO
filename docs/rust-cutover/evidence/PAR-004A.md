# PAR-004A TWAP High-Precision Remainder Fixture Evidence

Date: 2026-07-29
Executor: Codex
GitHub issue: #1185
Status: LOCAL VALIDATION PASSED / REVIEW_REQUIRED

## Root Cause

```text
high-precision activation = nautilus-model/high-precision dependency feature
stale test selector = cfg(feature = "high-precision") on nautilus-trading
observed branch = standard raw literal branch under a high-precision model
production TWAP calculation changed = false
```

The old test assumed `nautilus-trading` declared its own `high-precision`
feature. It does not. A dependency feature can change `QuantityRaw` and
`FIXED_PRECISION` without making that `cfg` true in the trading crate, so the
test compared a correct high-precision result with a standard-precision raw
literal.

## Repair

The replacement test has one precision-independent contract:

- both regular slices are equal;
- a regular slice differs from semantic one third by less than one active
  fixed-point resolution unit;
- the remainder equals one active resolution unit;
- three regular slices plus the remainder exactly conserve the raw total.

No production function, scheduling branch, public API, release boundary, or
trading capability changed.

## Validation

```text
cargo test -p nautilus-trading --lib \
  test_twap_calculates_size_schedule_with_remainder
result = PASS (1 passed)

cargo test -p nautilus-trading --lib \
  test_twap_calculates_size_schedule_with_remainder \
  --features nautilus-model/high-precision
result = PASS (1 passed)

cargo test -p nautilus-trading
result = PASS (179 unit + 8 integration; 4 doctests intentionally ignored)

cargo test -p nautilus-trading \
  --features nautilus-model/high-precision
result = PASS (179 unit + 8 integration; 4 doctests intentionally ignored)

cargo test -p nautilus-trading -p nautilus-risk \
  --features nautilus-model/high-precision
result = PASS
  nautilus-trading = 179 unit + 8 integration
  nautilus-risk = 146 executable tests
  remaining ignored = 1 PAR-005 account-balance placeholder

cargo clippy -p nautilus-trading --all-targets --all-features \
  --features nautilus-model/high-precision -- -D warnings
result = PASS

scripts/ai/check_backend_runtime_risk_inventory.sh
result = PASS (29,126 signals in 1,215 files; counts unchanged)

scripts/ai/verify_release.sh current-governance backend-freeze-baseline
result = PASS

scripts/ai/verify_fast.sh
result = PASS

cargo fmt --all -- --check
git diff --check
result = PASS
```

The risk inventory canonical hash changed only because the test edit shifted
line-number-bearing scan rows. Every ownership and signal count is unchanged.

## Review State

Independent review and hosted checks are pending. Auto-merge is not enabled.

## Rollback

Revert the PAR-004A PR. That restores the false high-precision failure and
returns the capability-readiness queue to a known precision-fixture blocker;
it does not change production TWAP behavior.
