import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";

import type { Run } from "../api/generated/productApi";

import { validStatusPayload } from "./fixtures";
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

function comparisonItem(runId: string) {
  return {
    run_id: runId,
    strategy_version_id: runMetricsFixture.data.strategy_version_id,
    data_ref: runMetricsFixture.data.data_ref,
    data_sha256: runMetricsFixture.data.data_sha256,
    config_sha256: runMetricsFixture.data.config_sha256,
    instrument_id: runMetricsFixture.data.instrument_id,
    parameters: runMetricsFixture.data.parameters,
    metrics: runMetricsFixture.data.metrics,
    risk: runAnalysisFixture.data.risk,
    provenance: runAnalysisFixture.data.provenance,
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
    schema_version: "ntpro.product_api.run_comparison.response.v1" as const,
    contract_version: "ntpro.product_api.v1" as const,
    request_id: "product-0000000000000001-0000000000000002",
    data: {
      baseline_run_id: runIds[0],
      run_ids: runIds,
      items: runIds.map(comparisonItem),
      compatibility: {
        same_strategy: true as const,
        same_strategy_version: true,
        same_data: true,
        same_instrument: true,
        same_currency: true,
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
