import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";

import type { DemoRunSnapshotResponse, Run } from "../api/generated/productApi";

import { validStatusPayload } from "./fixtures";
import liveAccountRefreshFixture from "./product-api-fixtures/live-account-refresh-connected.json";
import liveAdmissionFixture from "./product-api-fixtures/live-admission.json";
import liveRunCandidateFixture from "./product-api-fixtures/live-run-candidate.json";
import runDetailFixture from "./product-api-fixtures/run-detail.json";
import runAnalysisFixture from "./product-api-fixtures/run-analysis.json";
import runListFixture from "./product-api-fixtures/run-list.json";
import runMetricsFixture from "./product-api-fixtures/run-metrics.json";
import runReportFixture from "./product-api-fixtures/run-report.json";
import strategyDetailFixture from "./product-api-fixtures/strategy-detail.json";
import strategyListFixture from "./product-api-fixtures/strategy-list.json";
import strategyVersionDetailFixture from "./product-api-fixtures/strategy-version-detail.json";
import strategyVersionListFixture from "./product-api-fixtures/strategy-version-list.json";

const baselineBacktest = runListFixture.data.find(
  (run) => run.environment === "backtest",
)!;
const createdBacktest = {
  ...baselineBacktest,
  run_id: "backtest-created-001",
  config_ref: "artifact://backtests/backtest-created-001/request.toml",
  account_ref: "account://simulated/backtest-created-001",
  result: {
    status: "available" as const,
    result_ref: "artifact://backtests/backtest-created-001/summary.json",
    report_ref: "artifact://backtests/backtest-created-001/details.json",
    analysis_ref: "artifact://backtests/backtest-created-001/analysis.json",
    reproduction_ref: null,
  },
  risk: {
    status: "passed" as const,
    risk_ref:
      "artifact://backtests/backtest-created-001/run-manifest.json#risk",
  },
  source: {
    ...baselineBacktest.source,
    source_refs: [
      "mvp/identity_contract.json",
      "mvp/status_contract.json",
      "artifact://backtests/backtest-created-001/run-manifest.json",
    ],
  },
};

const reproducedBacktest = {
  ...createdBacktest,
  run_id: "backtest-reproduced-001",
  config_ref: "artifact://backtests/backtest-reproduced-001/request.toml",
  account_ref: "account://simulated/backtest-reproduced-001",
  result: {
    status: "available" as const,
    result_ref: "artifact://backtests/backtest-reproduced-001/summary.json",
    report_ref: "artifact://backtests/backtest-reproduced-001/details.json",
    analysis_ref: "artifact://backtests/backtest-reproduced-001/analysis.json",
    reproduction_ref:
      "artifact://backtests/backtest-reproduced-001/reproduction.json",
  },
  risk: {
    status: "passed" as const,
    risk_ref:
      "artifact://backtests/backtest-reproduced-001/run-manifest.json#risk",
  },
};

const reproductionProof = {
  schema_version: "ntpro.backtest_reproduction_proof.v1" as const,
  source_run_id: createdBacktest.run_id,
  reproduced_run_id: reproducedBacktest.run_id,
  proof_ref: reproducedBacktest.result.reproduction_ref,
  source_input_sha256:
    "sha256:1111111111111111111111111111111111111111111111111111111111111111",
  reproduced_input_sha256:
    "sha256:1111111111111111111111111111111111111111111111111111111111111111",
  source_output_sha256:
    "sha256:2222222222222222222222222222222222222222222222222222222222222222",
  reproduced_output_sha256:
    "sha256:2222222222222222222222222222222222222222222222222222222222222222",
  input_equivalent: true as const,
  output_equivalent: true as const,
  user_initiated: true as const,
  automatic_retry_allowed: false as const,
  automatic_remediation_allowed: false as const,
};

const backtestCreationBoundaries = {
  backtest_run_creation_allowed: true as const,
  sandbox_run_creation_allowed: false as const,
  live_run_creation_allowed: false as const,
  external_venue_connection: false as const,
  order_submission_allowed: false as const,
  order_mutation_allowed: false as const,
  automatic_retry_allowed: false as const,
  automatic_remediation_allowed: false as const,
  real_orders_submitted: false as const,
  trading_controls_enabled: false as const,
};

const demoBoundaries = {
  demo_run_creation_allowed: true as const,
  demo_start_allowed: true as const,
  demo_stop_allowed: true as const,
  live_run_creation_allowed: false as const,
  external_venue_connection: false as const,
  order_submission_allowed: false as const,
  order_mutation_allowed: false as const,
  automatic_retry_allowed: false as const,
  automatic_remediation_allowed: false as const,
  real_orders_submitted: false as const,
  trading_controls_enabled: false as const,
};

export const createdDemo = {
  ...baselineBacktest,
  run_id: "demo-created-001",
  environment: "sandbox" as const,
  data_ref: "market://sandbox/BTCUSDT.BINANCE",
  config_ref: "artifact://demo-runs/demo-created-001/request.json",
  adapter_ref: "adapter://sandbox/fixture-stream",
  account_ref: "account://sandbox/acct-sandbox-001",
  venue_ref: "venue://sandbox/BINANCE",
  lifecycle: "created" as const,
  result: {
    status: "pending" as const,
    result_ref: null,
    report_ref: null,
    analysis_ref: null,
    reproduction_ref: null,
  },
  started_at_unix_ms: null,
  completed_at_unix_ms: null,
  risk: {
    status: "pending" as const,
    risk_ref: "artifact://demo-runs/demo-created-001/run-manifest.json#risk",
  },
  runtime: {
    supervisor_node_id: "mvp-node-001",
    strategy_instance_id: "mvp-strategy-001",
    process_state: "not_started" as const,
    lifecycle_state: "stopped" as const,
  },
  source: {
    ...baselineBacktest.source,
    source_refs: [
      "mvp/identity_contract.json",
      "mvp/status_contract.json",
      "artifact://demo-runs/demo-created-001/run-manifest.json",
    ],
  },
} as Run;

export const createdDemoResponse = {
  schema_version: "ntpro.product_api.demo_run_create.response.v1" as const,
  contract_version: "ntpro.product_api.v1" as const,
  request_id: "product-0000000000000001-0000000000000010",
  data: createdDemo,
  boundaries: demoBoundaries,
};

export function demoActionResponse(action: "start" | "stop") {
  const currentRun: Run = {
    ...createdDemo,
    lifecycle: action === "start" ? ("running" as const) : ("stopped" as const),
    started_at_unix_ms: 1_786_400_000_000,
    completed_at_unix_ms: action === "stop" ? 1_786_400_001_000 : null,
    risk: {
      ...createdDemo.risk,
      status: action === "start" ? ("active" as const) : ("blocked" as const),
    },
    runtime: {
      ...createdDemo.runtime!,
      process_state:
        action === "start" ? ("running" as const) : ("stopped" as const),
      lifecycle_state:
        action === "start" ? ("running" as const) : ("stopped" as const),
    },
  };
  return {
    schema_version: "ntpro.product_api.demo_run_action.response.v1" as const,
    contract_version: "ntpro.product_api.v1" as const,
    request_id: "product-0000000000000001-0000000000000011",
    data: {
      run_id: currentRun.run_id,
      action,
      previous_lifecycle:
        action === "start" ? ("created" as const) : ("running" as const),
      current_run: currentRun,
    },
    boundaries: demoBoundaries,
  };
}

export function demoSnapshotResponse(run: Run): DemoRunSnapshotResponse {
  const snapshotStatus =
    run.lifecycle === "created"
      ? ("not_started" as const)
      : run.lifecycle === "stopped" || run.lifecycle === "failed"
        ? ("frozen" as const)
        : ("running" as const);
  const hasRuntimeData = snapshotStatus !== "not_started";
  const frozen = snapshotStatus === "frozen";
  return {
    schema_version: "ntpro.product_api.demo_run_snapshot.response.v2",
    contract_version: "ntpro.product_api.v1",
    request_id: "product-0000000000000001-0000000000000013",
    data: {
      schema_version: "ntpro.product_api.demo_run_result.v2",
      run_id: run.run_id,
      strategy_id: run.strategy_id,
      strategy_version_id: run.strategy_version_id,
      observed_at_unix_ms: 1_786_400_001_000,
      lifecycle: run.lifecycle,
      snapshot_status: snapshotStatus,
      runtime: {
        supervisor_node_id: run.runtime!.supervisor_node_id,
        strategy_instance_id: run.runtime!.strategy_instance_id,
        process_state: run.runtime!.process_state,
        lifecycle_state: run.runtime!.lifecycle_state,
        data_connection: hasRuntimeData ? "connected" : "not_configured",
        execution_connection: "not_configured",
        uptime_ms: hasRuntimeData ? 1_000 : null,
        generated_at_unix_ms: hasRuntimeData ? 1_786_400_000_500 : null,
      },
      market: hasRuntimeData
        ? {
            connection: "connected",
            state: frozen ? "stopped" : "exhausted",
            source: "fixture_stream",
            event_count: 12,
            last_event_at_unix_ms: 1_786_400_000_400,
            updated_at_unix_ms: 1_786_400_000_500,
            latest_event: {
              event_type: "fixture_bar",
              source: "fixture_stream",
              seq: 7,
              symbol: "BTCUSDT.BINANCE",
              price: 100.5,
              event_at_unix_ms: 1_786_400_000_400,
              recorded_at_unix_ms: 1_786_400_000_500,
            },
          }
        : null,
      session: hasRuntimeData
        ? {
            state: frozen ? "stopped" : "running",
            reason: frozen ? "user_stop" : "fixture_completed",
            event_count: 5,
            market_event_count: 12,
            signal_count: 3,
            intent_count: 3,
            risk_decision_count: 3,
            rejection_count: 3,
            actual_submission_count: 0,
            updated_at_unix_ms: 1_786_400_000_500,
          }
        : null,
      latest_signal: hasRuntimeData
        ? {
            symbol: "BTCUSDT.BINANCE",
            signal: "sell",
            confidence: 0.72,
            market_event_seq: 7,
            generated_at_unix_ms: 1_786_400_000_450,
          }
        : null,
      latest_order_intent: hasRuntimeData
        ? {
            intent_id: "intent-demo-001",
            symbol: "BTCUSDT.BINANCE",
            side: "sell",
            order_type: "market",
            quantity: 1,
            source_signal: "sell",
            confidence: 0.72,
            market_event_seq: 7,
            created_at_unix_ms: 1_786_400_000_460,
            submission_allowed: false,
            submission_status: "blocked",
          }
        : null,
      latest_risk_decision: hasRuntimeData
        ? {
            decision_id: "decision-demo-001",
            intent_id: "intent-demo-001",
            symbol: "BTCUSDT.BINANCE",
            decision: "rejected",
            reasons: ["order_submission_disabled"],
            mode: "sandbox",
            order_submission: "disabled",
            kill_switch_enabled: true,
            kill_switch_active: false,
            account_state: "sandbox",
            market_state: "fresh",
            actual_submission: false,
            evaluated_at_unix_ms: 1_786_400_000_470,
          }
        : null,
      simulation: hasRuntimeData
        ? {
            summary: {
              schema_version: "ntpro.demo_simulation_summary.v1",
              session_id: run.run_id,
              strategy_id: run.strategy_id,
              instrument_id: "BTCUSDT.BINANCE",
              engine: "nautilus_backtest::engine::BacktestEngine",
              execution_mode: "simulated",
              data_sha256:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
              parameters: {
                trade_size: "1.000000",
                fast_period: 3,
                slow_period: 5,
              },
              fill_count: 1,
              position_count: 1,
              equity_point_count: 2,
              boundaries: {
                simulation_only: true,
                external_venue_connection: false,
                order_submission_allowed: false,
                order_mutation_allowed: false,
                automatic_retry_allowed: false,
                automatic_remediation_allowed: false,
                real_orders_submitted: false,
                trading_controls_enabled: false,
              },
            },
            fills: [
              {
                schema_version: "ntpro.demo_simulated_fill.v1",
                session_id: run.run_id,
                strategy_id: run.strategy_id,
                simulation_only: true,
                trade_id: "trade-demo-001",
                client_order_id: "order-demo-001",
                venue_order_id: "simulated-001",
                position_id: "position-demo-001",
                side: "SELL",
                order_type: "MARKET",
                quantity: "1.000000",
                price: "100.50",
                currency: "USDT",
                liquidity_side: "TAKER",
                commission: "0.10050000 USDT",
                ts_event: "1786400000400000000",
              },
            ],
            positions: [
              {
                schema_version: "ntpro.demo_simulated_position.v1",
                session_id: run.run_id,
                strategy_id: run.strategy_id,
                simulation_only: true,
                position_id: "position-demo-001",
                account_id: "BINANCE-001",
                side: "SHORT",
                entry_side: "SELL",
                peak_quantity: "1.000000",
                buy_quantity: "0.000000",
                sell_quantity: "1.000000",
                avg_price_open: "100.5",
                avg_price_close: null,
                realized_return: "0",
                realized_pnl: null,
                trade_count: 1,
                ts_opened: "1786400000400000000",
                ts_closed: null,
                duration_ns: "0",
              },
            ],
            equity_curve: [
              {
                schema_version: "ntpro.demo_equity_point.v1",
                session_id: run.run_id,
                strategy_id: run.strategy_id,
                simulation_only: true,
                account_id: "BINANCE-001",
                currency: "USDT",
                total: "1000000.00000000 USDT",
                free: "1000000.00000000 USDT",
                locked: "0.00000000 USDT",
                ts_event: "1786400000000000000",
              },
              {
                schema_version: "ntpro.demo_equity_point.v1",
                session_id: run.run_id,
                strategy_id: run.strategy_id,
                simulation_only: true,
                account_id: "BINANCE-001",
                currency: "USDT",
                total: "999999.89950000 USDT",
                free: "999999.89950000 USDT",
                locked: "0.00000000 USDT",
                ts_event: "1786400000400000000",
              },
            ],
          }
        : null,
      technical_health: {
        status: hasRuntimeData ? "healthy" : "blocked",
        diagnostics: hasRuntimeData ? [] : ["demo_not_started"],
      },
      provenance: {
        source_refs: [`artifact://demo-runs/${run.run_id}/run-manifest.json`],
        manifest_sha256:
          "sha256:1111111111111111111111111111111111111111111111111111111111111111",
        result_ref: frozen
          ? `artifact://demo-runs/${run.run_id}/demo-result.json`
          : null,
        result_sha256: frozen
          ? "sha256:2222222222222222222222222222222222222222222222222222222222222222"
          : null,
      },
    },
    boundaries: {
      read_only: true,
      sandbox_only: true,
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

function comparisonItem(runId: string) {
  return {
    run_id: runId,
    environment: "backtest" as const,
    strategy_id: runMetricsFixture.data.strategy_id,
    strategy_version_id: runMetricsFixture.data.strategy_version_id,
    data_ref: runMetricsFixture.data.data_ref,
    data_sha256: runMetricsFixture.data.data_sha256,
    config_sha256: runMetricsFixture.data.config_sha256,
    instrument_id: runMetricsFixture.data.instrument_id,
    parameters: runMetricsFixture.data.parameters,
    metrics: {
      market_event_count: runMetricsFixture.data.metrics.quotes,
      fill_count: runReportFixture.data.trades.length,
      position_count: runReportFixture.data.positions.length,
    },
    risk: {
      currency: runAnalysisFixture.data.risk.currency,
      starting_equity: runAnalysisFixture.data.risk.starting_equity,
      ending_equity: runAnalysisFixture.data.risk.ending_equity,
      max_drawdown_rate: runAnalysisFixture.data.risk.max_drawdown_rate,
      open_positions: runAnalysisFixture.data.risk.open_positions,
      closed_positions: runAnalysisFixture.data.risk.closed_positions,
    },
    provenance: {
      engine: runAnalysisFixture.data.provenance.generator,
      data_ref: runAnalysisFixture.data.provenance.data_ref,
      data_sha256: runAnalysisFixture.data.provenance.data_sha256,
      source_refs: [runAnalysisFixture.data.provenance.summary_ref],
    },
    reproduction_ref:
      runId === reproducedBacktest.run_id
        ? reproducedBacktest.result.reproduction_ref
        : null,
  };
}

export function backtestComparisonResponse(
  runIds = [baselineBacktest.run_id, createdBacktest.run_id],
) {
  return {
    schema_version: "ntpro.product_api.run_comparison.response.v2" as const,
    contract_version: "ntpro.product_api.v1" as const,
    request_id: "product-0000000000000001-0000000000000002",
    data: {
      baseline_run_id: runIds[0],
      run_ids: runIds,
      items: runIds.map(comparisonItem),
      compatibility: {
        same_strategy: true as const,
        same_strategy_version: true,
        same_parameters: true,
        same_data: true,
        same_instrument: true,
        same_currency: true,
        same_environment: true,
        behaviorally_comparable: true,
        directly_comparable: true,
      },
    },
    boundaries: runDetailFixture.boundaries,
  };
}

export const backtestReproductionResponse = {
  schema_version: "ntpro.product_api.run_reproduction.response.v1" as const,
  contract_version: "ntpro.product_api.v1" as const,
  request_id: "product-0000000000000001-0000000000000003",
  data: {
    source_run_id: createdBacktest.run_id,
    reproduced_run: reproducedBacktest,
    proof: reproductionProof,
  },
  boundaries: backtestCreationBoundaries,
};

export const backtestReproductionProofResponse = {
  schema_version:
    "ntpro.product_api.run_reproduction_proof.response.v1" as const,
  contract_version: "ntpro.product_api.v1" as const,
  request_id: "product-0000000000000001-0000000000000004",
  data: reproductionProof,
  boundaries: runDetailFixture.boundaries,
};

export const createdBacktestResponse = {
  schema_version: "ntpro.product_api.run_create.response.v1" as const,
  contract_version: "ntpro.product_api.v1" as const,
  request_id: "product-0000000000000001-0000000000000001",
  data: createdBacktest,
  boundaries: backtestCreationBoundaries,
};

export const server = setupServer(
  http.get("/api/mvp/v1/status", () => HttpResponse.json(validStatusPayload)),
  http.get("/api/product/v1/strategies", () =>
    HttpResponse.json(strategyListFixture),
  ),
  http.get("/api/product/v1/strategies/:strategyId", () =>
    HttpResponse.json(strategyDetailFixture),
  ),
  http.get("/api/product/v1/strategies/:strategyId/versions", () =>
    HttpResponse.json(strategyVersionListFixture),
  ),
  http.get(
    "/api/product/v1/strategies/:strategyId/versions/:versionId/live-admission",
    () => HttpResponse.json(liveAdmissionFixture),
  ),
  http.post(
    "/api/product/v1/strategies/:strategyId/versions/:versionId/live-account/actions/refresh",
    () => HttpResponse.json(liveAccountRefreshFixture),
  ),
  http.post("/api/product/v1/live-run-candidates", () =>
    HttpResponse.json(liveRunCandidateFixture, { status: 201 }),
  ),
  http.get("/api/product/v1/live-run-candidates", () => {
    const response: Record<string, any> = structuredClone(
      liveRunCandidateFixture,
    );
    response.schema_version =
      "ntpro.product_api.live_run_candidate_list.response.v1";
    response.data = [];
    return HttpResponse.json(response);
  }),
  http.get("/api/product/v1/live-run-candidates/:runId", () => {
    const response: Record<string, any> = structuredClone(
      liveRunCandidateFixture,
    );
    response.schema_version =
      "ntpro.product_api.live_run_candidate_detail.response.v1";
    return HttpResponse.json(response);
  }),
  http.post(
    "/api/product/v1/live-run-candidates/:runId/actions",
    async ({ request }) => {
      const body = (await request.json()) as { action: "preflight" | "stop" };
      const response: Record<string, any> = structuredClone(
        liveRunCandidateFixture,
      );
      response.schema_version =
        "ntpro.product_api.live_run_candidate_action.response.v1";
      if (body.action === "preflight") {
        response.data.lifecycle = "preflight_ready";
        response.data.preflight_at_unix_ms = 1786406401000;
        response.data.account_connected = true;
        response.data.account_can_trade_verified = true;
        response.data.audit_anchor.revision = 1;
        response.data.audit_anchor.workspace_revision = 1;
        response.data.audit_anchor.receipt_ref = `sha256:${"d".repeat(64)}`;
        response.data.audit_anchor.anchored_at_unix_ms = 1786406401000;
      } else {
        response.data.lifecycle = "stopped";
        response.data.stopped_at_unix_ms = 1786406402000;
        response.data.audit_anchor.revision = 1;
        response.data.audit_anchor.workspace_revision = 1;
        response.data.audit_anchor.receipt_ref = `sha256:${"e".repeat(64)}`;
        response.data.audit_anchor.anchored_at_unix_ms = 1786406402000;
      }
      return HttpResponse.json(response);
    },
  ),
  http.get("/api/product/v1/strategies/:strategyId/versions/:versionId", () =>
    HttpResponse.json(strategyVersionDetailFixture),
  ),
  http.get("/api/product/v1/runs", () => HttpResponse.json(runListFixture)),
  http.post("/api/product/v1/runs", () =>
    HttpResponse.json(createdBacktestResponse, { status: 201 }),
  ),
  http.post("/api/product/v1/demo-runs", () =>
    HttpResponse.json(createdDemoResponse, { status: 201 }),
  ),
  http.post("/api/product/v1/demo-runs/:runId/actions", async ({ request }) => {
    const body = (await request.json()) as { action: "start" | "stop" };
    return HttpResponse.json(demoActionResponse(body.action));
  }),
  http.get("/api/product/v1/run-comparisons", ({ request }) => {
    const runIds =
      new URL(request.url).searchParams.get("run_ids")?.split(",") ?? [];
    return HttpResponse.json(backtestComparisonResponse(runIds));
  }),
  http.post("/api/product/v1/runs/:runId/reproduction", () =>
    HttpResponse.json(backtestReproductionResponse, { status: 201 }),
  ),
  http.get("/api/product/v1/runs/:runId/reproduction", () =>
    HttpResponse.json(backtestReproductionProofResponse),
  ),
  http.get("/api/product/v1/runs/:runId", ({ params }) => {
    const run =
      params.runId === createdBacktest.run_id
        ? createdBacktest
        : params.runId === createdDemo.run_id
          ? createdDemo
          : params.runId === reproducedBacktest.run_id
            ? reproducedBacktest
            : runListFixture.data.find((item) => item.run_id === params.runId);
    return HttpResponse.json(
      run ? { ...runDetailFixture, data: run } : runDetailFixture,
    );
  }),
  http.get("/api/product/v1/runs/:runId/demo-snapshot", () =>
    HttpResponse.json(demoSnapshotResponse(createdDemo)),
  ),
  http.get("/api/product/v1/runs/:runId/metrics", ({ params }) =>
    HttpResponse.json(
      params.runId === createdBacktest.run_id ||
        params.runId === reproducedBacktest.run_id
        ? {
            ...runMetricsFixture,
            data: {
              ...runMetricsFixture.data,
              run_id: String(params.runId),
              config_ref:
                params.runId === reproducedBacktest.run_id
                  ? reproducedBacktest.config_ref
                  : createdBacktest.config_ref,
              result_ref:
                params.runId === reproducedBacktest.run_id
                  ? reproducedBacktest.result.result_ref
                  : createdBacktest.result.result_ref,
            },
          }
        : runMetricsFixture,
    ),
  ),
  http.get("/api/product/v1/runs/:runId/report", ({ params }) =>
    HttpResponse.json(
      params.runId === createdBacktest.run_id ||
        params.runId === reproducedBacktest.run_id
        ? {
            ...runReportFixture,
            data: {
              ...runReportFixture.data,
              run_id: String(params.runId),
              config_ref:
                params.runId === reproducedBacktest.run_id
                  ? reproducedBacktest.config_ref
                  : createdBacktest.config_ref,
              details_ref:
                params.runId === reproducedBacktest.run_id
                  ? reproducedBacktest.result.report_ref
                  : createdBacktest.result.report_ref,
            },
          }
        : runReportFixture,
    ),
  ),
  http.get("/api/product/v1/runs/:runId/analysis", ({ params }) =>
    HttpResponse.json(
      params.runId === createdBacktest.run_id ||
        params.runId === reproducedBacktest.run_id
        ? {
            ...runAnalysisFixture,
            data: {
              ...runAnalysisFixture.data,
              run_id: String(params.runId),
              analysis_ref:
                params.runId === reproducedBacktest.run_id
                  ? reproducedBacktest.result.analysis_ref
                  : createdBacktest.result.analysis_ref,
              provenance: {
                ...runAnalysisFixture.data.provenance,
                config_ref:
                  params.runId === reproducedBacktest.run_id
                    ? reproducedBacktest.config_ref
                    : createdBacktest.config_ref,
                summary_ref:
                  params.runId === reproducedBacktest.run_id
                    ? reproducedBacktest.result.result_ref
                    : createdBacktest.result.result_ref,
                details_ref:
                  params.runId === reproducedBacktest.run_id
                    ? reproducedBacktest.result.report_ref
                    : createdBacktest.result.report_ref,
              },
            },
          }
        : runAnalysisFixture,
    ),
  ),
);
