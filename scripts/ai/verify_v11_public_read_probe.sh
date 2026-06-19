#!/usr/bin/env bash
set -euo pipefail

# V110-002: v0.11 production public read-only probe contract.
# This script is safe for local development and CI. It does not open network
# connections, does not read credentials, and does not submit or mutate orders.

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

PROBE_ROOT="${NTPRO_V11_PUBLIC_READ_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v11-public-read.XXXXXX")}"
OUTPUT_DIR="$PROBE_ROOT/command-output"
mkdir -p "$OUTPUT_DIR"

BLOCKED_JSON="$OUTPUT_DIR/blocked-public-read-probe.json"
BLOCKED_STDOUT="$OUTPUT_DIR/blocked.stdout.log"
BLOCKED_STDERR="$OUTPUT_DIR/blocked.stderr.log"
READY_JSON="$OUTPUT_DIR/ready-public-read-probe.json"
READY_STDOUT="$OUTPUT_DIR/ready.stdout.log"
READY_STDERR="$OUTPUT_DIR/ready.stderr.log"

env \
  -u NTPRO_ALLOW_PRODUCTION_PUBLIC_READ \
  -u NTPRO_CONFIRM_PRODUCTION_PUBLIC_READ_ONLY \
  -u NTPRO_CONFIRM_NO_PRODUCTION_ORDER_MUTATION \
  "$NAUTILUS_BIN" live production-public-read-probe \
    --endpoint server-time \
    --output "$BLOCKED_JSON" \
    >"$BLOCKED_STDOUT" \
    2>"$BLOCKED_STDERR"

if [[ -s "$BLOCKED_STDERR" ]]; then
  echo "v11 public read blocked path wrote stderr" >&2
  cat "$BLOCKED_STDERR" >&2
  exit 1
fi

grep -q "live.production_public_read_probe status=blocked_missing_gate" "$BLOCKED_STDOUT"
grep -q '"schema_version": "ntpro.v110_production_public_read_probe.v1"' "$BLOCKED_JSON"
grep -q '"status": "blocked_missing_gate"' "$BLOCKED_JSON"
grep -q '"endpoint_class": "production_public_read_only"' "$BLOCKED_JSON"
grep -q '"path": "/api/v3/time"' "$BLOCKED_JSON"
grep -q '"requires_api_key": false' "$BLOCKED_JSON"
grep -q '"requires_signature": false' "$BLOCKED_JSON"
grep -q '"read_allowed": false' "$BLOCKED_JSON"
grep -q '"mutation_allowed": false' "$BLOCKED_JSON"
grep -q '"network_attempted": false' "$BLOCKED_JSON"
grep -q '"credentials_used": false' "$BLOCKED_JSON"
grep -q '"production_order_submission_attempted": false' "$BLOCKED_JSON"
grep -q '"production_order_mutation_attempted": false' "$BLOCKED_JSON"
grep -q '"dashboard_order_controls_enabled": false' "$BLOCKED_JSON"

NTPRO_ALLOW_PRODUCTION_PUBLIC_READ=1 \
NTPRO_CONFIRM_PRODUCTION_PUBLIC_READ_ONLY=1 \
NTPRO_CONFIRM_NO_PRODUCTION_ORDER_MUTATION=1 \
  "$NAUTILUS_BIN" live production-public-read-probe \
    --endpoint exchange-info \
    --output "$READY_JSON" \
    --allow-production-public-read \
    --confirm-read-only \
    --confirm-no-order-mutation \
    >"$READY_STDOUT" \
    2>"$READY_STDERR"

if [[ -s "$READY_STDERR" ]]; then
  echo "v11 public read ready path wrote stderr" >&2
  cat "$READY_STDERR" >&2
  exit 1
fi

grep -q "live.production_public_read_probe status=ready_offline_contract" "$READY_STDOUT"
grep -q '"status": "ready_offline_contract"' "$READY_JSON"
grep -q '"endpoint": "exchange_info"' "$READY_JSON"
grep -q '"path": "/api/v3/exchangeInfo"' "$READY_JSON"
grep -q '"read_allowed": true' "$READY_JSON"
grep -q '"mutation_allowed": false' "$READY_JSON"
grep -q '"online_execution_supported": false' "$READY_JSON"
grep -q '"network_attempted": false' "$READY_JSON"
grep -q '"credentials_used": false' "$READY_JSON"
grep -q '"account_mutation_attempted": false' "$READY_JSON"
grep -q '"production_order_submission_attempted": false' "$READY_JSON"
grep -q '"production_order_mutation_attempted": false' "$READY_JSON"
grep -q '"dashboard_order_controls_enabled": false' "$READY_JSON"

echo "v11_public_read_probe status=ok root=$PROBE_ROOT"
