import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { describe, expect, it } from "vitest";

import errorFixture from "../test/product-api-fixtures/error.json";
import runAnalysisFixture from "../test/product-api-fixtures/run-analysis.json";
import runDetailFixture from "../test/product-api-fixtures/run-detail.json";
import runListFixture from "../test/product-api-fixtures/run-list.json";
import runMetricsFixture from "../test/product-api-fixtures/run-metrics.json";
import runReportFixture from "../test/product-api-fixtures/run-report.json";
import strategyListFixture from "../test/product-api-fixtures/strategy-list.json";
import strategyVersionDetailFixture from "../test/product-api-fixtures/strategy-version-detail.json";
import { createdBacktestResponse, server } from "../test/server";
import { createAppRouter } from "./router";

function renderWorkbench(path: string) {
  window.history.replaceState({}, "", `/strategy-workbench${path}`);
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const router = createAppRouter();
  render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
  return { queryClient, router };
}

describe("strategy workbench product slice", () => {
  it("creates a Backtest Run from the product page and opens its detail", async () => {
    let submittedBody: unknown;
    server.use(
      http.post("/api/product/v1/runs", async ({ request }) => {
        submittedBody = await request.json();
        return HttpResponse.json(createdBacktestResponse, { status: 201 });
      }),
    );
    renderWorkbench("/backtests");

    expect(
      await screen.findByRole("heading", { name: "创建策略回测" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Backtest" })).toHaveAttribute(
      "href",
      "/strategy-workbench/backtests",
    );
    await userEvent.click(screen.getByRole("button", { name: "创建并运行" }));
    expect(
      await screen.findByRole("heading", { name: "backtest-created-001" }),
    ).toBeInTheDocument();
    expect(submittedBody).toEqual({
      strategy_id: "ema-cross",
      strategy_version_id: "ema-cross@v1",
      environment: "backtest",
      data_ref: "dataset://fixtures/ema-cross",
      venue_ref: "venue://simulated/BINANCE",
      starting_balance: "1000000 USDT",
      quotes: 120,
      trade_size: "0.001000",
      fast_period: 3,
      slow_period: 5,
    });
  });

  it("renders Product API resources and opens a Run deep link", async () => {
    renderWorkbench("/overview");

    expect(await screen.findByText("产品资源已验证")).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "BTC/USDT EMA Cross" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: /ema-cross-live-001/ }),
    ).toHaveAttribute("href", "/strategy-workbench/runs/ema-cross-live-001");
    expect(
      screen
        .getAllByRole("button", { name: /Live/ })
        .every((button) => button.hasAttribute("disabled")),
    ).toBe(true);
    expect(
      screen.queryByRole("button", { name: /下单|撤单|改单|平仓/ }),
    ).not.toBeInTheDocument();

    await userEvent.click(
      screen.getByRole("link", { name: /ema-cross-live-001/ }),
    );
    expect(
      await screen.findByRole("heading", { name: "ema-cross-live-001" }),
    ).toBeInTheDocument();
    expect(screen.getByText("当前 Run 禁止能力")).toBeInTheDocument();
    expect(screen.getAllByText("关闭")).toHaveLength(7);
  });

  it("renders real Backtest metrics only for an available Backtest Run", async () => {
    renderWorkbench("/runs/backtest-001");

    expect(
      await screen.findByRole("heading", { name: "backtest-001" }),
    ).toBeInTheDocument();
    expect(screen.getByText("真实引擎回测结果")).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "Backtest 指标" }),
    ).toHaveTextContent("120");
    expect(screen.getByText("研究结果，不代表 Live 准入")).toBeInTheDocument();
    expect(screen.getByLabelText("Backtest 收益统计")).toHaveTextContent(
      "总损益",
    );
    expect(screen.getByText("-0.004000000000")).toBeInTheDocument();
    expect(screen.getAllByText("不可计算").length).toBeGreaterThan(0);
    expect(screen.getByText("BTCUSDT.BINANCE")).toBeInTheDocument();
    expect(
      screen.getByRole("img", { name: "账户权益随回测时间变化" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "交易明细" })).toHaveTextContent(
      "T-1",
    );
    expect(screen.getByRole("region", { name: "持仓明细" })).toHaveTextContent(
      "P-1",
    );
    expect(
      screen.getByRole("img", { name: "账户权益回撤随回测时间变化" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "Backtest 运行记录" }),
    ).toHaveTextContent("运行开始");
    expect(
      screen.getByRole("region", { name: "Backtest 分析来源" }),
    ).toHaveTextContent("artifact://backtests/backtest-001/summary.json");
  });

  it.each([404, 500, 503])(
    "keeps Run metrics visible when the report route returns %s",
    async (status) => {
      server.use(
        http.get("/api/product/v1/runs/:runId/report", () =>
          HttpResponse.json(errorFixture, { status }),
        ),
      );

      renderWorkbench("/runs/backtest-001");
      expect(
        await screen.findByRole("heading", { name: "backtest-001" }),
      ).toBeInTheDocument();
      expect(screen.getByText("真实引擎回测结果")).toBeInTheDocument();
      expect(
        await screen.findByRole("button", { name: "重试明细" }),
      ).toBeInTheDocument();
      expect(
        screen.queryByText("历史 Run 仅保留聚合指标"),
      ).not.toBeInTheDocument();
    },
  );

  it("retries a failed report without reloading the Run metrics", async () => {
    let shouldFail = true;
    let attempts = 0;
    server.use(
      http.get("/api/product/v1/runs/:runId/report", () => {
        attempts += 1;
        return shouldFail
          ? HttpResponse.json(errorFixture, { status: 503 })
          : HttpResponse.json(runReportFixture);
      }),
    );

    renderWorkbench("/runs/backtest-001");
    expect(
      await screen.findByRole("heading", { name: "backtest-001" }),
    ).toBeInTheDocument();
    const retry = await screen.findByRole("button", { name: "重试明细" });
    shouldFail = false;
    await userEvent.click(retry);
    expect(
      await screen.findByRole("img", { name: "账户权益随回测时间变化" }),
    ).toBeInTheDocument();
    expect(attempts).toBeGreaterThanOrEqual(2);
  });

  it("keeps historical Backtest metrics when report_ref is null", async () => {
    const historical = structuredClone(runDetailFixture) as Record<string, any>;
    const backtest = structuredClone(
      runListFixture.data.find((run) => run.run_id === "backtest-001")!,
    );
    backtest.result.report_ref = null;
    historical.data = backtest;
    let reportRequests = 0;
    server.use(
      http.get("/api/product/v1/runs/backtest-001", () =>
        HttpResponse.json(historical),
      ),
      http.get("/api/product/v1/runs/backtest-001/report", () => {
        reportRequests += 1;
        return HttpResponse.json(runReportFixture);
      }),
    );

    renderWorkbench("/runs/backtest-001");
    expect(
      await screen.findByText("历史 Run 仅保留聚合指标"),
    ).toBeInTheDocument();
    expect(screen.getByText("真实引擎回测结果")).toBeInTheDocument();
    expect(reportRequests).toBe(0);
  });

  it.each([404, 500, 503])(
    "keeps Run metrics and report visible when the analysis route returns %s",
    async (status) => {
      server.use(
        http.get("/api/product/v1/runs/:runId/analysis", () =>
          HttpResponse.json(errorFixture, { status }),
        ),
      );

      renderWorkbench("/runs/backtest-001");
      expect(
        await screen.findByRole("heading", { name: "backtest-001" }),
      ).toBeInTheDocument();
      expect(screen.getByText("真实引擎回测结果")).toBeInTheDocument();
      expect(
        await screen.findByRole("img", { name: "账户权益随回测时间变化" }),
      ).toBeInTheDocument();
      expect(
        await screen.findByRole("button", { name: "重试分析" }),
      ).toBeInTheDocument();
      expect(
        screen.queryByText("历史 Run 未生成分析产物"),
      ).not.toBeInTheDocument();
    },
  );

  it("retries only a failed analysis resource", async () => {
    let shouldFail = true;
    let analysisRequests = 0;
    let metricsRequests = 0;
    let reportRequests = 0;
    server.use(
      http.get("/api/product/v1/runs/:runId/metrics", () => {
        metricsRequests += 1;
        return HttpResponse.json(runMetricsFixture);
      }),
      http.get("/api/product/v1/runs/:runId/report", () => {
        reportRequests += 1;
        return HttpResponse.json(runReportFixture);
      }),
      http.get("/api/product/v1/runs/:runId/analysis", () => {
        analysisRequests += 1;
        return shouldFail
          ? HttpResponse.json(errorFixture, { status: 503 })
          : HttpResponse.json(runAnalysisFixture);
      }),
    );

    renderWorkbench("/runs/backtest-001");
    const retry = await screen.findByRole("button", { name: "重试分析" });
    const metricsBeforeRetry = metricsRequests;
    const reportBeforeRetry = reportRequests;
    shouldFail = false;
    await userEvent.click(retry);
    expect(
      await screen.findByRole("img", {
        name: "账户权益回撤随回测时间变化",
      }),
    ).toBeInTheDocument();
    expect(analysisRequests).toBeGreaterThanOrEqual(2);
    expect(metricsRequests).toBe(metricsBeforeRetry);
    expect(reportRequests).toBe(reportBeforeRetry);
  });

  it("keeps report visible and skips analysis for a historical Run", async () => {
    const historical = structuredClone(runDetailFixture) as Record<string, any>;
    const backtest = structuredClone(
      runListFixture.data.find((run) => run.run_id === "backtest-001")!,
    );
    backtest.result.analysis_ref = null;
    historical.data = backtest;
    let analysisRequests = 0;
    server.use(
      http.get("/api/product/v1/runs/backtest-001", () =>
        HttpResponse.json(historical),
      ),
      http.get("/api/product/v1/runs/backtest-001/analysis", () => {
        analysisRequests += 1;
        return HttpResponse.json(runAnalysisFixture);
      }),
    );

    renderWorkbench("/runs/backtest-001");
    expect(
      await screen.findByText("历史 Run 未生成分析产物"),
    ).toBeInTheDocument();
    expect(
      await screen.findByRole("img", { name: "账户权益随回测时间变化" }),
    ).toBeInTheDocument();
    expect(analysisRequests).toBe(0);
  });

  it("renders a verified empty strategy state", async () => {
    const empty = structuredClone(strategyListFixture);
    empty.data = [];
    empty.page.returned_count = 0;
    server.use(
      http.get("/api/product/v1/strategies", () => HttpResponse.json(empty)),
    );

    renderWorkbench("/overview");
    expect(await screen.findByText("当前没有已注册策略")).toBeInTheDocument();
  });

  it("fails closed when a Product API boundary opens", async () => {
    const invalid = structuredClone(strategyListFixture) as Record<string, any>;
    invalid.boundaries.order_submission_allowed = true;
    server.use(
      http.get("/api/product/v1/strategies", () => HttpResponse.json(invalid)),
    );

    renderWorkbench("/overview");
    expect(await screen.findByText("产品合同验证失败")).toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "BTC/USDT EMA Cross" }),
    ).not.toBeInTheDocument();
    expect(screen.getByTestId("strategy-name")).toHaveTextContent("策略未加载");
    expect(screen.getByText("产品阻断")).toBeInTheDocument();
  });

  it("clears the entire shell when the exact version identity fails", async () => {
    const invalid = structuredClone(strategyVersionDetailFixture) as Record<
      string,
      any
    >;
    invalid.data.strategy_version_id = "ema-cross@other";
    server.use(
      http.get(
        "/api/product/v1/strategies/:strategyId/versions/:versionId",
        () => HttpResponse.json(invalid),
      ),
    );

    renderWorkbench("/overview");
    expect(await screen.findByText("产品合同验证失败")).toBeInTheDocument();
    expect(screen.getByTestId("strategy-name")).toHaveTextContent("策略未加载");
    expect(screen.getByText("产品阻断")).toBeInTheDocument();
  });

  it("rejects a Run whose response identity differs from the deep link", async () => {
    const invalid = structuredClone(runDetailFixture) as Record<string, any>;
    invalid.data.run_id = "other-run";
    server.use(
      http.get("/api/product/v1/runs/:runId", () => HttpResponse.json(invalid)),
    );

    renderWorkbench("/runs/ema-cross-live-001");
    expect(await screen.findByText("产品合同验证失败")).toBeInTheDocument();
    expect(screen.getByTestId("strategy-name")).toHaveTextContent("策略未加载");
    expect(
      screen.queryByRole("heading", { name: "other-run" }),
    ).not.toBeInTheDocument();
  });

  it("renders the stable not-found error for an unknown Run deep link", async () => {
    server.use(
      http.get("/api/product/v1/runs/:runId", ({ params }) =>
        params.runId === "missing"
          ? HttpResponse.json(errorFixture, { status: 404 })
          : HttpResponse.json(runDetailFixture),
      ),
    );

    const { router } = renderWorkbench("/runs/ema-cross-live-001");
    expect(
      await screen.findByRole("heading", { name: "ema-cross-live-001" }),
    ).toBeInTheDocument();
    await router.navigate({ to: "/runs/$runId", params: { runId: "missing" } });
    expect(await screen.findByText("产品资源不存在")).toBeInTheDocument();
    expect(screen.getByText(/未找到指定运行记录/)).toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "ema-cross-live-001" }),
    ).not.toBeInTheDocument();
  });

  it("renders a distinct access-denied state", async () => {
    const denied = structuredClone(errorFixture);
    denied.error.code = "product_access_denied";
    denied.error.field = "institution_access";
    denied.error.summary = "当前机构无权读取产品资源";
    server.use(
      http.get("/api/product/v1/strategies", () =>
        HttpResponse.json(denied, { status: 403 }),
      ),
    );

    renderWorkbench("/overview");
    expect(await screen.findByText("没有产品资源访问权限")).toBeInTheDocument();
  });

  it("renders a distinct transport failure state", async () => {
    server.use(
      http.get("/api/product/v1/strategies", () => HttpResponse.error()),
    );

    renderWorkbench("/overview");
    expect(await screen.findByText("产品服务不可用")).toBeInTheDocument();
  });

  it("clears cached product context while a manual refresh is pending", async () => {
    let releaseRefresh: (() => void) | undefined;
    const refreshGate = new Promise<void>((resolve) => {
      releaseRefresh = resolve;
    });
    renderWorkbench("/overview");
    expect(await screen.findByText("产品资源已验证")).toBeInTheDocument();

    server.use(
      http.get("/api/product/v1/strategies", async () => {
        await refreshGate;
        return HttpResponse.json(strategyListFixture);
      }),
    );
    await userEvent.click(
      screen.getByRole("button", { name: "刷新产品与系统状态" }),
    );
    expect(await screen.findByText("产品验证中")).toBeInTheDocument();
    expect(screen.getByTestId("strategy-name")).toHaveTextContent("策略未加载");
    expect(screen.queryByText("来源新鲜")).not.toBeInTheDocument();

    releaseRefresh?.();
    expect(await screen.findByText("产品资源已验证")).toBeInTheDocument();
  });

  it("keeps cached product context cleared when refresh fails", async () => {
    renderWorkbench("/overview");
    expect(await screen.findByText("产品资源已验证")).toBeInTheDocument();
    server.use(
      http.get("/api/product/v1/strategies", () => HttpResponse.error()),
    );

    await userEvent.click(
      screen.getByRole("button", { name: "刷新产品与系统状态" }),
    );
    expect(await screen.findByText("产品服务不可用")).toBeInTheDocument();
    expect(screen.getByTestId("strategy-name")).toHaveTextContent("策略未加载");
    expect(screen.getByText("产品阻断")).toBeInTheDocument();
  });

  it("keeps system diagnostics separate from product resources", async () => {
    renderWorkbench("/system-status");
    expect(
      await screen.findByRole("heading", { name: "系统状态" }),
    ).toBeInTheDocument();
    expect(await screen.findByText("只读连接")).toBeInTheDocument();
    await waitFor(() =>
      expect(screen.getByTestId("strategy-name")).toHaveTextContent(
        "ema-cross",
      ),
    );
  });
});
