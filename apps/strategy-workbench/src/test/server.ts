import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";

import { validStatusPayload } from "./fixtures";
import runDetailFixture from "./product-api-fixtures/run-detail.json";
import runListFixture from "./product-api-fixtures/run-list.json";
import runMetricsFixture from "./product-api-fixtures/run-metrics.json";
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

export const createdBacktestResponse = {
  schema_version: "ntpro.product_api.run_create.response.v1" as const,
  contract_version: "ntpro.product_api.v1" as const,
  request_id: "product-0000000000000001-0000000000000001",
  data: createdBacktest,
  boundaries: {
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
  },
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
  http.get("/api/product/v1/runs/:runId", ({ params }) => {
    const run =
      params.runId === createdBacktest.run_id
        ? createdBacktest
        : runListFixture.data.find((item) => item.run_id === params.runId);
    return HttpResponse.json(
      run ? { ...runDetailFixture, data: run } : runDetailFixture,
    );
  }),
  http.get("/api/product/v1/runs/:runId/metrics", ({ params }) =>
    HttpResponse.json(
      params.runId === createdBacktest.run_id
        ? {
            ...runMetricsFixture,
            data: {
              ...runMetricsFixture.data,
              run_id: createdBacktest.run_id,
              config_ref: createdBacktest.config_ref,
              result_ref: createdBacktest.result.result_ref,
            },
          }
        : runMetricsFixture,
    ),
  ),
);
