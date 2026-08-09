import { describe, expect, it, vi } from "vitest";

import errorFixture from "../test/product-api-fixtures/error.json";
import runDetailFixture from "../test/product-api-fixtures/run-detail.json";
import runListFixture from "../test/product-api-fixtures/run-list.json";
import runMetricsFixture from "../test/product-api-fixtures/run-metrics.json";
import runReportFixture from "../test/product-api-fixtures/run-report.json";
import strategyDetailFixture from "../test/product-api-fixtures/strategy-detail.json";
import strategyListFixture from "../test/product-api-fixtures/strategy-list.json";
import strategyVersionDetailFixture from "../test/product-api-fixtures/strategy-version-detail.json";
import strategyVersionListFixture from "../test/product-api-fixtures/strategy-version-list.json";
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

describe("product API generated client", () => {
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
