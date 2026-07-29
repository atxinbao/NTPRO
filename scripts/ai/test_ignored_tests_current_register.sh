#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECK="$ROOT/scripts/ai/check_ignored_tests_current_register.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

fail() {
  echo "ignored tests current register self-test failed: $*" >&2
  exit 1
}

run_check() {
  local current="$1"
  local historical="$2"
  local scope="$3"
  local scan_roots="$4"

  NTPRO_IGNORED_CURRENT_REGISTER="$current" \
    NTPRO_IGNORED_HISTORICAL_REGISTER="$historical" \
    NTPRO_IGNORED_REGISTER_SCOPE="$scope" \
    NTPRO_IGNORED_SCAN_ROOTS="$scan_roots" \
    "$CHECK"
}

expect_failure() {
  local name="$1"
  local current="$2"
  local historical="$3"
  local scope="$4"
  local scan_roots="$5"

  if run_check "$current" "$historical" "$scope" "$scan_roots" >"$TMP_DIR/$name.out" 2>&1; then
    fail "$name unexpectedly passed"
  fi
}

mkdir -p "$TMP_DIR/scan/crates/demo/tests" "$TMP_DIR/registers"
direct_path="$TMP_DIR/scan/crates/demo/tests/direct.rs"
conditional_path="$TMP_DIR/scan/crates/demo/tests/conditional.rs"
conditional_bare_path="$TMP_DIR/scan/crates/demo/tests/conditional_bare.rs"
comment_path="$TMP_DIR/scan/crates/demo/tests/comment.rs"
current="$TMP_DIR/registers/current.md"
historical="$TMP_DIR/registers/historical.md"

cat >"$direct_path" <<'EOF'
#[ignore = "synthetic direct fixture"]
fn direct_fixture() {}
EOF

cat >"$conditional_path" <<'EOF'
#[cfg_attr(
    feature = "high-precision",
    ignore = "synthetic conditional fixture"
)]
fn conditional_fixture() {}
EOF

cat >"$conditional_bare_path" <<'EOF'
#[cfg_attr(feature = "slow", ignore)]
fn conditional_bare_fixture() {}
EOF

cat >"$comment_path" <<'EOF'
/*
#[ignore = "commented out"]
*/
const TEXT: &str = "#[ignore]";
EOF

cat >"$current" <<EOF
# Synthetic Current Register
Register status: CURRENT
Direct ignored attributes: 1
Conditional ignored attributes: 2
Total ignored attributes across configurations: 3
\`$direct_path\`
\`$conditional_path\`
\`$conditional_bare_path\`
EOF

cat >"$historical" <<'EOF'
# Synthetic Historical Register
Register status: HISTORICAL_EXTENSION
EOF

run_check "$current" "$historical" "$TMP_DIR/registers" "$TMP_DIR/scan/crates" >/dev/null

cp "$current" "$TMP_DIR/stale-count.md"
sed -i.bak 's/^Direct ignored attributes: 1$/Direct ignored attributes: 0/' \
  "$TMP_DIR/stale-count.md"
expect_failure stale_count \
  "$TMP_DIR/stale-count.md" "$historical" "$TMP_DIR/registers" "$TMP_DIR/scan/crates"

mkdir "$TMP_DIR/duplicate-scope"
cp "$current" "$TMP_DIR/duplicate-scope/current.md"
cp "$historical" "$TMP_DIR/duplicate-scope/historical.md"
sed -i.bak 's/^Register status: HISTORICAL_EXTENSION$/Register status: CURRENT/' \
  "$TMP_DIR/duplicate-scope/historical.md"
expect_failure duplicate_current \
  "$TMP_DIR/duplicate-scope/current.md" \
  "$TMP_DIR/duplicate-scope/historical.md" \
  "$TMP_DIR/duplicate-scope" \
  "$TMP_DIR/scan/crates"

cp "$current" "$TMP_DIR/missing-path.md"
sed -i.bak "s#\`$conditional_path\`#\`removed/conditional.rs\`#" \
  "$TMP_DIR/missing-path.md"
expect_failure missing_path \
  "$TMP_DIR/missing-path.md" "$historical" "$TMP_DIR/registers" "$TMP_DIR/scan/crates"

echo "ignored tests current register self-test OK: cases=4"
