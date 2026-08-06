import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { createRequire } from "node:module";

const playwrightPath = process.env.NTPRO_PLAYWRIGHT_CORE_PATH;
if (!playwrightPath) throw new Error("NTPRO_PLAYWRIGHT_CORE_PATH is required");
const require = createRequire(import.meta.url);
const { chromium } = require(playwrightPath);
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
const dist = path.resolve("apps/strategy-workbench/dist");
fs.mkdirSync(evidenceDir, { recursive: true });

const redact = (value) =>
  value.replace(/(access_token=)[^\s&]+/g, "$1[REDACTED]");
const serverLog = [];
const port = await new Promise((resolve, reject) => {
  const listener = net.createServer();
  listener.once("error", reject);
  listener.listen(0, "127.0.0.1", () => {
    const address = listener.address();
    listener.close((error) =>
      error ? reject(error) : resolve(address.port),
    );
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
let failure;
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
      if (!token) throw new Error("strategy bootstrap URL omitted access_token");
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
    unauthorizedProductBody.schema_version !==
      "ntpro.product_api.error.v1" ||
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
    throw new Error(`product strategy list contract drift: ${JSON.stringify(productList)}`);
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
    productDetail.data?.default_version_id !== productStrategy.default_version_id
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
    productRunList.schema_version !== "ntpro.product_api.run_list.response.v1" ||
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
    runCapabilityFields.some((field) => liveRun?.capabilities?.[field] !== false) ||
    expectedFalseBoundaries.some(
      (field) => productRunList.boundaries?.[field] !== false,
    )
  ) {
    throw new Error(`product run list contract drift: ${JSON.stringify(productRunList)}`);
  }

  const productRunDetailResponse = await fetch(
    `${baseUrl}/api/product/v1/runs/ema-cross-btcusdt-live-v1`,
    { headers: { cookie: institutionCookie } },
  );
  const productRunDetail = await productRunDetailResponse.json();
  if (
    productRunDetailResponse.status !== 200 ||
    productRunDetail.schema_version !== "ntpro.product_api.run_detail.response.v1" ||
    productRunDetail.data?.run_id !== liveRun.run_id ||
    productRunDetail.data?.source?.source_type !== "run_manifest" ||
    productRunDetail.data?.source?.freshness_status !== "fresh"
  ) {
    throw new Error(`product run detail contract drift: ${JSON.stringify(productRunDetail)}`);
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
  for (const [method, url, expected] of [
    ["GET", "/strategy-workbench/system-status", 200],
    ["POST", "/strategy-workbench/overview", 405],
    ["GET", "/strategy-workbench/assets/missing.js", 404],
    ["GET", "/api/product/v1/unknown", 404],
    ["POST", "/api/product/v1/strategies", 405],
    ["POST", "/api/product/v1/runs", 405],
    [
      "POST",
      "/api/product/v1/strategies/ema_cross_btcusdt_v1/versions",
      405,
    ],
  ]) {
    const response = await fetch(`${baseUrl}${url}`, {
      method,
      headers: { cookie: institutionCookie },
    });
    if (response.status !== expected) {
      throw new Error(`${method} ${url} expected ${expected}, got ${response.status}`);
    }
    if (url.startsWith("/api/product/v1/") && method === "POST") {
      const body = await response.json();
      if (
        response.headers.get("allow") !== "GET" ||
        body.schema_version !== "ntpro.product_api.error.v1" ||
        body.error?.code !== "product_method_not_allowed" ||
        body.error?.retryable !== false ||
        !body.request_id
      ) {
        throw new Error(`product method error contract drift: ${JSON.stringify(body)}`);
      }
    }
  }

  browser = await chromium.launch({ executablePath: chrome, headless: true });
  const context = await browser.newContext({
    viewport: { width: 1440, height: 1000 },
  });
  const page = await context.newPage();
  const browserErrors = [];
  const productionAssets = new Set();
  page.on("pageerror", (error) => browserErrors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") browserErrors.push(message.text());
  });
  page.on("response", (response) => {
    const url = new URL(response.url());
    if (url.pathname.startsWith("/strategy-workbench/assets/")) {
      productionAssets.add(url.pathname);
    }
  });
  let scenario = "valid";
  await page.route("**/api/mvp/v1/status", async (route) => {
    if (scenario === "valid") return route.continue();
    if (scenario === "http_error") {
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

  await page.goto(strategyAccessUrl.toString(), { waitUntil: "networkidle" });
  if (new URL(page.url()).searchParams.has("access_token")) {
    throw new Error("bootstrap token remained in browser URL");
  }
  await page.getByText("策略状态已验证").waitFor();
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
      throw new Error(`${phase} canvas origin drift: ${JSON.stringify(layout)}`);
    }
  };
  await assertCanvasOrigin("initial");
  if (!(await page.getByTestId("strategy-name").textContent())) {
    throw new Error("strategy identity did not render");
  }
  for (const liveButton of await page
    .getByRole("button", { name: /Live/ })
    .all()) {
    if (!(await liveButton.isDisabled())) {
      throw new Error("Live mode must remain disabled");
    }
  }
  if (
    await page
      .getByRole("button", { name: /下单|撤单|改单|平仓/ })
      .count()
  ) {
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
      /\/strategy-workbench\/assets\/[^/]+-[A-Za-z0-9_-]{6,}\.js$/.test(
        asset,
      ),
    )
  ) {
    throw new Error("browser did not load a hashed production JavaScript asset");
  }

  await page.getByRole("tab", { name: "日志" }).click();
  await page.getByText("原始日志不在主产品面暴露").waitFor();
  await page.getByRole("button", { name: "收起详情栏" }).click();
  if ((await page.getByTestId("app-shell").getAttribute("class"))?.includes("drawerOpen")) {
    throw new Error("details drawer did not close");
  }
  await page.getByRole("button", { name: "展开详情栏" }).click();
  await assertCanvasOrigin("drawer-reopened");
  await page.screenshot({
    path: path.join(evidenceDir, "strategy-workbench-1440.png"),
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
  await page.getByText("策略状态已验证").waitFor();
  if ((await page.getByTestId("app-shell").getAttribute("class"))?.includes("drawerOpen")) {
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
    throw new Error(`390 viewport layout drift: ${JSON.stringify(mobileLayout)}`);
  }
  await page.screenshot({
    path: path.join(evidenceDir, "strategy-workbench-390.png"),
    fullPage: true,
  });

  scenario = "boundary";
  await page.getByRole("button", { name: "刷新共享状态" }).click();
  await page.getByText("策略工作台已阻断").waitFor();
  if ((await page.getByTestId("strategy-name").textContent()) !== "策略未加载") {
    throw new Error("boundary failure retained stale strategy identity");
  }
  await page.screenshot({
    path: path.join(evidenceDir, "strategy-workbench-blocked.png"),
    fullPage: true,
  });

  scenario = "http_error";
  await page.getByRole("button", { name: "刷新共享状态" }).click();
  await page.getByText("策略工作台已阻断").waitFor();
  if (browserErrors.length > 0) {
    throw new Error(`browser errors: ${browserErrors.join("; ")}`);
  }
} catch (error) {
  failure = error instanceof Error ? error : new Error(String(error));
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
  writeEvidence({ status: "fail", error: redact(failure.message) });
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
  product_run_error: 1,
  product_run_live_boundary: 1,
  product_run_access_control: 1,
  asset_404: 1,
  method_405: 1,
  valid: 1,
  boundary: 1,
  http_error: 1,
  dock: 1,
  drawer: 1,
  live_disabled: 1,
  bootstrap_url_clean: 1,
});
console.log(`strategy_workbench_browser=pass evidence=${evidenceDir}`);
