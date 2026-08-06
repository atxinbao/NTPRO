import { describe, expect, it, vi } from "vitest";

import errorFixture from "../test/product-api-fixtures/error.json";
import runDetailFixture from "../test/product-api-fixtures/run-detail.json";
import runListFixture from "../test/product-api-fixtures/run-list.json";
import strategyDetailFixture from "../test/product-api-fixtures/strategy-detail.json";
import strategyListFixture from "../test/product-api-fixtures/strategy-list.json";
import strategyVersionDetailFixture from "../test/product-api-fixtures/strategy-version-detail.json";
import strategyVersionListFixture from "../test/product-api-fixtures/strategy-version-list.json";
import {
  createProductApiClient,
  ProductApiContractError,
  ProductApiTransportError,
} from "./productApi";

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
      fixture: runListFixture,
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
});
