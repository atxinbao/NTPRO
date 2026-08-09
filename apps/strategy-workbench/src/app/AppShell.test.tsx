import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { describe, expect, it } from "vitest";

import errorFixture from "../test/product-api-fixtures/error.json";
import runDetailFixture from "../test/product-api-fixtures/run-detail.json";
import strategyListFixture from "../test/product-api-fixtures/strategy-list.json";
import strategyVersionDetailFixture from "../test/product-api-fixtures/strategy-version-detail.json";
import { server } from "../test/server";
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
