import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { http, HttpResponse } from "msw";
import { describe, expect, it } from "vitest";

import errorFixture from "../test/product-api-fixtures/error.json";
import liveRunCandidateFixture from "../test/product-api-fixtures/live-run-candidate.json";
import runAnalysisFixture from "../test/product-api-fixtures/run-analysis.json";
import runDetailFixture from "../test/product-api-fixtures/run-detail.json";
import runListFixture from "../test/product-api-fixtures/run-list.json";
import runMetricsFixture from "../test/product-api-fixtures/run-metrics.json";
import runReportFixture from "../test/product-api-fixtures/run-report.json";
import strategyListFixture from "../test/product-api-fixtures/strategy-list.json";
import strategyVersionDetailFixture from "../test/product-api-fixtures/strategy-version-detail.json";
import {
  backtestComparisonResponse,
  createdBacktestResponse,
  createdDemo,
  createdDemoResponse,
  demoActionResponse,
  demoSnapshotResponse,
  server,
} from "../test/server";
import { createAppRouter } from "./router";
import type { Run, RunComparisonResponse } from "../api/generated/productApi";

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

describe("strategy workbench product slice", () => {
  it("creates and explicitly starts a Demo Run from the workbench", async () => {
    let created = false;
    let currentDemo: Run = structuredClone(createdDemoResponse.data);
    let createBody: unknown;
    let actionBody: unknown;
    let actionPosts = 0;
    const demoList = () => {
      const data: Run[] = runListFixture.data.filter(
        (run) => run.environment !== "sandbox",
      ) as Run[];
      if (created) data.push(currentDemo);
      return {
        ...structuredClone(runListFixture),
        data,
        page: {
          ...structuredClone(runListFixture.page),
          returned_count: data.length,
        },
      };
    };
    server.use(
      http.get("/api/product/v1/runs", () => HttpResponse.json(demoList())),
      http.post("/api/product/v1/demo-runs", async ({ request }) => {
        createBody = await request.json();
        created = true;
        return HttpResponse.json(createdDemoResponse, { status: 201 });
      }),
      http.get("/api/product/v1/runs/:runId", () =>
        HttpResponse.json({
          ...runDetailFixture,
          data: currentDemo,
        }),
      ),
      http.get("/api/product/v1/runs/:runId/demo-snapshot", () =>
        HttpResponse.json(demoSnapshotResponse(currentDemo)),
      ),
      http.post(
        "/api/product/v1/demo-runs/:runId/actions",
        async ({ request }) => {
          actionPosts += 1;
          actionBody = await request.json();
          const response = demoActionResponse("start");
          currentDemo = structuredClone(response.data.current_run);
          return HttpResponse.json(response);
        },
      ),
    );

    renderWorkbench("/demo");
    expect(
      await screen.findByRole("heading", { name: "Sandbox 策略运行" }),
    ).toBeInTheDocument();
    await userEvent.click(
      screen.getByRole("checkbox", {
        name: /我确认创建 Demo Run/,
      }),
    );
    await userEvent.click(
      screen.getByRole("button", { name: "创建 Demo Run" }),
    );

    expect(
      await screen.findByRole("heading", { name: "demo-created-001" }),
    ).toBeInTheDocument();
    expect(createBody).toEqual({
      strategy_id: "ema-cross",
      strategy_version_id: "ema-cross@v1",
      environment: "sandbox",
      supervisor_node_id: "mvp-node-001",
      account_ref: "account://sandbox/acct-sandbox-001",
      venue_ref: "venue://sandbox/BINANCE",
      user_confirmed: true,
    });

    await userEvent.click(screen.getByRole("button", { name: "启动" }));
    await waitFor(() =>
      expect(
        screen.getByRole("region", { name: "Demo 生命周期" }),
      ).toHaveTextContent("running"),
    );
    expect(
      await screen.findByRole("region", { name: "Demo 模拟 成交明细" }),
    ).toHaveTextContent("trade-demo-001");
    expect(
      screen.getByRole("region", { name: "Demo 模拟 持仓明细" }),
    ).toHaveTextContent("position-demo-001");
    expect(
      screen.getByRole("region", { name: "Demo 模拟 资金曲线" }),
    ).toHaveTextContent("999999.89950000 USDT");
    expect(actionPosts).toBe(1);
    expect(actionBody).toEqual({
      run_id: "demo-created-001",
      action: "start",
      user_confirmed: true,
    });
  });

  it("does not retry a failed Demo mutation", async () => {
    let createPosts = 0;
    const data = structuredClone(runListFixture.data).map((run) =>
      run.environment === "sandbox"
        ? { ...run, lifecycle: "stopped" as const }
        : run,
    );
    server.use(
      http.get("/api/product/v1/runs", () =>
        HttpResponse.json({
          ...structuredClone(runListFixture),
          data,
          page: {
            ...structuredClone(runListFixture.page),
            returned_count: data.length,
          },
        }),
      ),
      http.post("/api/product/v1/demo-runs", () => {
        createPosts += 1;
        return HttpResponse.json(errorFixture, { status: 500 });
      }),
    );

    renderWorkbench("/demo");
    await screen.findByRole("heading", { name: "Sandbox 策略运行" });
    await userEvent.click(
      screen.getByRole("checkbox", { name: /我确认创建 Demo Run/ }),
    );
    await userEvent.click(
      screen.getByRole("button", { name: "创建 Demo Run" }),
    );
    await screen.findByRole("alert");
    await new Promise((resolve) => setTimeout(resolve, 100));
    expect(createPosts).toBe(1);
  });

  it("creates a Backtest Run from the product page and opens its detail", async () => {
    let submittedBody: unknown;
    const registeredRuns = structuredClone(runListFixture);
    const registeredBaseline = registeredRuns.data.find(
      (run) => run.environment === "backtest",
    )!;
    registeredBaseline.data_ref = "dataset://fixtures/registered-baseline";
    registeredBaseline.venue_ref = "venue://simulated/KRAKEN";
    const multiVenueVersion = structuredClone(strategyVersionDetailFixture);
    multiVenueVersion.data.data_requirements.venues = ["BINANCE", "KRAKEN"];
    const registeredResponse = structuredClone(createdBacktestResponse);
    registeredResponse.data.data_ref = registeredBaseline.data_ref;
    registeredResponse.data.venue_ref = registeredBaseline.venue_ref;
    server.use(
      http.get("/api/product/v1/runs", () => HttpResponse.json(registeredRuns)),
      http.get(
        "/api/product/v1/strategies/:strategyId/versions/:versionId",
        () => HttpResponse.json(multiVenueVersion),
      ),
      http.post("/api/product/v1/runs", async ({ request }) => {
        submittedBody = await request.json();
        return HttpResponse.json(registeredResponse, { status: 201 });
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
      data_ref: "dataset://fixtures/registered-baseline",
      venue_ref: "venue://simulated/KRAKEN",
      starting_balance: "1000000 USDT",
      quotes: 120,
      trade_size: "0.001000",
      fast_period: 3,
      slow_period: 5,
    });
  });

  it("creates the first Backtest from a verified empty Run list", async () => {
    let submittedBody: unknown;
    const emptyRuns = structuredClone(runListFixture);
    emptyRuns.data = [];
    emptyRuns.page.returned_count = 0;
    server.use(
      http.get("/api/product/v1/runs", () => HttpResponse.json(emptyRuns)),
      http.post("/api/product/v1/runs", async ({ request }) => {
        submittedBody = await request.json();
        return HttpResponse.json(createdBacktestResponse, { status: 201 });
      }),
    );

    renderWorkbench("/backtests");

    expect(await screen.findByRole("status")).toHaveTextContent(
      "当前没有历史 Backtest",
    );
    expect(screen.getByRole("status")).toHaveTextContent("内置确定性数据");
    expect(
      screen.getByDisplayValue("dataset://fixtures/ema-cross"),
    ).toBeVisible();
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

  it("fails closed when an empty Run list has no unique Backtest source", async () => {
    const emptyRuns = structuredClone(runListFixture);
    emptyRuns.data = [];
    emptyRuns.page.returned_count = 0;
    const ambiguousVersion = structuredClone(
      strategyVersionDetailFixture,
    ) as Record<string, any>;
    ambiguousVersion.data.data_requirements.venues = ["BINANCE", "SIM"];
    server.use(
      http.get("/api/product/v1/runs", () => HttpResponse.json(emptyRuns)),
      http.get(
        "/api/product/v1/strategies/:strategyId/versions/:versionId",
        () => HttpResponse.json(ambiguousVersion),
      ),
    );

    renderWorkbench("/backtests");

    expect(
      await screen.findByText("当前策略版本没有唯一的内置回测来源"),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "创建并运行" }),
    ).not.toBeInTheDocument();
  });

  it("keeps Backtest creation closed while the Run list is pending", async () => {
    let releaseRuns: (() => void) | undefined;
    const runGate = new Promise<void>((resolve) => {
      releaseRuns = resolve;
    });
    server.use(
      http.get("/api/product/v1/runs", async () => {
        await runGate;
        return HttpResponse.json(runListFixture);
      }),
    );

    renderWorkbench("/backtests");

    expect(
      await screen.findByText("正在验证 Backtest 运行数据"),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "创建并运行" }),
    ).not.toBeInTheDocument();

    releaseRuns?.();
    expect(
      await screen.findByRole("heading", { name: "创建策略回测" }),
    ).toBeInTheDocument();
  });

  it("compares frozen Runs and creates a user-confirmed Backtest reproduction", async () => {
    const list = {
      ...structuredClone(runListFixture),
      data: [
        structuredClone(
          runListFixture.data.find((run) => run.run_id === "backtest-001")!,
        ),
        structuredClone(createdBacktestResponse.data),
      ],
      page: {
        ...structuredClone(runListFixture.page),
        returned_count: 2,
      },
    };
    server.use(http.get("/api/product/v1/runs", () => HttpResponse.json(list)));

    renderWorkbench("/backtests/compare");

    expect(
      await screen.findByRole("heading", {
        name: "Backtest 与 Demo 行为对比",
      }),
    ).toBeInTheDocument();
    expect(
      await screen.findByRole("region", { name: "比较兼容性" }),
    ).toHaveTextContent("结果可直接比较");
    expect(
      screen.getByRole("region", { name: "Run 比较结果" }),
    ).toHaveTextContent("backtest-created-001");

    await userEvent.click(
      screen.getByRole("button", { name: /backtest-created-001/ }),
    );
    const confirmation = await screen.findByRole("checkbox", {
      name: /我确认这是一次用户主动的确定性复现/,
    });
    await userEvent.click(confirmation);
    await userEvent.click(screen.getByRole("button", { name: "创建复现 Run" }));

    expect(
      await screen.findByRole("heading", { name: "backtest-reproduced-001" }),
    ).toBeInTheDocument();
    expect(
      await screen.findByRole("region", { name: "Backtest 确定性复现证明" }),
    ).toHaveTextContent("输入与输出均等价");
  });

  it("compares a verified Backtest with a stopped Demo without enabling reproduction", async () => {
    const stoppedDemo: Run = {
      ...structuredClone(createdDemoResponse.data),
      lifecycle: "stopped",
      started_at_unix_ms: 1_786_400_000_000,
      completed_at_unix_ms: 1_786_400_001_000,
      runtime: {
        ...structuredClone(createdDemoResponse.data.runtime!),
        process_state: "stopped",
        lifecycle_state: "stopped",
      },
    };
    const list = {
      ...structuredClone(runListFixture),
      data: [
        structuredClone(
          runListFixture.data.find((run) => run.run_id === "backtest-001")!,
        ),
        stoppedDemo,
      ],
      page: {
        ...structuredClone(runListFixture.page),
        returned_count: 2,
      },
    };
    const comparison = structuredClone(
      backtestComparisonResponse(["backtest-001", stoppedDemo.run_id]),
    ) as unknown as RunComparisonResponse;
    comparison.data.items[1] = {
      ...comparison.data.items[1],
      run_id: stoppedDemo.run_id,
      environment: "sandbox",
      strategy_id: stoppedDemo.strategy_id,
      strategy_version_id: stoppedDemo.strategy_version_id,
      data_ref: stoppedDemo.data_ref,
      data_sha256:
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      parameters: {
        ...comparison.data.items[1].parameters,
        trade_size: "1.000000",
      },
      metrics: {
        market_event_count: 12,
        fill_count: 1,
        position_count: 1,
      },
      provenance: {
        engine: "nautilus_backtest::engine::BacktestEngine",
        data_ref: stoppedDemo.data_ref,
        data_sha256:
          "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        source_refs: [
          `artifact://demo-runs/${stoppedDemo.run_id}/run-manifest.json`,
        ],
      },
      reproduction_ref: null,
    };
    comparison.data.compatibility = {
      ...comparison.data.compatibility,
      same_data: false,
      same_parameters: false,
      same_environment: false,
      behaviorally_comparable: false,
      directly_comparable: false,
    };
    server.use(
      http.get("/api/product/v1/runs", () => HttpResponse.json(list)),
      http.get("/api/product/v1/run-comparisons", () =>
        HttpResponse.json(comparison),
      ),
    );

    renderWorkbench("/backtests/compare");

    expect(
      await screen.findByRole("region", { name: "比较兼容性" }),
    ).toHaveTextContent("结果仅可并列查看");
    expect(
      screen.getByRole("region", { name: "Run 比较结果" }),
    ).toHaveTextContent(stoppedDemo.run_id);
    expect(
      screen.getByRole("button", { name: new RegExp(stoppedDemo.run_id) }),
    ).toBeDisabled();
  });

  it("never retries a failed deterministic reproduction automatically", async () => {
    let reproductionPosts = 0;
    const list = {
      ...structuredClone(runListFixture),
      data: [
        structuredClone(
          runListFixture.data.find((run) => run.run_id === "backtest-001")!,
        ),
        structuredClone(createdBacktestResponse.data),
      ],
      page: {
        ...structuredClone(runListFixture.page),
        returned_count: 2,
      },
    };
    server.use(
      http.get("/api/product/v1/runs", () => HttpResponse.json(list)),
      http.post("/api/product/v1/runs/:runId/reproduction", () => {
        reproductionPosts += 1;
        return HttpResponse.json(errorFixture, { status: 503 });
      }),
    );

    renderWorkbench("/backtests/compare");
    await userEvent.click(
      await screen.findByRole("button", { name: /backtest-created-001/ }),
    );
    await userEvent.click(
      await screen.findByRole("checkbox", {
        name: /我确认这是一次用户主动的确定性复现/,
      }),
    );
    await userEvent.click(screen.getByRole("button", { name: "创建复现 Run" }));

    await waitFor(() => expect(reproductionPosts).toBe(1));
    await new Promise((resolve) => setTimeout(resolve, 1_100));
    expect(reproductionPosts).toBe(1);
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
    expect(screen.getByRole("link", { name: "Live" })).toHaveAttribute(
      "href",
      "/strategy-workbench/live",
    );
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

  it("renders explicit Live account results without trading controls", async () => {
    renderWorkbench("/live");

    expect(
      await screen.findByRole("heading", { name: "Live 连接与独立准入" }),
    ).toBeInTheDocument();
    expect(screen.getByText("未准入")).toBeInTheDocument();
    expect(
      screen.getByText("真实 Live Runtime 启动尚未授权"),
    ).toBeInTheDocument();
    expect(screen.getByText("自动恢复尚未授权")).toBeInTheDocument();
    expect(screen.getByText("生产 API Key 尚未配置")).toBeInTheDocument();
    expect(screen.getByText("生产 API Secret 尚未配置")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "检查账户连接" }));
    const accountConnection = screen.getByRole("region", {
      name: "生产账户只读连接",
    });
    expect(
      await within(accountConnection).findByText("已连接"),
    ).toBeInTheDocument();
    expect(within(accountConnection).getByText("5/5")).toBeInTheDocument();
    expect(within(accountConnection).getByText("已尝试")).toBeInTheDocument();
    expect(within(accountConnection).getByText("SPOT")).toBeInTheDocument();
    expect(within(accountConnection).getByText("BTC")).toBeInTheDocument();
    expect(
      within(accountConnection).getByText("0.1234568"),
    ).toBeInTheDocument();
    expect(within(accountConnection).getByText("USDT")).toBeInTheDocument();
    expect(within(accountConnection).getByText("105")).toBeInTheDocument();
    expect(
      within(accountConnection).getByText(/未做跨币种估值/),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /启动|下单|撤单|改单|平仓/ }),
    ).not.toBeInTheDocument();
  });

  it("keeps the Live page usable but candidate creation blocked without an audit anchor", async () => {
    const anchorError = structuredClone(errorFixture);
    anchorError.error.code = "live_run_preflight_failed";
    anchorError.error.field = "live_run_audit_anchor_config";
    anchorError.error.summary = "Live Run 启动前检查未通过";
    server.use(
      http.get("/api/product/v1/live-run-candidates", () =>
        HttpResponse.json(anchorError, { status: 422 }),
      ),
    );

    renderWorkbench("/live");

    await screen.findByRole("heading", { name: "Live 连接与独立准入" });
    const region = screen.getByRole("region", { name: "Live Run 候选" });
    expect(
      await within(region).findByText(
        "外部审计锚点尚未配置，Live Run 候选保持阻断。",
      ),
    ).toBeInTheDocument();
    expect(
      within(region).getByRole("button", { name: "创建 Live Run 候选" }),
    ).toBeDisabled();
  });

  it("creates, preflights, starts market data and manually stops a Live Run candidate", async () => {
    let currentCandidate: Record<string, any> | null = null;
    let listGets = 0;
    server.use(
      http.get("/api/product/v1/live-run-candidates", () => {
        listGets += 1;
        const response: Record<string, any> = structuredClone(
          liveRunCandidateFixture,
        );
        response.schema_version =
          "ntpro.product_api.live_run_candidate_list.response.v1";
        response.data =
          currentCandidate?.data.lifecycle === "stopped"
            ? []
            : currentCandidate
              ? [currentCandidate.data]
              : [];
        return HttpResponse.json(response);
      }),
      http.post("/api/product/v1/live-run-candidates", () => {
        currentCandidate = structuredClone(liveRunCandidateFixture);
        return HttpResponse.json(currentCandidate, { status: 201 });
      }),
      http.post(
        "/api/product/v1/live-run-candidates/:runId/actions",
        async ({ request }) => {
          const body = (await request.json()) as {
            action: "preflight" | "start_market_data" | "stop";
          };
          currentCandidate ??= structuredClone(liveRunCandidateFixture);
          currentCandidate.schema_version =
            "ntpro.product_api.live_run_candidate_action.response.v1";
          if (body.action === "preflight") {
            currentCandidate.data.lifecycle = "preflight_ready";
            currentCandidate.data.preflight_at_unix_ms = 1786406401000;
            currentCandidate.data.account_connected = true;
            currentCandidate.data.account_can_trade_verified = true;
            currentCandidate.data.audit_anchor.revision = 1;
            currentCandidate.data.audit_anchor.workspace_revision = 1;
            currentCandidate.data.audit_anchor.receipt_ref = `sha256:${"d".repeat(64)}`;
            currentCandidate.data.audit_anchor.anchored_at_unix_ms = 1786406401000;
          } else if (body.action === "start_market_data") {
            currentCandidate.data.lifecycle = "market_data_running";
            currentCandidate.data.runtime_started = true;
            currentCandidate.data.market_data_connected = true;
            currentCandidate.data.runtime_node_id =
              currentCandidate.data.run_id;
            currentCandidate.data.runtime_process_state = "running";
            currentCandidate.data.runtime_error = null;
            currentCandidate.data.audit_anchor.revision = 3;
            currentCandidate.data.audit_anchor.workspace_revision = 3;
            currentCandidate.data.audit_anchor.receipt_ref = `sha256:${"f".repeat(64)}`;
            currentCandidate.data.audit_anchor.anchored_at_unix_ms = 1786406402000;
          } else {
            currentCandidate.data.lifecycle = "stopped";
            currentCandidate.data.stopped_at_unix_ms = 1786406403000;
            currentCandidate.data.runtime_started = false;
            currentCandidate.data.market_data_connected = false;
            currentCandidate.data.runtime_process_state = "stopped";
            currentCandidate.data.runtime_error = null;
            currentCandidate.data.audit_anchor.revision = 5;
            currentCandidate.data.audit_anchor.workspace_revision = 5;
            currentCandidate.data.audit_anchor.receipt_ref = `sha256:${"e".repeat(64)}`;
            currentCandidate.data.audit_anchor.anchored_at_unix_ms = 1786406403000;
          }
          return HttpResponse.json(currentCandidate);
        },
      ),
    );
    renderWorkbench("/live");
    await screen.findByRole("heading", { name: "Live 连接与独立准入" });
    await userEvent.click(screen.getByRole("button", { name: "检查账户连接" }));
    const region = screen.getByRole("region", { name: "Live Run 候选" });
    await within(region).findByText(/先显式检查生产账户/);
    await userEvent.click(within(region).getByRole("checkbox"));
    await userEvent.click(
      within(region).getByRole("button", { name: "创建 Live Run 候选" }),
    );
    expect(await within(region).findByText("created")).toBeInTheDocument();
    expect(within(region).getByText("真实 Runtime")).toBeInTheDocument();
    expect(within(region).getByText("未运行")).toBeInTheDocument();
    await userEvent.click(
      within(region).getByRole("button", { name: "执行启动前检查" }),
    );
    expect(
      await within(region).findByText("preflight_ready"),
    ).toBeInTheDocument();
    expect(
      within(region).getByRole("form", {
        name: "单笔真实限价单准入",
      }),
    ).toBeInTheDocument();
    expect(
      within(region).getByRole("button", { name: "提交负责人审批" }),
    ).toBeDisabled();
    await userEvent.click(
      within(region).getByRole("button", { name: "启动生产行情" }),
    );
    expect(
      await within(region).findByText("market_data_running"),
    ).toBeInTheDocument();
    expect(within(region).getByText("运行中")).toBeInTheDocument();
    expect(within(region).getByText("已连接")).toBeInTheDocument();
    expect(
      within(region).queryByRole("button", { name: /下单|撤单|改单|平仓/ }),
    ).not.toBeInTheDocument();
    const listGetsBeforeStop = listGets;
    await userEvent.click(
      within(region).getByRole("button", { name: "人工停止候选" }),
    );
    expect(
      await within(region).findByText(/先显式检查生产账户/),
    ).toBeInTheDocument();
    expect(
      within(region).getByRole("button", { name: "创建 Live Run 候选" }),
    ).toBeInTheDocument();
    expect(within(region).queryByText("stopped")).not.toBeInTheDocument();
    expect(listGets).toBeGreaterThan(listGetsBeforeStop);
  });

  it("binds owner approval to the latest frozen Demo strategy intent", async () => {
    const stoppedDemo: Run = {
      ...structuredClone(createdDemo),
      lifecycle: "stopped",
      started_at_unix_ms: 1_786_400_000_000,
      completed_at_unix_ms: 1_786_400_001_000,
      runtime: {
        ...structuredClone(createdDemo.runtime!),
        process_state: "stopped",
        lifecycle_state: "stopped",
      },
    };
    const candidate: Record<string, any> = structuredClone(
      liveRunCandidateFixture,
    );
    candidate.schema_version =
      "ntpro.product_api.live_run_candidate_list.response.v1";
    candidate.data.lifecycle = "preflight_ready";
    candidate.data.preflight_at_unix_ms = 1_786_406_401_000;
    candidate.data.account_connected = true;
    candidate.data.account_can_trade_verified = true;
    candidate.data.audit_anchor.revision = 1;
    candidate.data.audit_anchor.workspace_revision = 1;
    candidate.data.audit_anchor.receipt_ref = `sha256:${"d".repeat(64)}`;
    candidate.data.audit_anchor.anchored_at_unix_ms = 1_786_406_401_000;
    let ownerApprovalBody: Record<string, unknown> | null = null;
    let ownerApprovalAttempts = 0;
    server.use(
      http.get("/api/product/v1/runs", () => {
        const data = [...structuredClone(runListFixture.data), stoppedDemo];
        return HttpResponse.json({
          ...structuredClone(runListFixture),
          data,
          page: {
            ...structuredClone(runListFixture.page),
            returned_count: data.length,
          },
        });
      }),
      http.get("/api/product/v1/runs/:runId/demo-snapshot", () =>
        HttpResponse.json(demoSnapshotResponse(stoppedDemo)),
      ),
      http.get("/api/product/v1/live-run-candidates", () =>
        HttpResponse.json({ ...candidate, data: [candidate.data] }),
      ),
      http.post(
        "/api/product/v1/live-run-candidates/:runId/execution-approvals/owner",
        async ({ request }) => {
          ownerApprovalAttempts += 1;
          if (ownerApprovalAttempts === 1) {
            const rejected = structuredClone(errorFixture);
            rejected.error.field = "live_sizing_decision.account_budget";
            return HttpResponse.json(rejected, { status: 409 });
          }
          ownerApprovalBody = (await request.json()) as Record<string, unknown>;
          return HttpResponse.json({
            ...candidate,
            schema_version:
              "ntpro.product_api.live_run_candidate_action.response.v1",
          });
        },
      ),
    );

    renderWorkbench("/live");
    const region = await screen.findByRole("region", {
      name: "Live Run 候选",
    });
    const executionForm = await within(region).findByRole("form", {
      name: "单笔真实限价单准入",
    });
    expect(within(executionForm).getByLabelText("交易标的")).toHaveValue(
      "BTCUSDT.BINANCE",
    );
    expect(within(executionForm).getByLabelText("方向")).toHaveValue("SELL");
    expect(within(executionForm).getByLabelText("数量")).toHaveValue("1");
    expect(within(executionForm).getByLabelText("来源 Demo Run")).toHaveValue(
      stoppedDemo.run_id,
    );
    expect(within(executionForm).getByLabelText("策略意图")).toHaveValue(
      "intent-demo-001",
    );
    const ownerApproval = within(executionForm).getByRole("button", {
      name: "提交负责人审批",
    });
    expect(ownerApproval).toBeDisabled();
    await userEvent.type(
      within(executionForm).getByLabelText("限价"),
      "100.50",
    );
    await userEvent.type(
      within(executionForm).getByLabelText("最大名义金额"),
      "100.50",
    );
    await userEvent.click(
      within(executionForm).getByRole("checkbox", { name: /真实订单/ }),
    );
    await userEvent.click(ownerApproval);
    expect(
      await within(region).findByText("订单金额超过账户单笔预算"),
    ).toBeInTheDocument();
    await userEvent.click(ownerApproval);
    await waitFor(() => expect(ownerApprovalBody).not.toBeNull());
    expect(ownerApprovalAttempts).toBe(2);
    expect(ownerApprovalBody).toMatchObject({
      run_id: candidate.data.run_id,
      strategy_version_id: "ema-cross@v1",
      source_demo_run_id: stoppedDemo.run_id,
      strategy_intent_id: "intent-demo-001",
      instrument_id: "BTCUSDT.BINANCE",
      side: "SELL",
      order_type: "LIMIT",
      time_in_force: "GTC",
      price: "100.50",
      quantity: "1",
      max_notional: "100.50",
      user_confirmed: true,
    });
  });

  it("shows partial-fill reconciliation and submits one owner cancel request", async () => {
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
    running.data.strategy_intent = {
      schema_version: "ntpro.s3.live_strategy_order_intent.v1",
      source_demo_run_id: "demo-source-001",
      strategy_id: running.data.strategy_id,
      strategy_version_id: running.data.strategy_version_id,
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
    let cancelPosts = 0;
    let cancelBody: Record<string, unknown> = {};
    server.use(
      http.get("/api/product/v1/live-run-candidates", () => {
        const list = structuredClone(running);
        list.schema_version =
          "ntpro.product_api.live_run_candidate_list.response.v1";
        list.data = [list.data];
        return HttpResponse.json(list);
      }),
      http.post(
        "/api/product/v1/live-run-candidates/:runId/cancel-approvals/owner",
        async ({ request }) => {
          cancelPosts += 1;
          cancelBody = (await request.json()) as Record<string, unknown>;
          return HttpResponse.json(running);
        },
      ),
    );

    renderWorkbench("/live");
    const region = await screen.findByRole("region", { name: "Live Run 候选" });
    expect(within(region).getByText("价格步长")).toBeInTheDocument();
    expect(within(region).getByText("数量步长")).toBeInTheDocument();
    expect(within(region).getByText("数量范围")).toBeInTheDocument();
    expect(within(region).getByText("最小名义金额")).toBeInTheDocument();
    expect(within(region).getByText("账户预算比例")).toBeInTheDocument();
    expect(within(region).getByText("0.00000400")).toBeInTheDocument();
    expect(within(region).getByText("0.00000600")).toBeInTheDocument();
    expect(
      within(region).getByRole("button", { name: "刷新交易所订单状态" }),
    ).toBeEnabled();
    const submit = within(region).getByRole("button", {
      name: "提交人工撤单申请",
    });
    expect(submit).toBeDisabled();
    await userEvent.click(
      within(region).getByRole("checkbox", {
        name: /撤销当前订单的剩余未成交数量/,
      }),
    );
    await userEvent.click(submit);

    await waitFor(() => expect(cancelPosts).toBe(1));
    expect(cancelBody).toMatchObject({
      run_id: running.data.run_id,
      client_order_id: "S3LV008-001",
      source_order_state_sha256: running.data.execution_order_state_sha256,
      user_confirmed: true,
    });
    expect(String(cancelBody.request_id)).toMatch(/^cancel-\d+$/);
    await new Promise((resolve) => setTimeout(resolve, 100));
    expect(cancelPosts).toBe(1);
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
    expect(
      screen.getByRole("region", { name: "Backtest 成交明细" }),
    ).toHaveTextContent("T-1");
    expect(
      screen.getByRole("region", { name: "Backtest 持仓明细" }),
    ).toHaveTextContent("P-1");
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

  it("keeps Run identity visible when Backtest metrics are unavailable", async () => {
    let releaseRetry: (() => void) | undefined;
    const retryGate = new Promise<void>((resolve) => {
      releaseRetry = resolve;
    });
    let retryEnabled = false;
    server.use(
      http.get("/api/product/v1/runs/:runId/metrics", async () => {
        if (!retryEnabled) {
          return HttpResponse.json(errorFixture, { status: 503 });
        }
        await retryGate;
        return HttpResponse.json(runMetricsFixture);
      }),
    );

    renderWorkbench("/runs/backtest-001");
    expect(
      await screen.findByRole("heading", { name: "backtest-001" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Backtest · 已完成")).toBeInTheDocument();
    const retry = await screen.findByRole("button", { name: "重试指标" });
    expect(screen.queryByText("真实引擎回测结果")).not.toBeInTheDocument();
    retryEnabled = true;
    await userEvent.click(retry);
    expect(
      await screen.findByText("正在加载 Backtest 指标"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "backtest-001" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Backtest · 已完成")).toBeInTheDocument();
    releaseRetry?.();
    expect(
      await screen.findByRole("region", { name: "Backtest 指标" }),
    ).toBeInTheDocument();
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

  it("opens the independent strategy page from the left navigation", async () => {
    renderWorkbench("/overview");
    expect(
      await screen.findByRole("heading", { name: "BTC/USDT EMA Cross" }),
    ).toBeInTheDocument();

    const strategyLink = screen.getByRole("link", { name: "策略" });
    expect(strategyLink).toHaveAttribute(
      "href",
      "/strategy-workbench/strategies",
    );
    await userEvent.click(strategyLink);

    expect(
      await screen.findByRole("heading", { name: "策略管理" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "当前策略身份" }),
    ).toHaveTextContent("ema-cross");
    expect(
      screen.getByRole("region", { name: "默认不可变版本" }),
    ).toHaveTextContent("ema-cross@v1");
    expect(
      screen.getByRole("region", { name: "策略固定参数" }),
    ).toHaveTextContent("fast_period3");
    expect(
      screen.getByRole("region", { name: "策略运行模式摘要" }),
    ).toHaveTextContent("Backtest");
  });

  it("keeps Run counts unknown while the strategy page runtime query is pending", async () => {
    let releaseRuns: (() => void) | undefined;
    const runGate = new Promise<void>((resolve) => {
      releaseRuns = resolve;
    });
    server.use(
      http.get("/api/product/v1/runs", async () => {
        await runGate;
        return HttpResponse.json(runListFixture);
      }),
    );

    renderWorkbench("/strategies");
    expect(
      await screen.findByRole("heading", { name: "策略管理" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("策略目录已验证，正在验证运行数据"),
    ).toBeInTheDocument();
    expect(screen.getAllByText("运行状态未知")).toHaveLength(2);
    expect(screen.queryByText("来源新鲜")).not.toBeInTheDocument();
    const modes = screen.getByRole("region", { name: "策略运行模式摘要" });
    expect(within(modes).getAllByText("--")).toHaveLength(3);
    expect(within(modes).queryByText("0 个")).not.toBeInTheDocument();

    releaseRuns?.();
    expect(await screen.findByText("策略资源已验证")).toBeInTheDocument();
    expect(within(modes).queryByText("--")).not.toBeInTheDocument();
  });

  it("renders a verified empty strategy state on the strategy route", async () => {
    const empty = structuredClone(strategyListFixture);
    empty.data = [];
    empty.page.returned_count = 0;
    server.use(
      http.get("/api/product/v1/strategies", () => HttpResponse.json(empty)),
    );

    renderWorkbench("/strategies");
    expect(await screen.findByText("当前没有已注册策略")).toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "策略管理" }),
    ).not.toBeInTheDocument();
  });

  it("keeps the strategy page catalog visible when runtime data is stale", async () => {
    const stale = structuredClone(errorFixture);
    stale.error.code = "product_source_stale";
    stale.error.field = "node_status";
    stale.error.summary = "策略产品数据源已过期，需要刷新对应来源后重试";
    stale.error.retryable = true;
    server.use(
      http.get("/api/product/v1/runs", () =>
        HttpResponse.json(stale, { status: 503 }),
      ),
    );

    renderWorkbench("/strategies");
    expect(
      await screen.findByRole("heading", { name: "策略管理" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("策略目录已验证，运行摘要降级"),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("region", { name: "当前策略身份" }),
    ).toHaveTextContent("ema-cross");
    expect(
      screen.getByText(/策略产品数据源已过期，需要刷新对应来源后重试/),
    ).toBeInTheDocument();
    const modes = screen.getByRole("region", { name: "策略运行模式摘要" });
    expect(within(modes).getAllByText("--")).toHaveLength(3);
    expect(within(modes).queryByText("0 个")).not.toBeInTheDocument();
  });

  it("fails closed on the strategy route when a Product API boundary opens", async () => {
    const invalid = structuredClone(strategyListFixture) as Record<string, any>;
    invalid.boundaries.order_submission_allowed = true;
    server.use(
      http.get("/api/product/v1/strategies", () => HttpResponse.json(invalid)),
    );

    renderWorkbench("/strategies");
    expect(await screen.findByText("产品合同验证失败")).toBeInTheDocument();
    expect(
      screen.queryByRole("heading", { name: "策略管理" }),
    ).not.toBeInTheDocument();
    expect(screen.getByTestId("strategy-name")).toHaveTextContent("策略未加载");
    expect(screen.getByText("产品阻断")).toBeInTheDocument();
  });

  it("keeps the strategy catalog visible when runtime data is stale", async () => {
    const stale = structuredClone(errorFixture);
    stale.error.code = "product_source_stale";
    stale.error.field = "node_status";
    stale.error.summary = "策略产品数据源已过期，需要刷新对应来源后重试";
    stale.error.retryable = true;
    server.use(
      http.get("/api/product/v1/runs", () =>
        HttpResponse.json(stale, { status: 503 }),
      ),
    );

    renderWorkbench("/overview");
    expect(
      await screen.findByRole("heading", { name: "BTC/USDT EMA Cross" }),
    ).toBeInTheDocument();
    expect(
      await screen.findByText("策略目录已验证，运行数据降级"),
    ).toBeInTheDocument();
    expect(screen.getAllByText("ema-cross@v1").length).toBeGreaterThan(0);
    expect(
      screen.getByText("产品服务返回错误，策略与版本目录仍可查看"),
    ).toBeInTheDocument();
    expect(screen.queryByText("当前没有已注册策略")).not.toBeInTheDocument();
    expect(screen.queryByText("backtest-001")).not.toBeInTheDocument();
  });

  it("shows the Run source error instead of claiming the Backtest dataset is missing", async () => {
    const stale = structuredClone(errorFixture);
    stale.error.code = "product_source_stale";
    stale.error.field = "node_status";
    stale.error.summary = "策略产品数据源已过期，需要刷新对应来源后重试";
    stale.error.retryable = true;
    server.use(
      http.get("/api/product/v1/runs", () =>
        HttpResponse.json(stale, { status: 503 }),
      ),
    );

    renderWorkbench("/backtests");
    expect(await screen.findByText("产品服务返回错误")).toBeInTheDocument();
    expect(
      screen.getByText(/策略产品数据源已过期，需要刷新对应来源后重试/),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("当前版本没有已登记 Backtest 数据集"),
    ).not.toBeInTheDocument();
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
