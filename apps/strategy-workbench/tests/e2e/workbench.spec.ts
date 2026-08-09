import { expect, test } from "@playwright/test";
import { readFileSync } from "node:fs";

import { validStatusPayload } from "../../src/test/fixtures";

function readProductFixture(name: string): Record<string, unknown> {
  return JSON.parse(
    readFileSync(
      new URL(
        `../../src/test/product-api-fixtures/${name}.json`,
        import.meta.url,
      ),
      "utf8",
    ),
  ) as Record<string, unknown>;
}

const errorFixture = readProductFixture("error");
const runDetailFixture = readProductFixture("run-detail");
const runListFixture = readProductFixture("run-list");
const runMetricsFixture = readProductFixture("run-metrics");
const runReportFixture = readProductFixture("run-report");
const strategyDetailFixture = readProductFixture("strategy-detail");
const strategyListFixture = readProductFixture("strategy-list");
const strategyVersionDetailFixture = readProductFixture(
  "strategy-version-detail",
);
const strategyVersionListFixture = readProductFixture("strategy-version-list");
const baselineBacktest = (
  runListFixture.data as Array<Record<string, unknown>>
).find((run) => run.environment === "backtest")!;
const createdBacktest = {
  ...baselineBacktest,
  run_id: "backtest-browser-001",
  config_ref: "artifact://backtests/backtest-browser-001/request.toml",
  account_ref: "account://simulated/backtest-browser-001",
  result: {
    status: "available",
    result_ref: "artifact://backtests/backtest-browser-001/summary.json",
    report_ref: "artifact://backtests/backtest-browser-001/details.json",
  },
  risk: {
    status: "passed",
    risk_ref:
      "artifact://backtests/backtest-browser-001/run-manifest.json#risk",
  },
  source: {
    source_type: "run_manifest",
    freshness_status: "fresh",
    source_refs: [
      "mvp/identity_contract.json",
      "mvp/status_contract.json",
      "artifact://backtests/backtest-browser-001/run-manifest.json",
    ],
  },
};
const createdBacktestResponse = {
  schema_version: "ntpro.product_api.run_create.response.v1",
  contract_version: "ntpro.product_api.v1",
  request_id: "product-0000000000000001-0000000000000001",
  data: createdBacktest,
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

function productFixtureForPath(path: string): Record<string, unknown> {
  if (path === "/api/product/v1/strategies") return strategyListFixture;
  if (path === "/api/product/v1/strategies/ema-cross") {
    return strategyDetailFixture;
  }
  if (path === "/api/product/v1/strategies/ema-cross/versions") {
    return strategyVersionListFixture;
  }
  if (path === "/api/product/v1/strategies/ema-cross/versions/ema-cross@v1") {
    return strategyVersionDetailFixture;
  }
  if (path === "/api/product/v1/runs") return runListFixture;
  if (path === "/api/product/v1/runs/backtest-001/metrics") {
    return runMetricsFixture;
  }
  if (path === "/api/product/v1/runs/backtest-001/report") {
    return runReportFixture;
  }
  if (path === "/api/product/v1/runs/backtest-001") {
    const run = (runListFixture.data as Array<Record<string, unknown>>).find(
      (item) => item.run_id === "backtest-001",
    );
    return { ...runDetailFixture, data: run };
  }
  if (path === "/api/product/v1/runs/backtest-browser-001") {
    return { ...runDetailFixture, data: createdBacktest };
  }
  if (path === "/api/product/v1/runs/backtest-browser-001/metrics") {
    return {
      ...runMetricsFixture,
      data: {
        ...(runMetricsFixture.data as Record<string, unknown>),
        run_id: "backtest-browser-001",
        config_ref: createdBacktest.config_ref,
        result_ref: (createdBacktest.result as Record<string, unknown>)
          .result_ref,
      },
    };
  }
  if (path === "/api/product/v1/runs/backtest-browser-001/report") {
    return {
      ...runReportFixture,
      data: {
        ...(runReportFixture.data as Record<string, unknown>),
        run_id: "backtest-browser-001",
        config_ref: createdBacktest.config_ref,
        details_ref: (createdBacktest.result as Record<string, unknown>)
          .report_ref,
      },
    };
  }
  if (path === "/api/product/v1/runs/ema-cross-live-001") {
    return runDetailFixture;
  }
  return errorFixture;
}

test.beforeEach(async ({ page }) => {
  await page.route("**/api/mvp/v1/status", async (route) => {
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(validStatusPayload),
    });
  });
  await page.route("**/api/product/v1/**", async (route) => {
    const path = decodeURIComponent(new URL(route.request().url()).pathname);
    if (
      route.request().method() === "POST" &&
      path === "/api/product/v1/runs"
    ) {
      await route.fulfill({
        status: 201,
        contentType: "application/json",
        body: JSON.stringify(createdBacktestResponse),
      });
      return;
    }
    await route.fulfill({
      status: path === "/api/product/v1/runs/missing" ? 404 : 200,
      contentType: "application/json",
      body: JSON.stringify(productFixtureForPath(path)),
    });
  });
});

test("Backtest page creates a Run and stays inside the workbench shell", async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto("backtests");
  await expect(
    page.getByRole("heading", { name: "创建策略回测" }),
  ).toBeVisible();
  await expect(page.getByLabel("初始资金")).toHaveValue("1000000 USDT");
  await expect(page.getByLabel("每次交易数量")).toHaveValue("0.001000");
  await page.screenshot({
    path: testInfo.outputPath("strategy-workbench-backtest-create-1440.png"),
    fullPage: true,
  });
  await page.getByRole("button", { name: "创建并运行" }).click();
  await expect(
    page.getByRole("heading", { name: "backtest-browser-001" }),
  ).toBeVisible();
  await expect(page.getByText("真实引擎回测结果")).toBeVisible();
  expect(
    await page.evaluate(
      () =>
        document.documentElement.scrollWidth <=
        document.documentElement.clientWidth,
    ),
  ).toBe(true);
});

test("desktop shell renders verified read-only status", async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto("overview");
  await expect(page.getByText("产品资源已验证")).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "BTC/USDT EMA Cross" }),
  ).toBeVisible();
  await expect(page.getByTestId("strategy-name")).toHaveText("ema-cross");
  for (const liveButton of await page
    .getByRole("button", { name: /Live/ })
    .all()) {
    await expect(liveButton).toBeDisabled();
  }
  await expect(
    page.getByRole("button", { name: /下单|撤单|改单|平仓/ }),
  ).toHaveCount(0);
  await page.getByRole("button", { name: "收起详情栏" }).click();
  await page.getByRole("button", { name: "展开详情栏" }).click();
  await page.getByRole("link", { name: /ema-cross-live-001/ }).click();
  await expect(
    page.getByRole("heading", { name: "ema-cross-live-001" }),
  ).toBeVisible();
  await expect(page.getByText("当前 Run 禁止能力")).toBeVisible();
  const desktopOrigin = await page.evaluate(() => {
    const rail = document.querySelector("aside")?.getBoundingClientRect();
    const canvas = document.querySelector("main")?.getBoundingClientRect();
    return { railRight: rail?.right, canvasLeft: canvas?.left };
  });
  expect(desktopOrigin.canvasLeft).toBeGreaterThanOrEqual(
    desktopOrigin.railRight ?? Number.POSITIVE_INFINITY,
  );
  expect(
    await page.evaluate(
      () =>
        document.documentElement.scrollWidth <=
        document.documentElement.clientWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: testInfo.outputPath("strategy-workbench-1440.png"),
    fullPage: true,
  });

  await page.getByRole("link", { name: "系统状态" }).click();
  await expect(page.getByRole("heading", { name: "系统状态" })).toBeVisible();
});

test("Backtest Run deep link renders immutable engine metrics", async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto("runs/backtest-001");
  await expect(
    page.getByRole("heading", { name: "backtest-001" }),
  ).toBeVisible();
  await expect(page.getByText("真实引擎回测结果")).toBeVisible();
  await expect(
    page.getByRole("region", { name: "Backtest 指标" }),
  ).toContainText("120");
  await expect(page.getByText("研究结果，不代表 Live 准入")).toBeVisible();
  await expect(page.getByLabel("Backtest 收益统计")).toContainText("总损益");
  await expect(
    page.getByRole("img", { name: "账户权益随回测时间变化" }),
  ).toBeVisible();
  await expect(page.getByRole("region", { name: "交易明细" })).toContainText(
    "T-1",
  );
  await expect(page.getByRole("region", { name: "持仓明细" })).toContainText(
    "P-1",
  );
  await page.screenshot({
    path: testInfo.outputPath("strategy-workbench-backtest-report-1440.png"),
    fullPage: true,
  });
});

test("mobile shell keeps the drawer closed and has no page overflow", async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("overview");
  await expect(page.getByText("产品资源已验证")).toBeVisible();
  await expect(page.getByTestId("app-shell")).not.toHaveClass(/drawerOpen/);
  const layout = await page.evaluate(() => ({
    documentWidth: document.documentElement.scrollWidth,
    viewportWidth: document.documentElement.clientWidth,
    scrollX: window.scrollX,
  }));
  expect(layout.documentWidth).toBeLessThanOrEqual(layout.viewportWidth);
  expect(layout.scrollX).toBe(0);
  const runTable = page.getByTestId("run-table-scroll");
  const tableLayout = await runTable.evaluate((element) => ({
    clientWidth: element.clientWidth,
    scrollWidth: element.scrollWidth,
  }));
  expect(tableLayout.scrollWidth).toBeGreaterThan(tableLayout.clientWidth);
  await runTable.evaluate((element) => {
    element.scrollLeft = element.scrollWidth;
  });
  await expect(
    page.getByRole("columnheader", { name: "更新时间" }),
  ).toBeVisible();
  await page.screenshot({
    path: testInfo.outputPath("strategy-workbench-390.png"),
    fullPage: true,
  });
  await page.goto("runs/ema-cross-live-001");
  await expect(
    page.getByRole("heading", { name: "ema-cross-live-001" }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () =>
        document.documentElement.scrollWidth <=
        document.documentElement.clientWidth,
    ),
  ).toBe(true);

  await page.goto("runs/backtest-001");
  await expect(page.getByText("真实引擎回测结果")).toBeVisible();
  await expect(page.getByLabel("Backtest 收益统计")).toContainText("夏普比率");
  await expect(
    page.getByRole("img", { name: "账户权益随回测时间变化" }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () =>
        document.documentElement.scrollWidth <=
        document.documentElement.clientWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: testInfo.outputPath("strategy-workbench-backtest-390.png"),
    fullPage: true,
  });
});

test("technical boundary violation stays separate from Product API resources", async ({
  page,
}) => {
  let blocked = false;
  await page.unroute("**/api/mvp/v1/status");
  await page.route("**/api/mvp/v1/status", async (route) => {
    const payload = structuredClone(validStatusPayload);
    if (blocked) payload.boundaries.real_orders_submitted = true;
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(payload),
    });
  });
  await page.goto("overview");
  await expect(page.getByTestId("strategy-name")).toHaveText("ema-cross");
  blocked = true;
  await page.getByRole("button", { name: "刷新产品与系统状态" }).click();
  await expect(page.getByText("连接阻断")).toBeVisible();
  await expect(page.getByTestId("strategy-name")).toHaveText("ema-cross");
  await expect(page.getByText("产品资源已验证")).toBeVisible();
});

test("real browser consumes every Rust product API fixture through the generated client", async ({
  context,
  page,
}) => {
  await context.addCookies([
    {
      name: "ntpro_mvp_institution_access",
      value: "browser-fixture",
      url: "http://127.0.0.1:4174",
    },
  ]);
  const requests: Array<{ accept: string; cookie: string; method: string }> =
    [];
  await page.route("**/api/product/v1/**", async (route) => {
    const request = route.request();
    const path = decodeURIComponent(new URL(request.url()).pathname);
    const fixture = productFixtureForPath(path);
    requests.push({
      accept: request.headers().accept ?? "",
      cookie: request.headers().cookie ?? "",
      method: request.method(),
    });
    await route.fulfill({
      status: path === "/api/product/v1/runs/missing" ? 404 : 200,
      contentType: "application/json",
      body: JSON.stringify(fixture),
    });
  });

  await page.goto("tests/e2e/product-api-harness.html");
  await expect(page.locator("body")).toHaveAttribute(
    "data-contract-ready",
    "true",
  );
  const result = await page.evaluate(async () => {
    const api = (
      window as typeof window & {
        __ntproProductApi: {
          getRun: (path: { run_id: string }) => Promise<{
            data: { run_id: string };
          }>;
          getStrategy: (path: { strategy_id: string }) => Promise<{
            data: { strategy_id: string };
          }>;
          getStrategyVersion: (path: {
            strategy_id: string;
            version_id: string;
          }) => Promise<{ data: { strategy_version_id: string } }>;
          listRuns: () => Promise<{ data: unknown[] }>;
          listStrategies: () => Promise<{ data: unknown[] }>;
          listStrategyVersions: (path: {
            strategy_id: string;
          }) => Promise<{ data: unknown[] }>;
        };
      }
    ).__ntproProductApi;
    const [strategies, strategy, versions, version, runs, run] =
      await Promise.all([
        api.listStrategies(),
        api.getStrategy({ strategy_id: "ema-cross" }),
        api.listStrategyVersions({ strategy_id: "ema-cross" }),
        api.getStrategyVersion({
          strategy_id: "ema-cross",
          version_id: "ema-cross@v1",
        }),
        api.listRuns(),
        api.getRun({ run_id: "ema-cross-live-001" }),
      ]);
    let error: Record<string, unknown> = {};
    try {
      await api.getRun({ run_id: "missing" });
    } catch (caught) {
      error = {
        code: (caught as { code?: unknown }).code,
        requestId: (caught as { requestId?: unknown }).requestId,
        status: (caught as { status?: unknown }).status,
      };
    }
    return {
      run: run.data.run_id,
      runs: runs.data.length,
      strategies: strategies.data.length,
      strategy: strategy.data.strategy_id,
      version: version.data.strategy_version_id,
      versions: versions.data.length,
      error,
    };
  });

  expect(result).toEqual({
    run: "ema-cross-live-001",
    runs: 3,
    strategies: 1,
    strategy: "ema-cross",
    version: "ema-cross@v1",
    versions: 1,
    error: {
      code: "run_not_found",
      requestId: errorFixture.request_id,
      status: 404,
    },
  });
  expect(requests).toHaveLength(7);
  for (const request of requests) {
    expect(request.method).toBe("GET");
    expect(request.accept).toBe("application/json");
    expect(request.cookie).toContain(
      "ntpro_mvp_institution_access=browser-fixture",
    );
  }
});
