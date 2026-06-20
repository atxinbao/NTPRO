#!/usr/bin/env bash
set -euo pipefail

# V110-003: v0.11 authenticated production account snapshot contract.
# This script is safe for local development and CI. It does not open network
# connections, does not persist credential values, and does not access order or
# mutation endpoints.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V11_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V11_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V11_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

SNAPSHOT_ROOT="${NTPRO_V11_ACCOUNT_SNAPSHOT_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v11-account-snapshot.XXXXXX")}"
OUTPUT_DIR="$SNAPSHOT_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

BLOCKED_JSON="$OUTPUT_DIR/blocked-account-snapshot.json"
BLOCKED_STDOUT="$OUTPUT_DIR/blocked.stdout.log"
BLOCKED_STDERR="$OUTPUT_DIR/blocked.stderr.log"
MISSING_CREDENTIALS_JSON="$OUTPUT_DIR/missing-credentials-account-snapshot.json"
MISSING_CREDENTIALS_STDOUT="$OUTPUT_DIR/missing-credentials.stdout.log"
MISSING_CREDENTIALS_STDERR="$OUTPUT_DIR/missing-credentials.stderr.log"
READY_JSON="$OUTPUT_DIR/ready-account-snapshot.json"
READY_STDOUT="$OUTPUT_DIR/ready.stdout.log"
READY_STDERR="$OUTPUT_DIR/ready.stderr.log"

SYNTHETIC_API_KEY="ntpro_v110003_script_synthetic_api_key_value"
SYNTHETIC_API_SECRET="ntpro_v110003_script_synthetic_api_secret_value"

env \
  -u NTPRO_ALLOW_PRODUCTION_AUTHENTICATED_READ \
  -u NTPRO_OWNER_APPROVED_PRODUCTION_READ_ONLY \
  -u NTPRO_CONFIRM_PRODUCTION_ACCOUNT_NO_ORDER_MUTATION \
  -u NTPRO_CONFIRM_NO_SECRET_PERSISTENCE \
  -u BINANCE_PRODUCTION_READONLY_API_KEY \
  -u BINANCE_PRODUCTION_READONLY_API_SECRET \
  "$NAUTILUS_BIN" live production-account-snapshot-contract \
    --output "$BLOCKED_JSON" \
    >"$BLOCKED_STDOUT" \
    2>"$BLOCKED_STDERR"

if [[ -s "$BLOCKED_STDERR" ]]; then
  echo "v11 account snapshot blocked path wrote stderr" >&2
  cat "$BLOCKED_STDERR" >&2
  exit 1
fi

grep -q "live.production_account_snapshot_contract status=blocked_missing_gate" "$BLOCKED_STDOUT"
grep -q '"schema_version": "ntpro.v110_authenticated_account_snapshot_contract.v1"' "$BLOCKED_JSON"
grep -q '"status": "blocked_missing_gate"' "$BLOCKED_JSON"
grep -q '"endpoint_class": "production_authenticated_read_only"' "$BLOCKED_JSON"
grep -q '"path": "/api/v3/account"' "$BLOCKED_JSON"
grep -q '"requires_api_key": true' "$BLOCKED_JSON"
grep -q '"requires_signature": true' "$BLOCKED_JSON"
grep -q '"read_allowed": false' "$BLOCKED_JSON"
grep -q '"mutation_allowed": false' "$BLOCKED_JSON"
grep -q '"network_attempted": false' "$BLOCKED_JSON"
grep -q '"env_credentials_only": true' "$BLOCKED_JSON"
grep -q '"account_read_attempted": false' "$BLOCKED_JSON"
grep -q '"account_mutation_attempted": false' "$BLOCKED_JSON"
grep -q '"order_endpoint_access_attempted": false' "$BLOCKED_JSON"
grep -q '"production_order_submission_attempted": false' "$BLOCKED_JSON"
grep -q '"production_order_mutation_attempted": false' "$BLOCKED_JSON"
grep -q '"dashboard_order_controls_enabled": false' "$BLOCKED_JSON"
grep -q '"secrets_redacted": true' "$BLOCKED_JSON"

NTPRO_ALLOW_PRODUCTION_AUTHENTICATED_READ=1 \
NTPRO_OWNER_APPROVED_PRODUCTION_READ_ONLY=1 \
NTPRO_CONFIRM_PRODUCTION_ACCOUNT_NO_ORDER_MUTATION=1 \
NTPRO_CONFIRM_NO_SECRET_PERSISTENCE=1 \
  "$NAUTILUS_BIN" live production-account-snapshot-contract \
    --output "$MISSING_CREDENTIALS_JSON" \
    --allow-production-authenticated-read \
    --confirm-owner-approved-read-only \
    --confirm-no-order-mutation \
    --confirm-no-secret-persistence \
    >"$MISSING_CREDENTIALS_STDOUT" \
    2>"$MISSING_CREDENTIALS_STDERR"

if [[ -s "$MISSING_CREDENTIALS_STDERR" ]]; then
  echo "v11 account snapshot missing-credentials path wrote stderr" >&2
  cat "$MISSING_CREDENTIALS_STDERR" >&2
  exit 1
fi

grep -q "live.production_account_snapshot_contract status=blocked_missing_credentials" "$MISSING_CREDENTIALS_STDOUT"
grep -q '"status": "blocked_missing_credentials"' "$MISSING_CREDENTIALS_JSON"
grep -q '"api_key_present": false' "$MISSING_CREDENTIALS_JSON"
grep -q '"api_secret_present": false' "$MISSING_CREDENTIALS_JSON"
grep -q '"network_attempted": false' "$MISSING_CREDENTIALS_JSON"
grep -q '"production_order_mutation_attempted": false' "$MISSING_CREDENTIALS_JSON"

NTPRO_ALLOW_PRODUCTION_AUTHENTICATED_READ=1 \
NTPRO_OWNER_APPROVED_PRODUCTION_READ_ONLY=1 \
NTPRO_CONFIRM_PRODUCTION_ACCOUNT_NO_ORDER_MUTATION=1 \
NTPRO_CONFIRM_NO_SECRET_PERSISTENCE=1 \
BINANCE_PRODUCTION_READONLY_API_KEY="$SYNTHETIC_API_KEY" \
BINANCE_PRODUCTION_READONLY_API_SECRET="$SYNTHETIC_API_SECRET" \
  "$NAUTILUS_BIN" live production-account-snapshot-contract \
    --output "$READY_JSON" \
    --allow-production-authenticated-read \
    --confirm-owner-approved-read-only \
    --confirm-no-order-mutation \
    --confirm-no-secret-persistence \
    >"$READY_STDOUT" \
    2>"$READY_STDERR"

if [[ -s "$READY_STDERR" ]]; then
  echo "v11 account snapshot ready path wrote stderr" >&2
  cat "$READY_STDERR" >&2
  exit 1
fi

grep -q "live.production_account_snapshot_contract status=ready_offline_contract" "$READY_STDOUT"
grep -q '"status": "ready_offline_contract"' "$READY_JSON"
grep -q '"read_allowed": true' "$READY_JSON"
grep -q '"api_key_present": true' "$READY_JSON"
grep -q '"api_secret_present": true' "$READY_JSON"
grep -q '"api_key_value_recorded": false' "$READY_JSON"
grep -q '"api_secret_value_recorded": false' "$READY_JSON"
grep -q '"signature_recorded": false' "$READY_JSON"
grep -q '"signed_query_recorded": false' "$READY_JSON"
grep -q '"signed_url_recorded": false' "$READY_JSON"
grep -q '"network_attempted": false' "$READY_JSON"
grep -q '"account_read_attempted": false' "$READY_JSON"
grep -q '"account_mutation_attempted": false' "$READY_JSON"
grep -q '"order_endpoint_access_attempted": false' "$READY_JSON"
grep -q '"production_order_submission_attempted": false' "$READY_JSON"
grep -q '"production_order_mutation_attempted": false' "$READY_JSON"
grep -q '"dashboard_order_controls_enabled": false' "$READY_JSON"
grep -q '"secrets_redacted": true' "$READY_JSON"

if grep -q "$SYNTHETIC_API_KEY\\|$SYNTHETIC_API_SECRET" "$READY_JSON" "$READY_STDOUT" "$READY_STDERR"; then
  echo "v11 account snapshot leaked synthetic credential value" >&2
  exit 1
fi

echo "v11_authenticated_account_snapshot_contract status=ok root=$SNAPSHOT_ROOT"
