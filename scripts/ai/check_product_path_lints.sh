#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
source scripts/ai/toolchain_env.sh

PACKAGES="${NTPRO_PRODUCT_LINT_PACKAGES:-nautilus-cli nautilus-live nautilus-backtest nautilus-sandbox}"
EXTRA_CARGO_ARGS="${NTPRO_PRODUCT_LINT_EXTRA_CARGO_ARGS:-}"

echo "== product-path lint warning rollout =="
echo "packages=$PACKAGES"
echo "extra_cargo_args=${EXTRA_CARGO_ARGS:-<none>}"
echo "lints=unwrap_used,expect_used,indexing_slicing,unused_result_ok"
echo "mode=warning-only; existing warnings are tracked in docs/rust-cutover/quality/product_path_lint_register.md"

# This rollout must remain warning-only even when CI uses global rustflags for
# stricter gates in earlier steps.
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS

pkg_args=()
for package in $PACKAGES; do
  pkg_args+=("-p" "$package")
done

if [ -n "$EXTRA_CARGO_ARGS" ]; then
  # shellcheck disable=SC2206
  extra_args=($EXTRA_CARGO_ARGS)
  cargo clippy "${pkg_args[@]}" --lib --tests "${extra_args[@]}" -- \
    -W clippy::unwrap_used \
    -W clippy::expect_used \
    -W clippy::indexing_slicing \
    -W clippy::unused_result_ok
else
  cargo clippy "${pkg_args[@]}" --lib --tests -- \
    -W clippy::unwrap_used \
    -W clippy::expect_used \
    -W clippy::indexing_slicing \
    -W clippy::unused_result_ok
fi
