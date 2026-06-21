#!/usr/bin/env bash
set -euo pipefail

# V130-003: owner-run production online read-only proof pack.
# Default mode is CI-safe and only proves fail-closed preflight behavior:
#
#   scripts/ai/verify_v13_online_readonly_proof_pack.sh
#
# Optional owner-run online mode must be explicit and still uses only the
# existing v0.12 production GET read-only public/account probes:
#
#   NTPRO_V13_OWNER_RUN_ONLINE_READONLY_PROOF=1
#   NTPRO_V13_OWNER_ACCEPTS_PRODUCTION_READONLY_RISK=1
#   NTPRO_V12_MANUAL_ONLINE=1
#   ... plus the v0.12 public/account read-only gates and read-only credentials
#
# This script never creates an order endpoint, Dashboard control, listenKey
# lifecycle, or production mutation path.

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"
source scripts/ai/toolchain_env.sh

if [[ "${NTPRO_V13_SKIP_BUILD:-0}" != "1" && -z "${NTPRO_V13_NAUTILUS_BIN:-}" ]]; then
  cargo build -p nautilus-cli --bin nautilus
fi

NAUTILUS_BIN="${NTPRO_V13_NAUTILUS_BIN:-$ROOT_DIR/target/debug/nautilus}"
if [[ ! -x "$NAUTILUS_BIN" ]]; then
  echo "missing nautilus binary: $NAUTILUS_BIN" >&2
  exit 1
fi

PACK_ROOT="${NTPRO_V13_ONLINE_READONLY_PROOF_PACK_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/ntpro-v13-online-readonly-proof-pack.XXXXXX")}"
PUBLIC_ROOT="$PACK_ROOT/public"
ACCOUNT_ROOT="$PACK_ROOT/account"
LOG_ROOT="$PACK_ROOT/script-logs"
MANIFEST_JSON="$PACK_ROOT/proof-pack-manifest.json"

mkdir -p "$PUBLIC_ROOT" "$ACCOUNT_ROOT" "$LOG_ROOT"

OWNER_MODE=false
if [[ "${NTPRO_V13_OWNER_RUN_ONLINE_READONLY_PROOF:-0}" == "1" ]]; then
  OWNER_MODE=true
fi

export NTPRO_V12_SKIP_BUILD=1
export NTPRO_V12_NAUTILUS_BIN="$NAUTILUS_BIN"

run_offline_preflight() {
  echo "== v13 proof pack: offline public production GET preflight =="
  env -u NTPRO_V12_MANUAL_ONLINE \
    NTPRO_V12_PUBLIC_READ_ROOT="$PUBLIC_ROOT" \
    scripts/ai/verify_v12_public_online_read_probe.sh \
      >"$LOG_ROOT/public.stdout.log" \
      2>"$LOG_ROOT/public.stderr.log"
  echo 0 >"$LOG_ROOT/public.exit_code"

  echo "== v13 proof pack: offline authenticated account GET preflight =="
  env -u NTPRO_V12_MANUAL_ONLINE \
    NTPRO_V12_ACCOUNT_SNAPSHOT_ROOT="$ACCOUNT_ROOT" \
    scripts/ai/verify_v12_authenticated_account_snapshot_online_read.sh \
      >"$LOG_ROOT/account.stdout.log" \
      2>"$LOG_ROOT/account.stderr.log"
  echo 0 >"$LOG_ROOT/account.exit_code"
}

run_owner_online() {
  if [[ "${NTPRO_V13_OWNER_ACCEPTS_PRODUCTION_READONLY_RISK:-0}" != "1" ]]; then
    echo "owner-run proof pack requires NTPRO_V13_OWNER_ACCEPTS_PRODUCTION_READONLY_RISK=1" >&2
    exit 2
  fi
  if [[ "${NTPRO_V12_MANUAL_ONLINE:-0}" != "1" ]]; then
    echo "owner-run proof pack requires NTPRO_V12_MANUAL_ONLINE=1" >&2
    exit 2
  fi

  echo "== v13 proof pack: owner-run public production GET read-only proof =="
  set +e
  NTPRO_V12_PUBLIC_READ_ROOT="$PUBLIC_ROOT" \
    scripts/ai/verify_v12_public_online_read_probe.sh \
      >"$LOG_ROOT/public.stdout.log" \
      2>"$LOG_ROOT/public.stderr.log"
  public_code=$?
  set -e
  echo "$public_code" >"$LOG_ROOT/public.exit_code"

  echo "== v13 proof pack: owner-run authenticated account GET read-only proof =="
  set +e
  NTPRO_V12_ACCOUNT_SNAPSHOT_ROOT="$ACCOUNT_ROOT" \
    scripts/ai/verify_v12_authenticated_account_snapshot_online_read.sh \
      >"$LOG_ROOT/account.stdout.log" \
      2>"$LOG_ROOT/account.stderr.log"
  account_code=$?
  set -e
  echo "$account_code" >"$LOG_ROOT/account.exit_code"
}

if [[ "$OWNER_MODE" == true ]]; then
  run_owner_online
else
  run_offline_preflight
fi

python3 - "$PACK_ROOT" "$MANIFEST_JSON" "$OWNER_MODE" <<'PY'
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

pack_root = Path(sys.argv[1])
manifest_path = Path(sys.argv[2])
owner_mode = sys.argv[3] == "true"

public_artifact = (
    pack_root / "public/command-output/manual-online-public-read-probe.json"
    if owner_mode
    else pack_root / "public/command-output/manual-preflight-public-read-probe.json"
)
account_artifact = (
    pack_root / "account/command-output/manual-online-account-snapshot.json"
    if owner_mode
    else pack_root / "account/command-output/manual-preflight-account-snapshot.json"
)

public_exit = int((pack_root / "script-logs/public.exit_code").read_text().strip())
account_exit = int((pack_root / "script-logs/account.exit_code").read_text().strip())

def require(condition, message):
    if not condition:
        raise SystemExit(message)

def load_json(path):
    require(path.exists(), f"missing expected artifact: {path}")
    return json.loads(path.read_text())

public_report = load_json(public_artifact)
account_report = load_json(account_artifact)

secret_values = [
    os.environ.get("BINANCE_PRODUCTION_READONLY_API_KEY", ""),
    os.environ.get("BINANCE_PRODUCTION_READONLY_API_SECRET", ""),
    "ntpro_v120002_script_synthetic_api_key_value",
    "ntpro_v120002_script_synthetic_api_secret_value",
]
secret_values = [value for value in secret_values if value]

for path in pack_root.rglob("*"):
    if not path.is_file():
        continue
    text = path.read_text(errors="ignore")
    for secret in secret_values:
        require(secret not in text, f"secret-like value leaked into {path}")

def assert_no_mutation(report, label):
    require(report.get("production_order_submission_attempted") is False, label)
    require(report.get("production_order_mutation_attempted") is False, label)
    require(report.get("dashboard_order_controls_enabled") is False, label)
    require(report.get("account_mutation_attempted", False) is False, label)
    require(report.get("order_endpoint_access_attempted", False) is False, label)
    require(report.get("signed_url_recorded", False) is False, label)
    require(report.get("signed_query_recorded", False) is False, label)
    require(report.get("signature_recorded", False) is False, label)
    if "secrets_redacted" in report:
        require(report["secrets_redacted"] is True, label)
    summary = report.get("response_shape_summary")
    if isinstance(summary, dict):
        require(summary.get("raw_account_response_recorded") is False, label)
        require(summary.get("raw_balances_recorded") is False, label)
        require(summary.get("raw_permissions_recorded") is False, label)

assert_no_mutation(public_report, "public probe mutation invariant failed")
assert_no_mutation(account_report, "account snapshot mutation invariant failed")

if owner_mode:
    require(public_report.get("manual_online_requested") is True, public_report)
    require(account_report.get("manual_online_requested") is True, account_report)
    require(public_report.get("network_attempted") is True, public_report)
    require(account_report.get("network_attempted") is True, account_report)
    public_ok = public_exit == 0 and public_report.get("error_code") == "none"
    account_ok = account_exit == 0 and account_report.get("error_code") == "none"
    status = "owner_run_online_ok" if public_ok and account_ok else "owner_run_classified_failure"
else:
    require(public_exit == 0, public_report)
    require(account_exit == 0, account_report)
    require(public_report.get("status") == "blocked_missing_manual_online_gate", public_report)
    require(account_report.get("status") == "blocked_missing_manual_online_gate", account_report)
    require(public_report.get("network_attempted") is False, public_report)
    require(account_report.get("network_attempted") is False, account_report)
    require(public_report.get("production_public_online_read_attempted") is False, public_report)
    require(account_report.get("account_read_attempted") is False, account_report)
    status = "offline_preflight_ok"

manifest = {
    "schema_version": "ntpro.v130_online_readonly_proof_pack.v1",
    "generated_at_utc": datetime.now(timezone.utc).isoformat(),
    "status": status,
    "mode": "owner_run_online" if owner_mode else "offline_preflight",
    "default_ci_network_required": False,
    "owner_run_online_proof_required_for_release": False,
    "production_order_submission_allowed": False,
    "production_order_mutation_allowed": False,
    "production_order_state_reads_allowed": False,
    "listen_key_lifecycle_allowed": False,
    "dashboard_order_controls_enabled": False,
    "real_funds_enabled": False,
    "production_trading_enabled": False,
    "artifacts_redacted": True,
    "public_probe": {
        "artifact": str(public_artifact.relative_to(pack_root)),
        "script_exit_code": public_exit,
        "status": public_report.get("status"),
        "error_code": public_report.get("error_code"),
        "method": public_report.get("method"),
        "path": public_report.get("path"),
        "network_attempted": public_report.get("network_attempted"),
        "production_public_online_read_attempted": public_report.get(
            "production_public_online_read_attempted"
        ),
        "response_shape": public_report.get("response_shape"),
        "response_shape_validated": public_report.get("response_shape_validated"),
        "credentials_used": public_report.get("credentials_used"),
        "production_order_submission_attempted": public_report.get(
            "production_order_submission_attempted"
        ),
        "production_order_mutation_attempted": public_report.get(
            "production_order_mutation_attempted"
        ),
    },
    "account_snapshot": {
        "artifact": str(account_artifact.relative_to(pack_root)),
        "script_exit_code": account_exit,
        "status": account_report.get("status"),
        "error_code": account_report.get("error_code"),
        "method": account_report.get("method"),
        "path": account_report.get("path"),
        "network_attempted": account_report.get("network_attempted"),
        "account_read_attempted": account_report.get("account_read_attempted"),
        "response_shape": account_report.get("response_shape"),
        "response_shape_validated": account_report.get("response_shape_validated"),
        "secrets_redacted": account_report.get("secrets_redacted"),
        "production_order_submission_attempted": account_report.get(
            "production_order_submission_attempted"
        ),
        "production_order_mutation_attempted": account_report.get(
            "production_order_mutation_attempted"
        ),
    },
    "diagnostic": (
        "Owner-run classified failures are evidence packages only, not trading "
        "readiness. The proof pack does not authorize production order "
        "submission, mutation, order-state reads, listenKey lifecycle, real "
        "funds, or Dashboard order controls."
    ),
}

manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")

if owner_mode and status != "owner_run_online_ok":
    print(
        "v13_online_readonly_proof_pack "
        f"status={status} root={pack_root} manifest={manifest_path} "
        "trading_readiness=false"
    )
    raise SystemExit(20)

print(
    "v13_online_readonly_proof_pack "
    f"status={status} root={pack_root} manifest={manifest_path} "
    "network_required_for_ci=false production_order_mutation_allowed=false "
    "dashboard_order_controls_enabled=false"
)
PY
