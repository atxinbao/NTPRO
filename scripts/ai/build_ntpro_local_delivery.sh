#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
output="${NTPRO_LOCAL_DELIVERY_OUTPUT:-$repo_root/target/ntpro-local-delivery}"
skip_build="${NTPRO_LOCAL_DELIVERY_SKIP_BUILD:-0}"
cargo_bin="${NTPRO_CARGO_BIN:-cargo}"

fail() {
  printf 'local_delivery_build=fail reason=%s\n' "$1" >&2
  exit 1
}

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

if [[ "$skip_build" != "0" && "$skip_build" != "1" ]]; then
  fail "NTPRO_LOCAL_DELIVERY_SKIP_BUILD must be 0 or 1"
fi

if [[ "$skip_build" == "0" ]]; then
  "$cargo_bin" build -p nautilus-cli --bin nautilus --bin ntpro-node
  npm --prefix "$repo_root/apps/strategy-workbench" ci
  npm --prefix "$repo_root/apps/strategy-workbench" run build
fi

nautilus_bin="${NTPRO_NAUTILUS_BIN:-$repo_root/target/debug/nautilus}"
node_bin="${NTPRO_NODE_BIN:-$repo_root/target/debug/ntpro-node}"
frontend_dist="${NTPRO_STRATEGY_WORKBENCH_DIST:-$repo_root/apps/strategy-workbench/dist}"
launcher="$repo_root/scripts/ai/ntpro_local_delivery_launcher.sh"
operations="$repo_root/docs/product/ntpro_local_delivery.md"
node_config="$repo_root/configs/nodes/btc-ema-shadow.toml"
backtest_config="$repo_root/configs/backtests/ema-cross-btcusdt-product.toml"

[[ -x "$nautilus_bin" ]] || fail "nautilus binary is missing: $nautilus_bin"
[[ -x "$node_bin" ]] || fail "ntpro-node binary is missing: $node_bin"
[[ -f "$frontend_dist/index.html" ]] || fail "strategy workbench dist is missing: $frontend_dist/index.html"
for file in "$launcher" "$operations" "$node_config" "$backtest_config"; do
  [[ -f "$file" ]] || fail "required delivery source is missing: $file"
done

output_parent="$(dirname -- "$output")"
mkdir -p "$output_parent"
if [[ -e "$output" && ! -f "$output/.ntpro-local-delivery-root" ]]; then
  fail "refusing to replace unrecognized output directory: $output"
fi

staging="${output}.tmp.$$"
rm -rf "$staging"
mkdir -p \
  "$staging/bin" \
  "$staging/configs/nodes" \
  "$staging/configs/backtests" \
  "$staging/apps/strategy-workbench/dist"
cleanup() {
  rm -rf "$staging"
}
trap cleanup EXIT

install -m 0755 "$nautilus_bin" "$staging/bin/nautilus"
install -m 0755 "$node_bin" "$staging/bin/ntpro-node"
install -m 0755 "$launcher" "$staging/start-ntpro"
install -m 0644 "$node_config" "$staging/configs/nodes/btc-ema-shadow.toml"
install -m 0644 "$backtest_config" "$staging/configs/backtests/ema-cross-btcusdt-product.toml"
install -m 0644 "$operations" "$staging/操作说明.md"
cp -R "$frontend_dist/." "$staging/apps/strategy-workbench/dist/"
printf '%s\n' 'ntpro.local_delivery.v1' >"$staging/.ntpro-local-delivery-root"

source_sha="$(git -C "$repo_root" rev-parse HEAD 2>/dev/null || printf 'unknown')"
built_at="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
nautilus_sha="$(sha256_file "$staging/bin/nautilus")"
node_sha="$(sha256_file "$staging/bin/ntpro-node")"
frontend_sha="$(sha256_file "$staging/apps/strategy-workbench/dist/index.html")"
cat >"$staging/delivery-manifest.json" <<EOF
{
  "schema_version": "ntpro.local_delivery_manifest.v1",
  "source_sha": "$source_sha",
  "built_at": "$built_at",
  "entrypoint": "start-ntpro",
  "workspace_policy": "external_persistent_user_data",
  "components": {
    "nautilus_sha256": "$nautilus_sha",
    "ntpro_node_sha256": "$node_sha",
    "strategy_workbench_index_sha256": "$frontend_sha"
  }
}
EOF

if [[ -e "$output" ]]; then
  rm -rf "$output"
fi
mv "$staging" "$output"
trap - EXIT

printf 'local_delivery_build=pass output=%s source_sha=%s\n' "$output" "$source_sha"
printf 'user_entry=%s/start-ntpro\n' "$output"
