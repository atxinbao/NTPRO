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
const runAnalysisFixture = readProductFixture("run-analysis");
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
    analysis_ref: "artifact://backtests/backtest-browser-001/analysis.json",
    reproduction_ref: null,
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
const reproducedBacktest = {
  ...createdBacktest,
  run_id: "backtest-browser-reproduced-001",
  config_ref:
    "artifact://backtests/backtest-browser-reproduced-001/request.toml",
  account_ref: "account://simulated/backtest-browser-reproduced-001",
  result: {
    status: "available",
    result_ref:
      "artifact://backtests/backtest-browser-reproduced-001/summary.json",
    report_ref:
      "artifact://backtests/backtest-browser-reproduced-001/details.json",
    analysis_ref:
      "artifact://backtests/backtest-browser-reproduced-001/analysis.json",
    reproduction_ref:
      "artifact://backtests/backtest-browser-reproduced-001/reproduction.json",
  },
};
const reproductionProof = {
  schema_version: "ntpro.backtest_reproduction_proof.v1",
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
  input_equivalent: true,
  output_equivalent: true,
  user_initiated: true,
  automatic_retry_allowed: false,
  automatic_remediation_allowed: false,
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

const createdDemo = {
  ...baselineBacktest,
  run_id: "demo-browser-001",
  environment: "sandbox",
  data_ref: "market://sandbox/BTCUSDT.BINANCE",
  config_ref: "artifact://demo-runs/demo-browser-001/request.json",
  adapter_ref: "adapter://sandbox/fixture-stream",
  account_ref: "account://sandbox/acct-sandbox-001",
  venue_ref: "venue://sandbox/BINANCE",
  lifecycle: "created",
  result: {
    status: "pending",
    result_ref: null,
    report_ref: null,
    analysis_ref: null,
    reproduction_ref: null,
  },
  risk: {
    status: "pending",
    risk_ref: "artifact://demo-runs/demo-browser-001/run-manifest.json#risk",
  },
  started_at_unix_ms: null,
  completed_at_unix_ms: null,
  runtime: {
    supervisor_node_id: "mvp-node-001",
    strategy_instance_id: "mvp-strategy-001",
    process_state: "not_started",
    lifecycle_state: "stopped",
  },
  source: {
    source_type: "run_manifest",
    freshness_status: "fresh",
    source_refs: [
      "mvp/identity_contract.json",
      "mvp/status_contract.json",
      "artifact://demo-runs/demo-browser-001/run-manifest.json",
    ],
  },
};

const demoBoundaries = {
  demo_run_creation_allowed: true,
  demo_start_allowed: true,
  demo_stop_allowed: true,
  live_run_creation_allowed: false,
  external_venue_connection: false,
  order_submission_allowed: false,
  order_mutation_allowed: false,
  automatic_retry_allowed: false,
  automatic_remediation_allowed: false,
  real_orders_submitted: false,
  trading_controls_enabled: false,
};

function demoSnapshotFixture(run: Record<string, unknown>) {
  const lifecycle = String(run.lifecycle);
  const status =
    lifecycle === "created"
      ? "not_started"
      : lifecycle === "stopped" || lifecycle === "failed"
        ? "frozen"
        : "running";
  const hasRuntimeData = status !== "not_started";
  const frozen = status === "frozen";
  const runtime = run.runtime as Record<string, unknown>;
  return {
    schema_version: "ntpro.product_api.demo_run_snapshot.response.v1",
    contract_version: "ntpro.product_api.v1",
    request_id: "product-0000000000000001-0000000000000013",
    data: {
      schema_version: "ntpro.product_api.demo_run_result.v1",
      run_id: run.run_id,
      strategy_id: run.strategy_id,
      strategy_version_id: run.strategy_version_id,
      observed_at_unix_ms: 1_786_400_001_000,
      lifecycle,
      snapshot_status: status,
      runtime: {
        supervisor_node_id: runtime.supervisor_node_id,
        strategy_instance_id: runtime.strategy_instance_id,
        process_state: runtime.process_state,
        lifecycle_state: runtime.lifecycle_state,
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
            event_count: 8,
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
            market_event_count: 8,
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
      technical_health: {
        status: hasRuntimeData ? "healthy" : "blocked",
        diagnostics: hasRuntimeData ? [] : ["demo_not_started"],
      },
      provenance: {
        source_refs: [
          `artifact://demo-runs/${String(run.run_id)}/run-manifest.json`,
        ],
        manifest_sha256:
          "sha256:1111111111111111111111111111111111111111111111111111111111111111",
        result_ref: frozen
          ? `artifact://demo-runs/${String(run.run_id)}/demo-result.json`
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

const comparisonItem = (runId: string) => ({
  run_id: runId,
  strategy_version_id: (runMetricsFixture.data as Record<string, unknown>)
    .strategy_version_id,
  data_ref: (runMetricsFixture.data as Record<string, unknown>).data_ref,
  data_sha256: (runMetricsFixture.data as Record<string, unknown>).data_sha256,
  config_sha256: (runMetricsFixture.data as Record<string, unknown>)
    .config_sha256,
  instrument_id: (runMetricsFixture.data as Record<string, unknown>)
    .instrument_id,
  parameters: (runMetricsFixture.data as Record<string, unknown>).parameters,
  metrics: (runMetricsFixture.data as Record<string, unknown>).metrics,
  risk: (runAnalysisFixture.data as Record<string, unknown>).risk,
  provenance: (runAnalysisFixture.data as Record<string, unknown>).provenance,
  reproduction_ref: null,
});

const comparisonResponse = {
  schema_version: "ntpro.product_api.run_comparison.response.v1",
  contract_version: "ntpro.product_api.v1",
  request_id: "product-0000000000000001-0000000000000002",
  data: {
    baseline_run_id: baselineBacktest.run_id,
    run_ids: [baselineBacktest.run_id, createdBacktest.run_id],
    items: [
      comparisonItem(String(baselineBacktest.run_id)),
      comparisonItem(createdBacktest.run_id),
    ],
    compatibility: {
      same_strategy: true,
      same_strategy_version: true,
      same_data: true,
      same_instrument: true,
      same_currency: true,
      directly_comparable: true,
    },
  },
  boundaries: runDetailFixture.boundaries,
};

const reproductionResponse = {
  schema_version: "ntpro.product_api.run_reproduction.response.v1",
  contract_version: "ntpro.product_api.v1",
  request_id: "product-0000000000000001-0000000000000003",
  data: {
    source_run_id: createdBacktest.run_id,
    reproduced_run: reproducedBacktest,
    proof: reproductionProof,
  },
  boundaries: createdBacktestResponse.boundaries,
};

const reproductionProofResponse = {
  schema_version: "ntpro.product_api.run_reproduction_proof.response.v1",
  contract_version: "ntpro.product_api.v1",
  request_id: "product-0000000000000001-0000000000000004",
  data: reproductionProof,
  boundaries: runDetailFixture.boundaries,
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
  if (path === "/api/product/v1/runs") {
    return {
      ...runListFixture,
      data: [
        ...(runListFixture.data as Array<Record<string, unknown>>),
        createdBacktest,
      ],
      page: {
        ...(runListFixture.page as Record<string, unknown>),
        returned_count:
          (runListFixture.data as Array<Record<string, unknown>>).length + 1,
      },
    };
  }
  if (path === "/api/product/v1/run-comparisons") {
    return comparisonResponse;
  }
  if (path === "/api/product/v1/runs/backtest-001/metrics") {
    return runMetricsFixture;
  }
  if (path === "/api/product/v1/runs/backtest-001/report") {
    return runReportFixture;
  }
  if (path === "/api/product/v1/runs/backtest-001/analysis") {
    return runAnalysisFixture;
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
  if (path === "/api/product/v1/runs/backtest-browser-001/analysis") {
    return {
      ...runAnalysisFixture,
      data: {
        ...(runAnalysisFixture.data as Record<string, unknown>),
        run_id: "backtest-browser-001",
        analysis_ref: (createdBacktest.result as Record<string, unknown>)
          .analysis_ref,
        provenance: {
          ...((runAnalysisFixture.data as Record<string, unknown>)
            .provenance as Record<string, unknown>),
          config_ref: createdBacktest.config_ref,
          summary_ref: (createdBacktest.result as Record<string, unknown>)
            .result_ref,
          details_ref: (createdBacktest.result as Record<string, unknown>)
            .report_ref,
        },
      },
    };
  }
  if (path === "/api/product/v1/runs/backtest-browser-reproduced-001") {
    return { ...runDetailFixture, data: reproducedBacktest };
  }
  if (path === "/api/product/v1/runs/backtest-browser-reproduced-001/metrics") {
    return {
      ...runMetricsFixture,
      data: {
        ...(runMetricsFixture.data as Record<string, unknown>),
        run_id: reproducedBacktest.run_id,
        config_ref: reproducedBacktest.config_ref,
        result_ref: reproducedBacktest.result.result_ref,
      },
    };
  }
  if (path === "/api/product/v1/runs/backtest-browser-reproduced-001/report") {
    return {
      ...runReportFixture,
      data: {
        ...(runReportFixture.data as Record<string, unknown>),
        run_id: reproducedBacktest.run_id,
        config_ref: reproducedBacktest.config_ref,
        details_ref: reproducedBacktest.result.report_ref,
      },
    };
  }
  if (
    path === "/api/product/v1/runs/backtest-browser-reproduced-001/analysis"
  ) {
    return {
      ...runAnalysisFixture,
      data: {
        ...(runAnalysisFixture.data as Record<string, unknown>),
        run_id: reproducedBacktest.run_id,
        analysis_ref: reproducedBacktest.result.analysis_ref,
        provenance: {
          ...((runAnalysisFixture.data as Record<string, unknown>)
            .provenance as Record<string, unknown>),
          config_ref: reproducedBacktest.config_ref,
          summary_ref: reproducedBacktest.result.result_ref,
          details_ref: reproducedBacktest.result.report_ref,
        },
      },
    };
  }
  if (
    path === "/api/product/v1/runs/backtest-browser-reproduced-001/reproduction"
  ) {
    return reproductionProofResponse;
  }
  if (path === "/api/product/v1/runs/ema-cross-live-001") {
    return runDetailFixture;
  }
  return errorFixture;
}

test.beforeEach(async ({ page }) => {
  let currentDemo: Record<string, unknown> | undefined;
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
      path === "/api/product/v1/demo-runs"
    ) {
      currentDemo = structuredClone(createdDemo);
      await route.fulfill({
        status: 201,
        contentType: "application/json",
        body: JSON.stringify({
          schema_version: "ntpro.product_api.demo_run_create.response.v1",
          contract_version: "ntpro.product_api.v1",
          request_id: "product-0000000000000001-0000000000000010",
          data: currentDemo,
          boundaries: demoBoundaries,
        }),
      });
      return;
    }
    if (
      route.request().method() === "POST" &&
      path === "/api/product/v1/demo-runs/demo-browser-001/actions" &&
      currentDemo
    ) {
      const request = route.request().postDataJSON() as { action: string };
      const previousLifecycle = currentDemo.lifecycle;
      const running = request.action === "start";
      currentDemo = {
        ...currentDemo,
        lifecycle: running ? "running" : "stopped",
        started_at_unix_ms: 1_786_400_000_000,
        completed_at_unix_ms: running ? null : 1_786_400_001_000,
        risk: {
          ...(currentDemo.risk as Record<string, unknown>),
          status: running ? "active" : "blocked",
        },
        runtime: {
          ...(currentDemo.runtime as Record<string, unknown>),
          process_state: running ? "running" : "stopped",
          lifecycle_state: running ? "running" : "stopped",
        },
      };
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          schema_version: "ntpro.product_api.demo_run_action.response.v1",
          contract_version: "ntpro.product_api.v1",
          request_id:
            request.action === "start"
              ? "product-0000000000000001-0000000000000011"
              : "product-0000000000000001-0000000000000012",
          data: {
            run_id: "demo-browser-001",
            action: request.action,
            previous_lifecycle: previousLifecycle,
            current_run: currentDemo,
          },
          boundaries: demoBoundaries,
        }),
      });
      return;
    }
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
    if (
      route.request().method() === "POST" &&
      path === "/api/product/v1/runs/backtest-browser-001/reproduction"
    ) {
      await route.fulfill({
        status: 201,
        contentType: "application/json",
        body: JSON.stringify(reproductionResponse),
      });
      return;
    }
    if (route.request().method() === "GET" && path === "/api/product/v1/runs") {
      const response = productFixtureForPath(path);
      const data = response.data as Array<Record<string, unknown>>;
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          ...response,
          data: currentDemo ? [...data, currentDemo] : data,
          page: {
            ...(response.page as Record<string, unknown>),
            returned_count: data.length + (currentDemo ? 1 : 0),
          },
        }),
      });
      return;
    }
    if (
      route.request().method() === "GET" &&
      path === "/api/product/v1/runs/demo-browser-001/demo-snapshot" &&
      currentDemo
    ) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(demoSnapshotFixture(currentDemo)),
      });
      return;
    }
    if (
      route.request().method() === "GET" &&
      path === "/api/product/v1/runs/demo-browser-001" &&
      currentDemo
    ) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({ ...runDetailFixture, data: currentDemo }),
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

test("Demo page creates a Run and explicitly controls Supervisor lifecycle", async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto("demo");
  await expect(
    page.getByRole("heading", { name: "Sandbox 策略运行" }),
  ).toBeVisible();
  await expect(page.getByText("真实订单关闭")).toBeVisible();
  const createButton = page.getByRole("button", { name: "创建 Demo Run" });
  await expect(createButton).toBeDisabled();
  await page.getByRole("checkbox", { name: /我确认创建 Demo Run/ }).check();
  await createButton.click();
  await expect(
    page.getByRole("heading", { name: "demo-browser-001" }),
  ).toBeVisible();
  await expect(
    page.getByRole("region", { name: "Demo 生命周期" }),
  ).toContainText("not_started");

  await page.getByRole("button", { name: "启动" }).click();
  await expect(
    page.getByRole("region", { name: "Demo 生命周期" }),
  ).toContainText("running");
  await expect(
    page.getByRole("region", { name: "Demo 运行结果" }),
  ).toContainText("实时策略快照");
  await expect(
    page.getByRole("region", { name: "Demo 运行结果" }),
  ).toContainText("sell");
  await page.getByRole("button", { name: "停止" }).click();
  await expect(
    page.getByRole("region", { name: "Demo 生命周期" }),
  ).toContainText("stopped");
  await expect(
    page.getByRole("region", { name: "Demo 运行结果" }),
  ).toContainText("终态冻结快照");
  await expect(
    page.getByRole("region", { name: "Demo 运行结果" }),
  ).toContainText("sha256:2222");
  await expect(
    page.getByRole("button", { name: /下单|撤单|改单|平仓/ }),
  ).toHaveCount(0);
  expect(
    await page.evaluate(
      () =>
        document.documentElement.scrollWidth <=
        document.documentElement.clientWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: testInfo.outputPath("strategy-workbench-demo-lifecycle-1440.png"),
    fullPage: true,
  });
});

test("Backtest comparison reproduces a Run only after explicit confirmation", async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto("backtests/compare");
  await expect(
    page.getByRole("heading", { name: "多 Run 对比与确定性复现" }),
  ).toBeVisible();
  await expect(page.getByRole("region", { name: "比较兼容性" })).toContainText(
    "结果可直接比较",
  );
  await expect(
    page.getByRole("region", { name: "Backtest 比较结果" }),
  ).toContainText("backtest-browser-001");
  await page.screenshot({
    path: testInfo.outputPath("strategy-workbench-backtest-compare-1440.png"),
    fullPage: true,
  });

  await page.getByRole("button", { name: /backtest-browser-001/ }).click();
  const createButton = page.getByRole("button", { name: "创建复现 Run" });
  await expect(createButton).toBeDisabled();
  await page
    .getByRole("checkbox", { name: /我确认这是一次用户主动的确定性复现/ })
    .check();
  await createButton.click();
  await expect(
    page.getByRole("heading", { name: "backtest-browser-reproduced-001" }),
  ).toBeVisible();
  await expect(
    page.getByRole("region", { name: "Backtest 确定性复现证明" }),
  ).toContainText("输入与输出均等价");
  expect(
    await page.evaluate(
      () =>
        document.documentElement.scrollWidth <=
        document.documentElement.clientWidth,
    ),
  ).toBe(true);
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
  await expect(
    page.getByRole("img", { name: "账户权益回撤随回测时间变化" }),
  ).toBeVisible();
  await expect(
    page.getByRole("region", { name: "Backtest 运行记录" }),
  ).toContainText("运行开始");
  await expect(
    page.getByRole("region", { name: "Backtest 分析来源" }),
  ).toContainText("artifact://backtests/backtest-001/summary.json");
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
  await expect(
    page.getByRole("img", { name: "账户权益回撤随回测时间变化" }),
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
