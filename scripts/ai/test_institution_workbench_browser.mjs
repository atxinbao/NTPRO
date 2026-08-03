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

const root = fs.mkdtempSync(path.join(os.tmpdir(), "ntpro-mvp-006-browser-"));
const evidenceDir = process.env.NTPRO_BROWSER_EVIDENCE_DIR || path.join(root, "evidence");
fs.mkdirSync(evidenceDir, { recursive: true });
const workspace = path.join(root, "workspace");
const config = path.resolve("configs/nodes/btc-ema-shadow.toml");
if (!fs.existsSync(config)) throw new Error(`MVP browser fixture is missing: ${config}`);

const readIfPresent = (filePath) => fs.existsSync(filePath) ? fs.readFileSync(filePath, "utf8") : "";
const redactAccessTokens = (value) => value.replace(/(access_token=)[^\s&]+/g, "$1[REDACTED]");
const passResult = {
  status: "pass",
  viewports: ["1440x1000", "390x844"],
  valid: 1,
  boundary: 1,
  http_error: 1,
  event_mismatch: 1,
  duplicate_event: 1,
  cross_portal_jump: 1,
  unauthorized: 1,
  wrong_role: 1,
  bootstrap_url_clean: 1,
  stale_clear: 4,
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
    `${JSON.stringify(result, null, 2)}\n`,
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
const recordFailure = (error) => {
  const next = error instanceof Error ? error : new Error(String(error));
  failure = failure ? new Error(`${failure.message}\nCleanup failure: ${next.message}`) : next;
};
try {
  let payload;
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
      const institutionToken = institutionAccessUrl.searchParams.get("access_token");
      if (!institutionToken) throw new Error("institution bootstrap URL omitted access_token");
      const cookie = `ntpro_mvp_institution_access=${institutionToken}`;
      const [response, correlationResponse] = await Promise.all([
        fetch(`${baseUrl}/api/mvp/v1/status`, { headers: { cookie } }),
        fetch(`${baseUrl}/api/mvp/v1/event-correlation`, { headers: { cookie } }),
      ]);
      if (response.ok && correlationResponse.ok) {
        [payload, correlationPayload] = await Promise.all([response.json(), correlationResponse.json()]);
        break;
      }
    } catch {}
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  if (!payload || !correlationPayload || !institutionAccessUrl || !operatorAccessUrl) {
    throw new Error(`MVP status, role bootstrap and event correlation APIs did not become ready:\n${redactAccessTokens(serverLog.join(""))}`);
  }

  const unauthorized = await fetch(`${baseUrl}/institution-workbench`, { redirect: "manual" });
  if (unauthorized.status !== 403) throw new Error(`unauthorized institution page expected 403, got ${unauthorized.status}`);
  const institutionToken = institutionAccessUrl.searchParams.get("access_token");
  if (!institutionToken) throw new Error("institution bootstrap token missing");
  const wrongRole = await fetch(`${baseUrl}/control-center`, {
    headers: { cookie: `ntpro_mvp_institution_access=${institutionToken}` },
    redirect: "manual",
  });
  if (wrongRole.status !== 403) throw new Error(`institution role reached control center: ${wrongRole.status}`);

  browser = await chromium.launch({ executablePath: chrome, headless: true });
  const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
  const page = await context.newPage();
  const browserErrors = [];
  page.on("pageerror", (error) => browserErrors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error" && scenario !== "http_error") browserErrors.push(message.text());
  });
  let scenario = "valid";
  await page.route("**/api/mvp/v1/status", async (route) => {
    if (scenario === "valid") {
      await route.continue();
      return;
    }
    if (scenario === "http_error") {
      await route.fulfill({ status: 503, contentType: "application/json", body: "{}" });
      return;
    }
    const response = structuredClone(payload);
    if (scenario === "boundary_violation") response.boundaries.order_submission_allowed = true;
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(response) });
  });
  await page.route("**/api/mvp/v1/event-correlation", async (route) => {
    if (scenario !== "event_mismatch") {
      await route.continue();
      return;
    }
    const response = structuredClone(correlationPayload);
    response.event.node_id = "mismatched-node";
    await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(response) });
  });

  const waitForTitle = (title) => page.waitForFunction(
    (expected) => document.getElementById("connection-title")?.textContent === expected,
    title,
  );
  const assertCleared = async (name) => {
    const state = await page.evaluate(() => ({
      strategy: document.getElementById("context-strategy")?.textContent,
      business: document.getElementById("business-grid")?.textContent,
    }));
    if (state.strategy !== "策略未加载" || !state.business?.includes("等待共享状态")) {
      throw new Error(`${name} retained stale institution data`);
    }
  };

  await page.goto(operatorAccessUrl.toString(), { waitUntil: "networkidle" });
  if (new URL(page.url()).searchParams.has("access_token")) throw new Error("operator bootstrap token remained in browser URL");
  await page.goto(institutionAccessUrl.toString(), { waitUntil: "networkidle" });
  if (new URL(page.url()).searchParams.has("access_token")) throw new Error("institution bootstrap token remained in browser URL");
  await waitForTitle("共享状态已验证");
  const strategy = await page.locator("#context-strategy").textContent();
  if (!strategy || strategy === "策略未加载") throw new Error("valid browser contract did not render identity");
  const eventId = correlationPayload.event.event_id;
  const technicalLink = page.locator("#event-correlation-panel .portal-link");
  if (!await technicalLink.isVisible()) throw new Error("institution workbench did not render technical root jump");
  const cjkGlyphCheck = await page.evaluate(() => {
    const canvas = document.createElement("canvas");
    canvas.width = 64;
    canvas.height = 64;
    const context = canvas.getContext("2d");
    if (!context) return { rendered: false, reason: "canvas_2d_unavailable" };
    context.font = `32px ${getComputedStyle(document.body).fontFamily}`;
    context.textBaseline = "top";
    const signatures = [..."共享状态验证"].map((character) => {
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
  await page.screenshot({ path: path.join(evidenceDir, "institution-workbench-1440.png"), fullPage: true });

  await technicalLink.click();
  await page.waitForURL((url) => url.pathname === "/control-center" && url.searchParams.get("event_id") === eventId && url.hash === "#event-correlation");
  await waitForTitle("共享与运维状态已对齐");
  if (!await page.locator("#event-correlation-panel").textContent().then((value) => value?.includes(eventId))) {
    throw new Error("control center did not preserve the correlated event");
  }

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${baseUrl}/institution-workbench`, { waitUntil: "networkidle" });
  await waitForTitle("共享状态已验证");
  const narrowOverflow = await page.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth);
  if (narrowOverflow) throw new Error("390 viewport has horizontal overflow");
  await page.screenshot({ path: path.join(evidenceDir, "institution-workbench-390.png"), fullPage: true });

  scenario = "boundary_violation";
  await page.locator("#refresh").click();
  await waitForTitle("机构工作台已阻断");
  await assertCleared("boundary violation");
  await page.screenshot({ path: path.join(evidenceDir, "institution-workbench-boundary-blocked.png"), fullPage: true });

  scenario = "valid";
  await page.locator("#refresh").click();
  await waitForTitle("共享状态已验证");
  scenario = "http_error";
  await page.locator("#refresh").click();
  await waitForTitle("机构工作台已阻断");
  await assertCleared("HTTP error");
  await page.screenshot({ path: path.join(evidenceDir, "institution-workbench-http-error.png"), fullPage: true });

  scenario = "valid";
  await page.locator("#refresh").click();
  await waitForTitle("共享状态已验证");
  scenario = "event_mismatch";
  await page.locator("#refresh").click();
  await waitForTitle("机构工作台已阻断");
  await assertCleared("event mismatch");
  await page.screenshot({ path: path.join(evidenceDir, "institution-workbench-event-mismatch.png"), fullPage: true });

  scenario = "valid";
  await page.goto(`${baseUrl}/institution-workbench?event_id=${encodeURIComponent(eventId)}&event_id=forged`, { waitUntil: "networkidle" });
  await waitForTitle("机构工作台已阻断");
  await assertCleared("duplicate event parameter");
  await page.screenshot({ path: path.join(evidenceDir, "institution-workbench-duplicate-event-blocked.png"), fullPage: true });

  if (browserErrors.length > 0) throw new Error(`browser console errors: ${browserErrors.join(" | ")}`);
} catch (error) {
  recordFailure(error);
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
      if (!await waitForServerExit(5_000)) {
        recordFailure(new Error("MVP server did not exit after SIGKILL"));
      }
    }
  }
  const logs = collectRuntimeLogs();
  if (!failure) {
    if (server.exitCode !== 0) recordFailure(new Error(`MVP server exited with code ${server.exitCode}`));
    if (!logs.server.includes("mvp.serve status=stopped")) {
      recordFailure(new Error("MVP server did not report stopped status"));
    }
    if (!logs.nodeStdout.includes("final_state=Stopped")) {
      recordFailure(new Error("ntpro-node did not report final_state=Stopped"));
    }
    if (!logs.nodeStdout.includes("external_venue_connection=false real_orders_submitted=false")) {
      recordFailure(new Error("ntpro-node shutdown evidence did not preserve trading boundaries"));
    }
  }
  writeDiagnostics(
    failure ? { status: "fail", error: failure.message } : passResult,
    logs,
  );
  fs.rmSync(root, { recursive: true, force: true });
}

if (failure) throw failure;
console.log("institution_workbench_browser=pass viewports=1440x1000,390x844 valid=1 boundary=1 http_error=1 event_mismatch=1 duplicate_event=1 cross_portal_jump=1 unauthorized=1 wrong_role=1 bootstrap_url_clean=1 stale_clear=4 cjk_glyphs=1 graceful_shutdown=1");
