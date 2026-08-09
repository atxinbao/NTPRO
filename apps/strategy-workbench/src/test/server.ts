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
  http.get("/api/product/v1/runs/:runId", ({ params }) => {
    const run = runListFixture.data.find(
      (item) => item.run_id === params.runId,
    );
    return HttpResponse.json(
      run ? { ...runDetailFixture, data: run } : runDetailFixture,
    );
  }),
  http.get("/api/product/v1/runs/:runId/metrics", () =>
    HttpResponse.json(runMetricsFixture),
  ),
);
