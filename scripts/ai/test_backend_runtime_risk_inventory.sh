#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

fail() {
  echo "backend runtime risk inventory self-test failed: $*" >&2
  exit 1
}

mkdir -p \
  "$TMP_DIR/crates/demo/src/generated" \
  "$TMP_DIR/crates/demo/src/ffi" \
  "$TMP_DIR/crates/demo/tests" \
  "$TMP_DIR/plain/crates/demo/src"

cat >"$TMP_DIR/crates/demo/src/lib.rs" <<'EOF'
pub fn production() {
    result.expect("production");
}

#[cfg(test)]
#[expect(clippy::needless_return)]
mod tests {
    #[test]
    fn inline_test() {
        let normal = "a string with a closing brace }";
        let raw = r###"a raw string with braces { }"###;
        /* a nested block comment { /* } */ } */
        option.unwrap();
        if normal == raw {
            panic!("inline");
        }
    }
} pub fn production_on_closing_line() { result.expect("production"); }

#[cfg(test)]
fn conservatively_owned_item() {
    result.expect("not an explicit test module");
}

pub fn after_inline_module() {
    todo!();
}
EOF

cat >"$TMP_DIR/crates/demo/src/generated/model.rs" <<'EOF'
#[cfg(test)]
mod tests {
    fn generated_test() {
        option.unwrap();
    }
}
EOF

cat >"$TMP_DIR/crates/demo/src/ffi/boundary.rs" <<'EOF'
#[cfg(test)]
mod tests {
    unsafe fn ffi_test() {}
}
EOF

cat >"$TMP_DIR/crates/demo/tests/integration.rs" <<'EOF'
fn integration_test() {
    option.unwrap();
}
EOF

cat >"$TMP_DIR/plain/crates/demo/src/lib.rs" <<'EOF'
pub fn production_only() {
    option.unwrap();
}
EOF

canonical="$TMP_DIR/canonical.tsv"
NTPRO_RISK_SCAN_ROOT="$TMP_DIR/crates" \
  "$ROOT/scripts/ai/check_backend_runtime_risk_inventory.sh" --canonical \
  >"$canonical"

count_row() {
  local owner="$1"
  local signal="$2"
  awk -F '\t' -v owner="$owner" -v signal="$signal" \
    '$1 == owner && $2 == signal { count++ } END { print count + 0 }' "$canonical"
}

[[ "$(count_row production expect)" == "3" ]] \
  || fail "production, conservative cfg(test) item, or closing-line ownership drift"
[[ "$(count_row production todo_macro)" == "1" ]] \
  || fail "production code after inline module was misclassified"
[[ "$(count_row test_inline unwrap)" == "1" ]] \
  || fail "inline test unwrap was not classified as test_inline"
[[ "$(count_row test_inline panic)" == "1" ]] \
  || fail "inline test panic was not classified as test_inline"
[[ "$(count_row generated unwrap)" == "1" ]] \
  || fail "generated ownership did not take precedence"
[[ "$(count_row ffi unsafe)" == "1" ]] \
  || fail "ffi ownership did not take precedence"
[[ "$(count_row test unwrap)" == "1" ]] \
  || fail "dedicated test ownership drift"

plain_canonical="$TMP_DIR/plain-canonical.tsv"
NTPRO_RISK_SCAN_ROOT="$TMP_DIR/plain/crates" \
  "$ROOT/scripts/ai/check_backend_runtime_risk_inventory.sh" --canonical \
  >"$plain_canonical"
[[ "$(awk -F '\t' '$1 == "production" && $2 == "unwrap" { count++ } END { print count + 0 }' "$plain_canonical")" == "1" ]] \
  || fail "scan root without inline modules was not production-classified"

echo "backend runtime risk inventory self-test OK: cases=8"
