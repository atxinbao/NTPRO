# PAR-004A TWAP High-Precision Remainder Fixture Evidence

Date: 2026-07-29
Executor: Codex
GitHub issue: #1185
Status: LOCAL VALIDATION PASSED / REVIEW_REQUIRED

## Root Cause

```text
failing invocation = --features nautilus-model/high-precision
local forwarding feature = nautilus-trading/high-precision
stale selector = cfg(feature = "high-precision") on nautilus-trading
observed branch = standard raw literal branch under a high-precision model
production TWAP calculation changed = false
```

`nautilus-trading` declares a local `high-precision` feature that forwards to
`nautilus-model/high-precision`. The failing audit command directly enabled the
dependency feature instead. Cargo feature activation does not flow backward
from a dependency to a dependent crate, so the trading crate's local `cfg`
remained false while `QuantityRaw` and `FIXED_PRECISION` were high precision.
The test therefore compared a correct high-precision result with a
standard-precision raw literal.

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

```text
first pass = P2 evidence incorrectly said the trading crate did not declare a
             local high-precision feature
repair = task, evidence, PR body, and local review records now distinguish the
         existing forwarding feature from direct dependency-feature activation
rereview = pending
```

Hosted checks are pending. Auto-merge is not enabled.

## Rollback

Revert the PAR-004A PR. That restores the false high-precision failure and
returns the capability-readiness queue to a known precision-fixture blocker;
it does not change production TWAP behavior.
