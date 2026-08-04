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
const chrome = process.env.NTPRO_CHROME_BIN || "google-chrome";

const root = fs.mkdtempSync(path.join(os.tmpdir(), "ntpro-mvp-010-browser-"));
const evidenceDir = process.env.NTPRO_BROWSER_EVIDENCE_DIR || path.join(root, "evidence");
fs.mkdirSync(evidenceDir, { recursive: true });
const workspace = path.join(root, "workspace");
const config = path.resolve("configs/nodes/btc-ema-shadow.toml");
if (!fs.existsSync(config)) throw new Error(`MVP browser fixture is missing: ${config}`);

const readIfPresent = (filePath) => fs.existsSync(filePath) ? fs.readFileSync(filePath, "utf8") : "";
const redactAccessTokens = (value) => value.replace(/(access_token=)[^\s&]+/g, "$1[REDACTED]");
const sanitizeDiagnosticResult = (result) => ({
  ...result,
  ...(typeof result.error === "string" ? { error: redactAccessTokens(result.error) } : {}),
});
const diagnosticProbe = sanitizeDiagnosticResult({
  status: "fail",
  error: "goto http://127.0.0.1/control-center?access_token=probe-secret&event_id=1",
});
if (diagnosticProbe.error.includes("probe-secret") || !diagnosticProbe.error.includes("access_token=[REDACTED]")) {
  throw new Error("browser diagnostic token redaction self-test failed");
}
const passResult = {
  status: "pass",
  viewports: ["1440x1000", "390x844"],
  valid: 1,
  lifecycle_action: 1,
  lifecycle_operator_only: 1,
  shared_boundary: 1,
  node_mismatch: 1,
  ops_http_error: 1,
  event_mismatch: 1,
  duplicate_event: 1,
  cross_portal_jump: 1,
  unauthorized: 1,
  wrong_role: 1,
  bootstrap_url_clean: 1,
  diagnostic_redaction_selftest: 1,
  stale_clear: 5,
  cjk_glyphs: 1,
  graceful_shutdown: 1,
};

const port = await new Promise((resolve, reject) => {
  const listener = net.createServer();
  listener.once("error", reject);
  listener.listen(0, "127.0.0.1", () => {
    const address = listener.address();
    listener.close((error) => error ? reject(error) : resolve(address.port));
  });
});
const baseUrl = `http://127.0.0.1:${port}`;
const serverLog = [];
const server = spawn(
  "target/debug/nautilus",
  [
    "mvp", "serve", "--config", config, "--workspace", workspace,
    "--bind", `127.0.0.1:${port}`, "--ntpro-node-bin", "target/debug/ntpro-node",
    "--startup-timeout-ms", "10000", "--node-max-runtime-ms", "120000",
  ],
  { stdio: ["ignore", "pipe", "pipe"] },
);
server.stdout.on("data", (chunk) => serverLog.push(chunk.toString()));
server.stderr.on("data", (chunk) => serverLog.push(chunk.toString()));
const collectRuntimeLogs = () => {
  const nodeLogDir = path.join(workspace, "nodes", "mvp-node-001", "logs");
  return {
    server: redactAccessTokens(serverLog.join("")),
    nodeStdout: readIfPresent(path.join(nodeLogDir, "stdout.log")),
    nodeStderr: readIfPresent(path.join(nodeLogDir, "stderr.log")),
  };
};
const writeDiagnostics = (result, logs) => {
  fs.writeFileSync(path.join(evidenceDir, "mvp-server.log"), logs.server);
  fs.writeFileSync(path.join(evidenceDir, "ntpro-node-stdout.log"), logs.nodeStdout);
  fs.writeFileSync(path.join(evidenceDir, "ntpro-node-stderr.log"), logs.nodeStderr);
  fs.writeFileSync(
    path.join(evidenceDir, "result.json"),
    `${JSON.stringify(sanitizeDiagnosticResult(result), null, 2)}\n`,
  );
};
const serverExited = () => server.exitCode !== null || server.signalCode !== null;
const waitForServerExit = (timeoutMs) => {
  if (serverExited()) return Promise.resolve(true);
  return new Promise((resolve) => {
    const onExit = () => {
      clearTimeout(timer);
      resolve(true);
    };
    const timer = setTimeout(() => {
      server.off("exit", onExit);
      resolve(false);
    }, timeoutMs);
    server.once("exit", onExit);
  });
};

let browser;
let failure;
let scenario = "startup";
const recordFailure = (error) => {
  const next = error instanceof Error ? error : new Error(String(error));
  const message = redactAccessTokens(next.message);
  failure = failure ? new Error(`${failure.message}\nCleanup failure: ${message}`) : new Error(message);
};
try {
  let sharedPayload;
  let snapshotPayload;
  let correlationPayload;
  let institutionAccessUrl;
  let operatorAccessUrl;
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    try {
      const log = serverLog.join("");
      const institutionMatch = log.match(/institution_workbench_url=(\S+)/);
      const operatorMatch = log.match(/control_center_url=(\S+)/);
      if (!institutionMatch || !operatorMatch) {
        await new Promise((resolve) => setTimeout(resolve, 250));
        continue;
      }
      institutionAccessUrl = new URL(institutionMatch[1]);
      operatorAccessUrl = new URL(operatorMatch[1]);
      const operatorToken = operatorAccessUrl.searchParams.get("access_token");
      if (!operatorToken) throw new Error("operator bootstrap URL omitted access_token");
      const cookie = `ntpro_mvp_operator_access=${operatorToken}`;
      const [sharedResponse, snapshotResponse, correlationResponse] = await Promise.all([
        fetch(`${baseUrl}/api/mvp/v1/status`, { headers: { cookie } }),
        fetch(`${baseUrl}/api/mvp/v1/control-center`, { headers: { cookie } }),
        fetch(`${baseUrl}/api/mvp/v1/event-correlation`, { headers: { cookie } }),
      ]);
      if (sharedResponse.ok && snapshotResponse.ok && correlationResponse.ok) {
        [sharedPayload, snapshotPayload, correlationPayload] = await Promise.all([
          sharedResponse.json(), snapshotResponse.json(), correlationResponse.json(),
        ]);
        break;
      }
    } catch {}
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  if (!sharedPayload || !snapshotPayload || !correlationPayload || !institutionAccessUrl || !operatorAccessUrl) {
    throw new Error(`control center APIs and role bootstrap did not become ready:\n${redactAccessTokens(serverLog.join(""))}`);
  }
  if (process.env.NTPRO_BROWSER_FORCE_BOOTSTRAP_DIAGNOSTIC_FAILURE === "1") {
    throw new Error(`forced bootstrap diagnostic failure: ${operatorAccessUrl}`);
  }
  const unauthorized = await fetch(`${baseUrl}/control-center`, { redirect: "manual" });
  if (unauthorized.status !== 403) throw new Error(`unauthorized control center expected 403, got ${unauthorized.status}`);
  const operatorToken = operatorAccessUrl.searchParams.get("access_token");
  if (!operatorToken) throw new Error("operator bootstrap token missing");
  const wrongRole = await fetch(`${baseUrl}/institution-workbench`, {
    headers: { cookie: `ntpro_mvp_operator_access=${operatorToken}` },
    redirect: "manual",
  });
  if (wrongRole.status !== 403) throw new Error(`operator role reached institution workbench: ${wrongRole.status}`);
  const institutionToken = institutionAccessUrl.searchParams.get("access_token");
  if (!institutionToken) throw new Error("institution bootstrap token missing");
  const lifecycleActionPath = `${baseUrl}/api/mvp/v1/control-center/nodes/mvp-node-001/actions/stop`;
  const unauthorizedAction = await fetch(lifecycleActionPath, { method: "POST", redirect: "manual" });
  if (unauthorizedAction.status !== 403) throw new Error(`unauthorized lifecycle action expected 403, got ${unauthorizedAction.status}`);
  const institutionAction = await fetch(lifecycleActionPath, {
    method: "POST",
    headers: { cookie: `ntpro_mvp_institution_access=${institutionToken}` },
    redirect: "manual",
  });
  if (institutionAction.status !== 403) throw new Error(`institution role reached lifecycle action: ${institutionAction.status}`);
  const operatorCookie = `ntpro_mvp_operator_access=${operatorToken}`;
  const lifecycleGet = await fetch(lifecycleActionPath, { headers: { cookie: operatorCookie }, redirect: "manual" });
  if (lifecycleGet.status !== 405) throw new Error(`lifecycle GET expected 405, got ${lifecycleGet.status}`);
  const unsupportedAction = await fetch(`${baseUrl}/api/mvp/v1/control-center/nodes/mvp-node-001/actions/pause`, {
    method: "POST",
    headers: { cookie: operatorCookie },
    redirect: "manual",
  });
  if (unsupportedAction.status !== 404) throw new Error(`unsupported lifecycle action expected 404, got ${unsupportedAction.status}`);
  const serializedSnapshot = JSON.stringify(snapshotPayload);
  for (const forbidden of ["controls", "production_mutation_evidence", "read_model_runtime", "last_error", "message", "notes", "account_ref"]) {
    if (serializedSnapshot.includes(`\"${forbidden}\"`)) throw new Error(`operational API exposed forbidden field: ${forbidden}`);
  }
  const serializedCorrelation = JSON.stringify(correlationPayload);
  for (const forbidden of ["source_refs", "registry_path", "last_error", "message", "credential", "controls"]) {
    if (serializedCorrelation.includes(`\"${forbidden}\"`)) throw new Error(`event correlation API exposed forbidden field: ${forbidden}`);
  }

  browser = await chromium.launch({ executablePath: chrome, headless: true });
  const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const page = await context.newPage();
  const browserErrors = [];
  scenario = "valid";
  page.on("pageerror", (error) => browserErrors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error" && scenario !== "ops_http_error") browserErrors.push(message.text());
  });
  await page.route("**/api/mvp/v1/status", async (route) => {
    if (scenario !== "shared_boundary") {
      await route.continue();
      return;
    }
    const response = structuredClone(sharedPayload);
    response.boundaries.order_submission_allowed = true;
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(response) });
  });
  await page.route("**/api/mvp/v1/control-center", async (route) => {
    if (scenario === "ops_http_error") {
      await route.fulfill({ status: 503, contentType: "application/json", body: "{}" });
      return;
    }
    if (scenario === "node_mismatch") {
      const response = structuredClone(snapshotPayload);
      response.node.node_id = "mismatched-node";
      await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(response) });
      return;
    }
    await route.continue();
  });
  await page.route("**/api/mvp/v1/event-correlation", async (route) => {
    if (scenario !== "event_mismatch") {
      await route.continue();
      return;
    }
    const response = structuredClone(correlationPayload);
    response.event.strategy_instance_id = "mismatched-instance";
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(response) });
  });

  const waitForTitle = (title) => page.waitForFunction(
    (expected) => document.getElementById("connection-title")?.textContent === expected,
    title,
  );
  const refreshPage = async (expectedTitle) => {
    await page.evaluate(() => refreshControlCenter());
    const state = await page.evaluate(() => ({
      title: document.getElementById("connection-title")?.textContent,
      detail: document.getElementById("connection-detail")?.textContent,
    }));
    if (state.title !== expectedTitle) {
      throw new Error(`refresh expected ${expectedTitle}, got ${state.title}: ${state.detail}`);
    }
  };
  const assertCleared = async (name) => {
    const state = await page.evaluate(() => ({
      node: document.getElementById("context-node")?.textContent,
      components: document.getElementById("component-table")?.textContent,
      actions: document.getElementById("lifecycle-action-buttons")?.textContent,
    }));
    if (state.node !== "节点未加载" || !state.components?.includes("旧数据已清空") || !state.actions?.includes("操作不可用")) {
      throw new Error(`${name} retained stale control center data`);
    }
  };

  await page.goto(institutionAccessUrl.toString(), { waitUntil: "networkidle" });
  if (new URL(page.url()).searchParams.has("access_token")) throw new Error("institution bootstrap token remained in browser URL");
  await page.goto(operatorAccessUrl.toString(), { waitUntil: "networkidle" });
  if (new URL(page.url()).searchParams.has("access_token")) throw new Error("operator bootstrap token remained in browser URL");
  await waitForTitle("共享与运维状态已对齐");
  const node = await page.locator("#context-node").textContent();
  if (!node || node === "节点未加载") throw new Error("valid browser contract did not render node identity");
  const buttons = await page.locator("button").allTextContents();
  if (buttons.length !== 3 || !buttons.includes("刷新") || !buttons.includes("启动") || !buttons.includes("停止")) {
    throw new Error(`control center exposed unexpected action controls: ${buttons.join(" | ")}`);
  }
  const startButton = page.locator('[data-lifecycle-action="start"]');
  const stopButton = page.locator('[data-lifecycle-action="stop"]');
  if (!await startButton.isDisabled() || await stopButton.isDisabled()) {
    throw new Error("running node did not expose stop-only lifecycle capability");
  }
  const businessImpact = await page.locator("#business-impact-list").textContent();
  if (!businessImpact?.includes("风险")) throw new Error("valid browser contract did not render business impact");
  const eventId = correlationPayload.event.event_id;
  const businessLink = page.locator("#event-correlation-panel .portal-link");
  if (!await businessLink.isVisible()) throw new Error("control center did not render business impact jump");
  const cjkGlyphCheck = await page.evaluate(() => {
    const canvas = document.createElement("canvas");
    canvas.width = 64;
    canvas.height = 64;
    const context = canvas.getContext("2d");
    if (!context) return { rendered: false, reason: "canvas_2d_unavailable" };
    context.font = `32px ${getComputedStyle(document.body).fontFamily}`;
    context.textBaseline = "top";
    const signatures = [..."控制中心状态"].map((character) => {
      context.clearRect(0, 0, canvas.width, canvas.height);
      context.fillText(character, 0, 0);
      return Array.from(context.getImageData(0, 0, canvas.width, canvas.height).data).join(",");
    });
    return {
      rendered: signatures.every((signature) => /[1-9]/.test(signature))
        && new Set(signatures).size === signatures.length,
      reason: "pixel_signature",
    };
  });
  if (!cjkGlyphCheck.rendered) throw new Error(`Chinese glyph check failed: ${cjkGlyphCheck.reason}`);
  const wideOverflow = await page.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth);
  if (wideOverflow) throw new Error("1440 viewport has horizontal overflow");
  await page.screenshot({ path: path.join(evidenceDir, "control-center-1440.png"), fullPage: true });

  await stopButton.click();
  if (await stopButton.textContent() !== "确认停止") throw new Error("stop action did not require explicit confirmation");
  await stopButton.click();
  await page.waitForFunction(() => document.getElementById("lifecycle-action-result")?.textContent?.includes("running → stopped"));
  if (await startButton.isDisabled() || !await stopButton.isDisabled()) {
    throw new Error("stopped node did not expose start-only lifecycle capability");
  }
  await page.screenshot({ path: path.join(evidenceDir, "control-center-lifecycle-stopped.png"), fullPage: true });

  await startButton.click();
  if (await startButton.textContent() !== "确认启动") throw new Error("start action did not require explicit confirmation");
  await startButton.click();
  await page.waitForFunction(() => document.getElementById("lifecycle-action-result")?.textContent?.includes("stopped → running"));
  if (!await startButton.isDisabled() || await stopButton.isDisabled()) {
    throw new Error("restarted node did not restore stop-only lifecycle capability");
  }
  await page.screenshot({ path: path.join(evidenceDir, "control-center-lifecycle-running.png"), fullPage: true });

  await businessLink.click();
  await page.waitForURL((url) => url.pathname === "/institution-workbench" && url.searchParams.get("event_id") === eventId && url.hash === "#event-correlation");
  await waitForTitle("共享状态已验证");
  if (!await page.locator("#event-correlation-panel").textContent().then((value) => value?.includes(eventId))) {
    throw new Error("institution workbench did not preserve the correlated event");
  }

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${baseUrl}/control-center`, { waitUntil: "networkidle" });
  await waitForTitle("共享与运维状态已对齐");
  const narrowOverflow = await page.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth);
  if (narrowOverflow) throw new Error("390 viewport has horizontal overflow");
  await page.screenshot({ path: path.join(evidenceDir, "control-center-390.png"), fullPage: true });

  scenario = "shared_boundary";
  await refreshPage("控制中心已阻断");
  await assertCleared("shared boundary violation");
  await page.screenshot({ path: path.join(evidenceDir, "control-center-shared-boundary-blocked.png"), fullPage: true });

  scenario = "valid";
  await refreshPage("共享与运维状态已对齐");
  scenario = "node_mismatch";
  await refreshPage("控制中心已阻断");
  await assertCleared("node mismatch");

  scenario = "valid";
  await refreshPage("共享与运维状态已对齐");
  scenario = "ops_http_error";
  await refreshPage("控制中心已阻断");
  await assertCleared("operations HTTP error");
  await page.screenshot({ path: path.join(evidenceDir, "control-center-ops-http-error.png"), fullPage: true });

  scenario = "valid";
  await refreshPage("共享与运维状态已对齐");
  scenario = "event_mismatch";
  await refreshPage("控制中心已阻断");
  await assertCleared("event mismatch");
  await page.screenshot({ path: path.join(evidenceDir, "control-center-event-mismatch.png"), fullPage: true });

  scenario = "valid";
  await page.goto(`${baseUrl}/control-center?event_id=forged&event_id=${encodeURIComponent(eventId)}`, { waitUntil: "networkidle" });
  await waitForTitle("控制中心已阻断");
  await assertCleared("duplicate event parameter");
  await page.screenshot({ path: path.join(evidenceDir, "control-center-duplicate-event-blocked.png"), fullPage: true });

  if (browserErrors.length > 0) throw new Error(`browser console errors: ${browserErrors.join(" | ")}`);
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  recordFailure(new Error(`scenario=${scenario}: ${message}`));
} finally {
  if (browser) {
    try {
      await browser.close();
    } catch (error) {
      recordFailure(error);
    }
  }
  if (!serverExited()) {
    try {
      server.kill("SIGINT");
    } catch (error) {
      recordFailure(error);
    }
    if (!await waitForServerExit(5_000)) {
      recordFailure(new Error("MVP server did not stop within 5000 ms and required SIGKILL"));
      try {
        server.kill("SIGKILL");
      } catch (error) {
        recordFailure(error);
      }
      if (!await waitForServerExit(5_000)) recordFailure(new Error("MVP server did not exit after SIGKILL"));
    }
  }
  const logs = collectRuntimeLogs();
  if (!failure) {
    if (server.exitCode !== 0) recordFailure(new Error(`MVP server exited with code ${server.exitCode}`));
    if (!logs.server.includes("mvp.serve status=stopped")) recordFailure(new Error("MVP server did not report stopped status"));
    if (!logs.nodeStdout.includes("final_state=Stopped")) recordFailure(new Error("ntpro-node did not report final_state=Stopped"));
    if (!logs.nodeStdout.includes("external_venue_connection=false real_orders_submitted=false")) {
      recordFailure(new Error("ntpro-node shutdown evidence did not preserve trading boundaries"));
    }
  }
  writeDiagnostics(failure ? { status: "fail", error: failure.message } : passResult, logs);
  fs.rmSync(root, { recursive: true, force: true });
}

if (failure) throw failure;
console.log("control_center_browser=pass viewports=1440x1000,390x844 valid=1 lifecycle_action=1 lifecycle_operator_only=1 shared_boundary=1 node_mismatch=1 ops_http_error=1 event_mismatch=1 duplicate_event=1 cross_portal_jump=1 unauthorized=1 wrong_role=1 bootstrap_url_clean=1 diagnostic_redaction_selftest=1 stale_clear=5 cjk_glyphs=1 graceful_shutdown=1");
