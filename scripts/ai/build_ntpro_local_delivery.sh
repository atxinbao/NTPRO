#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)"
output="${NTPRO_LOCAL_DELIVERY_OUTPUT:-$repo_root/target/ntpro-local-delivery}"
allow_dirty="${NTPRO_LOCAL_DELIVERY_ALLOW_DIRTY:-0}"

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

if [[ "$allow_dirty" != "0" && "$allow_dirty" != "1" ]]; then
  fail "NTPRO_LOCAL_DELIVERY_ALLOW_DIRTY must be 0 or 1"
fi

source_dirty=false
if [[ -n "$(git -C "$repo_root" status --porcelain --untracked-files=normal)" ]]; then
  source_dirty=true
fi
if [[ "$source_dirty" == "true" && "$allow_dirty" != "1" ]]; then
  fail "source tree is dirty; commit the delivery sources or set NTPRO_LOCAL_DELIVERY_ALLOW_DIRTY=1 for development-only output"
fi

source "$repo_root/scripts/ai/toolchain_env.sh"
"$NTPRO_CARGO" build -p nautilus-cli --bin nautilus --bin ntpro-node
npm --prefix "$repo_root/apps/strategy-workbench" ci
npm --prefix "$repo_root/apps/strategy-workbench" run build

nautilus_bin="$repo_root/target/debug/nautilus"
node_bin="$repo_root/target/debug/ntpro-node"
frontend_dist="$repo_root/apps/strategy-workbench/dist"
launcher="$repo_root/scripts/ai/ntpro_local_delivery_launcher.sh"
operations="$repo_root/docs/product/ntpro_local_delivery.md"
node_config="$repo_root/configs/nodes/btc-ema-shadow.toml"
backtest_config="$repo_root/configs/backtests/ema-cross-btcusdt-product.toml"
license="$repo_root/LICENSE"

[[ -x "$nautilus_bin" ]] || fail "nautilus binary is missing: $nautilus_bin"
[[ -x "$node_bin" ]] || fail "ntpro-node binary is missing: $node_bin"
[[ -f "$frontend_dist/index.html" ]] || fail "strategy workbench dist is missing: $frontend_dist/index.html"
for file in "$launcher" "$operations" "$node_config" "$backtest_config" "$license"; do
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
install -m 0644 "$license" "$staging/LICENSE"
cp -R "$frontend_dist/." "$staging/apps/strategy-workbench/dist/"
printf '%s\n' 'ntpro.local_delivery.v1' >"$staging/.ntpro-local-delivery-root"

source_sha="$(git -C "$repo_root" rev-parse HEAD 2>/dev/null || printf 'unknown')"
built_at="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
platform_os="$(uname -s)"
platform_arch="$(uname -m)"
rust_target="$("$NTPRO_RUSTC" -vV | awk '/^host: /{host=$2} END{print host}')"
if [[ "$source_dirty" == "true" ]]; then
  source_binding="git_head_dirty_workspace_build"
else
  source_binding="git_head_clean_workspace_build"
fi
nautilus_sha="$(sha256_file "$staging/bin/nautilus")"
node_sha="$(sha256_file "$staging/bin/ntpro-node")"
frontend_sha="$(sha256_file "$staging/apps/strategy-workbench/dist/index.html")"
launcher_sha="$(sha256_file "$staging/start-ntpro")"
node_config_sha="$(sha256_file "$staging/configs/nodes/btc-ema-shadow.toml")"
backtest_config_sha="$(sha256_file "$staging/configs/backtests/ema-cross-btcusdt-product.toml")"
cat >"$staging/delivery-manifest.json" <<EOF
{
  "schema_version": "ntpro.local_delivery_manifest.v1",
  "source_sha": "$source_sha",
  "source_tree_dirty": $source_dirty,
  "source_binding": "$source_binding",
  "built_at": "$built_at",
  "platform": {
    "os": "$platform_os",
    "arch": "$platform_arch",
    "rust_target": "$rust_target"
  },
  "entrypoint": "start-ntpro",
  "workspace_policy": "external_persistent_user_data",
  "components": {
    "nautilus_sha256": "$nautilus_sha",
    "ntpro_node_sha256": "$node_sha",
    "strategy_workbench_index_sha256": "$frontend_sha",
    "launcher_sha256": "$launcher_sha",
    "node_config_sha256": "$node_config_sha",
    "backtest_config_sha256": "$backtest_config_sha"
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
