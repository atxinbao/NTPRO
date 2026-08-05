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
const chrome = process.env.NTPRO_CHROME_BIN || (process.platform === "darwin"
  ? "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
  : "google-chrome");

const root = fs.mkdtempSync(path.join(os.tmpdir(), "ntpro-swb-001-browser-"));
const evidenceDir = process.env.NTPRO_BROWSER_EVIDENCE_DIR || path.join(root, "evidence");
const workspace = path.join(root, "workspace");
const config = path.resolve("configs/nodes/btc-ema-shadow.toml");
fs.mkdirSync(evidenceDir, { recursive: true });

const redact = (value) => value.replace(/(access_token=)[^\s&]+/g, "$1[REDACTED]");
const serverLog = [];
const port = await new Promise((resolve, reject) => {
  const listener = net.createServer();
  listener.once("error", reject);
  listener.listen(0, "127.0.0.1", () => {
    const address = listener.address();
    listener.close((error) => error ? reject(error) : resolve(address.port));
  });
});
const baseUrl = `http://127.0.0.1:${port}`;
const server = spawn("target/debug/nautilus", [
  "mvp", "serve", "--config", config, "--workspace", workspace,
  "--bind", `127.0.0.1:${port}`, "--ntpro-node-bin", "target/debug/ntpro-node",
  "--startup-timeout-ms", "10000", "--node-max-runtime-ms", "120000",
], { stdio: ["ignore", "pipe", "pipe"] });
server.stdout.on("data", (chunk) => serverLog.push(chunk.toString()));
server.stderr.on("data", (chunk) => serverLog.push(chunk.toString()));

let browser;
let failure;
const writeEvidence = (result) => {
  fs.writeFileSync(path.join(evidenceDir, "mvp-server.log"), redact(serverLog.join("")));
  fs.writeFileSync(path.join(evidenceDir, "result.json"), `${JSON.stringify(result, null, 2)}\n`);
};
try {
  let strategyAccessUrl;
  let payload;
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    const match = serverLog.join("").match(/strategy_workbench_url=(\S+)/);
    if (match) {
      strategyAccessUrl = new URL(match[1]);
      const token = strategyAccessUrl.searchParams.get("access_token");
      if (!token) throw new Error("strategy bootstrap URL omitted access_token");
      const response = await fetch(`${baseUrl}/api/mvp/v1/status`, { headers: { cookie: `ntpro_mvp_institution_access=${token}` } });
      if (response.ok) {
        payload = await response.json();
        break;
      }
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  if (!strategyAccessUrl || !payload) throw new Error(`strategy workbench did not become ready:\n${redact(serverLog.join(""))}`);

  const unauthorized = await fetch(`${baseUrl}/strategy-workbench`, { redirect: "manual" });
  if (unauthorized.status !== 403) throw new Error(`unauthorized strategy page expected 403, got ${unauthorized.status}`);

  browser = await chromium.launch({ executablePath: chrome, headless: true });
  const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const page = await context.newPage();
  const browserErrors = [];
  page.on("pageerror", (error) => browserErrors.push(error.message));
  page.on("console", (message) => { if (message.type() === "error") browserErrors.push(message.text()); });
  let scenario = "valid";
  await page.route("**/api/mvp/v1/status", async (route) => {
    if (scenario === "valid") return route.continue();
    if (scenario === "http_error") return route.fulfill({ status: 503, contentType: "application/json", body: "{}" });
    const response = structuredClone(payload);
    response.boundaries.real_orders_submitted = true;
    return route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(response) });
  });

  await page.goto(strategyAccessUrl.toString(), { waitUntil: "networkidle" });
  if (new URL(page.url()).searchParams.has("access_token")) throw new Error("bootstrap token remained in browser URL");
  await page.waitForFunction(() => document.getElementById("connection-title")?.textContent === "策略状态已验证");
  if (!await page.locator("#strategy-name").textContent()) throw new Error("strategy identity did not render");
  if (!await page.locator('.mode-tabs button:has-text("Live")').isDisabled()) throw new Error("Live mode must remain disabled");
  if (await page.locator('button').allTextContents().then((values) => values.some((value) => /下单|撤单|改单|平仓/.test(value)))) throw new Error("trading control appeared in strategy shell");
  if (await page.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth)) throw new Error("1440 viewport has horizontal overflow");

  await page.locator('[data-dock="logs"]').click();
  if (!await page.locator("#dock-content").textContent().then((value) => value?.includes("原始日志不在主产品面暴露"))) throw new Error("logs dock did not switch content");
  await page.locator("#drawer-toggle").click();
  if (await page.locator("#strategy-workbench").evaluate((node) => node.classList.contains("drawer-open"))) throw new Error("details drawer did not close");
  await page.locator("#drawer-toggle").click();
  await page.screenshot({ path: path.join(evidenceDir, "strategy-workbench-1440.png"), fullPage: true });

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${baseUrl}/strategy-workbench`, { waitUntil: "networkidle" });
  await page.waitForFunction(() => document.getElementById("connection-title")?.textContent === "策略状态已验证");
  if (await page.locator("#strategy-workbench").evaluate((node) => node.classList.contains("drawer-open"))) throw new Error("mobile details drawer must default closed");
  const mobileLayout = await page.evaluate(() => ({
    scrollX: window.scrollX,
    documentWidth: document.documentElement.scrollWidth,
    viewportWidth: document.documentElement.clientWidth,
    stageLeft: document.querySelector(".stage")?.getBoundingClientRect().left,
    canvasLeft: document.querySelector(".canvas")?.getBoundingClientRect().left,
    headingLeft: document.querySelector(".canvas-heading")?.getBoundingClientRect().left,
    firstScopeLeft: document.querySelector(".scope")?.getBoundingClientRect().left,
  }));
  if (mobileLayout.documentWidth > mobileLayout.viewportWidth || mobileLayout.scrollX !== 0 || mobileLayout.stageLeft !== 0 || mobileLayout.canvasLeft !== 0 || mobileLayout.headingLeft < 0 || mobileLayout.firstScopeLeft < 0) throw new Error(`390 viewport layout drift: ${JSON.stringify(mobileLayout)}`);
  await page.screenshot({ path: path.join(evidenceDir, "strategy-workbench-390.png"), fullPage: true });

  scenario = "boundary";
  await page.locator("#refresh").click();
  await page.waitForFunction(() => document.getElementById("connection-title")?.textContent === "策略工作台已阻断");
  if ((await page.locator("#strategy-name").textContent()) !== "策略未加载") throw new Error("boundary failure retained stale strategy identity");
  await page.screenshot({ path: path.join(evidenceDir, "strategy-workbench-blocked.png"), fullPage: true });

  scenario = "http_error";
  await page.locator("#refresh").click();
  await page.waitForFunction(() => document.getElementById("connection-title")?.textContent === "策略工作台已阻断");
  if (browserErrors.length > 0) throw new Error(`browser errors: ${browserErrors.join("; ")}`);
} catch (error) {
  failure = error instanceof Error ? error : new Error(String(error));
} finally {
  if (browser) await browser.close().catch(() => {});
  if (server.exitCode === null) server.kill("SIGINT");
  await new Promise((resolve) => {
    if (server.exitCode !== null) return resolve();
    const timer = setTimeout(() => { if (server.exitCode === null) server.kill("SIGKILL"); resolve(); }, 10_000);
    server.once("exit", () => { clearTimeout(timer); resolve(); });
  });
}

if (failure) {
  writeEvidence({ status: "fail", error: redact(failure.message) });
  throw failure;
}
writeEvidence({ status: "pass", viewports: ["1440x1000", "390x844"], valid: 1, boundary: 1, http_error: 1, dock: 1, drawer: 1, live_disabled: 1, bootstrap_url_clean: 1 });
console.log(`strategy_workbench_browser=pass evidence=${evidenceDir}`);
