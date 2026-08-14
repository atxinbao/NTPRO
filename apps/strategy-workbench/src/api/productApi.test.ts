import { describe, expect, it, vi } from "vitest";

import errorFixture from "../test/product-api-fixtures/error.json";
import liveAccountRefreshConnectedFixture from "../test/product-api-fixtures/live-account-refresh-connected.json";
import liveAccountRefreshFixture from "../test/product-api-fixtures/live-account-refresh.json";
import liveAdmissionFixture from "../test/product-api-fixtures/live-admission.json";
import liveRunCandidateFixture from "../test/product-api-fixtures/live-run-candidate.json";
import runAnalysisFixture from "../test/product-api-fixtures/run-analysis.json";
import runDetailFixture from "../test/product-api-fixtures/run-detail.json";
import runListFixture from "../test/product-api-fixtures/run-list.json";
import runMetricsFixture from "../test/product-api-fixtures/run-metrics.json";
import runReportFixture from "../test/product-api-fixtures/run-report.json";
import strategyDetailFixture from "../test/product-api-fixtures/strategy-detail.json";
import strategyListFixture from "../test/product-api-fixtures/strategy-list.json";
import strategyVersionDetailFixture from "../test/product-api-fixtures/strategy-version-detail.json";
import strategyVersionListFixture from "../test/product-api-fixtures/strategy-version-list.json";
import {
  backtestComparisonResponse,
  backtestReproductionProofResponse,
  backtestReproductionResponse,
  createdDemo,
  createdDemoResponse,
  demoActionResponse,
  demoSnapshotResponse,
} from "../test/server";
import {
  createProductApiClient,
  ProductApiContractError,
  ProductApiTransportError,
} from "./productApi";

const createBacktestBody = {
  strategy_id: "ema-cross",
  strategy_version_id: "ema-cross@v1",
  environment: "backtest" as const,
  data_ref: "dataset://fixtures/ema-cross",
  venue_ref: "venue://simulated/BINANCE",
  starting_balance: "100000 USDT",
  quotes: 120,
  trade_size: "0.001000",
  fast_period: 3,
  slow_period: 5,
};

const createDemoBody = {
  strategy_id: "ema-cross",
  strategy_version_id: "ema-cross@v1",
  environment: "sandbox" as const,
  supervisor_node_id: "mvp-node-001",
  account_ref: "account://sandbox/acct-sandbox-001",
  venue_ref: "venue://sandbox/BINANCE",
  user_confirmed: true as const,
};

const createLiveRunCandidateBody = {
  strategy_id: "ema-cross",
  strategy_version_id: "ema-cross@v1",
  environment: "live" as const,
  account_ref: "account://live/binance/primary" as const,
  venue_ref: "venue://live/BINANCE" as const,
  user_confirmed: true as const,
};

function createBacktestResponse() {
  const baseline = structuredClone(
    runListFixture.data.find((run) => run.environment === "backtest")!,
  );
  return {
    schema_version: "ntpro.product_api.run_create.response.v1",
    contract_version: "ntpro.product_api.v1",
    request_id: "product-0000000000000001-0000000000000001",
    data: {
      ...baseline,
      run_id: "backtest-created-001",
      config_ref: "artifact://backtests/backtest-created-001/request.toml",
      account_ref: "account://simulated/backtest-created-001",
      result: {
        status: "available",
        result_ref: "artifact://backtests/backtest-created-001/summary.json",
        report_ref: "artifact://backtests/backtest-created-001/details.json",
        analysis_ref: "artifact://backtests/backtest-created-001/analysis.json",
        reproduction_ref: null,
      },
    },
    boundaries: {
      backtest_run_creation_allowed: true,
      sandbox_run_creation_allowed: false,
      live_run_creation_allowed: false,
      external_venue_connection: false,
      order_submission_allowed: false,
      order_mutation_allowed: false,
      automatic_retry_allowed: false,
      automatic_remediation_allowed: false,
      real_orders_submitted: false,
      trading_controls_enabled: false,
    },
  };
}

function jsonFetch(payload: unknown, status = 200) {
  return vi.fn<typeof fetch>(async () =>
    Promise.resolve(
      new Response(JSON.stringify(payload), {
        status,
        headers: { "Content-Type": "application/json" },
      }),
    ),
  );
}

function liveExecutionControlResponse(): Record<string, any> {
  const running: Record<string, any> = structuredClone(liveRunCandidateFixture);
  running.schema_version =
    "ntpro.product_api.live_run_candidate_action.response.v1";
  running.data.lifecycle = "market_data_running";
  running.data.preflight_at_unix_ms = 1786406401000;
  running.data.account_connected = true;
  running.data.account_can_trade_verified = true;
  running.data.runtime_started = true;
  running.data.market_data_connected = true;
  running.data.runtime_node_id = running.data.run_id;
  running.data.runtime_process_state = "running";
  running.data.order_admission = {
    status: "consumed_single_shot",
    submit: "blocked",
    cancel: "dual_approval_required",
    replace: "blocked",
    fill_reconciliation: "explicit_manual_available",
    owner_approved: true,
    risk_approved: true,
    operator_approved: true,
    blockers: ["single_shot_admission_consumed", "additional_orders_blocked"],
  };
  running.data.strategy_intent = strategyIntent(running.data);
  running.data.strategy_intent_sha256 = `sha256:${"9".repeat(64)}`;
  attachSizingDecision(running.data);
  running.data.execution_order = {
    schema_version: "ntpro.s3.live_execution_order_state.v4",
    admission_id: "manual-001",
    source_demo_run_id: "demo-source-001",
    strategy_intent_id: "intent-001",
    strategy_intent_sha256: `sha256:${"9".repeat(64)}`,
    sizing_decision_sha256: `sha256:${"7".repeat(64)}`,
    strategy_version_id: running.data.strategy_version_id,
    instrument_id: "BTCUSDT.BINANCE",
    client_order_id: "S3LV008-001",
    venue_order_id: "1001",
    original_quantity: "0.00001000",
    filled_quantity: "0.00000400",
    remaining_quantity: "0.00000600",
    status: "canceled",
    terminal: true,
    new_orders_blocked: true,
    actual_submission_attempted: true,
    automatic_retry_attempted: false,
    cancel_attempted: true,
    replace_attempted: false,
    last_error: null,
    updated_at_unix_ms: 1786406404000,
  };
  running.data.execution_order_state_sha256 = `sha256:${"a".repeat(64)}`;
  running.data.execution_control = {
    schema_version: "ntpro.s3.live_execution_control_result.v1",
    request_sha256: `sha256:${"b".repeat(64)}`,
    request_id: "cancel-control-001",
    action: "cancel",
    run_id: running.data.run_id,
    admission_id: "manual-001",
    strategy_version_id: running.data.strategy_version_id,
    instrument_id: "BTCUSDT.BINANCE",
    client_order_id: "S3LV008-001",
    venue_order_id: "1001",
    status: "cancel_confirmed",
    exchange_order_status: "canceled",
    original_quantity: "0.00001000",
    filled_quantity: "0.00000400",
    remaining_quantity: "0.00000600",
    query_attempted: true,
    cancel_attempted: true,
    cancel_confirmed: true,
    automatic_retry_attempted: false,
    manual_review_required: false,
    error_code: null,
    completed_at_unix_ms: 1786406403500,
  };
  running.data.audit_anchor.revision = 4;
  running.data.audit_anchor.workspace_revision = 4;
  running.data.audit_anchor.receipt_ref = `sha256:${"8".repeat(64)}`;
  running.data.audit_anchor.anchored_at_unix_ms = 1786406402000;
  running.boundaries.execution_adapter_send_attempted = true;
  running.boundaries.real_orders_submitted = true;
  return running;
}

function strategyIntent(candidate: Record<string, any>) {
  return {
    schema_version: "ntpro.s3.live_strategy_order_intent.v1",
    source_demo_run_id: "demo-source-001",
    strategy_id: candidate.strategy_id,
    strategy_version_id: candidate.strategy_version_id,
    intent_id: "intent-001",
    instrument_id: "BTCUSDT.BINANCE",
    side: "BUY",
    source_order_type: "market",
    quantity: "0.00001000",
    source_signal: "long",
    confidence: "0.72",
    market_event_seq: 1,
    created_at_unix_ms: 1786406300000,
    source_manifest_sha256: `sha256:${"5".repeat(64)}`,
    source_result_sha256: `sha256:${"6".repeat(64)}`,
  };
}

function attachSizingDecision(candidate: Record<string, any>) {
  candidate.sizing_decision = {
    schema_version: "ntpro.s3.live_sizing_decision.v1",
    run_id: candidate.run_id,
    source_manifest_sha256: `sha256:${"4".repeat(64)}`,
    source_preflight_sha256: `sha256:${"5".repeat(64)}`,
    strategy_intent_sha256: candidate.strategy_intent_sha256,
    instrument_id: candidate.strategy_intent.instrument_id,
    side: candidate.strategy_intent.side,
    price: "1.00",
    price_tick: "0.01",
    source_quantity: candidate.strategy_intent.quantity,
    approved_quantity: candidate.strategy_intent.quantity,
    quantity_step: "0.00001000",
    min_quantity: "0.00001000",
    max_quantity: "9000.00000000",
    min_notional: "0.000001",
    max_account_budget_fraction: "0.10",
    order_notional: "0.00001",
    account_budget_notional: "1.00",
    request_max_notional: "1.00",
    risk_policy_max_notional: "10.00",
    sizing_source_ref: `sizing-config-sha256:${"6".repeat(64)}`,
    evaluated_at_unix_ms: 1786406401000,
    evidence_expires_at_unix_ms: 1786406701000,
  };
  candidate.sizing_decision_sha256 = `sha256:${"7".repeat(64)}`;
}

describe("product API generated client", () => {
  it("creates and reads a fail-closed Live Run candidate", async () => {
    const createFetch = jsonFetch(liveRunCandidateFixture, 201);
    const created = await createProductApiClient({
      fetch: createFetch,
    }).createLiveRunCandidate(createLiveRunCandidateBody);
    expect(created.data.lifecycle).toBe("created");
    expect(created.data.runtime_started).toBe(false);
    expect(created.data.order_admission.status).toBe("blocked");
    expect(
      created.data.audit_anchor.workspace_snapshot_rollback_detectable,
    ).toBe(true);
    expect(created.data.audit_anchor.trading_authority_granted).toBe(false);
    expect(created.data.audit_anchor.workspace_revision).toBe(0);
    expect(createFetch).toHaveBeenCalledTimes(1);

    const detail = structuredClone(liveRunCandidateFixture);
    detail.schema_version =
      "ntpro.product_api.live_run_candidate_detail.response.v1";
    const result = await createProductApiClient({
      fetch: jsonFetch(detail),
    }).getLiveRunCandidate(detail.data.run_id);
    expect(result.data.run_id).toBe(detail.data.run_id);

    const list: Record<string, any> = structuredClone(liveRunCandidateFixture);
    list.schema_version =
      "ntpro.product_api.live_run_candidate_list.response.v1";
    list.data = [list.data];
    const listed = await createProductApiClient({
      fetch: jsonFetch(list),
    }).listLiveRunCandidates();
    expect(listed.data).toHaveLength(1);
    expect(listed.data[0]?.lifecycle).toBe("created");
  });

  it("accepts explicit preflight and stop without opening order mutation", async () => {
    const preflight: Record<string, any> = structuredClone(
      liveRunCandidateFixture,
    );
    preflight.schema_version =
      "ntpro.product_api.live_run_candidate_action.response.v1";
    preflight.data.lifecycle = "preflight_ready";
    preflight.data.preflight_at_unix_ms = 1786406401000;
    preflight.data.account_connected = true;
    preflight.data.account_can_trade_verified = true;
    preflight.data.audit_anchor.revision = 1;
    preflight.data.audit_anchor.workspace_revision = 1;
    preflight.data.audit_anchor.receipt_ref = `sha256:${"d".repeat(64)}`;
    preflight.data.audit_anchor.anchored_at_unix_ms = 1786406401000;
    const client = createProductApiClient({ fetch: jsonFetch(preflight) });
    const result = await client.actOnLiveRunCandidate(
      preflight.data.run_id,
      "preflight",
    );
    expect(result.data.lifecycle).toBe("preflight_ready");
    expect(result.boundaries.order_submission_allowed).toBe(false);
  });

  it("authorizes exactly one anchored Live order while cancel and replace remain blocked", async () => {
    const authorized: Record<string, any> = structuredClone(
      liveRunCandidateFixture,
    );
    authorized.schema_version =
      "ntpro.product_api.live_run_candidate_action.response.v1";
    authorized.data.lifecycle = "preflight_ready";
    authorized.data.preflight_at_unix_ms = 1786406401000;
    authorized.data.account_connected = true;
    authorized.data.account_can_trade_verified = true;
    authorized.data.order_admission = {
      status: "authorized_single_shot",
      submit: "authorized_single_shot",
      cancel: "blocked",
      replace: "blocked",
      fill_reconciliation: "runtime_event_projection",
      owner_approved: true,
      risk_approved: true,
      operator_approved: true,
      blockers: [
        "additional_orders_blocked",
        "cancel_not_scoped",
        "replace_not_scoped",
      ],
    };
    authorized.data.strategy_intent = strategyIntent(authorized.data);
    authorized.data.strategy_intent_sha256 = `sha256:${"9".repeat(64)}`;
    attachSizingDecision(authorized.data);
    authorized.data.audit_anchor.revision = 2;
    authorized.data.audit_anchor.workspace_revision = 2;
    authorized.data.audit_anchor.receipt_ref = `sha256:${"7".repeat(64)}`;
    authorized.data.audit_anchor.anchored_at_unix_ms = 1786406402000;
    authorized.boundaries.order_endpoint_access_allowed = true;
    authorized.boundaries.order_submission_allowed = true;
    authorized.boundaries.trading_controls_enabled = true;

    const body = {
      run_id: authorized.data.run_id,
      strategy_version_id: authorized.data.strategy_version_id,
      account_ref: "account://live/binance/primary" as const,
      venue_ref: "venue://live/BINANCE" as const,
      admission_id: "manual-001",
      source_demo_run_id: "demo-source-001",
      strategy_intent_id: "intent-001",
      instrument_id: "BTCUSDT.BINANCE",
      side: "BUY" as const,
      order_type: "LIMIT" as const,
      time_in_force: "GTC" as const,
      price: "1.00",
      quantity: "0.00001000",
      max_notional: "1.00",
      expires_at_unix_ms: 1786406700000,
      user_confirmed: true as const,
    };
    const result = await createProductApiClient({
      fetch: jsonFetch(authorized),
    }).approveLiveExecutionAsOwner(authorized.data.run_id, body);
    expect(result.data.order_admission.status).toBe("authorized_single_shot");
    expect(result.boundaries.order_submission_allowed).toBe(true);
    expect(result.boundaries.cancel_order_allowed).toBe(false);
    expect(result.boundaries.replace_order_allowed).toBe(false);
    expect(result.boundaries.automatic_retry_allowed).toBe(false);
  });

  it.each([
    ["price tick drift", "price", "1.001"],
    ["quantity step drift", "approved_quantity", "0.00001500"],
    ["order notional drift", "order_notional", "0.00002"],
    ["request budget drift", "request_max_notional", "0.000009"],
    ["request exceeds risk policy", "request_max_notional", "100.00"],
    ["risk policy drift", "risk_policy_max_notional", "0.000009"],
  ])("rejects sizing semantic drift: %s", async (_case, field, value) => {
    const authorized: Record<string, any> = structuredClone(
      liveRunCandidateFixture,
    );
    authorized.schema_version =
      "ntpro.product_api.live_run_candidate_action.response.v1";
    authorized.data.lifecycle = "preflight_ready";
    authorized.data.preflight_at_unix_ms = 1786406401000;
    authorized.data.account_connected = true;
    authorized.data.account_can_trade_verified = true;
    authorized.data.order_admission = {
      status: "authorized_single_shot",
      submit: "authorized_single_shot",
      cancel: "blocked",
      replace: "blocked",
      fill_reconciliation: "runtime_event_projection",
      owner_approved: true,
      risk_approved: true,
      operator_approved: true,
      blockers: [
        "additional_orders_blocked",
        "cancel_not_scoped",
        "replace_not_scoped",
      ],
    };
    authorized.data.strategy_intent = strategyIntent(authorized.data);
    authorized.data.strategy_intent_sha256 = `sha256:${"9".repeat(64)}`;
    attachSizingDecision(authorized.data);
    authorized.data.sizing_decision[field] = value;
    authorized.data.audit_anchor.revision = 2;
    authorized.data.audit_anchor.workspace_revision = 2;
    authorized.data.audit_anchor.receipt_ref = `sha256:${"7".repeat(64)}`;
    authorized.data.audit_anchor.anchored_at_unix_ms = 1786406402000;
    authorized.boundaries.order_endpoint_access_allowed = true;
    authorized.boundaries.order_submission_allowed = true;
    authorized.boundaries.trading_controls_enabled = true;

    await expect(
      createProductApiClient({
        fetch: jsonFetch(authorized),
      }).approveLiveExecutionAsOwner(authorized.data.run_id, {
        run_id: authorized.data.run_id,
        strategy_version_id: authorized.data.strategy_version_id,
        account_ref: "account://live/binance/primary",
        venue_ref: "venue://live/BINANCE",
        admission_id: "manual-001",
        source_demo_run_id: "demo-source-001",
        strategy_intent_id: "intent-001",
        instrument_id: "BTCUSDT.BINANCE",
        side: "BUY",
        order_type: "LIMIT",
        time_in_force: "GTC",
        price: "1.00",
        quantity: "0.00001000",
        max_notional: "1.00",
        expires_at_unix_ms: 1786406700000,
        user_confirmed: true,
      }),
    ).rejects.toBeInstanceOf(ProductApiContractError);
  });

  it("projects an exchange-accepted single order without reopening follow-up controls", async () => {
    const running: Record<string, any> = structuredClone(
      liveRunCandidateFixture,
    );
    running.schema_version =
      "ntpro.product_api.live_run_candidate_action.response.v1";
    running.data.lifecycle = "market_data_running";
    running.data.preflight_at_unix_ms = 1786406401000;
    running.data.account_connected = true;
    running.data.account_can_trade_verified = true;
    running.data.runtime_started = true;
    running.data.market_data_connected = true;
    running.data.runtime_node_id = running.data.run_id;
    running.data.runtime_process_state = "running";
    running.data.order_admission = {
      status: "consumed_single_shot",
      submit: "blocked",
      cancel: "dual_approval_required",
      replace: "blocked",
      fill_reconciliation: "explicit_manual_available",
      owner_approved: true,
      risk_approved: true,
      operator_approved: true,
      blockers: ["single_shot_admission_consumed", "additional_orders_blocked"],
    };
    running.data.strategy_intent = strategyIntent(running.data);
    running.data.strategy_intent_sha256 = `sha256:${"9".repeat(64)}`;
    attachSizingDecision(running.data);
    running.data.execution_order = {
      schema_version: "ntpro.s3.live_execution_order_state.v4",
      admission_id: "manual-001",
      source_demo_run_id: "demo-source-001",
      strategy_intent_id: "intent-001",
      strategy_intent_sha256: `sha256:${"9".repeat(64)}`,
      sizing_decision_sha256: `sha256:${"7".repeat(64)}`,
      strategy_version_id: running.data.strategy_version_id,
      instrument_id: "BTCUSDT.BINANCE",
      client_order_id: "S3LV007-001",
      venue_order_id: "1001",
      original_quantity: "0.00001000",
      filled_quantity: "0",
      remaining_quantity: "0.00001000",
      status: "accepted",
      terminal: false,
      new_orders_blocked: true,
      actual_submission_attempted: true,
      automatic_retry_attempted: false,
      cancel_attempted: false,
      replace_attempted: false,
      last_error: null,
      updated_at_unix_ms: 1786406403000,
    };
    running.data.execution_order_state_sha256 = `sha256:${"a".repeat(64)}`;
    running.data.audit_anchor.revision = 4;
    running.data.audit_anchor.workspace_revision = 4;
    running.data.audit_anchor.receipt_ref = `sha256:${"8".repeat(64)}`;
    running.data.audit_anchor.anchored_at_unix_ms = 1786406402000;
    running.boundaries.execution_adapter_send_attempted = true;
    running.boundaries.real_orders_submitted = true;

    const result = await createProductApiClient({
      fetch: jsonFetch(running),
    }).actOnLiveRunCandidate(running.data.run_id, "start_execution");
    expect(result.data.execution_order?.status).toBe("accepted");
    expect(result.boundaries.execution_adapter_send_attempted).toBe(true);
    expect(result.boundaries.real_orders_submitted).toBe(true);
    expect(result.boundaries.order_submission_allowed).toBe(false);
    expect(result.boundaries.cancel_order_allowed).toBe(false);
    expect(result.boundaries.replace_order_allowed).toBe(false);
  });

  it("submits one owner cancel approval without automatic retry", async () => {
    const running: Record<string, any> = structuredClone(
      liveRunCandidateFixture,
    );
    running.schema_version =
      "ntpro.product_api.live_run_candidate_action.response.v1";
    running.data.lifecycle = "market_data_running";
    running.data.preflight_at_unix_ms = 1786406401000;
    running.data.account_connected = true;
    running.data.account_can_trade_verified = true;
    running.data.runtime_started = true;
    running.data.market_data_connected = true;
    running.data.runtime_node_id = running.data.run_id;
    running.data.runtime_process_state = "running";
    running.data.order_admission = {
      status: "consumed_single_shot",
      submit: "blocked",
      cancel: "dual_approval_required",
      replace: "blocked",
      fill_reconciliation: "explicit_manual_available",
      owner_approved: true,
      risk_approved: true,
      operator_approved: true,
      blockers: ["single_shot_admission_consumed", "additional_orders_blocked"],
    };
    running.data.strategy_intent = strategyIntent(running.data);
    running.data.strategy_intent_sha256 = `sha256:${"9".repeat(64)}`;
    attachSizingDecision(running.data);
    running.data.execution_order = {
      schema_version: "ntpro.s3.live_execution_order_state.v4",
      admission_id: "manual-001",
      source_demo_run_id: "demo-source-001",
      strategy_intent_id: "intent-001",
      strategy_intent_sha256: `sha256:${"9".repeat(64)}`,
      sizing_decision_sha256: `sha256:${"7".repeat(64)}`,
      strategy_version_id: running.data.strategy_version_id,
      instrument_id: "BTCUSDT.BINANCE",
      client_order_id: "S3LV008-001",
      venue_order_id: "1001",
      original_quantity: "0.00001000",
      filled_quantity: "0.00000400",
      remaining_quantity: "0.00000600",
      status: "partially_filled",
      terminal: false,
      new_orders_blocked: true,
      actual_submission_attempted: true,
      automatic_retry_attempted: false,
      cancel_attempted: false,
      replace_attempted: false,
      last_error: null,
      updated_at_unix_ms: 1786406403000,
    };
    running.data.execution_order_state_sha256 = `sha256:${"a".repeat(64)}`;
    running.data.audit_anchor.revision = 4;
    running.data.audit_anchor.workspace_revision = 4;
    running.data.audit_anchor.receipt_ref = `sha256:${"8".repeat(64)}`;
    running.data.audit_anchor.anchored_at_unix_ms = 1786406402000;
    running.boundaries.cancel_order_allowed = true;
    running.boundaries.fill_reconciliation_allowed = true;
    running.boundaries.execution_adapter_send_attempted = true;
    running.boundaries.real_orders_submitted = true;
    const request = {
      run_id: running.data.run_id,
      request_id: "cancel-owner-001",
      client_order_id: "S3LV008-001",
      source_order_state_sha256: running.data.execution_order_state_sha256,
      expires_at_unix_ms: 1786406700000,
      user_confirmed: true as const,
    };
    const fetch = jsonFetch(running);

    const result = await createProductApiClient({
      fetch,
    }).approveLiveExecutionCancelAsOwner(running.data.run_id, request);

    expect(result.data.execution_order?.remaining_quantity).toBe("0.00000600");
    expect(result.boundaries.cancel_order_allowed).toBe(true);
    expect(result.boundaries.replace_order_allowed).toBe(false);
    expect(result.boundaries.automatic_retry_allowed).toBe(false);
    expect(fetch).toHaveBeenCalledTimes(1);
    const [input] = fetch.mock.calls[0];
    const submitted = input as Request;
    expect(submitted.url).toContain(
      `/api/product/v1/live-run-candidates/${running.data.run_id}/cancel-approvals/owner`,
    );
    expect(submitted.method).toBe("POST");
    expect(await submitted.clone().json()).toEqual(request);
  });

  it("accepts a confirmed manual cancel result", async () => {
    const running = liveExecutionControlResponse();
    const result = await createProductApiClient({
      fetch: jsonFetch(running),
    }).actOnLiveRunCandidate(running.data.run_id, "start_execution");
    expect(result.data.execution_control?.status).toBe("cancel_confirmed");
    expect(result.data.execution_order?.cancel_attempted).toBe(true);
  });

  it("accepts a terminal fill after a real cancel attempt without claiming confirmation", async () => {
    const running = liveExecutionControlResponse();
    running.data.execution_order.status = "filled";
    running.data.execution_order.filled_quantity = "0.00001000";
    running.data.execution_order.remaining_quantity = "0";
    running.data.execution_control.status = "cancel_sent_readback_pending";
    running.data.execution_control.exchange_order_status = "filled";
    running.data.execution_control.filled_quantity = "0.00001000";
    running.data.execution_control.remaining_quantity = "0";
    running.data.execution_control.cancel_confirmed = false;
    running.data.execution_control.manual_review_required = true;

    const result = await createProductApiClient({
      fetch: jsonFetch(running),
    }).actOnLiveRunCandidate(running.data.run_id, "start_execution");
    expect(result.data.execution_order?.status).toBe("filled");
    expect(result.data.execution_control?.manual_review_required).toBe(true);
  });

  it.each(["submission_requested", "submitted"])(
    "accepts marker-ahead %s manual-review projection",
    async (status) => {
      const running = liveExecutionControlResponse();
      running.data.execution_order.status = status;
      running.data.execution_order.terminal = false;
      running.data.execution_order.venue_order_id = null;
      running.data.execution_order.filled_quantity = "0";
      running.data.execution_order.remaining_quantity = "0.00001000";
      running.data.execution_control.status = "unknown_manual_review";
      running.data.execution_control.venue_order_id = null;
      running.data.execution_control.exchange_order_status = null;
      running.data.execution_control.original_quantity = null;
      running.data.execution_control.filled_quantity = null;
      running.data.execution_control.remaining_quantity = null;
      running.data.execution_control.query_attempted = true;
      running.data.execution_control.cancel_confirmed = false;
      running.data.execution_control.manual_review_required = true;
      running.data.execution_control.error_code =
        "previous_attempt_interrupted_no_retry";

      const result = await createProductApiClient({
        fetch: jsonFetch(running),
      }).actOnLiveRunCandidate(running.data.run_id, "start_execution");
      expect(result.data.execution_order?.status).toBe(status);
      expect(result.data.execution_order?.cancel_attempted).toBe(true);
      expect(result.data.execution_order_state_sha256).toBe(
        running.data.execution_order_state_sha256,
      );
      expect(result.data.execution_control?.status).toBe(
        "unknown_manual_review",
      );
    },
  );

  it.each([
    [
      "cross-action status",
      (value: Record<string, any>) =>
        (value.data.execution_control.action = "reconcile"),
    ],
    [
      "manual review on confirmed cancel",
      (value: Record<string, any>) =>
        (value.data.execution_control.manual_review_required = true),
    ],
    [
      "cancel-not-required after venue send",
      (value: Record<string, any>) => {
        value.data.execution_control.status =
          "cancel_not_required_terminal_or_pending";
        value.data.execution_control.cancel_confirmed = false;
      },
    ],
    [
      "admitted quantity mismatch",
      (value: Record<string, any>) =>
        (value.data.execution_control.original_quantity = "0.00002000"),
    ],
    [
      "venue order identity mismatch",
      (value: Record<string, any>) =>
        (value.data.execution_control.venue_order_id = "different-order"),
    ],
    [
      "admission identity mismatch",
      (value: Record<string, any>) =>
        (value.data.execution_control.admission_id = "different-admission"),
    ],
  ])("rejects Live execution control %s", async (_, mutate) => {
    const running = liveExecutionControlResponse();
    mutate(running);
    await expect(
      createProductApiClient({
        fetch: jsonFetch(running),
      }).actOnLiveRunCandidate(running.data.run_id, "start_execution"),
    ).rejects.toBeInstanceOf(ProductApiContractError);
  });

  it("accepts an expired admission that failed before any adapter send", async () => {
    const failed: Record<string, any> = structuredClone(
      liveRunCandidateFixture,
    );
    failed.schema_version =
      "ntpro.product_api.live_run_candidate_action.response.v1";
    failed.data.lifecycle = "market_data_running";
    failed.data.preflight_at_unix_ms = 1786406401000;
    failed.data.account_connected = true;
    failed.data.account_can_trade_verified = true;
    failed.data.runtime_started = true;
    failed.data.market_data_connected = true;
    failed.data.runtime_node_id = failed.data.run_id;
    failed.data.runtime_process_state = "running";
    failed.data.order_admission = {
      status: "consumed_single_shot",
      submit: "blocked",
      cancel: "dual_approval_required",
      replace: "blocked",
      fill_reconciliation: "explicit_manual_available",
      owner_approved: true,
      risk_approved: true,
      operator_approved: true,
      blockers: ["single_shot_admission_consumed", "additional_orders_blocked"],
    };
    failed.data.strategy_intent = strategyIntent(failed.data);
    failed.data.strategy_intent_sha256 = `sha256:${"9".repeat(64)}`;
    attachSizingDecision(failed.data);
    failed.data.execution_order = {
      schema_version: "ntpro.s3.live_execution_order_state.v4",
      admission_id: "manual-001",
      source_demo_run_id: "demo-source-001",
      strategy_intent_id: "intent-001",
      strategy_intent_sha256: `sha256:${"9".repeat(64)}`,
      sizing_decision_sha256: `sha256:${"7".repeat(64)}`,
      strategy_version_id: failed.data.strategy_version_id,
      instrument_id: "BTCUSDT.BINANCE",
      client_order_id: null,
      venue_order_id: null,
      original_quantity: "0.00001000",
      filled_quantity: "0",
      remaining_quantity: "0.00001000",
      status: "submission_failed",
      terminal: true,
      new_orders_blocked: true,
      actual_submission_attempted: false,
      automatic_retry_attempted: false,
      cancel_attempted: false,
      replace_attempted: false,
      last_error: "execution admission expired before submission",
      updated_at_unix_ms: 1786406403000,
    };
    failed.data.execution_order_state_sha256 = `sha256:${"b".repeat(64)}`;
    failed.data.audit_anchor.revision = 4;
    failed.data.audit_anchor.workspace_revision = 4;
    failed.data.audit_anchor.receipt_ref = `sha256:${"9".repeat(64)}`;
    failed.data.audit_anchor.anchored_at_unix_ms = 1786406402000;

    const result = await createProductApiClient({
      fetch: jsonFetch(failed),
    }).actOnLiveRunCandidate(failed.data.run_id, "start_execution");
    expect(result.data.execution_order?.status).toBe("submission_failed");
    expect(result.boundaries.execution_adapter_send_attempted).toBe(false);
    expect(result.boundaries.real_orders_submitted).toBe(false);
  });

  it("accepts an explicitly started production market-data Runtime with orders blocked", async () => {
    const running: Record<string, any> = structuredClone(
      liveRunCandidateFixture,
    );
    running.schema_version =
      "ntpro.product_api.live_run_candidate_action.response.v1";
    running.data.lifecycle = "market_data_running";
    running.data.preflight_at_unix_ms = 1786406401000;
    running.data.account_connected = true;
    running.data.account_can_trade_verified = true;
    running.data.runtime_started = true;
    running.data.market_data_connected = true;
    running.data.runtime_node_id = running.data.run_id;
    running.data.runtime_process_state = "running";
    running.data.runtime_error = null;
    running.data.audit_anchor.revision = 3;
    running.data.audit_anchor.workspace_revision = 3;
    running.data.audit_anchor.receipt_ref = `sha256:${"f".repeat(64)}`;
    running.data.audit_anchor.anchored_at_unix_ms = 1786406402000;

    const result = await createProductApiClient({
      fetch: jsonFetch(running),
    }).actOnLiveRunCandidate(running.data.run_id, "start_market_data");
    expect(result.data.lifecycle).toBe("market_data_running");
    expect(result.data.market_data_connected).toBe(true);
    expect(result.data.order_admission.status).toBe("blocked");
    expect(result.boundaries.live_runtime_start_allowed).toBe(true);
    expect(result.boundaries.external_market_data_connection_allowed).toBe(
      true,
    );
    expect(result.boundaries.order_submission_allowed).toBe(false);
    expect(result.boundaries.automatic_retry_allowed).toBe(false);
  });

  it("accepts a running Live Runtime failure anchored at revision four", async () => {
    const failed: Record<string, any> = structuredClone(
      liveRunCandidateFixture,
    );
    failed.schema_version =
      "ntpro.product_api.live_run_candidate_detail.response.v1";
    failed.data.lifecycle = "failed";
    failed.data.preflight_at_unix_ms = 1786406401000;
    failed.data.account_connected = true;
    failed.data.account_can_trade_verified = true;
    failed.data.runtime_started = false;
    failed.data.market_data_connected = false;
    failed.data.runtime_node_id = failed.data.run_id;
    failed.data.runtime_process_state = "stopped";
    failed.data.runtime_error = "data client disconnected during live run";
    failed.data.audit_anchor.revision = 4;
    failed.data.audit_anchor.workspace_revision = 4;
    failed.data.audit_anchor.receipt_ref = `sha256:${"9".repeat(64)}`;
    failed.data.audit_anchor.anchored_at_unix_ms = 1786406403000;

    const result = await createProductApiClient({
      fetch: jsonFetch(failed),
    }).getLiveRunCandidate(failed.data.run_id);
    expect(result.data.lifecycle).toBe("failed");
    expect(result.data.runtime_started).toBe(false);
    expect(result.data.runtime_error).toContain("disconnected");
    expect(result.boundaries.order_submission_allowed).toBe(false);
  });

  it.each([
    [
      "runtime started",
      (value: Record<string, any>) => (value.data.runtime_started = true),
    ],
    [
      "order submit opened",
      (value: Record<string, any>) =>
        (value.boundaries.order_submission_allowed = true),
    ],
    [
      "order admission opened",
      (value: Record<string, any>) =>
        (value.data.order_admission.submit = "ready"),
    ],
    [
      "identity drift",
      (value: Record<string, any>) =>
        (value.data.account_ref = "account://live/binance/other"),
    ],
    [
      "unknown field",
      (value: Record<string, any>) => (value.data.secret = "leak"),
    ],
    [
      "missing risk content source",
      (value: Record<string, any>) => value.data.source_refs.splice(2, 1),
    ],
    [
      "malformed risk content source",
      (value: Record<string, any>) =>
        (value.data.source_refs[2] = "risk-config-sha256:not-a-hash"),
    ],
    [
      "created lifecycle with preflight facts",
      (value: Record<string, any>) => {
        value.data.preflight_at_unix_ms = 1786406401000;
        value.data.account_connected = true;
        value.data.account_can_trade_verified = true;
      },
    ],
    [
      "preflight timestamp before creation",
      (value: Record<string, any>) => {
        value.data.lifecycle = "preflight_ready";
        value.data.preflight_at_unix_ms = value.data.created_at_unix_ms - 1;
        value.data.account_connected = true;
        value.data.account_can_trade_verified = true;
      },
    ],
    [
      "stale external audit anchor revision",
      (value: Record<string, any>) => {
        value.data.lifecycle = "preflight_ready";
        value.data.preflight_at_unix_ms = 1786406401000;
        value.data.account_connected = true;
        value.data.account_can_trade_verified = true;
        value.data.audit_anchor.anchored_at_unix_ms = 1786406401000;
      },
    ],
    [
      "audit anchor grants trading authority",
      (value: Record<string, any>) =>
        (value.data.audit_anchor.trading_authority_granted = true),
    ],
  ])("rejects Live candidate %s", async (_, mutate) => {
    const payload = structuredClone(liveRunCandidateFixture);
    mutate(payload);
    await expect(
      createProductApiClient({
        fetch: jsonFetch(payload, 201),
      }).createLiveRunCandidate(createLiveRunCandidateBody),
    ).rejects.toBeInstanceOf(ProductApiContractError);
  });

  const sandboxRunListFixture = structuredClone(runListFixture);
  sandboxRunListFixture.data = sandboxRunListFixture.data.filter(
    (run) => run.environment === "sandbox",
  );
  sandboxRunListFixture.page.returned_count = sandboxRunListFixture.data.length;

  const routeCases = [
    {
      name: "strategy list",
      fixture: strategyListFixture,
      path: "/api/product/v1/strategies?limit=20",
      invoke: (fetch: typeof globalThis.fetch) =>
        createProductApiClient({ fetch }).listStrategies({ limit: 20 }),
    },
    {
      name: "strategy detail",
      fixture: strategyDetailFixture,
      path: "/api/product/v1/strategies/ema-cross",
      invoke: (fetch: typeof globalThis.fetch) =>
        createProductApiClient({ fetch }).getStrategy({
          strategy_id: "ema-cross",
        }),
    },
    {
      name: "strategy version list",
      fixture: strategyVersionListFixture,
      path: "/api/product/v1/strategies/ema-cross/versions",
      invoke: (fetch: typeof globalThis.fetch) =>
        createProductApiClient({ fetch }).listStrategyVersions({
          strategy_id: "ema-cross",
        }),
    },
    {
      name: "strategy version detail",
      fixture: strategyVersionDetailFixture,
      path: "/api/product/v1/strategies/ema-cross/versions/ema-cross%40v1",
      invoke: (fetch: typeof globalThis.fetch) =>
        createProductApiClient({ fetch }).getStrategyVersion({
          strategy_id: "ema-cross",
          version_id: "ema-cross@v1",
        }),
    },
    {
      name: "live admission",
      fixture: liveAdmissionFixture,
      path: "/api/product/v1/strategies/ema-cross/versions/ema-cross%40v1/live-admission",
      invoke: (fetch: typeof globalThis.fetch) =>
        createProductApiClient({ fetch }).getLiveAdmission({
          strategy_id: "ema-cross",
          version_id: "ema-cross@v1",
        }),
    },
    {
      name: "run list",
      fixture: sandboxRunListFixture,
      path: "/api/product/v1/runs?environment=sandbox",
      invoke: (fetch: typeof globalThis.fetch) =>
        createProductApiClient({ fetch }).listRuns({
          environment: "sandbox",
        }),
    },
    {
      name: "run detail",
      fixture: runDetailFixture,
      path: "/api/product/v1/runs/ema-cross-live-001",
      invoke: (fetch: typeof globalThis.fetch) =>
        createProductApiClient({ fetch }).getRun({
          run_id: "ema-cross-live-001",
        }),
    },
    {
      name: "run metrics",
      fixture: runMetricsFixture,
      path: "/api/product/v1/runs/backtest-001/metrics",
      invoke: (fetch: typeof globalThis.fetch) =>
        createProductApiClient({ fetch }).getRunMetrics({
          run_id: "backtest-001",
        }),
    },
    {
      name: "run analysis",
      fixture: runAnalysisFixture,
      path: "/api/product/v1/runs/backtest-001/analysis",
      invoke: (fetch: typeof globalThis.fetch) =>
        createProductApiClient({ fetch }).getRunAnalysis({
          run_id: "backtest-001",
        }),
    },
    {
      name: "run report",
      fixture: runReportFixture,
      path: "/api/product/v1/runs/backtest-001/report",
      invoke: (fetch: typeof globalThis.fetch) =>
        createProductApiClient({ fetch }).getRunReport({
          run_id: "backtest-001",
        }),
    },
  ] as const;

  it.each(routeCases)("consumes the Rust $name fixture", async (testCase) => {
    const fetch = jsonFetch(testCase.fixture);
    const result = await testCase.invoke(fetch);
    const request = fetch.mock.calls[0]?.[0];

    expect(result).toEqual(testCase.fixture);
    expect(request).toBeInstanceOf(Request);
    expect((request as Request).url).toBe(
      `${globalThis.location.origin}${testCase.path}`,
    );
    expect((request as Request).credentials).toBe("same-origin");
    expect((request as Request).headers.get("Accept")).toBe("application/json");
    expect((request as Request).method).toBe("GET");
  });

  const liveAdmissionPath = {
    strategy_id: "ema-cross",
    version_id: "ema-cross@v1",
  };

  it("uses an explicit POST command for the Live account refresh", async () => {
    const fetch = jsonFetch(liveAccountRefreshConnectedFixture);
    const result = await createProductApiClient({ fetch }).refreshLiveAccount(
      liveAdmissionPath,
    );
    const request = fetch.mock.calls[0]?.[0] as Request;

    expect(result).toEqual(liveAccountRefreshConnectedFixture);
    expect(request.method).toBe("POST");
    expect(request.url).toBe(
      `${globalThis.location.origin}/api/product/v1/strategies/ema-cross/versions/ema-cross%40v1/live-account/actions/refresh`,
    );
    expect(await request.json()).toEqual({ action: "refresh" });
  });

  it("accepts a fail-closed Live account refresh without result values", async () => {
    await expect(
      createProductApiClient({
        fetch: jsonFetch(liveAccountRefreshFixture),
      }).refreshLiveAccount(liveAdmissionPath),
    ).resolves.toEqual(liveAccountRefreshFixture);
  });

  it("accepts a failed Live read only when normalized results are absent", async () => {
    const failed = structuredClone(
      liveAccountRefreshConnectedFixture,
    ) as Record<string, any>;
    failed.data.connection_status = "failed";
    failed.data.error_code = "account_result_invalid";
    failed.data.account_result = null;
    failed.data.asset_balances = [];
    failed.data.funds_summary = {
      native_asset_units: true,
      non_zero_asset_count: 0,
      portfolio_value: null,
      source_balance_entry_count: null,
      valuation_currency: null,
      valuation_status: "not_evaluated",
      zero_balance_entry_count: null,
    };
    failed.boundaries.normalized_account_results_exposed = false;

    await expect(
      createProductApiClient({ fetch: jsonFetch(failed) }).refreshLiveAccount(
        liveAdmissionPath,
      ),
    ).resolves.toEqual(failed);
  });

  it.each([
    [
      "unknown field",
      (payload: Record<string, any>) => {
        payload.data.unexpected_authority = false;
      },
    ],
    [
      "mismatched identity",
      (payload: Record<string, any>) => {
        payload.data.strategy_version_id = "ema-cross@v2";
      },
    ],
    [
      "secret exposure",
      (payload: Record<string, any>) => {
        payload.data.credentials.secret_values_exposed = true;
      },
    ],
    [
      "order lifecycle drift",
      (payload: Record<string, any>) => {
        payload.data.order_lifecycle.cancel = "allowed";
      },
    ],
    [
      "duplicate blocker",
      (payload: Record<string, any>) => {
        payload.data.blockers.push(payload.data.blockers[0]);
      },
    ],
    [
      "missing required blocker",
      (payload: Record<string, any>) => {
        payload.data.blockers = payload.data.blockers.filter(
          (blocker: string) => blocker !== "production_network_not_authorized",
        );
      },
    ],
  ] as const)("fails closed for Live admission %s", async (_, mutate) => {
    const payload = structuredClone(liveAdmissionFixture) as Record<
      string,
      any
    >;
    mutate(payload);
    await expect(
      createProductApiClient({ fetch: jsonFetch(payload) }).getLiveAdmission(
        liveAdmissionPath,
      ),
    ).rejects.toBeInstanceOf(ProductApiContractError);
  });

  it.each([
    ["read_only", false],
    ["independent_live_admission_required", false],
    ["owner_approval_granted", true],
    ["inherited_from_backtest", true],
    ["inherited_from_demo", true],
    ["external_venue_connection", true],
    ["production_venue_connection", true],
    ["production_network_allowed", true],
    ["external_network_attempted", true],
    ["authenticated_account_read_allowed", true],
    ["live_run_creation_allowed", true],
    ["order_submission_allowed", true],
    ["cancel_order_allowed", true],
    ["replace_order_allowed", true],
    ["order_mutation_allowed", true],
    ["fill_reconciliation_allowed", true],
    ["automatic_retry_allowed", true],
    ["automatic_remediation_allowed", true],
    ["automatic_recovery_allowed", true],
    ["real_orders_submitted", true],
    ["trading_controls_enabled", true],
  ] as const)("rejects Live boundary drift for %s", async (field, value) => {
    const payload = structuredClone(liveAdmissionFixture) as Record<
      string,
      any
    >;
    payload.boundaries[field] = value;
    await expect(
      createProductApiClient({ fetch: jsonFetch(payload) }).getLiveAdmission(
        liveAdmissionPath,
      ),
    ).rejects.toBeInstanceOf(ProductApiContractError);
  });

  it.each([
    [
      "mismatched identity",
      (payload: Record<string, any>) => {
        payload.data.strategy_version_id = "ema-cross@v2";
      },
    ],
    [
      "closed runtime gate without missing reference",
      (payload: Record<string, any>) => {
        payload.data.runtime_gates.manual_online = false;
      },
    ],
    [
      "duplicate runtime gate reference",
      (payload: Record<string, any>) => {
        payload.data.missing_runtime_gate_refs.push(
          payload.data.missing_runtime_gate_refs[0],
        );
      },
    ],
    [
      "connected without network attempt",
      (payload: Record<string, any>) => {
        payload.data.network_attempted = false;
        payload.data.account_read_attempted = false;
        payload.boundaries.external_network_attempted = false;
      },
    ],
    [
      "connected with non-success status",
      (payload: Record<string, any>) => {
        payload.data.response_status_code = 401;
      },
    ],
    [
      "connected without account type shape proof",
      (payload: Record<string, any>) => {
        payload.data.shape_summary.account_type_present = false;
      },
    ],
    [
      "connected without permission count",
      (payload: Record<string, any>) => {
        payload.data.shape_summary.permission_entry_count = null;
      },
    ],
    [
      "response shape identity drift",
      (payload: Record<string, any>) => {
        payload.data.response_shape = "unknown_account_shape";
      },
    ],
    [
      "order authority",
      (payload: Record<string, any>) => {
        payload.boundaries.order_submission_allowed = true;
      },
    ],
    [
      "raw account exposure",
      (payload: Record<string, any>) => {
        payload.data.shape_summary.raw_balances_exposed = true;
      },
    ],
    [
      "blocked response with account results",
      (payload: Record<string, any>) => {
        payload.data.connection_status = "blocked";
      },
    ],
    [
      "missing account result",
      (payload: Record<string, any>) => {
        payload.data.account_result = null;
      },
    ],
    [
      "duplicate asset result",
      (payload: Record<string, any>) => {
        payload.data.asset_balances[1].asset =
          payload.data.asset_balances[0].asset;
      },
    ],
    [
      "unsorted asset result",
      (payload: Record<string, any>) => {
        payload.data.asset_balances.reverse();
      },
    ],
    [
      "asset count mismatch",
      (payload: Record<string, any>) => {
        payload.data.funds_summary.non_zero_asset_count = 1;
      },
    ],
    [
      "asset total mismatch",
      (payload: Record<string, any>) => {
        payload.data.asset_balances[0].total = "0.1234567";
      },
    ],
    [
      "zero asset total",
      (payload: Record<string, any>) => {
        payload.data.asset_balances[0].free = "0";
        payload.data.asset_balances[0].locked = "0";
        payload.data.asset_balances[0].total = "0";
      },
    ],
    [
      "fabricated cross currency valuation",
      (payload: Record<string, any>) => {
        payload.data.funds_summary.portfolio_value = "105.1234568";
      },
    ],
    [
      "account result persistence drift",
      (payload: Record<string, any>) => {
        payload.boundaries.account_results_persisted = true;
      },
    ],
  ] as const)("fails closed for Live account refresh %s", async (_, mutate) => {
    const payload = structuredClone(
      liveAccountRefreshConnectedFixture,
    ) as Record<string, any>;
    mutate(payload);
    await expect(
      createProductApiClient({ fetch: jsonFetch(payload) }).refreshLiveAccount(
        liveAdmissionPath,
      ),
    ).rejects.toBeInstanceOf(ProductApiContractError);
  });

  it.each([
    [
      "none error",
      (payload: Record<string, any>) => {
        payload.data.error_code = "none";
      },
    ],
    [
      "HTTP status",
      (payload: Record<string, any>) => {
        payload.data.response_status_code = 200;
      },
    ],
    [
      "latency",
      (payload: Record<string, any>) => {
        payload.data.latency_ms = 1;
      },
    ],
    [
      "validated shape",
      (payload: Record<string, any>) => {
        payload.data.response_shape_validated = true;
      },
    ],
    [
      "partial shape",
      (payload: Record<string, any>) => {
        payload.data.shape_summary.account_type_present = true;
      },
    ],
  ] as const)(
    "rejects blocked Live account refresh with %s",
    async (_, mutate) => {
      const payload = structuredClone(liveAccountRefreshFixture) as Record<
        string,
        any
      >;
      mutate(payload);
      await expect(
        createProductApiClient({
          fetch: jsonFetch(payload),
        }).refreshLiveAccount(liveAdmissionPath),
      ).rejects.toBeInstanceOf(ProductApiContractError);
    },
  );

  it.each([
    {
      name: "mismatched Run identity",
      mutate: (payload: Record<string, any>) => {
        payload.data.run_id = "backtest-other";
      },
      field: "run_analysis.path.run_id",
    },
    {
      name: "mismatched summary provenance",
      mutate: (payload: Record<string, any>) => {
        payload.data.provenance.summary_ref =
          "artifact://backtests/backtest-other/summary.json";
      },
      field: "run_analysis.data.provenance",
    },
  ])("fails closed for $name in Run analysis", async ({ mutate, field }) => {
    const payload = structuredClone(runAnalysisFixture) as Record<string, any>;
    mutate(payload);

    await expect(
      createProductApiClient({ fetch: jsonFetch(payload) }).getRunAnalysis({
        run_id: "backtest-001",
      }),
    ).rejects.toMatchObject({ name: "ProductApiContractError", field });
  });

  it("creates a Backtest Run through the generated POST client", async () => {
    const payload = createBacktestResponse();
    const fetch = jsonFetch(payload, 201);

    await expect(
      createProductApiClient({ fetch }).createBacktestRun(createBacktestBody),
    ).resolves.toEqual(payload);

    const request = fetch.mock.calls[0]?.[0];
    expect(request).toBeInstanceOf(Request);
    expect((request as Request).url).toBe(
      `${globalThis.location.origin}/api/product/v1/runs`,
    );
    expect((request as Request).method).toBe("POST");
    await expect((request as Request).json()).resolves.toEqual(
      createBacktestBody,
    );
  });

  it("creates a Demo Run and preserves the Supervisor identity", async () => {
    const fetch = jsonFetch(createdDemoResponse, 201);

    await expect(
      createProductApiClient({ fetch }).createDemoRun(createDemoBody),
    ).resolves.toEqual(createdDemoResponse);

    const request = fetch.mock.calls[0]?.[0] as Request;
    expect(request.url).toBe(
      `${globalThis.location.origin}/api/product/v1/demo-runs`,
    );
    expect(request.method).toBe("POST");
    await expect(request.json()).resolves.toEqual(createDemoBody);
  });

  it("starts a Demo Run once without automatic client retry", async () => {
    const response = demoActionResponse("start");
    const fetch = jsonFetch(response);

    await expect(
      createProductApiClient({ fetch }).actOnDemoRun(
        "demo-created-001",
        "start",
      ),
    ).resolves.toEqual(response);

    expect(fetch).toHaveBeenCalledTimes(1);
    const request = fetch.mock.calls[0]?.[0] as Request;
    expect(request.url).toBe(
      `${globalThis.location.origin}/api/product/v1/demo-runs/demo-created-001/actions`,
    );
    await expect(request.json()).resolves.toEqual({
      run_id: "demo-created-001",
      action: "start",
      user_confirmed: true,
    });
  });

  it("rejects a Demo response that enables real-order capability", async () => {
    const payload = structuredClone(createdDemoResponse);
    payload.data.capabilities.order_submission_allowed = true as false;

    await expect(
      createProductApiClient({ fetch: jsonFetch(payload, 201) }).createDemoRun(
        createDemoBody,
      ),
    ).rejects.toBeInstanceOf(ProductApiContractError);
  });

  it("reads a strict Demo snapshot and preserves its Run identity", async () => {
    const payload = demoSnapshotResponse(createdDemo);
    const fetch = jsonFetch(payload);

    await expect(
      createProductApiClient({ fetch }).getDemoRunSnapshot({
        run_id: createdDemo.run_id,
      }),
    ).resolves.toEqual(payload);

    const request = fetch.mock.calls[0]?.[0] as Request;
    expect(request.url).toBe(
      `${globalThis.location.origin}/api/product/v1/runs/demo-created-001/demo-snapshot`,
    );
    expect(request.method).toBe("GET");
  });

  it.each([
    [
      "submission boundary",
      (payload: Record<string, any>) => {
        payload.boundaries.order_submission_allowed = true;
      },
    ],
    [
      "unknown field",
      (payload: Record<string, any>) => {
        payload.data.unexpected = true;
      },
    ],
    [
      "mismatched Run",
      (payload: Record<string, any>) => {
        payload.data.run_id = "demo-other";
      },
    ],
    [
      "partial not-started market",
      (payload: Record<string, any>) => {
        payload.data.market = {
          connection: "connected",
          state: "exhausted",
          source: "fixture",
          event_count: 0,
          last_event_at_unix_ms: null,
          updated_at_unix_ms: 1,
          latest_event: null,
        };
      },
    ],
    [
      "partial not-started signal",
      (payload: Record<string, any>) => {
        payload.data.latest_signal = {
          symbol: "BTCUSDT.BINANCE",
          signal: "sell",
          confidence: 0.5,
          market_event_seq: 1,
          generated_at_unix_ms: 1,
        };
      },
    ],
  ])("fails closed for Demo snapshot %s", async (_, mutate) => {
    const payload = structuredClone(demoSnapshotResponse(createdDemo));
    mutate(payload);
    await expect(
      createProductApiClient({ fetch: jsonFetch(payload) }).getDemoRunSnapshot({
        run_id: createdDemo.run_id,
      }),
    ).rejects.toBeInstanceOf(ProductApiContractError);
  });

  it("compares two verified Backtest Runs in request order", async () => {
    const runIds = ["backtest-001", "backtest-created-001"];
    const payload = backtestComparisonResponse(runIds);
    const fetch = jsonFetch(payload);

    await expect(
      createProductApiClient({ fetch }).compareRuns(runIds),
    ).resolves.toEqual(payload);

    const request = fetch.mock.calls[0]?.[0] as Request;
    expect(request.url).toBe(
      `${globalThis.location.origin}/api/product/v1/run-comparisons?run_ids=backtest-001%2Cbacktest-created-001`,
    );
    expect(request.method).toBe("GET");
  });

  it.each([
    {
      name: "different StrategyVersion",
      mutate: (payload: Record<string, any>) => {
        payload.data.items[1].strategy_version_id = "ema-cross@v2";
        payload.data.compatibility.same_strategy_version = false;
        payload.data.compatibility.behaviorally_comparable = false;
        payload.data.compatibility.directly_comparable = false;
      },
    },
    {
      name: "different strategy",
      mutate: (payload: Record<string, any>) => {
        payload.data.items[1].strategy_id = "mean-reversion";
        payload.data.compatibility.same_strategy = false;
        payload.data.compatibility.behaviorally_comparable = false;
        payload.data.compatibility.directly_comparable = false;
      },
    },
  ])("accepts a comparison for $name as view-only", async ({ mutate }) => {
    const runIds = ["backtest-001", "backtest-created-001"];
    const payload = structuredClone(
      backtestComparisonResponse(runIds),
    ) as unknown as Record<string, any>;
    mutate(payload);
    await expect(
      createProductApiClient({ fetch: jsonFetch(payload) }).compareRuns(runIds),
    ).resolves.toEqual(payload);
  });

  it("creates and verifies an explicit deterministic reproduction", async () => {
    const sourceRunId = "backtest-created-001";
    const createFetch = jsonFetch(backtestReproductionResponse, 201);
    await expect(
      createProductApiClient({ fetch: createFetch }).reproduceBacktestRun(
        sourceRunId,
      ),
    ).resolves.toEqual(backtestReproductionResponse);
    const createRequest = createFetch.mock.calls[0]?.[0] as Request;
    expect(createRequest.method).toBe("POST");
    await expect(createRequest.json()).resolves.toEqual({
      source_run_id: sourceRunId,
      deterministic_replay: true,
    });

    const proofFetch = jsonFetch(backtestReproductionProofResponse);
    await expect(
      createProductApiClient({ fetch: proofFetch }).getRunReproductionProof({
        run_id: "backtest-reproduced-001",
      }),
    ).resolves.toEqual(backtestReproductionProofResponse);
  });

  it.each([
    {
      name: "server reorders comparison items",
      payload: () => {
        const value = backtestComparisonResponse();
        value.data.items.reverse();
        return value;
      },
      invoke: (fetch: typeof globalThis.fetch) =>
        createProductApiClient({ fetch }).compareRuns([
          "backtest-001",
          "backtest-created-001",
        ]),
      field: "run_comparison.data.identity",
    },
    {
      name: "direct comparability contradicts component checks",
      payload: () => {
        const value = backtestComparisonResponse();
        value.data.compatibility.same_data = false;
        return value;
      },
      invoke: (fetch: typeof globalThis.fetch) =>
        createProductApiClient({ fetch }).compareRuns([
          "backtest-001",
          "backtest-created-001",
        ]),
      field: "run_comparison.data.compatibility",
    },
    {
      name: "parameter compatibility contradicts the compared items",
      payload: () => {
        const value = backtestComparisonResponse();
        value.data.items[1].parameters = {
          ...value.data.items[1].parameters,
          trade_size: "2.000000",
        };
        return value;
      },
      invoke: (fetch: typeof globalThis.fetch) =>
        createProductApiClient({ fetch }).compareRuns([
          "backtest-001",
          "backtest-created-001",
        ]),
      field: "run_comparison.data.compatibility",
    },
    {
      name: "reproduction proof targets another Run",
      payload: () => {
        const value = structuredClone(backtestReproductionProofResponse);
        value.data.reproduced_run_id = "backtest-other";
        return value;
      },
      invoke: (fetch: typeof globalThis.fetch) =>
        createProductApiClient({ fetch }).getRunReproductionProof({
          run_id: "backtest-reproduced-001",
        }),
      field: "run_reproduction_proof.data.identity",
    },
  ])("fails closed when $name", async ({ payload, invoke, field }) => {
    await expect(invoke(jsonFetch(payload()))).rejects.toMatchObject({ field });
  });

  it.each([
    {
      name: "open Live creation boundary",
      mutate: (payload: Record<string, any>) => {
        payload.boundaries.live_run_creation_allowed = true;
      },
      field: "run_create",
    },
    {
      name: "mismatched dataset identity",
      mutate: (payload: Record<string, any>) => {
        payload.data.data_ref = "dataset://fixtures/other";
      },
      field: "run_create.body.data_ref",
    },
    {
      name: "missing result reference",
      mutate: (payload: Record<string, any>) => {
        payload.data.result.result_ref = null;
      },
      field: "run_create.data.lifecycle",
    },
    {
      name: "missing report reference",
      mutate: (payload: Record<string, any>) => {
        payload.data.result.report_ref = null;
      },
      field: "run_create.data.lifecycle",
    },
    {
      name: "missing analysis reference",
      mutate: (payload: Record<string, any>) => {
        payload.data.result.analysis_ref = null;
      },
      field: "run_create.data.lifecycle",
    },
  ])("fails closed for a $name", async ({ mutate, field }) => {
    const payload = createBacktestResponse();
    mutate(payload);
    await expect(
      createProductApiClient({
        fetch: jsonFetch(payload, 201),
      }).createBacktestRun(createBacktestBody),
    ).rejects.toMatchObject({ field });
  });

  it("preserves stable error identity and retry semantics", async () => {
    const fetch = jsonFetch(errorFixture, 404);
    const request = createProductApiClient({ fetch }).getRun({
      run_id: "missing",
    });

    await expect(request).rejects.toMatchObject({
      name: "ProductApiRequestError",
      status: 404,
      requestId: errorFixture.request_id,
      code: errorFixture.error.code,
      field: errorFixture.error.field,
      retryable: false,
    });
  });

  const invalidCases: Array<[string, (payload: Record<string, any>) => void]> =
    [
      [
        "unknown enum",
        (payload) => {
          payload.data[0].lifecycle = "deleted";
        },
      ],
      [
        "stale source",
        (payload) => {
          payload.data[0].source.freshness_status = "stale";
        },
      ],
      [
        "open boundary",
        (payload) => {
          payload.boundaries.order_submission_allowed = true;
        },
      ],
      [
        "unknown field",
        (payload) => {
          payload.data[0].unexpected = true;
        },
      ],
      [
        "pagination count mismatch",
        (payload) => {
          payload.page.returned_count = 0;
        },
      ],
      [
        "pagination cursor mismatch",
        (payload) => {
          payload.page.has_more = true;
        },
      ],
      [
        "empty pagination cursor",
        (payload) => {
          payload.page.has_more = true;
          payload.page.next_cursor = "";
        },
      ],
      [
        "page larger than limit",
        (payload) => {
          payload.data.push(structuredClone(payload.data[0]));
          payload.page.limit = 1;
          payload.page.returned_count = 2;
        },
      ],
      [
        "empty page with continuation",
        (payload) => {
          payload.data = [];
          payload.page.returned_count = 0;
          payload.page.has_more = true;
          payload.page.next_cursor = "strategy-v1-next";
        },
      ],
    ];

  it.each(invalidCases)("fails closed for %s", async (_name, mutate) => {
    const payload = structuredClone(strategyListFixture) as Record<string, any>;
    mutate(payload);
    const request = createProductApiClient({
      fetch: jsonFetch(payload),
    }).listStrategies();
    await expect(request).rejects.toBeInstanceOf(ProductApiContractError);
  });

  it("fails closed for an invalid error envelope", async () => {
    const payload = structuredClone(errorFixture) as Record<string, any>;
    payload.error.code = "unknown_error";
    await expect(
      createProductApiClient({ fetch: jsonFetch(payload, 500) }).listRuns(),
    ).rejects.toBeInstanceOf(ProductApiContractError);
  });

  it("separates network failure from contract failure", async () => {
    const fetch = vi.fn<typeof globalThis.fetch>(async () => {
      throw new TypeError("network unavailable");
    });
    await expect(
      createProductApiClient({ fetch }).listStrategies(),
    ).rejects.toBeInstanceOf(ProductApiTransportError);
  });

  const identityMismatchCases: Array<{
    name: string;
    fixture: Record<string, any>;
    mutate: (payload: Record<string, any>) => void;
    invoke: (fetch: typeof globalThis.fetch) => Promise<unknown>;
    field: string;
  }> = [
    {
      name: "strategy detail path",
      fixture: strategyDetailFixture,
      mutate: (payload) => {
        payload.data.strategy_id = "other-strategy";
      },
      invoke: (fetch) =>
        createProductApiClient({ fetch }).getStrategy({
          strategy_id: "ema-cross",
        }),
      field: "strategy_detail.path.strategy_id",
    },
    {
      name: "version list strategy filter",
      fixture: strategyVersionListFixture,
      mutate: (payload) => {
        payload.data[0].strategy_id = "other-strategy";
      },
      invoke: (fetch) =>
        createProductApiClient({ fetch }).listStrategyVersions({
          strategy_id: "ema-cross",
        }),
      field: "strategy_version_list.path.strategy_id",
    },
    {
      name: "version detail path",
      fixture: strategyVersionDetailFixture,
      mutate: (payload) => {
        payload.data.strategy_version_id = "ema-cross@v2";
      },
      invoke: (fetch) =>
        createProductApiClient({ fetch }).getStrategyVersion({
          strategy_id: "ema-cross",
          version_id: "ema-cross@v1",
        }),
      field: "strategy_version_detail.path.version_id",
    },
    {
      name: "run list strategy filter",
      fixture: runListFixture,
      mutate: (payload) => {
        payload.data[0].strategy_id = "other-strategy";
      },
      invoke: (fetch) =>
        createProductApiClient({ fetch }).listRuns({
          strategy_id: "ema-cross",
        }),
      field: "run_list.query.strategy_id",
    },
    {
      name: "run detail path",
      fixture: runDetailFixture,
      mutate: (payload) => {
        payload.data.run_id = "other-run";
      },
      invoke: (fetch) =>
        createProductApiClient({ fetch }).getRun({
          run_id: "ema-cross-live-001",
        }),
      field: "run_detail.path.run_id",
    },
  ];

  it.each(identityMismatchCases)(
    "fails closed for $name mismatch",
    async ({ fixture, mutate, invoke, field }) => {
      const payload = structuredClone(fixture);
      mutate(payload);
      await expect(invoke(jsonFetch(payload))).rejects.toMatchObject({ field });
    },
  );
});
