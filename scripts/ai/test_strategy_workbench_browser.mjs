import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { spawn, spawnSync } from "node:child_process";
import { createRequire } from "node:module";

const playwrightPath = process.env.NTPRO_PLAYWRIGHT_CORE_PATH;
if (!playwrightPath) throw new Error("NTPRO_PLAYWRIGHT_CORE_PATH is required");
const require = createRequire(import.meta.url);
const { chromium } = require(path.resolve(playwrightPath));
const chrome =
  process.env.NTPRO_CHROME_BIN ||
  (process.platform === "darwin"
    ? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
    : "google-chrome");

const root = fs.mkdtempSync(path.join(os.tmpdir(), "ntpro-fei-001-browser-"));
const evidenceDir =
  process.env.NTPRO_BROWSER_EVIDENCE_DIR || path.join(root, "evidence");
const workspace = path.join(root, "workspace");
const config = path.resolve("configs/nodes/btc-ema-shadow.toml");
const backtestConfig = path.resolve(
  "configs/backtests/ema-cross-btcusdt-product.toml",
);
const dist = path.resolve("apps/strategy-workbench/dist");
fs.mkdirSync(evidenceDir, { recursive: true });
const backtestRunId = "ema-cross-btcusdt-baseline-v1";
const backtestOutput = path.join(
  workspace,
  "artifacts",
  "backtests",
  backtestRunId,
);
const backtest = spawnSync(
  "target/debug/nautilus",
  ["backtest", "run", "--config", backtestConfig, "--output", backtestOutput],
  { encoding: "utf8" },
);
if (backtest.status !== 0) {
  throw new Error(
    `product backtest failed before browser smoke: ${backtest.stdout}${backtest.stderr}`,
  );
}

const redact = (value) =>
  value.replace(/(access_token=)[^\s&]+/g, "$1[REDACTED]");
const serverLog = [];
const port = await new Promise((resolve, reject) => {
  const listener = net.createServer();
  listener.once("error", reject);
  listener.listen(0, "127.0.0.1", () => {
    const address = listener.address();
    listener.close((error) => (error ? reject(error) : resolve(address.port)));
  });
});
const baseUrl = `http://127.0.0.1:${port}`;
const server = spawn(
  "target/debug/nautilus",
  [
    "mvp",
    "serve",
    "--config",
    config,
    "--workspace",
    workspace,
    "--bind",
    `127.0.0.1:${port}`,
    "--strategy-workbench-dist",
    dist,
    "--ntpro-node-bin",
    "target/debug/ntpro-node",
    "--startup-timeout-ms",
    "10000",
    "--node-max-runtime-ms",
    "120000",
  ],
  { stdio: ["ignore", "pipe", "pipe"] },
);
server.stdout.on("data", (chunk) => serverLog.push(chunk.toString()));
server.stderr.on("data", (chunk) => serverLog.push(chunk.toString()));

let browser;
let page;
let failure;
const productResponseErrors = [];
let failurePageUrl;
let failurePageText;
const writeEvidence = (result) => {
  fs.writeFileSync(
    path.join(evidenceDir, "mvp-server.log"),
    redact(serverLog.join("")),
  );
  fs.writeFileSync(
    path.join(evidenceDir, "result.json"),
    `${JSON.stringify(result, null, 2)}\n`,
  );
};

try {
  let strategyAccessUrl;
  let payload;
  let institutionCookie;
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    const match = serverLog.join("").match(/strategy_workbench_url=(\S+)/);
    if (match) {
      strategyAccessUrl = new URL(match[1]);
      const token = strategyAccessUrl.searchParams.get("access_token");
      if (!token)
        throw new Error("strategy bootstrap URL omitted access_token");
      institutionCookie = `ntpro_mvp_institution_access=${token}`;
      const response = await fetch(`${baseUrl}/api/mvp/v1/status`, {
        headers: { cookie: institutionCookie },
      });
      if (response.ok) {
        payload = await response.json();
        break;
      }
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  if (!strategyAccessUrl || !payload || !institutionCookie) {
    throw new Error(
      `strategy workbench did not become ready:\n${redact(serverLog.join(""))}`,
    );
  }

  const unauthorized = await fetch(`${baseUrl}/strategy-workbench/overview`, {
    redirect: "manual",
  });
  if (unauthorized.status !== 403) {
    throw new Error(
      `unauthorized strategy page expected 403, got ${unauthorized.status}`,
    );
  }
  const unauthorizedProductApi = await fetch(
    `${baseUrl}/api/product/v1/strategies`,
  );
  const unauthorizedProductBody = await unauthorizedProductApi.json();
  if (
    unauthorizedProductApi.status !== 403 ||
    unauthorizedProductBody.schema_version !== "ntpro.product_api.error.v1" ||
    unauthorizedProductBody.contract_version !== "ntpro.product_api.v1" ||
    unauthorizedProductBody.error?.code !== "product_access_denied" ||
    unauthorizedProductBody.error?.retryable !== false ||
    !unauthorizedProductBody.request_id ||
    unauthorizedProductBody.boundaries?.read_only !== true ||
    unauthorizedProductBody.boundaries?.order_submission_allowed !== false
  ) {
    throw new Error(
      `unauthorized product API contract drift: status=${unauthorizedProductApi.status} body=${JSON.stringify(unauthorizedProductBody)}`,
    );
  }
  const unauthorizedRunApi = await fetch(`${baseUrl}/api/product/v1/runs`);
  const unauthorizedRunBody = await unauthorizedRunApi.json();
  if (
    unauthorizedRunApi.status !== 403 ||
    unauthorizedRunBody.error?.code !== "product_access_denied" ||
    unauthorizedRunBody.error?.retryable !== false ||
    unauthorizedRunBody.boundaries?.run_mutation_allowed !== false
  ) {
    throw new Error(
      `unauthorized run API contract drift: status=${unauthorizedRunApi.status} body=${JSON.stringify(unauthorizedRunBody)}`,
    );
  }
  const unauthorizedRunCreateApi = await fetch(
    `${baseUrl}/api/product/v1/runs`,
    {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: "{}",
    },
  );
  const unauthorizedRunCreateBody = await unauthorizedRunCreateApi.json();
  if (
    unauthorizedRunCreateApi.status !== 403 ||
    unauthorizedRunCreateBody.error?.code !== "product_access_denied" ||
    unauthorizedRunCreateBody.error?.retryable !== false
  ) {
    throw new Error(
      `unauthorized run creation did not fail closed: ${JSON.stringify(unauthorizedRunCreateBody)}`,
    );
  }
  const unauthorizedMetricsApi = await fetch(
    `${baseUrl}/api/product/v1/runs/${backtestRunId}/metrics`,
  );
  const unauthorizedMetricsBody = await unauthorizedMetricsApi.json();
  if (
    unauthorizedMetricsApi.status !== 403 ||
    unauthorizedMetricsBody.error?.code !== "product_access_denied" ||
    unauthorizedMetricsBody.boundaries?.read_only !== true ||
    unauthorizedMetricsBody.boundaries?.run_mutation_allowed !== false
  ) {
    throw new Error(
      `unauthorized run metrics contract drift: status=${unauthorizedMetricsApi.status} body=${JSON.stringify(unauthorizedMetricsBody)}`,
    );
  }

  const productListResponse = await fetch(
    `${baseUrl}/api/product/v1/strategies?limit=1&sort=updated_at&order=desc`,
    { headers: { cookie: institutionCookie } },
  );
  if (productListResponse.status !== 200) {
    throw new Error(
      `product strategy list expected 200, got ${productListResponse.status}`,
    );
  }
  const productList = await productListResponse.json();
  const productStrategy = productList.data?.[0];
  if (
    productList.schema_version !==
      "ntpro.product_api.strategy_list.response.v1" ||
    productList.contract_version !== "ntpro.product_api.v1" ||
    productStrategy?.strategy_id !== "ema_cross_btcusdt_v1" ||
    productStrategy?.name !== "BTC/USDT EMA Cross" ||
    productStrategy?.default_version_id !== "ema_cross_btcusdt_v1@v1" ||
    productList.page?.returned_count !== 1
  ) {
    throw new Error(
      `product strategy list contract drift: ${JSON.stringify(productList)}`,
    );
  }
  const expectedFalseBoundaries = [
    "strategy_mutation_allowed",
    "run_mutation_allowed",
    "external_venue_connection",
    "order_submission_allowed",
    "order_mutation_allowed",
    "automatic_retry_allowed",
    "automatic_remediation_allowed",
    "real_orders_submitted",
    "trading_controls_enabled",
  ];
  if (
    productList.boundaries?.read_only !== true ||
    expectedFalseBoundaries.some(
      (field) => productList.boundaries?.[field] !== false,
    )
  ) {
    throw new Error(
      `product strategy list boundary drift: ${JSON.stringify(productList.boundaries)}`,
    );
  }

  const productDetailResponse = await fetch(
    `${baseUrl}/api/product/v1/strategies/ema_cross_btcusdt_v1`,
    { headers: { cookie: institutionCookie } },
  );
  if (productDetailResponse.status !== 200) {
    throw new Error(
      `product strategy detail expected 200, got ${productDetailResponse.status}`,
    );
  }
  const productDetail = await productDetailResponse.json();
  if (
    productDetail.schema_version !==
      "ntpro.product_api.strategy_detail.response.v1" ||
    productDetail.data?.strategy_id !== productStrategy.strategy_id ||
    productDetail.data?.default_version_id !==
      productStrategy.default_version_id
  ) {
    throw new Error(
      `product strategy detail contract drift: ${JSON.stringify(productDetail)}`,
    );
  }

  const productVersionListResponse = await fetch(
    `${baseUrl}/api/product/v1/strategies/ema_cross_btcusdt_v1/versions?limit=1&sort=created_at&order=desc&status=registered`,
    { headers: { cookie: institutionCookie } },
  );
  if (productVersionListResponse.status !== 200) {
    throw new Error(
      `product strategy version list expected 200, got ${productVersionListResponse.status}`,
    );
  }
  const productVersionList = await productVersionListResponse.json();
  const productVersion = productVersionList.data?.[0];
  if (
    productVersionList.schema_version !==
      "ntpro.product_api.strategy_version_list.response.v1" ||
    productVersionList.contract_version !== "ntpro.product_api.v1" ||
    productVersion?.strategy_version_id !== "ema_cross_btcusdt_v1@v1" ||
    productVersion?.strategy_id !== productStrategy.strategy_id ||
    productVersion?.version !== "v1" ||
    !/^sha256:[0-9a-f]{64}$/.test(productVersion?.content_hash || "") ||
    productVersion?.code_ref !==
      "git://NTPRO@e24de1825b66f9e7b9bfb2fc4662c928e56d6c18/crates/cli/src/strategy_session.rs#ema_cross_demo" ||
    productVersion?.parameter_schema?.additionalProperties !== false ||
    productVersion?.data_requirements?.deterministic_replay_required !== true ||
    productVersion?.risk_config?.kill_switch_required !== true ||
    productVersion?.risk_config?.order_submission_default !== false ||
    productVersion?.status !== "registered" ||
    productVersionList.page?.returned_count !== 1
  ) {
    throw new Error(
      `product strategy version list contract drift: ${JSON.stringify(productVersionList)}`,
    );
  }
  if (
    productVersionList.boundaries?.read_only !== true ||
    expectedFalseBoundaries.some(
      (field) => productVersionList.boundaries?.[field] !== false,
    )
  ) {
    throw new Error(
      `product strategy version boundary drift: ${JSON.stringify(productVersionList.boundaries)}`,
    );
  }

  const productVersionDetailResponse = await fetch(
    `${baseUrl}/api/product/v1/strategies/ema_cross_btcusdt_v1/versions/ema_cross_btcusdt_v1@v1`,
    { headers: { cookie: institutionCookie } },
  );
  if (productVersionDetailResponse.status !== 200) {
    throw new Error(
      `product strategy version detail expected 200, got ${productVersionDetailResponse.status}`,
    );
  }
  const productVersionDetail = await productVersionDetailResponse.json();
  if (
    productVersionDetail.schema_version !==
      "ntpro.product_api.strategy_version_detail.response.v1" ||
    productVersionDetail.data?.strategy_version_id !==
      productVersion.strategy_version_id ||
    productVersionDetail.data?.content_hash !== productVersion.content_hash
  ) {
    throw new Error(
      `product strategy version detail contract drift: ${JSON.stringify(productVersionDetail)}`,
    );
  }

  const liveAdmissionResponse = await fetch(
    `${baseUrl}/api/product/v1/strategies/ema_cross_btcusdt_v1/versions/ema_cross_btcusdt_v1@v1/live-admission`,
    { headers: { cookie: institutionCookie } },
  );
  const liveAdmission = await liveAdmissionResponse.json();
  if (
    liveAdmissionResponse.status !== 200 ||
    liveAdmission.schema_version !==
      "ntpro.product_api.live_admission.response.v1" ||
    liveAdmission.data?.strategy_version_id !==
      productVersion.strategy_version_id ||
    liveAdmission.data?.admission_status !== "blocked" ||
    liveAdmission.data?.venue?.venue_id !== "BINANCE" ||
    liveAdmission.data?.venue?.connection_state !== "not_attempted" ||
    liveAdmission.data?.credentials?.secret_values_exposed !== false ||
    !liveAdmission.data?.blockers?.includes(
      "automatic_recovery_not_authorized",
    ) ||
    liveAdmission.boundaries?.owner_approval_granted !== false ||
    liveAdmission.boundaries?.production_network_allowed !== false ||
    liveAdmission.boundaries?.external_network_attempted !== false ||
    liveAdmission.boundaries?.order_submission_allowed !== false ||
    liveAdmission.boundaries?.automatic_recovery_allowed !== false ||
    liveAdmission.boundaries?.real_orders_submitted !== false
  ) {
    throw new Error(
      `Live admission contract drift: status=${liveAdmissionResponse.status} body=${JSON.stringify(liveAdmission)}`,
    );
  }

  const liveAccountRefreshPath =
    "/api/product/v1/strategies/ema_cross_btcusdt_v1/versions/ema_cross_btcusdt_v1@v1/live-account/actions/refresh";
  const liveAccountRefreshResponse = await fetch(
    `${baseUrl}${liveAccountRefreshPath}`,
    {
      method: "POST",
      headers: {
        cookie: institutionCookie,
        "content-type": "application/json",
      },
      body: JSON.stringify({ action: "refresh" }),
    },
  );
  const liveAccountRefresh = await liveAccountRefreshResponse.json();
  if (
    liveAccountRefreshResponse.status !== 200 ||
    liveAccountRefresh.schema_version !==
      "ntpro.product_api.live_account_refresh.response.v1" ||
    liveAccountRefresh.data?.connection_status !== "blocked" ||
    liveAccountRefresh.data?.error_code !== "credentials_missing" ||
    liveAccountRefresh.data?.network_attempted !== false ||
    liveAccountRefresh.data?.account_read_attempted !== false ||
    liveAccountRefresh.data?.missing_runtime_gate_refs?.length !== 5 ||
    liveAccountRefresh.data?.shape_summary?.raw_account_response_exposed !==
      false ||
    liveAccountRefresh.data?.shape_summary?.raw_balances_exposed !== false ||
    liveAccountRefresh.boundaries?.external_network_attempted !== false ||
    liveAccountRefresh.boundaries?.account_mutation_allowed !== false ||
    liveAccountRefresh.boundaries?.order_endpoint_access_allowed !== false ||
    liveAccountRefresh.boundaries?.order_submission_allowed !== false ||
    liveAccountRefresh.boundaries?.automatic_retry_allowed !== false ||
    liveAccountRefresh.boundaries?.automatic_remediation_allowed !== false ||
    liveAccountRefresh.boundaries?.automatic_recovery_allowed !== false ||
    liveAccountRefresh.boundaries?.secret_values_exposed !== false ||
    liveAccountRefresh.boundaries?.raw_account_response_exposed !== false ||
    liveAccountRefresh.boundaries?.trading_controls_enabled !== false
  ) {
    throw new Error(
      `Live account refresh did not fail closed without runtime gates: status=${liveAccountRefreshResponse.status} body=${JSON.stringify(liveAccountRefresh)}`,
    );
  }

  const missingProductVersion = await fetch(
    `${baseUrl}/api/product/v1/strategies/ema_cross_btcusdt_v1/versions/ema_cross_btcusdt_v1@v2`,
    { headers: { cookie: institutionCookie } },
  );
  const missingProductVersionBody = await missingProductVersion.json();
  if (
    missingProductVersion.status !== 404 ||
    missingProductVersionBody.error?.code !== "strategy_version_not_found" ||
    missingProductVersionBody.error?.retryable !== false
  ) {
    throw new Error(
      `missing product strategy version did not fail closed: ${JSON.stringify(missingProductVersionBody)}`,
    );
  }

  const productRunListResponse = await fetch(
    `${baseUrl}/api/product/v1/runs?strategy_id=ema_cross_btcusdt_v1&strategy_version_id=ema_cross_btcusdt_v1%40v1&environment=live&lifecycle=created`,
    { headers: { cookie: institutionCookie } },
  );
  const productRunList = await productRunListResponse.json();
  const liveRun = productRunList.data?.[0];
  const runCapabilityFields = [
    "external_venue_connection",
    "order_submission_allowed",
    "order_mutation_allowed",
    "automatic_retry_allowed",
    "automatic_remediation_allowed",
    "real_orders_submitted",
    "trading_controls_enabled",
  ];
  if (
    productRunListResponse.status !== 200 ||
    productRunList.schema_version !==
      "ntpro.product_api.run_list.response.v1" ||
    productRunList.contract_version !== "ntpro.product_api.v1" ||
    productRunList.page?.returned_count !== 1 ||
    liveRun?.run_id !== "ema-cross-btcusdt-live-v1" ||
    liveRun?.strategy_id !== productStrategy.strategy_id ||
    liveRun?.strategy_version_id !== productVersion.strategy_version_id ||
    liveRun?.environment !== "live" ||
    liveRun?.lifecycle !== "created" ||
    liveRun?.result?.status !== "pending" ||
    liveRun?.risk?.status !== "blocked" ||
    liveRun?.adapter_ref !== "adapter://live/disabled" ||
    liveRun?.account_ref !== "account://live/unconfigured" ||
    runCapabilityFields.some(
      (field) => liveRun?.capabilities?.[field] !== false,
    ) ||
    expectedFalseBoundaries.some(
      (field) => productRunList.boundaries?.[field] !== false,
    )
  ) {
    throw new Error(
      `product run list contract drift: ${JSON.stringify(productRunList)}`,
    );
  }

  const productRunDetailResponse = await fetch(
    `${baseUrl}/api/product/v1/runs/ema-cross-btcusdt-live-v1`,
    { headers: { cookie: institutionCookie } },
  );
  const productRunDetail = await productRunDetailResponse.json();
  if (
    productRunDetailResponse.status !== 200 ||
    productRunDetail.schema_version !==
      "ntpro.product_api.run_detail.response.v1" ||
    productRunDetail.data?.run_id !== liveRun.run_id ||
    productRunDetail.data?.source?.source_type !== "run_manifest" ||
    productRunDetail.data?.source?.freshness_status !== "fresh"
  ) {
    throw new Error(
      `product run detail contract drift: ${JSON.stringify(productRunDetail)}`,
    );
  }

  const productRunMetricsResponse = await fetch(
    `${baseUrl}/api/product/v1/runs/${backtestRunId}/metrics`,
    { headers: { cookie: institutionCookie } },
  );
  const productRunMetrics = await productRunMetricsResponse.json();
  if (
    productRunMetricsResponse.status !== 200 ||
    productRunMetrics.schema_version !==
      "ntpro.product_api.run_metrics.response.v1" ||
    productRunMetrics.data?.run_id !== backtestRunId ||
    productRunMetrics.data?.instrument_id !== "BTCUSDT.BINANCE" ||
    productRunMetrics.data?.metrics?.quotes !== 120 ||
    productRunMetrics.data?.metrics?.iterations !== 120 ||
    productRunMetrics.data?.boundaries?.read_only !== true ||
    runCapabilityFields.some(
      (field) => productRunMetrics.data?.boundaries?.[field] !== false,
    )
  ) {
    throw new Error(
      `product run metrics contract drift: ${JSON.stringify(productRunMetrics)}`,
    );
  }

  const liveRunMetricsResponse = await fetch(
    `${baseUrl}/api/product/v1/runs/${liveRun.run_id}/metrics`,
    { headers: { cookie: institutionCookie } },
  );
  const liveRunMetrics = await liveRunMetricsResponse.json();
  if (
    liveRunMetricsResponse.status !== 404 ||
    liveRunMetrics.error?.code !== "run_not_found" ||
    liveRunMetrics.error?.field !== "run_metrics"
  ) {
    throw new Error(
      `non-backtest metrics did not fail closed: ${JSON.stringify(liveRunMetrics)}`,
    );
  }

  const backtestCreateRequest = {
    strategy_id: productStrategy.strategy_id,
    strategy_version_id: productVersion.strategy_version_id,
    environment: "backtest",
    data_ref: "dataset://fixtures/ema-cross-btcusdt-v1",
    venue_ref: "venue://simulated/BINANCE_TESTNET",
    starting_balance: "1000000 USDT",
    quotes: 120,
    trade_size: "0.001000",
    fast_period: 3,
    slow_period: 5,
  };
  const productRunCreateResponse = await fetch(
    `${baseUrl}/api/product/v1/runs`,
    {
      method: "POST",
      headers: {
        cookie: institutionCookie,
        "content-type": "application/json",
      },
      body: JSON.stringify(backtestCreateRequest),
    },
  );
  const productRunCreate = await productRunCreateResponse.json();
  const createdRun = productRunCreate.data;
  const creationBoundaryFields = [
    "sandbox_run_creation_allowed",
    "live_run_creation_allowed",
    "external_venue_connection",
    "order_submission_allowed",
    "order_mutation_allowed",
    "automatic_retry_allowed",
    "automatic_remediation_allowed",
    "real_orders_submitted",
    "trading_controls_enabled",
  ];
  if (
    productRunCreateResponse.status !== 201 ||
    productRunCreate.schema_version !==
      "ntpro.product_api.run_create.response.v1" ||
    productRunCreate.contract_version !== "ntpro.product_api.v1" ||
    !productRunCreate.request_id ||
    !createdRun?.run_id?.startsWith("backtest-") ||
    createdRun?.strategy_id !== backtestCreateRequest.strategy_id ||
    createdRun?.strategy_version_id !==
      backtestCreateRequest.strategy_version_id ||
    createdRun?.data_ref !== backtestCreateRequest.data_ref ||
    createdRun?.venue_ref !== backtestCreateRequest.venue_ref ||
    createdRun?.environment !== "backtest" ||
    createdRun?.lifecycle !== "completed" ||
    createdRun?.result?.status !== "available" ||
    !createdRun?.result?.result_ref ||
    productRunCreate.boundaries?.backtest_run_creation_allowed !== true ||
    creationBoundaryFields.some(
      (field) => productRunCreate.boundaries?.[field] !== false,
    )
  ) {
    throw new Error(
      `product run creation contract drift: ${JSON.stringify(productRunCreate)}`,
    );
  }
  const createdRunDetailResponse = await fetch(
    `${baseUrl}/api/product/v1/runs/${createdRun.run_id}`,
    { headers: { cookie: institutionCookie } },
  );
  const createdRunDetail = await createdRunDetailResponse.json();
  const createdRunMetricsResponse = await fetch(
    `${baseUrl}/api/product/v1/runs/${createdRun.run_id}/metrics`,
    { headers: { cookie: institutionCookie } },
  );
  const createdRunMetrics = await createdRunMetricsResponse.json();
  if (
    createdRunDetailResponse.status !== 200 ||
    createdRunDetail.data?.run_id !== createdRun.run_id ||
    createdRunMetricsResponse.status !== 200 ||
    createdRunMetrics.data?.run_id !== createdRun.run_id ||
    createdRunMetrics.data?.metrics?.quotes !== 120 ||
    createdRunMetrics.data?.metrics?.total_orders !== 5
  ) {
    throw new Error(
      `created run readback drift: detail=${JSON.stringify(createdRunDetail)} metrics=${JSON.stringify(createdRunMetrics)}`,
    );
  }

  const missingProductRun = await fetch(
    `${baseUrl}/api/product/v1/runs/missing`,
    { headers: { cookie: institutionCookie } },
  );
  const missingProductRunBody = await missingProductRun.json();
  if (
    missingProductRun.status !== 404 ||
    missingProductRunBody.error?.code !== "run_not_found" ||
    missingProductRunBody.error?.retryable !== false
  ) {
    throw new Error(
      `missing product run did not fail closed: ${JSON.stringify(missingProductRunBody)}`,
    );
  }

  const invalidProductQuery = await fetch(
    `${baseUrl}/api/product/v1/strategies?limit=1&limit=2`,
    { headers: { cookie: institutionCookie } },
  );
  const invalidProductBody = await invalidProductQuery.json();
  if (
    invalidProductQuery.status !== 400 ||
    invalidProductBody.error?.code !== "product_query_invalid" ||
    invalidProductBody.error?.retryable !== false ||
    !invalidProductBody.request_id
  ) {
    throw new Error(
      `invalid product query did not fail closed: ${JSON.stringify(invalidProductBody)}`,
    );
  }
  const malformedProductPath = await fetch(
    `${baseUrl}/api/product/v1/strategies/%FF`,
    { headers: { cookie: institutionCookie } },
  );
  const malformedProductPathBody = await malformedProductPath.json();
  if (
    malformedProductPath.status !== 400 ||
    malformedProductPathBody.error?.code !== "product_query_invalid" ||
    malformedProductPathBody.error?.field !== "strategy_id" ||
    !malformedProductPathBody.request_id
  ) {
    throw new Error(
      `malformed product path did not use the product error contract: ${JSON.stringify(malformedProductPathBody)}`,
    );
  }
  for (const [method, url, expected, expectedAllow] of [
    ["GET", "/strategy-workbench/system-status", 200, null],
    ["POST", "/strategy-workbench/overview", 405, null],
    ["GET", "/strategy-workbench/assets/missing.js", 404, null],
    ["GET", "/api/product/v1/unknown", 404, null],
    ["POST", "/api/product/v1/strategies", 405, "GET"],
    ["PUT", "/api/product/v1/runs", 405, "GET, POST"],
    ["POST", `/api/product/v1/runs/${backtestRunId}/metrics`, 405, "GET"],
    [
      "POST",
      "/api/product/v1/strategies/ema_cross_btcusdt_v1/versions",
      405,
      "GET",
    ],
    [
      "POST",
      "/api/product/v1/strategies/ema_cross_btcusdt_v1/versions/ema_cross_btcusdt_v1@v1/live-admission",
      405,
      "GET",
    ],
    ["GET", liveAccountRefreshPath, 405, "POST"],
  ]) {
    const response = await fetch(`${baseUrl}${url}`, {
      method,
      headers: { cookie: institutionCookie },
    });
    if (response.status !== expected) {
      throw new Error(
        `${method} ${url} expected ${expected}, got ${response.status}`,
      );
    }
    if (expectedAllow) {
      const body = await response.json();
      if (
        response.headers.get("allow") !== expectedAllow ||
        body.schema_version !== "ntpro.product_api.error.v1" ||
        body.error?.code !== "product_method_not_allowed" ||
        body.error?.retryable !== false ||
        !body.request_id
      ) {
        throw new Error(
          `product method error contract drift: ${JSON.stringify(body)}`,
        );
      }
    }
  }

  browser = await chromium.launch({ executablePath: chrome, headless: true });
  const context = await browser.newContext({
    viewport: { width: 1440, height: 1000 },
  });
  page = await context.newPage();
  const browserErrors = [];
  const productionAssets = new Set();
  let expectedHttpErrorResponses = 0;
  let liveAccountRefreshBrowserRequests = 0;
  page.on("pageerror", (error) => browserErrors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") browserErrors.push(message.text());
  });
  page.on("response", (response) => {
    const url = new URL(response.url());
    if (url.pathname.startsWith("/strategy-workbench/assets/")) {
      productionAssets.add(url.pathname);
    }
    if (url.pathname.startsWith("/api/product/") && response.status() >= 400) {
      productResponseErrors.push(`${response.status()} ${url.pathname}`);
    }
  });
  page.on("request", (request) => {
    if (
      request.method() === "POST" &&
      decodeURIComponent(new URL(request.url()).pathname) ===
        liveAccountRefreshPath
    ) {
      liveAccountRefreshBrowserRequests += 1;
    }
  });
  let scenario = "valid";
  await page.route("**/api/mvp/v1/status", async (route) => {
    if (scenario === "valid") return route.continue();
    if (scenario === "http_error") {
      expectedHttpErrorResponses += 1;
      return route.fulfill({
        status: 503,
        contentType: "application/json",
        body: "{}",
      });
    }
    const response = structuredClone(payload);
    response.boundaries.real_orders_submitted = true;
    return route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(response),
    });
  });

  await page.goto(strategyAccessUrl.toString(), {
    waitUntil: "domcontentloaded",
  });
  if (new URL(page.url()).searchParams.has("access_token")) {
    throw new Error("bootstrap token remained in browser URL");
  }
  await page.getByText("产品资源已验证").waitFor();
  const assertCanvasOrigin = async (phase) => {
    const layout = await page.evaluate(() => {
      const canvas = document.querySelector("main");
      const heading = canvas?.querySelector("h1");
      const rail = document.querySelector("aside");
      const stage = document.querySelector("main")?.closest("section");
      const canvasRect = canvas?.getBoundingClientRect();
      const headingRect = heading?.getBoundingClientRect();
      return {
        scrollLeft: canvas?.scrollLeft,
        scrollTop: canvas?.scrollTop,
        railRight: rail?.getBoundingClientRect().right,
        stageLeft: stage?.getBoundingClientRect().left,
        canvasLeft: canvasRect?.left,
        headingLeft: headingRect?.left,
        headingText: heading?.textContent,
      };
    });
    if (
      layout.scrollLeft !== 0 ||
      layout.scrollTop !== 0 ||
      layout.railRight === undefined ||
      layout.stageLeft === undefined ||
      layout.canvasLeft === undefined ||
      layout.headingLeft === undefined ||
      layout.stageLeft < layout.railRight ||
      layout.canvasLeft < layout.railRight ||
      layout.headingLeft < layout.canvasLeft
    ) {
      throw new Error(
        `${phase} canvas origin drift: ${JSON.stringify(layout)}`,
      );
    }
  };
  await assertCanvasOrigin("initial");
  if (
    (await page.getByTestId("strategy-name").textContent()) !==
    productStrategy.strategy_id
  ) {
    throw new Error("Product API strategy identity did not render");
  }
  const liveLink = page.getByRole("link", { name: "Live", exact: true });
  if ((await liveLink.getAttribute("href")) !== "/strategy-workbench/live") {
    throw new Error(
      "Live admission route is not available from the product shell",
    );
  }
  if (await page.getByRole("button", { name: /下单|撤单|改单|平仓/ }).count()) {
    throw new Error("trading control appeared in strategy shell");
  }
  if (
    await page.evaluate(
      () =>
        document.documentElement.scrollWidth >
        document.documentElement.clientWidth,
    )
  ) {
    throw new Error("1440 viewport has horizontal overflow");
  }
  if (
    ![...productionAssets].some((asset) =>
      /\/strategy-workbench\/assets\/[^/]+-[A-Za-z0-9_-]{6,}\.js$/.test(asset),
    )
  ) {
    throw new Error(
      "browser did not load a hashed production JavaScript asset",
    );
  }

  await page.getByRole("tab", { name: "日志" }).click();
  await page.getByText("原始技术日志不在主产品面暴露").waitFor();
  await page.getByRole("button", { name: "收起详情栏" }).click();
  if (
    (await page.getByTestId("app-shell").getAttribute("class"))?.includes(
      "drawerOpen",
    )
  ) {
    throw new Error("details drawer did not close");
  }
  await page.getByRole("button", { name: "展开详情栏" }).click();
  await assertCanvasOrigin("drawer-reopened");
  await page.screenshot({
    path: path.join(evidenceDir, "strategy-workbench-1440.png"),
    fullPage: true,
  });

  await liveLink.click();
  await page
    .getByRole("heading", { name: "Live 连接与独立准入" })
    .waitFor();
  await page.getByText("尚未获得 Live 独立审批").waitFor();
  if (liveAccountRefreshBrowserRequests !== 0) {
    throw new Error("Live page load issued a production account refresh");
  }
  const liveRefreshResponsePromise = page.waitForResponse((response) => {
    const url = new URL(response.url());
    return (
      response.request().method() === "POST" &&
      decodeURIComponent(url.pathname) === liveAccountRefreshPath
    );
  });
  await page.getByRole("button", { name: "检查账户连接" }).click();
  const liveRefreshBrowserResponse = await liveRefreshResponsePromise;
  const liveRefreshBrowserBody = await liveRefreshBrowserResponse.json();
  if (
    liveAccountRefreshBrowserRequests !== 1 ||
    liveRefreshBrowserResponse.status() !== 200 ||
    liveRefreshBrowserBody.data?.connection_status !== "blocked" ||
    liveRefreshBrowserBody.data?.network_attempted !== false ||
    liveRefreshBrowserBody.data?.account_read_attempted !== false
  ) {
    throw new Error(
      `browser Live account refresh did not remain blocked: status=${liveRefreshBrowserResponse.status()} body=${JSON.stringify(liveRefreshBrowserBody)}`,
    );
  }
  const liveAccountRegion = page.getByRole("region", {
    name: "生产账户只读连接",
  });
  await liveAccountRegion.getByText("已阻断", { exact: true }).waitFor();
  await liveAccountRegion.getByText("0/5").waitFor();
  await liveAccountRegion.getByText("未尝试", { exact: true }).waitFor();
  if (
    await page.getByRole("button", { name: /启动|下单|撤单|改单|平仓/ }).count()
  ) {
    throw new Error("trading control appeared on blocked Live admission page");
  }
  await page.screenshot({
    path: path.join(evidenceDir, "strategy-workbench-live-admission-1440.png"),
    fullPage: true,
  });

  await page.getByRole("link", { name: "Demo", exact: true }).click();
  await page.getByRole("heading", { name: "Sandbox 策略运行" }).waitFor();
  await page.getByRole("checkbox", { name: /我确认创建 Demo Run/ }).check();
  const demoCreateResponsePromise = page.waitForResponse((response) => {
    const url = new URL(response.url());
    return (
      response.request().method() === "POST" &&
      url.pathname === "/api/product/v1/demo-runs"
    );
  });
  await page.getByRole("button", { name: "创建 Demo Run" }).click();
  const demoCreateResponse = await demoCreateResponsePromise;
  const demoCreateBody = await demoCreateResponse.json();
  if (
    demoCreateResponse.status() !== 201 ||
    !demoCreateBody.data?.run_id?.startsWith("demo-")
  ) {
    throw new Error(
      `browser Demo creation failed: status=${demoCreateResponse.status()} body=${JSON.stringify(demoCreateBody)}`,
    );
  }
  const browserDemoRunId = demoCreateBody.data.run_id;
  const demoReadbackResponse = await fetch(
    `${baseUrl}/api/product/v1/runs/${browserDemoRunId}`,
    { headers: { cookie: institutionCookie } },
  );
  const demoReadbackBody = await demoReadbackResponse.json();
  if (
    demoReadbackResponse.status !== 200 ||
    demoReadbackBody.data?.run_id !== browserDemoRunId
  ) {
    throw new Error(
      `browser Demo readback failed: status=${demoReadbackResponse.status} body=${JSON.stringify(demoReadbackBody)}`,
    );
  }
  await page.waitForURL(
    (url) => url.pathname === `/strategy-workbench/runs/${browserDemoRunId}`,
  );
  if (!browserDemoRunId?.startsWith("demo-")) {
    throw new Error(`browser-created Demo Run ID drifted: ${page.url()}`);
  }
  const demoLifecycle = page.getByRole("region", { name: "Demo 生命周期" });
  const demoRunState = demoLifecycle
    .getByText("运行状态", {
      exact: true,
    })
    .locator("..");
  await demoLifecycle.waitFor();
  await page.getByRole("button", { name: "启动" }).click();
  await demoRunState.getByText("running", { exact: true }).waitFor();
  const demoResult = page.getByRole("region", { name: "Demo 运行结果" });
  await demoResult.waitFor();
  await demoResult.getByRole("heading", { name: "实时策略快照" }).waitFor();
  const demoTrades = page.getByRole("region", {
    name: "Demo 模拟 成交明细",
  });
  const demoPositions = page.getByRole("region", {
    name: "Demo 模拟 持仓明细",
  });
  const demoEquity = page.getByRole("region", {
    name: "Demo 模拟 资金曲线",
  });
  await demoTrades.waitFor();
  await demoPositions.waitFor();
  await demoEquity.waitFor();
  if (
    (await demoTrades.locator("tbody tr").count()) === 0 ||
    (await demoPositions.locator("tbody tr").count()) === 0 ||
    !(await demoEquity
      .getByRole("img", {
        name: "账户权益随回测时间变化",
      })
      .isVisible())
  ) {
    throw new Error("Demo simulation result panels are incomplete");
  }
  const runningResultHash = demoResult.getByText("结果哈希").locator("..");
  await runningResultHash.getByText("运行中", { exact: true }).waitFor();
  await page.screenshot({
    path: path.join(evidenceDir, "strategy-workbench-demo-running-1440.png"),
    fullPage: true,
  });
  await page.getByRole("button", { name: "停止" }).click();
  await demoRunState.getByText("stopped", { exact: true }).waitFor();
  await demoResult.getByRole("heading", { name: "终态冻结快照" }).waitFor();
  const frozenResultHash = await runningResultHash
    .locator("strong")
    .textContent();
  if (!frozenResultHash || !/^sha256:[a-f0-9]{64}$/.test(frozenResultHash)) {
    throw new Error(`Demo frozen result hash drifted: ${frozenResultHash}`);
  }
  await page.screenshot({
    path: path.join(evidenceDir, "strategy-workbench-demo-stopped-1440.png"),
    fullPage: true,
  });
  await page.getByRole("link", { name: "运行对比" }).click();
  await page
    .getByRole("heading", { name: "Backtest 与 Demo 行为对比" })
    .waitFor();
  const demoComparisonOption = page.getByRole("checkbox", {
    name: new RegExp(browserDemoRunId),
  });
  if (!(await demoComparisonOption.isChecked())) {
    await demoComparisonOption.check();
  }
  await page
    .getByRole("region", { name: "Run 比较结果" })
    .getByText(browserDemoRunId, { exact: true })
    .waitFor();
  if (
    !(await page
      .getByRole("button", { name: new RegExp(browserDemoRunId) })
      .isDisabled())
  ) {
    throw new Error("Demo comparison unexpectedly enabled reproduction");
  }
  await page.screenshot({
    path: path.join(evidenceDir, "strategy-workbench-demo-comparison-1440.png"),
    fullPage: true,
  });
  await page.getByRole("link", { name: "Backtest", exact: true }).click();
  await page.getByRole("heading", { name: "创建策略回测" }).waitFor();
  await page.screenshot({
    path: path.join(evidenceDir, "strategy-workbench-backtest-create-1440.png"),
    fullPage: true,
  });
  await page.getByRole("button", { name: "创建并运行" }).click();
  await page.waitForURL(/\/strategy-workbench\/runs\/backtest-/);
  const browserCreatedRunId = page.url().split("/").at(-1);
  if (!browserCreatedRunId?.startsWith("backtest-")) {
    throw new Error(`browser-created Run ID drifted: ${page.url()}`);
  }
  await page.getByRole("heading", { name: browserCreatedRunId }).waitFor();
  await page.getByText("真实引擎回测结果").waitFor();
  await page.screenshot({
    path: path.join(
      evidenceDir,
      "strategy-workbench-backtest-created-1440.png",
    ),
    fullPage: true,
  });
  await page.getByRole("link", { name: "返回策略总览" }).click();
  const baselineBacktestLink = page.getByRole("link", {
    name: new RegExp(backtestRunId),
  });
  await baselineBacktestLink.waitFor();
  await baselineBacktestLink.click();
  await page.getByRole("heading", { name: backtestRunId }).waitFor();
  await page.getByText("真实引擎回测结果").waitFor();
  await page.getByRole("region", { name: "Backtest 指标" }).waitFor();
  await page.screenshot({
    path: path.join(
      evidenceDir,
      "strategy-workbench-backtest-metrics-1440.png",
    ),
    fullPage: true,
  });
  await page.getByRole("link", { name: "返回策略总览" }).click();
  const liveRunLink = page.getByRole("link", {
    name: new RegExp(liveRun.run_id),
  });
  await liveRunLink.waitFor();
  await liveRunLink.click();
  await page.getByRole("heading", { name: liveRun.run_id }).waitFor();
  if (!page.url().endsWith(`/strategy-workbench/runs/${liveRun.run_id}`)) {
    throw new Error(`Run deep link drifted: ${page.url()}`);
  }
  if (
    (await page.getByTestId("strategy-name").textContent()) !==
    liveRun.strategy_id
  ) {
    throw new Error(
      "Run detail did not bind the Product API strategy identity",
    );
  }
  await page.getByText("当前 Run 禁止能力").waitFor();
  await page.reload({ waitUntil: "networkidle" });
  await page.getByRole("heading", { name: liveRun.run_id }).waitFor();
  await page.screenshot({
    path: path.join(evidenceDir, "strategy-workbench-run-detail-1440.png"),
    fullPage: true,
  });

  await page.getByRole("link", { name: "系统状态" }).click();
  await page.getByRole("heading", { name: "系统状态" }).waitFor();
  await page.reload({ waitUntil: "networkidle" });
  await page.getByRole("heading", { name: "系统状态" }).waitFor();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${baseUrl}/strategy-workbench/overview`, {
    waitUntil: "networkidle",
  });
  await page.getByText("产品资源已验证").waitFor();
  if (
    (await page.getByTestId("app-shell").getAttribute("class"))?.includes(
      "drawerOpen",
    )
  ) {
    throw new Error("mobile details drawer must default closed");
  }
  const mobileLayout = await page.evaluate(() => ({
    scrollX: window.scrollX,
    documentWidth: document.documentElement.scrollWidth,
    viewportWidth: document.documentElement.clientWidth,
  }));
  if (
    mobileLayout.documentWidth > mobileLayout.viewportWidth ||
    mobileLayout.scrollX !== 0
  ) {
    throw new Error(
      `390 viewport layout drift: ${JSON.stringify(mobileLayout)}`,
    );
  }
  const mobileRunTable = await page
    .getByTestId("run-table-scroll")
    .evaluate((element) => ({
      clientWidth: element.clientWidth,
      scrollWidth: element.scrollWidth,
    }));
  if (mobileRunTable.scrollWidth <= mobileRunTable.clientWidth) {
    throw new Error(
      `mobile Run table did not preserve readable columns: ${JSON.stringify(mobileRunTable)}`,
    );
  }
  await page.screenshot({
    path: path.join(evidenceDir, "strategy-workbench-390.png"),
    fullPage: true,
  });
  await page.goto(`${baseUrl}/strategy-workbench/runs/${backtestRunId}`, {
    waitUntil: "networkidle",
  });
  await page.getByText("真实引擎回测结果").waitFor();
  await page.getByLabel("Backtest 收益统计").waitFor();
  const mobileMetricsLayout = await page.evaluate(() => ({
    scrollX: window.scrollX,
    documentWidth: document.documentElement.scrollWidth,
    viewportWidth: document.documentElement.clientWidth,
  }));
  if (
    mobileMetricsLayout.documentWidth > mobileMetricsLayout.viewportWidth ||
    mobileMetricsLayout.scrollX !== 0
  ) {
    throw new Error(
      `390 Backtest metrics layout drift: ${JSON.stringify(mobileMetricsLayout)}`,
    );
  }
  await page.screenshot({
    path: path.join(evidenceDir, "strategy-workbench-backtest-metrics-390.png"),
    fullPage: true,
  });
  await page.goto(`${baseUrl}/strategy-workbench/overview`, {
    waitUntil: "networkidle",
  });
  await page.getByText("产品资源已验证").waitFor();

  scenario = "boundary";
  await page.getByRole("button", { name: "刷新产品与系统状态" }).click();
  await page.getByText("连接阻断").waitFor({ state: "attached" });
  await page.waitForFunction(
    (strategyId) =>
      document.querySelector('[data-testid="strategy-name"]')?.textContent ===
      strategyId,
    productStrategy.strategy_id,
  );
  await page.screenshot({
    path: path.join(evidenceDir, "strategy-workbench-blocked.png"),
    fullPage: true,
  });

  scenario = "http_error";
  const errorsBeforeHttpScenario = browserErrors.length;
  await page.getByRole("button", { name: "刷新产品与系统状态" }).click();
  await page.getByText("连接阻断").waitFor({ state: "attached" });
  if (expectedHttpErrorResponses < 1) {
    throw new Error("HTTP error scenario did not intercept the status request");
  }
  const httpScenarioErrors = browserErrors.slice(errorsBeforeHttpScenario);
  const expectedHttpConsoleErrors = httpScenarioErrors.filter(
    (message) =>
      message ===
      "Failed to load resource: the server responded with a status of 503 (Service Unavailable)",
  );
  if (expectedHttpConsoleErrors.length > expectedHttpErrorResponses) {
    throw new Error(
      `HTTP error scenario emitted unbound 503 console errors: ${expectedHttpConsoleErrors.length}/${expectedHttpErrorResponses}`,
    );
  }
  const unexpectedHttpScenarioErrors = httpScenarioErrors.filter(
    (message) =>
      message !==
      "Failed to load resource: the server responded with a status of 503 (Service Unavailable)",
  );
  const unexpectedBrowserErrors = [
    ...browserErrors.slice(0, errorsBeforeHttpScenario),
    ...unexpectedHttpScenarioErrors,
  ];
  if (unexpectedBrowserErrors.length > 0) {
    throw new Error(`browser errors: ${unexpectedBrowserErrors.join("; ")}`);
  }
} catch (error) {
  failure = error instanceof Error ? error : new Error(String(error));
  if (page) {
    failurePageUrl = page.url();
    failurePageText = await page
      .locator("body")
      .innerText()
      .catch(() => undefined);
    await page
      .screenshot({
        path: path.join(evidenceDir, "failure.png"),
        fullPage: true,
      })
      .catch(() => {});
  }
} finally {
  if (browser) await browser.close().catch(() => {});
  if (server.exitCode === null) server.kill("SIGINT");
  await new Promise((resolve) => {
    if (server.exitCode !== null) return resolve();
    const timer = setTimeout(() => {
      if (server.exitCode === null) server.kill("SIGKILL");
      resolve();
    }, 10_000);
    server.once("exit", () => {
      clearTimeout(timer);
      resolve();
    });
  });
}

if (failure) {
  writeEvidence({
    status: "fail",
    error: redact(failure.message),
    page_url: failurePageUrl,
    page_text: failurePageText ? redact(failurePageText) : undefined,
    product_response_errors: productResponseErrors,
  });
  throw failure;
}
writeEvidence({
  status: "pass",
  viewports: ["1440x1000", "390x844"],
  production_bundle: 1,
  hashed_asset: 1,
  spa_deep_refresh: 1,
  api_fallback_isolation: 1,
  product_strategy_list: 1,
  product_strategy_detail: 1,
  product_strategy_error: 1,
  product_strategy_access_control: 1,
  product_run_list: 1,
  product_run_detail: 1,
  product_run_metrics: 1,
  product_run_metrics_mobile: 1,
  product_run_create_api: 1,
  product_run_create_browser: 1,
  product_run_create_readback: 1,
  product_run_create_access_control: 1,
  product_run_metrics_non_backtest_closed: 1,
  product_run_error: 1,
  product_run_live_boundary: 1,
  product_run_access_control: 1,
  product_run_deep_link: 1,
  demo_snapshot_running: 1,
  demo_snapshot_frozen: 1,
  demo_simulation_results: 1,
  demo_backtest_comparison: 1,
  asset_404: 1,
  method_405: 1,
  valid: 1,
  boundary: 1,
  http_error: 1,
  dock: 1,
  drawer: 1,
  live_admission: 1,
  live_account_refresh: 1,
  bootstrap_url_clean: 1,
});
console.log(`strategy_workbench_browser=pass evidence=${evidenceDir}`);
