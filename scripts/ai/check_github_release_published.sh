#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CURRENT_RELEASE_VERSION="${NTPRO_CURRENT_RELEASE_VERSION:-v0.26.0}"
CURRENT_RELEASE_TAG="${NTPRO_CURRENT_RELEASE_TAG:-ntpro-rust-only-${CURRENT_RELEASE_VERSION}}"
RELEASE_NAME="${NTPRO_CURRENT_RELEASE_NAME:-NTPRO Rust-only ${CURRENT_RELEASE_VERSION}}"
RELEASE_URL="${NTPRO_CURRENT_RELEASE_URL:-https://github.com/atxinbao/NTPRO/releases/tag/${CURRENT_RELEASE_TAG}}"
GH_BIN="${NTPRO_RELEASE_PUBLICATION_GH_BIN:-gh}"
PREPUBLISH_TAG_GATE="${NTPRO_RELEASE_PUBLICATION_PREPUBLISH_TAG_GATE:-0}"

CURRENT_RELEASE_STEM="v${CURRENT_RELEASE_VERSION#v}"
CURRENT_RELEASE_STEM="${CURRENT_RELEASE_STEM//./_}"
CURRENT_RELEASE_NOTES="${NTPRO_CURRENT_RELEASE_NOTES:-docs/rust-cutover/release/${CURRENT_RELEASE_STEM}_release_notes.md}"

fail() {
  echo "release publication drift: $*" >&2
  exit 1
}

offline_skip() {
  local reason="$1"
  if [[ "${NTPRO_RELEASE_PUBLICATION_ALLOW_OFFLINE:-0}" == "1" ]]; then
    echo "release_publication_guard=offline_skip reason=$reason"
    exit 0
  fi
  fail "$reason"
}

require_file() {
  local file="$1"
  [[ -f "$file" ]] || fail "missing required file: $file"
}

require_contains_text() {
  local haystack="$1"
  local needle="$2"
  local description="$3"
  if [[ "$haystack" != *"$needle"* ]]; then
    echo "expected: $description" >&2
    echo "needle: $needle" >&2
    fail "missing release body key field"
  fi
}

require_file_contains() {
  local file="$1"
  local needle="$2"
  local description="$3"
  if ! grep -F -- "$needle" "$file" >/dev/null; then
    echo "expected: $description" >&2
    echo "file: $file" >&2
    echo "needle: $needle" >&2
    fail "missing release notes key field"
  fi
}

ensure_origin_main_ref() {
  if git rev-parse -q --verify origin/main^{commit} >/dev/null; then
    return 0
  fi

  if git remote get-url origin >/dev/null 2>&1; then
    git fetch --no-tags --depth=1 origin +refs/heads/main:refs/remotes/origin/main >/dev/null 2>&1 || true
  fi
}

extract_json_field() {
  python3 - "$1" "$2" <<'PY'
import json
import sys

payload = json.loads(sys.argv[1])
value = payload.get(sys.argv[2])
if value is None:
    value = ""
print(value)
PY
}

echo "== GitHub release publication guard =="
echo "current_release_version=$CURRENT_RELEASE_VERSION"
echo "current_release_tag=$CURRENT_RELEASE_TAG"
echo "release_name=$RELEASE_NAME"
echo "release_url=$RELEASE_URL"
echo "release_notes=$CURRENT_RELEASE_NOTES"
echo "prepublish_tag_gate=$PREPUBLISH_TAG_GATE"

require_file "$CURRENT_RELEASE_NOTES"

if ! command -v "$GH_BIN" >/dev/null 2>&1; then
  offline_skip "gh_unavailable"
fi

if ! "$GH_BIN" auth status >/dev/null 2>&1; then
  offline_skip "gh_auth_unavailable"
fi

if ! git rev-parse -q --verify "${CURRENT_RELEASE_TAG}^{commit}" >/dev/null; then
  offline_skip "missing_local_git_tag:$CURRENT_RELEASE_TAG"
fi

ensure_origin_main_ref

if ! git rev-parse -q --verify origin/main^{commit} >/dev/null; then
  fail "missing local origin/main ref"
fi

tag_sha="$(git rev-list -n 1 "$CURRENT_RELEASE_TAG")"
origin_main_sha="$(git rev-parse origin/main)"

if ! git merge-base --is-ancestor "$tag_sha" "$origin_main_sha"; then
  fail "release tag $CURRENT_RELEASE_TAG is not reachable from origin/main"
fi

release_json=""
if [[ "$PREPUBLISH_TAG_GATE" == "1" ]]; then
  release_json="$("$GH_BIN" release view "$CURRENT_RELEASE_TAG" --json tagName,name,isDraft,isPrerelease,url,body,publishedAt,targetCommitish 2>/dev/null || true)"
else
  release_json="$("$GH_BIN" release view "$CURRENT_RELEASE_TAG" --json tagName,name,isDraft,isPrerelease,url,body,publishedAt,targetCommitish 2>/dev/null)" \
    || offline_skip "github_release_unavailable"
fi

tag_name=""
name=""
is_draft=""
is_prerelease=""
url=""
published_at=""
target_commitish=""
body=""

if [[ -n "$release_json" ]]; then
  tag_name="$(extract_json_field "$release_json" tagName)"
  name="$(extract_json_field "$release_json" name)"
  is_draft="$(extract_json_field "$release_json" isDraft)"
  is_prerelease="$(extract_json_field "$release_json" isPrerelease)"
  url="$(extract_json_field "$release_json" url)"
  published_at="$(extract_json_field "$release_json" publishedAt)"
  target_commitish="$(extract_json_field "$release_json" targetCommitish)"
  body="$(extract_json_field "$release_json" body)"

  [[ "$tag_name" == "$CURRENT_RELEASE_TAG" ]] || fail "release tag mismatch: $tag_name"
  [[ "$name" == "$RELEASE_NAME" ]] || fail "release name mismatch: $name"
  [[ "$is_draft" == "False" || "$is_draft" == "false" ]] || fail "release is draft"
  [[ "$is_prerelease" == "False" || "$is_prerelease" == "false" ]] || fail "release is prerelease"
  [[ "$url" == "$RELEASE_URL" ]] || fail "release URL mismatch: $url"
  [[ -n "$published_at" ]] || fail "release publishedAt is empty"
elif [[ "$PREPUBLISH_TAG_GATE" != "1" ]]; then
  fail "GitHub Release is unavailable"
fi

case "$CURRENT_RELEASE_VERSION" in
  v0.14.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Production Order-State Read-Only + Live Alpha Dry-Run"
      "no-production-mutation"
      "owner-gated production order-state GET proof"
      "live-alpha dry-run"
      "production order submission"
      "production cancel, replace, amend, retry, correction, or flatten"
      "listenKey lifecycle"
      "real funds"
      "production trading"
      "Dashboard order/cancel/replace/amend/retry/reconnect controls"
    )
    ;;
  v0.15.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Guarded Live Alpha Mutation Scope + Execution Dry-Run Harness"
      "production mutation endpoint classification"
      "redacted local order request preview"
      "manual approval lifecycle for preview artifact creation"
      "kill switch runtime gate"
      "local dry-run execution adapter artifact"
      "dry-run mutation golden trace replay"
      "read-only Dashboard mutation preflight panel"
      "production request sending"
      "production order submission"
      "production order mutation"
      "production cancel, replace, amend, retry, correction, or flatten"
      "listenKey lifecycle"
      "real funds"
      "production trading"
      "Dashboard order/cancel/replace/amend/retry/reconnect controls"
    )
    ;;
  v0.16.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Minimum Owner-Approved Production Order Mutation Candidate"
      "owner-approved runtime gates"
      "production signing-material approval evidence"
      "single \`LIMIT\` \`GTC\` request builder"
      "guarded production HTTP send path"
      "production mutation response redaction"
      "post-submit order-state readback proof contract"
      "kill-switch checks around the send boundary"
      "production mutation audit trail"
      "failure-mode and no-retry semantics"
      "read-only Dashboard production mutation evidence panel"
      "strategy-driven production execution"
      "multiple orders"
      "MARKET orders"
      "cancel, replace, amend, retry, correction, flatten, or remediation"
      "Dashboard order controls"
      "listenKey lifecycle"
      "real-funds proof in CI"
      "production trading platform claim"
    )
    ;;
  v0.17.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Production Reconciliation And Orphan Recovery Evidence"
      "capability_expansion_from_v16 = reconciliation_evidence_only"
      "lineage_scope = single_v16_mutation_candidate"
      "local ledger"
      "redacted readback mapper"
      "reconciliation classifier"
      "orphan order detector"
      "restart recovery evidence"
      "failure incident semantics"
      "read-only Dashboard evidence"
      "network readback execution = not included"
      "production order submission = not included"
      "production order mutation = not included"
      "actual cancel send = deferred"
      "automatic cancel = disabled"
      "Dashboard order controls = disabled"
      "Dashboard cancel controls = disabled"
      "retry_attempted = false"
      "cancel_attempted = false"
      "remediation_attempted = false"
      "strategy-driven production execution"
      "multi-account production execution"
      "multi-venue production execution"
      "real-funds proof in CI"
      "general production trading platform claim"
    )
    ;;
  v0.18.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Owner-Approved Cancel Recovery Preview"
      "capability_expansion = preview_gate_approval_only"
      "lineage_scope = single_v16_mutation_candidate"
      "cancel request preview"
      "cancel risk gate"
      "manual owner approval lifecycle"
      "cancel response redaction contract"
      "post-cancel readback contract"
      "incident/audit closeout contract"
      "read-only Dashboard evidence"
      "aggregate v0.18 release gate"
      "actual cancel send = not included"
      "automatic cancel = disabled"
      "automatic remediation = disabled"
      "Dashboard order controls = disabled"
      "Dashboard cancel controls = disabled"
      "retry_attempted = false"
      "cancel_attempted = false"
      "remediation_attempted = false"
      "DELETE /api/v3/order"
      "DELETE /api/v3/openOrders"
      "strategy-driven cancel"
      "cancel all open orders"
      "bulk cancel"
      "multi-account cancel recovery"
      "multi-venue cancel recovery"
      "production trading claim"
    )
    ;;
  v0.19.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Owner-Approved Single-Shot Actual Cancel"
      "actual cancel only owner-approved single-shot"
      "owner approval = required"
      "single order = required"
      "single venue = required"
      "single execution attempt = required"
      "post-cancel readback = required"
      "failure evidence = required"
      "Dashboard cancel button = not included"
      "production order submit lifecycle = not included"
      "automatic cancel = not included"
      "bulk cancel = not included"
      "retry / replace / amend / flatten = not included"
    )
    ;;
  v0.20.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Owner-Approved Production Order Lifecycle Foundation"
      "owner approval = required"
      "single submit attempt = required"
      "post-submit readback = required"
      "failure/no-retry evidence = required"
      "production submit lifecycle foundation = included"
      "Dashboard order controls = not included"
      "implicit retry = not included"
      "automatic cancel = not included"
      "automatic remediation = not included"
      "bulk order execution = not included"
      "retry / replace / amend / flatten = not included"
      "strategy-driven production execution = not included"
      "general production trading platform claim = not included"
    )
    ;;
  v0.20.1)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Production Order Lifecycle Release Closeout & Provenance Hardening Patch"
      "This patch does not expand production submit capability"
      "V201-001"
      "V201-007"
      "scripts/ai/verify_release.sh v20.1-release-gates"
      "new production submit capability"
      "implicit retry"
      "automatic cancel"
      "automatic remediation"
      "bulk order execution"
      "retry, replace, amend, correction, or flatten"
      "strategy-driven production execution"
      "multi-account or multi-venue execution"
      "product-grade live trading terminal readiness"
      "Dashboard order, approval, cancel, or retry controls"
    )
    ;;
  v0.21.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Unified Read Model Foundation"
      "This release does not expand production submit capability"
      "V210-000"
      "V210-008"
      "scripts/ai/verify_release.sh v21-release-gates"
      "scripts/ai/verify_release.sh v21-strict-provenance"
      "scripts/ai/verify_release_strict.sh v21"
      "unified_read_model_foundation"
      "read_only_foundation"
      "new production submit capability"
      "production order mutation"
      "implicit retry"
      "automatic cancel"
      "automatic remediation"
      "retry, replace, amend, correction, or flatten"
      "strategy-driven production execution"
      "multi-account or multi-venue execution expansion"
      "product-grade live trading terminal readiness"
      "Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, or order-ticket controls"
    )
    ;;
  v0.21.1)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Unified Read Model Foundation Hardening Patch"
      "This patch is not the Trader Terminal workbench"
      "This patch does not add submit capability"
      "V211-001"
      "V211-006"
      "scripts/ai/verify_release.sh v21.1-release-gates"
      "scripts/ai/verify_release.sh v21.1-strict-provenance"
      "scripts/ai/verify_v21_1_strict_provenance.sh"
      "v0.22.0"
      "new production submit capability"
      "production order mutation"
      "implicit retry"
      "automatic cancel"
      "automatic remediation"
      "retry, replace, amend, correction, or flatten"
      "strategy-driven production execution"
      "multi-account or multi-venue execution expansion"
      "Trader Terminal workbench"
      "product-grade live trading terminal readiness"
      "Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, or order-ticket controls"
    )
    ;;
  v0.22.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Trader Terminal Workbench"
      "This release is read-only first"
      "This release is not a product-grade live trading terminal"
      "This release does not add submit capability"
      "V220-000"
      "V220-007"
      "scripts/ai/verify_release.sh v22-runtime-boundary-tests"
      "scripts/ai/verify_release.sh v22-release-gates"
      "scripts/ai/verify_release.sh v22-strict-provenance"
      "scripts/ai/verify_release_strict.sh v22"
      "read_only_first"
      "gated_operation_boundary"
      "owner approval"
      "risk gate"
      "audit gate"
      "product-grade live trading terminal readiness"
      "new production submit capability"
      "production order mutation"
      "ungated submit/cancel/retry/replace/amend/flatten"
      "automatic cancel"
      "automatic remediation"
      "retry, replace, amend, correction, or flatten"
      "strategy-driven production execution"
      "multi-account production execution expansion"
      "multi-venue production execution expansion"
      "Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, or order-ticket controls"
      "manual operation entry that can mutate live state without owner approval, risk gate, and audit gate"
    )
    ;;
  v0.22.1)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Trader Terminal Workbench hardening patch"
      "This release is read-only first"
      "This release is not a product-grade live trading terminal"
      "This release does not add submit capability"
      "V221-001"
      "V221-006"
      "required_false_runtime_boundary"
      "read_model executable_replay rows = 28"
      "workbench render smoke = required"
      "gate_before_publish = required"
      "scripts/ai/verify_release.sh v22.1-release-gates"
      "scripts/ai/verify_release.sh v22.1-strict-provenance"
      "scripts/ai/verify_v22_1_strict_provenance.sh"
      "product-grade live trading terminal readiness"
      "complete executable read-model runtime coverage"
      "new production submit capability"
      "production order mutation"
      "manual operation entry that can mutate live state"
      "automatic cancel"
      "automatic cancel, repair, alert, audit, provenance, risk, or operation action"
      "strategy-driven production execution"
      "multi-account production execution expansion"
      "multi-strategy production execution expansion"
      "multi-venue production execution expansion"
      "Dashboard order, approval, cancel, retry, submit, replace, amend, flatten"
      "v0.23.0"
    )
    ;;
  v0.23.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Multi-Account / Multi-Strategy / Multi-Venue Node Isolation"
      "This release does not add submit capability"
      "This release is not a product-grade live trading terminal"
      "V230-000"
      "V230-007"
      "multi_account_isolation = true"
      "multi_strategy_isolation = true"
      "multi_venue_node_isolation = true"
      "read_only_dashboard_observability = true"
      "gate_before_publish = required"
      "strict provenance = required"
      "scripts/ai/verify_release.sh v23-release-gates"
      "scripts/ai/verify_release.sh v23-strict-provenance"
      "scripts/ai/verify_v23_strict_provenance.sh"
      "product-grade live trading terminal"
      "new production submit capability"
      "production order mutation"
      "strategy-driven production execution"
      "automatic cancel"
      "automatic remediation"
      "cross-account implicit operation"
      "cross-strategy implicit operation"
      "cross-venue implicit operation"
      "shared approval consumption"
      "Dashboard operation controls"
      "complete executable read-model runtime coverage"
      "v0.24.0"
    )
    ;;
  v0.23.1)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "v0.23.1 is a patch closeout release"
      "This release is a patch closeout"
      "This release does not add submit capability"
      "This release is not a product-grade live trading terminal"
      "V231-001"
      "V231-006"
      "patch_closeout_only = true"
      "v0.23.0 closeout facts = required"
      "candidate / pending / in-progress marker cleanup = required"
      "publication evidence strategy = source_tree_plus_github_remote"
      "v0.24.0 start gate = blocked until v0.23.1 release evidence is published"
      "new_submit_capability = false"
      "production_order_mutation_allowed = false"
      "dashboard_operation_controls_enabled = false"
      "scripts/ai/verify_release.sh v23.1-release-gates"
      "scripts/ai/verify_release.sh v23.1-strict-provenance"
      "scripts/ai/verify_v23_1_release_gates.sh"
      "scripts/ai/verify_v23_1_strict_provenance.sh"
      "scripts/ai/publish_ntpro_release_after_gate.sh"
      "v0.24.0 execution algorithms implementation"
      "product-grade live trading terminal"
      "complete executable read-model runtime coverage"
      "new production submit capability"
      "production order mutation"
      "ungated submit, cancel, retry, replace, amend, or flatten"
      "manual operation entry that can mutate live state"
      "automatic cancel, retry, remediation, repair, alert, audit, provenance, risk"
      "strategy-driven production execution"
      "shared approval consumption"
      "Dashboard order, approval, cancel, retry, submit, replace, amend, flatten"
      "v0.24.0"
    )
    ;;
  v0.24.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Execution Algorithms And Order Control Foundation"
      "This release does not add submit capability"
      "This release is not a product-grade live trading terminal"
      "V240-000"
      "V240-009"
      "order_control_foundation_preview_only = true"
      "preview_evidence_only = true"
      "v24 release gates = required"
      "v24 strict provenance = required"
      "release surface current guard = required"
      "release publication guard = required"
      "release publish after gate = required"
      "publication evidence strategy = source_tree_plus_github_remote"
      "v0.25.0 start gate = blocked until v0.24.0 release evidence is published"
      "new_submit_capability = false"
      "production_order_mutation_allowed = false"
      "execution_adapter_call_allowed = false"
      "live_exchange_request_allowed = false"
      "implicit_retry_allowed = false"
      "retry_scheduler_enabled = false"
      "dashboard_operation_controls_enabled = false"
      "trader_terminal_order_ticket_enabled = false"
      "product_grade_trading_terminal_claim = false"
      "scripts/ai/verify_release.sh v24-release-gates"
      "scripts/ai/verify_release.sh v24-strict-provenance"
      "scripts/ai/verify_v24_release_gates.sh"
      "scripts/ai/verify_v24_strict_provenance.sh"
      "scripts/ai/publish_ntpro_release_after_gate.sh"
      "product-grade live trading terminal"
      "complete executable order-control runtime"
      "real submit/cancel/replace/amend/flatten"
      "execution adapter or exchange network send"
      "retry scheduler or implicit retry"
      "strategy-driven production execution"
      "shared approval consumption"
      "Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, or order-ticket controls"
      "v0.25.0"
    )
    ;;
  v0.24.1)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "v0.24.1 is a hardening patch"
      "This release does not add submit capability"
      "This release is not a product-grade live trading terminal"
      "V241-001"
      "V241-007"
      "patch_hardening_only = true"
      "v24.1 release gates = required"
      "v24.1 strict provenance = required"
      "release surface current guard = required"
      "release publication guard = required"
      "release publish after gate = required"
      "publication evidence strategy = source_tree_plus_github_remote"
      "local generated publication evidence required in source tree = false"
      "remote reconstruction required = true"
      "v0.25.0 start gate = blocked until v0.24.1 release evidence is published"
      "new_submit_capability = false"
      "production_order_mutation_allowed = false"
      "execution_adapter_call_allowed = false"
      "live_exchange_request_allowed = false"
      "implicit_retry_allowed = false"
      "retry_scheduler_enabled = false"
      "dashboard_operation_controls_enabled = false"
      "trader_terminal_order_ticket_enabled = false"
      "product_grade_trading_terminal_claim = false"
      "scripts/ai/verify_release.sh v24.1-release-gates"
      "scripts/ai/verify_release.sh v24.1-strict-provenance"
      "scripts/ai/verify_v24_1_release_gates.sh"
      "scripts/ai/verify_v24_1_strict_provenance.sh"
      "scripts/ai/publish_ntpro_release_after_gate.sh"
      "product-grade live trading terminal"
      "complete executable order-control runtime"
      "real submit/cancel/replace/amend/flatten"
      "execution adapter or exchange network send"
      "retry scheduler or implicit retry"
      "strategy-driven production execution"
      "shared approval consumption"
      "Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, or order-ticket controls"
      "v0.25.0"
    )
    ;;
  v0.25.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "v0.25.0 publishes the Monitoring, Incident, and Disaster-Recovery Foundation"
      "This release does not add submit capability"
      "This release is not a product-grade live trading terminal"
      "V250-000"
      "V250-008"
      "monitoring_incident_dr_foundation = true"
      "v25 release gates = required"
      "v25 strict provenance = required"
      "release surface current guard = required"
      "release publication guard = required"
      "release publish after gate = required"
      "publication evidence strategy = source_tree_plus_github_remote"
      "local generated publication evidence required in source tree = false"
      "remote reconstruction required = true"
      "new_submit_capability = false"
      "production_order_mutation_allowed = false"
      "execution_adapter_call_allowed = false"
      "adapter_send_allowed = false"
      "live_exchange_request_allowed = false"
      "implicit_retry_allowed = false"
      "retry_scheduler_enabled = false"
      "automatic_remediation_allowed = false"
      "dashboard_operation_controls_enabled = false"
      "dashboard_trading_controls_enabled = false"
      "trader_terminal_order_ticket_enabled = false"
      "product_grade_trading_terminal_claim = false"
      "scripts/ai/verify_release.sh v25-release-gates"
      "scripts/ai/verify_release.sh v25-strict-provenance"
      "scripts/ai/verify_v25_release_gates.sh"
      "scripts/ai/verify_v25_strict_provenance.sh"
      "scripts/ai/publish_ntpro_release_after_gate.sh"
      "product-grade live trading terminal"
      "real submit/cancel/replace/amend/flatten"
      "production order mutation"
      "execution adapter or exchange network send"
      "retry scheduler or implicit retry"
      "automatic remediation"
      "strategy-driven production execution"
      "Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, remediation, or order-ticket controls"
      "v0.26.0"
    )
    ;;
  v0.25.1)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "v0.25.1 is a patch governance and evidence hardening release"
      "V251-001"
      "V251-006"
      "v25.1 release gates = required"
      "v25.1 strict provenance = required"
      "release surface current guard = required"
      "release publication guard = required"
      "release publish after gate = required"
      "publication evidence strategy = source_tree_plus_github_remote"
      "local generated publication evidence required in source tree = false"
      "remote reconstruction required = true"
      "new_submit_capability = false"
      "production_order_submission_allowed = false"
      "production_order_mutation_allowed = false"
      "execution_adapter_call_allowed = false"
      "adapter_send_allowed = false"
      "live_exchange_request_allowed = false"
      "retry_scheduler_enabled = false"
      "automatic_remediation_allowed = false"
      "dashboard_operation_controls_enabled = false"
      "dashboard_trading_controls_enabled = false"
      "trader_terminal_order_ticket_enabled = false"
      "manual_operation_submit_allowed = false"
      "product_grade_trading_terminal_claim = false"
      "scripts/ai/verify_release.sh v25.1-release-gates"
      "scripts/ai/verify_release.sh v25.1-strict-provenance"
      "scripts/ai/verify_v25_1_release_gates.sh"
      "scripts/ai/verify_v25_1_strict_provenance.sh"
      "scripts/ai/publish_ntpro_release_after_gate.sh"
      "v0.26.0"
    )
    ;;
  v0.26.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "v0.26.0 publishes the Product Hardening Foundation"
      "This release does not add submit capability"
      "This release is not a product-grade live trading terminal"
      "V260-000"
      "V260-008"
      "product_hardening_foundation = true"
      "v26 release gates = required"
      "v26 strict provenance = required"
      "release surface current guard = required"
      "release publication guard = required"
      "release publish after gate = required"
      "hosted release gate success before public GitHub Release = required"
      "publication evidence strategy = source_tree_plus_github_remote"
      "local generated publication evidence required in source tree = false"
      "remote reconstruction required = true"
      "new_submit_capability = false"
      "production_order_submission_allowed = false"
      "production_order_mutation_allowed = false"
      "execution_adapter_call_allowed = false"
      "adapter_send_allowed = false"
      "live_exchange_request_allowed = false"
      "implicit_retry_allowed = false"
      "retry_scheduler_enabled = false"
      "automatic_remediation_allowed = false"
      "dashboard_operation_controls_enabled = false"
      "dashboard_trading_controls_enabled = false"
      "trader_terminal_order_ticket_enabled = false"
      "manual_operation_submit_allowed = false"
      "product_grade_trading_terminal_claim = false"
      "scripts/ai/verify_release.sh v26-release-gates"
      "scripts/ai/verify_release.sh v26-strict-provenance"
      "scripts/ai/verify_v26_release_gates.sh"
      "scripts/ai/verify_v26_strict_provenance.sh"
      "scripts/ai/publish_ntpro_release_after_gate.sh"
      "product-grade live trading terminal"
      "real submit/cancel/replace/amend/flatten"
      "production order mutation"
      "execution adapter or exchange network send"
      "retry scheduler or implicit retry"
      "automatic remediation"
      "Dashboard order, approval, cancel, retry, submit, replace, amend, flatten, remediation, or order-ticket controls"
      "v0.27.0"
    )
    ;;
  v0.26.1)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "v0.26.1 is a patch governance and evidence hardening release"
      "V261-001"
      "V261-006"
      "v26.1 release gates = required"
      "v26.1 strict provenance = required"
      "release surface current guard = required"
      "release publication guard = required"
      "release publish after gate = required"
      "v27 intake gate = hard-blocked until v0.26.1 publication evidence exists"
      "publication evidence strategy = source_tree_plus_github_remote"
      "local generated publication evidence required in source tree = false"
      "remote reconstruction required = true"
      "new_submit_capability = false"
      "production_order_submission_allowed = false"
      "production_order_mutation_allowed = false"
      "execution_adapter_call_allowed = false"
      "adapter_send_allowed = false"
      "live_exchange_request_allowed = false"
      "retry_scheduler_enabled = false"
      "automatic_remediation_allowed = false"
      "dashboard_operation_controls_enabled = false"
      "dashboard_trading_controls_enabled = false"
      "trader_terminal_order_ticket_enabled = false"
      "manual_operation_submit_allowed = false"
      "product_grade_trading_terminal_claim = false"
      "scripts/ai/verify_release.sh v26.1-release-gates"
      "scripts/ai/verify_release.sh v26.1-strict-provenance"
      "scripts/ai/verify_v26_1_release_gates.sh"
      "scripts/ai/verify_v26_1_strict_provenance.sh"
      "scripts/ai/verify_v27_intake_gate.sh"
      "scripts/ai/publish_ntpro_release_after_gate.sh"
      "v0.27.0 start gate = blocked until v0.26.1 release gate passes"
    )
    ;;
  v0.27.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Base release: \`ntpro-rust-only-v0.26.1\`"
      "v0.27.0 publishes the Product Operations Runtime Integration Foundation"
      "V270-000"
      "V270-010"
      "V270 final release scope issue count = 11"
      "V270 final release scope evidence count = 11"
      "v27 release gates = required"
      "v27 strict provenance = required"
      "release surface current guard = required"
      "release publication guard = required"
      "release publish after gate = required"
      "hosted release gate success before public GitHub Release = required"
      "publication evidence strategy = source_tree_plus_github_remote"
      "local generated publication evidence required in source tree = false"
      "remote reconstruction required = true"
      "new_submit_capability = false"
      "production_order_submission_allowed = false"
      "production_order_mutation_allowed = false"
      "execution_adapter_call_allowed = false"
      "adapter_send_allowed = false"
      "live_exchange_request_allowed = false"
      "retry_scheduler_enabled = false"
      "automatic_remediation_allowed = false"
      "dashboard_operation_controls_enabled = false"
      "dashboard_trading_controls_enabled = false"
      "admin_workbench_operation_controls_enabled = false"
      "admin_workbench_trading_controls_enabled = false"
      "trader_terminal_order_ticket_enabled = false"
      "manual_operation_submit_allowed = false"
      "product_grade_trading_terminal_claim = false"
      "scripts/ai/verify_release.sh v27-release-gates"
      "scripts/ai/verify_release.sh v27-strict-provenance"
      "scripts/ai/verify_v27_release_gates.sh"
      "scripts/ai/verify_v27_strict_provenance.sh"
      "scripts/ai/check_github_release_published.sh"
      "scripts/ai/golden_trace_runner.py"
      "scripts/ai/publish_ntpro_release_after_gate.sh"
      "v0.28.0"
    )
    ;;
  v0.27.1)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Base release: \`ntpro-rust-only-v0.27.0\`"
      "v0.27.1 is a patch governance and evidence hardening release"
      "V271-001"
      "V271-006"
      "v27.1 release gates = required"
      "v27.1 strict provenance = required"
      "release surface current guard = required"
      "release publication guard = required"
      "release publish after gate = required"
      "v28 intake gate = hard-blocked until v0.27.1 publication evidence exists"
      "publication evidence strategy = source_tree_plus_github_remote"
      "local generated publication evidence required in source tree = false"
      "remote reconstruction required = true"
      "new_submit_capability = false"
      "production_order_submission_allowed = false"
      "production_order_mutation_allowed = false"
      "execution_adapter_call_allowed = false"
      "adapter_send_allowed = false"
      "live_exchange_request_allowed = false"
      "retry_scheduler_enabled = false"
      "automatic_remediation_allowed = false"
      "dashboard_operation_controls_enabled = false"
      "dashboard_trading_controls_enabled = false"
      "admin_workbench_operation_controls_enabled = false"
      "admin_workbench_trading_controls_enabled = false"
      "trader_terminal_order_ticket_enabled = false"
      "manual_operation_submit_allowed = false"
      "product_grade_trading_terminal_claim = false"
      "scripts/ai/verify_release.sh v27.1-release-gates"
      "scripts/ai/verify_release.sh v27.1-strict-provenance"
      "scripts/ai/verify_v27_1_release_gates.sh"
      "scripts/ai/verify_v27_1_strict_provenance.sh"
      "scripts/ai/publish_ntpro_release_after_gate.sh"
      "v0.28.0 start gate = blocked until v0.27.1 release gate passes"
    )
    ;;
  v0.28.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Base release: \`ntpro-rust-only-v0.27.1\`"
      "v0.28.0 publishes the Backend Closure / Product Operations Runtime Finalization track"
      "V280-000"
      "V280-009"
      "V280 final release scope issue count = 10"
      "V280 final release scope evidence count = 10"
      "V280 exact milestone issue set = #893-#902"
      "V280 registered corrective-scope exception count = 0"
      "v28 release gates = required"
      "v28 strict provenance = required"
      "backend closure boundary contract = required"
      "release surface current guard = required"
      "release publication guard = required"
      "release publish after gate = required"
      "hosted release gate success before public GitHub Release = required"
      "publication evidence strategy = source_tree_plus_github_remote"
      "local generated publication evidence required in source tree = false"
      "remote reconstruction required = true"
      "new_submit_capability = false"
      "production_order_submission_allowed = false"
      "production_order_mutation_allowed = false"
      "execution_adapter_call_allowed = false"
      "adapter_send_allowed = false"
      "live_exchange_request_allowed = false"
      "network_attempted = false"
      "retry_scheduler_enabled = false"
      "automatic_remediation_allowed = false"
      "automatic_operation_action_allowed = false"
      "dashboard_operation_controls_enabled = false"
      "dashboard_trading_controls_enabled = false"
      "admin_workbench_operation_controls_enabled = false"
      "admin_workbench_trading_controls_enabled = false"
      "trader_terminal_order_ticket_enabled = false"
      "manual_operation_submit_allowed = false"
      "product_grade_trading_terminal_claim = false"
      "scripts/ai/verify_release.sh v28-release-gates"
      "scripts/ai/verify_release.sh v28-strict-provenance"
      "scripts/ai/verify_v28_release_gates.sh"
      "scripts/ai/verify_v28_strict_provenance.sh"
      "scripts/ai/check_github_release_published.sh"
      "scripts/ai/publish_ntpro_release_after_gate.sh"
      "The next patch track is \`v0.28.1\`"
      "The next capability track is \`v0.29.0\`"
    )
    ;;
  v0.28.1)
    required_fields=(
      "Status: RELEASE GATE READY"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Base release: \`ntpro-rust-only-v0.28.0\`"
      "v0.28.1 is a patch governance and provenance hardening release"
      "V281-001"
      "V281-007"
      "V281-008"
      "V281-009"
      "V281-010"
      "V281 final release scope issue count = 10"
      "V281 final release scope evidence count = 10"
      "V281 exact milestone issue set = #919-#925, #944, #946, #948"
      "V281 registered corrective-scope exception count = 3"
      "v28.1 release gates = required"
      "v28.1 strict provenance = required"
      "v29 intake gate = hard-blocked until v0.28.1 publication evidence exists"
      "v28 release gates = required"
      "v28 strict provenance = required"
      "release body hash normalization = required"
      "runtime-closed terminology hardening = required"
      "release publish after gate current-release binding = required"
      "release surface current guard = required"
      "release publication guard = required"
      "release publish after gate = required"
      "post-publication closeout evidence path = docs/rust-cutover/release/v0_28_1_release_closeout_evidence.md"
      "publication evidence strategy = source_tree_plus_github_remote"
      "local generated publication evidence required in source tree = false"
      "remote reconstruction required = true"
      "new_submit_capability = false"
      "production_order_submission_allowed = false"
      "production_order_mutation_allowed = false"
      "execution_adapter_call_allowed = false"
      "adapter_send_allowed = false"
      "live_exchange_request_allowed = false"
      "network_attempted = false"
      "retry_scheduler_enabled = false"
      "automatic_remediation_allowed = false"
      "automatic_operation_action_allowed = false"
      "dashboard_operation_controls_enabled = false"
      "dashboard_trading_controls_enabled = false"
      "admin_workbench_operation_controls_enabled = false"
      "admin_workbench_trading_controls_enabled = false"
      "trader_terminal_order_ticket_enabled = false"
      "manual_operation_submit_allowed = false"
      "product_grade_trading_terminal_claim = false"
      "scripts/ai/verify_release.sh v28.1-release-gates"
      "scripts/ai/verify_release.sh v28.1-strict-provenance"
      "scripts/ai/verify_release.sh v29-intake-gate"
      "scripts/ai/verify_v28_1_release_gates.sh"
      "scripts/ai/verify_v28_1_strict_provenance.sh"
      "scripts/ai/verify_v29_intake_gate.sh"
      "scripts/ai/publish_ntpro_release_after_gate.sh"
      "v0.29.0 start gate = blocked until v0.28.1 release gate passes"
    )
    ;;
  v0.29.0)
    required_fields=(
      "Status: RELEASED"
      "Tag: \`$CURRENT_RELEASE_TAG\`"
      "Release name: \`$RELEASE_NAME\`"
      "Release URL: \`$RELEASE_URL\`"
      "Base release: \`ntpro-rust-only-v0.28.1\`"
      "v0.29.0 publishes the Backend Production Readiness Foundation track"
      "V290-000"
      "V290-010"
      "V290-011"
      "V290 final release scope issue count = 12"
      "V290 final release scope evidence count = 12"
      "V290 exact milestone issue set = #926-#936, #961"
      "V290 registered corrective-scope exception count = 1"
      "v29 release gates = required"
      "v29 strict provenance = required"
      "backend production readiness boundary contract = required"
      "backend production readiness fail-closed hardening = required"
      "release surface current guard = required"
      "release publication guard = required"
      "release publish after gate = required"
      "post-publication closeout gate = required"
      "hosted release gate success before public GitHub Release = required"
      "publication evidence strategy = source_tree_plus_github_remote"
      "local generated publication evidence required in source tree = false"
      "remote reconstruction required = true"
      "generated publication evidence sole proof allowed = false"
      "published release closeout evidence = docs/rust-cutover/release/v0_29_0_release_closeout_evidence.md"
      "published release status = published_after_gate"
      "hosted release gate run = 29091765148"
      "release body hash semantics = normalized_sha256"
      "v0.30.0 backend production go-live candidate = next track"
      "new_submit_capability = false"
      "production_order_submission_allowed = false"
      "production_order_mutation_allowed = false"
      "cancel_order_allowed = false"
      "replace_order_allowed = false"
      "amend_order_allowed = false"
      "flatten_position_allowed = false"
      "execution_adapter_call_allowed = false"
      "adapter_send_allowed = false"
      "live_exchange_request_allowed = false"
      "network_attempted = false"
      "retry_scheduler_enabled = false"
      "automatic_remediation_allowed = false"
      "automatic_operation_action_allowed = false"
      "dashboard_operation_controls_enabled = false"
      "dashboard_trading_controls_enabled = false"
      "admin_workbench_operation_controls_enabled = false"
      "admin_workbench_trading_controls_enabled = false"
      "trader_terminal_order_ticket_enabled = false"
      "manual_operation_submit_allowed = false"
      "backend_go_live_claim = false"
      "product_grade_trading_terminal_claim = false"
      "scripts/ai/verify_release.sh v29-release-gates"
      "scripts/ai/verify_release.sh v29-strict-provenance"
      "scripts/ai/verify_v29_release_gates.sh"
      "scripts/ai/verify_v29_strict_provenance.sh"
      "scripts/ai/verify_v29_1_post_publication_closeout_gate.sh"
      "scripts/ai/check_github_release_published.sh"
      "scripts/ai/publish_ntpro_release_after_gate.sh"
      "The next patch track is \`v0.29.1\`"
      "The next capability track is \`v0.30.0\`"
    )
    ;;
  *)
    fail "unsupported release publication guard version: $CURRENT_RELEASE_VERSION"
    ;;
esac

for field in "${required_fields[@]}"; do
  require_file_contains "$CURRENT_RELEASE_NOTES" "$field" "release notes key field"
done

if [[ "$PREPUBLISH_TAG_GATE" == "1" ]]; then
  echo "release_publication_guard=prepublish_tag_gate"
  echo "publication_evidence_strategy=source_tree_plus_github_remote"
  echo "local_evidence_path_is_generated_artifact=true"
  echo "local_evidence_path_required_in_source_tree=false"
  echo "remote_reconstruction_required=true"
  echo "tag_sha=$tag_sha"
  echo "origin_main_sha=$origin_main_sha"
  echo "existing_release_seen=$([[ -n "$release_json" ]] && echo true || echo false)"
  exit 0
fi

for field in "${required_fields[@]}"; do
  require_contains_text "$body" "$field" "GitHub Release body key field"
done

body_hash_report="$(python3 - "$release_json" "$CURRENT_RELEASE_NOTES" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

release = json.loads(sys.argv[1])
notes = Path(sys.argv[2]).read_text(encoding="utf-8")
body = release.get("body") or ""


def normalize(value: str) -> str:
    return "\n".join(line.rstrip() for line in value.splitlines()).strip()


def sha256(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


normalized_body = normalize(body)
normalized_notes = normalize(notes)
raw_match = body == notes
normalized_match = normalized_body == normalized_notes


def flag(value: bool) -> str:
    return "true" if value else "false"


print("release_body_hash_semantics=normalized_sha256")
print("release_body_normalization=line_rstrip_and_outer_strip")
print(f"release_body_normalized_sha256={sha256(normalized_body)}")
print(f"tracked_release_notes_normalized_sha256={sha256(normalized_notes)}")
print(f"release_body_normalized_sha256_matches_tracked_release_notes={flag(normalized_match)}")
print(f"release_body_raw_sha256={sha256(body)}")
print(f"tracked_release_notes_raw_sha256={sha256(notes)}")
print(f"release_body_raw_sha256_matches_tracked_release_notes={flag(raw_match)}")
print("release_body_raw_sha256_is_acceptance_rule=false")
PY
)"

if [[ "${NTPRO_RELEASE_PUBLICATION_STRICT_BODY:-0}" == "1" ]]; then
  if ! grep -F "release_body_normalized_sha256_matches_tracked_release_notes=true" <<<"$body_hash_report" >/dev/null; then
    fail "release body does not match release notes under normalized_sha256 semantics"
  fi
fi

echo "release_publication_guard=pass"
printf '%s\n' "$body_hash_report"
echo "publication_evidence_strategy=source_tree_plus_github_remote"
echo "local_evidence_path_is_generated_artifact=true"
echo "local_evidence_path_required_in_source_tree=false"
echo "remote_reconstruction_required=true"
echo "tag_sha=$tag_sha"
echo "origin_main_sha=$origin_main_sha"
echo "target_commitish=$target_commitish"
echo "published_at=$published_at"
