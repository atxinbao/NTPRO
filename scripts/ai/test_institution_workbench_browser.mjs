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
const passResult = {
  status: "pass",
  viewports: ["1440x1000", "390x844"],
  valid: 1,
  boundary: 1,
  http_error: 1,
  stale_clear: 2,
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
    server: serverLog.join(""),
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
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(`${baseUrl}/api/mvp/v1/status`);
      if (response.ok) {
        payload = await response.json();
        break;
      }
    } catch {}
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  if (!payload) throw new Error(`MVP status API did not become ready:\n${serverLog.join("")}`);

  browser = await chromium.launch({ executablePath: chrome, headless: true });
  const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
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

  await page.goto(`${baseUrl}/institution-workbench`, { waitUntil: "networkidle" });
  await waitForTitle("共享状态已验证");
  const strategy = await page.locator("#context-strategy").textContent();
  if (!strategy || strategy === "策略未加载") throw new Error("valid browser contract did not render identity");
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

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload({ waitUntil: "networkidle" });
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
console.log("institution_workbench_browser=pass viewports=1440x1000,390x844 valid=1 boundary=1 http_error=1 stale_clear=2 cjk_glyphs=1 graceful_shutdown=1");
