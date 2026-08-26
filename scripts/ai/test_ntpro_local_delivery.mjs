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

const repoRoot = path.resolve(".");
const root = fs.mkdtempSync(path.join(os.tmpdir(), "ntpro-upv1-005-"));
const packageDir = path.resolve(
  process.env.NTPRO_LOCAL_DELIVERY_PACKAGE || path.join(root, "delivery"),
);
const evidenceDir = path.resolve(
  process.env.NTPRO_LOCAL_DELIVERY_EVIDENCE_DIR ||
    fs.mkdtempSync(path.join(os.tmpdir(), "ntpro-upv1-005-evidence-")),
);
const workspace = path.join(root, "user-data", "usable-product-v1");
const launcher = path.join(packageDir, "start-ntpro");
const evidenceLog = [];
const activeChildren = new Set();

fs.mkdirSync(evidenceDir, { recursive: true });

const assert = (condition, message) => {
  if (!condition) throw new Error(message);
};
const sleep = (millis) => new Promise((resolve) => setTimeout(resolve, millis));
const redact = (value) =>
  String(value)
    .replace(/(access_token=)[^\s&]+/g, "$1[REDACTED]")
    .replace(/(ntpro_mvp_(?:institution|operator)_access=)[^;\s]+/g, "$1[REDACTED]")
    .replace(/(risk_api_token=)[^\s]+/g, "$1[REDACTED]");
const exited = (child) => child.exitCode !== null || child.signalCode !== null;
const waitForExit = (child, timeoutMs) => {
  if (exited(child)) return Promise.resolve(true);
  return new Promise((resolve) => {
    const onExit = () => {
      clearTimeout(timer);
      resolve(true);
    };
    const timer = setTimeout(() => {
      child.off("exit", onExit);
      resolve(false);
    }, timeoutMs);
    child.once("exit", onExit);
  });
};
const freePort = () =>
  new Promise((resolve, reject) => {
    const listener = net.createServer();
    listener.once("error", reject);
    listener.listen(0, "127.0.0.1", () => {
      const address = listener.address();
      listener.close((error) =>
        error ? reject(error) : resolve(address.port),
      );
    });
  });
const launcherEnv = (workspacePath, port) => ({
  ...process.env,
  HOME: path.join(root, "home"),
  XDG_DATA_HOME: path.join(root, "xdg-data"),
  NTPRO_WORKSPACE: workspacePath,
  NTPRO_BIND: `127.0.0.1:${port}`,
  NTPRO_NODE_MAX_RUNTIME_MS: "120000",
  NTPRO_STARTUP_TIMEOUT_MS: "10000",
  NTPRO_NODE_SHUTDOWN_TIMEOUT_MS: "10000",
  NO_COLOR: "1",
});
const startLauncher = (workspacePath, port, label) => {
  const chunks = [];
  const child = spawn(launcher, [], {
    cwd: packageDir,
    env: launcherEnv(workspacePath, port),
    detached: true,
    stdio: ["ignore", "pipe", "pipe"],
  });
  activeChildren.add(child);
  child.once("exit", () => activeChildren.delete(child));
  child.stdout.on("data", (chunk) => chunks.push(chunk.toString()));
  child.stderr.on("data", (chunk) => chunks.push(chunk.toString()));
  evidenceLog.push({ label, pid: child.pid, chunks });
  return { child, chunks, port };
};
const combinedLog = (launch) => launch.chunks.join("");
const waitForReady = async (launch) => {
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    const match = [...combinedLog(launch).matchAll(/strategy_workbench_url=(\S+)/g)].at(-1);
    if (match) {
      const accessUrl = new URL(match[1]);
      const token = accessUrl.searchParams.get("access_token");
      assert(token, "strategy bootstrap URL omitted access token");
      const cookie = `ntpro_mvp_institution_access=${token}`;
      try {
        const response = await fetch(
          `http://127.0.0.1:${launch.port}/api/mvp/v1/status`,
          { headers: { cookie }, signal: AbortSignal.timeout(5_000) },
        );
        if (response.ok) return { accessUrl, cookie };
      } catch {
        // Axum 可能在真正开始接收请求前一刻打印访问地址。
      }
    }
    if (exited(launch.child)) {
      throw new Error(`local delivery exited before ready: ${redact(combinedLog(launch))}`);
    }
    await sleep(150);
  }
  throw new Error(`local delivery did not become ready: ${redact(combinedLog(launch))}`);
};
const stopGracefully = async (launch) => {
  if (exited(launch.child)) return;
  launch.child.kill("SIGINT");
  if (await waitForExit(launch.child, 15_000)) return;
  try {
    process.kill(-launch.child.pid, "SIGKILL");
  } catch {
    // 进程组可能在等待超时与发送信号之间已经退出。
  }
  assert(await waitForExit(launch.child, 5_000), "local delivery did not stop");
};
const killAbnormally = async (launch) => {
  if (!exited(launch.child)) process.kill(-launch.child.pid, "SIGKILL");
  assert(await waitForExit(launch.child, 5_000), "abnormal process group did not exit");
};
const runSyncLauncher = (workspacePath, port, timeout = 20_000) =>
  spawnSync(launcher, [], {
    cwd: packageDir,
    env: launcherEnv(workspacePath, port),
    encoding: "utf8",
    timeout,
  });
const createBacktest = async (port, cookie) => {
  const response = await fetch(`http://127.0.0.1:${port}/api/product/v1/runs`, {
    method: "POST",
    headers: { cookie, "content-type": "application/json" },
    body: JSON.stringify({
      strategy_id: "ema_cross_btcusdt_v1",
      strategy_version_id: "ema_cross_btcusdt_v1@v1",
      environment: "backtest",
      data_ref: "dataset://fixtures/ema-cross-btcusdt-v1",
      venue_ref: "venue://simulated/BINANCE_TESTNET",
      starting_balance: "1000000 USDT",
      quotes: 120,
      trade_size: "0.001000",
      fast_period: 3,
      slow_period: 5,
    }),
    signal: AbortSignal.timeout(30_000),
  });
  const body = await response.json();
  assert(response.status === 201, `package Backtest creation failed: ${JSON.stringify(body)}`);
  assert(body.data?.run_id?.startsWith("backtest-"), "package Backtest run ID drifted");
  return body.data.run_id;
};
const readRun = async (port, cookie, runId) => {
  const response = await fetch(
    `http://127.0.0.1:${port}/api/product/v1/runs/${runId}`,
    { headers: { cookie }, signal: AbortSignal.timeout(5_000) },
  );
  const body = await response.json();
  assert(response.status === 200, `persisted Run readback failed: ${JSON.stringify(body)}`);
  assert(body.data?.run_id === runId, "persisted Run identity drifted");
};

let browser;
let page;
let currentLaunch;
let failure;

try {
  if (!process.env.NTPRO_LOCAL_DELIVERY_PACKAGE) {
    const build = spawnSync("scripts/ai/build_ntpro_local_delivery.sh", [], {
      cwd: repoRoot,
      env: {
        ...process.env,
        NTPRO_LOCAL_DELIVERY_SKIP_BUILD: "1",
        NTPRO_LOCAL_DELIVERY_OUTPUT: packageDir,
      },
      encoding: "utf8",
      timeout: 120_000,
    });
    assert(!build.error, `delivery builder failed to start: ${build.error?.message}`);
    assert(build.status === 0, `delivery builder failed: ${build.stderr || build.stdout}`);
  }
  for (const required of [
    launcher,
    path.join(packageDir, "bin", "nautilus"),
    path.join(packageDir, "bin", "ntpro-node"),
    path.join(packageDir, "apps", "strategy-workbench", "dist", "index.html"),
    path.join(packageDir, "delivery-manifest.json"),
    path.join(packageDir, "操作说明.md"),
  ]) {
    assert(fs.existsSync(required), `delivery package omitted ${required}`);
  }

  const port = await freePort();
  currentLaunch = startLauncher(workspace, port, "initial");
  const initial = await waitForReady(currentLaunch);

  const duplicate = runSyncLauncher(workspace, port);
  assert(duplicate.status === 73, `duplicate launch exited ${duplicate.status}`);
  assert(
    `${duplicate.stdout}${duplicate.stderr}`.includes("已经有一个 NTPRO 实例在运行"),
    "duplicate launch omitted actionable error",
  );
  assert(!exited(currentLaunch.child), "duplicate launch disturbed running product");

  browser = await chromium.launch({ executablePath: chrome, headless: true });
  page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
  const browserErrors = [];
  page.on("pageerror", (error) => browserErrors.push(error.message));
  page.on("response", (response) => {
    if (response.status() >= 500) browserErrors.push(`${response.status()} ${response.url()}`);
  });
  await page.goto(initial.accessUrl.toString(), { waitUntil: "networkidle" });
  assert(
    new URL(page.url()).pathname === "/strategy-workbench/overview",
    `browser did not reach strategy overview: ${page.url()}`,
  );
  assert(!page.url().includes("access_token"), "browser retained bootstrap token");
  await page.getByText("策略工作台", { exact: true }).first().waitFor();
  await page.screenshot({
    path: path.join(evidenceDir, "local-delivery-overview-1440.png"),
    fullPage: true,
  });
  assert(browserErrors.length === 0, `browser errors: ${browserErrors.join("; ")}`);

  const createdRunId = await createBacktest(port, initial.cookie);
  const retentionProbe = path.join(workspace, "local-delivery-retention-probe.txt");
  fs.writeFileSync(retentionProbe, `${createdRunId}\n`);
  await stopGracefully(currentLaunch);
  assert(!fs.existsSync(path.join(workspace, ".local-delivery.lock")), "normal stop retained lock");

  currentLaunch = startLauncher(workspace, port, "restart");
  const restarted = await waitForReady(currentLaunch);
  assert(fs.readFileSync(retentionProbe, "utf8").trim() === createdRunId, "workspace data was lost");
  await readRun(port, restarted.cookie, createdRunId);
  await killAbnormally(currentLaunch);
  await sleep(250);
  assert(fs.existsSync(path.join(workspace, ".local-delivery.lock")), "abnormal exit did not exercise stale lock");

  currentLaunch = startLauncher(workspace, port, "abnormal-recovery");
  await waitForReady(currentLaunch);
  assert(
    combinedLog(currentLaunch).includes("已清理失效运行锁"),
    "restart did not report stale-lock recovery",
  );
  await stopGracefully(currentLaunch);

  const occupiedPort = await freePort();
  const listener = net.createServer();
  await new Promise((resolve, reject) => {
    listener.once("error", reject);
    listener.listen(occupiedPort, "127.0.0.1", resolve);
  });
  const occupiedWorkspace = path.join(root, "occupied-workspace");
  const occupied = runSyncLauncher(occupiedWorkspace, occupiedPort);
  await new Promise((resolve) => listener.close(resolve));
  assert(occupied.status !== 0, "occupied port unexpectedly started product");
  assert(
    `${occupied.stdout}${occupied.stderr}`.includes("端口") ||
      `${occupied.stdout}${occupied.stderr}`.includes("地址"),
    "occupied port omitted actionable error",
  );
  assert(
    !fs.existsSync(path.join(occupiedWorkspace, ".local-delivery.lock")),
    "occupied port retained a misleading lock",
  );

  const packagedNode = path.join(packageDir, "bin", "ntpro-node");
  const missingNode = `${packagedNode}.missing`;
  fs.renameSync(packagedNode, missingNode);
  let missing;
  try {
    missing = runSyncLauncher(path.join(root, "missing-workspace"), await freePort());
  } finally {
    fs.renameSync(missingNode, packagedNode);
  }
  assert(missing.status === 66, `missing dependency exited ${missing.status}`);
  assert(
    `${missing.stdout}${missing.stderr}`.includes("缺少可执行的NTPRO 节点程序"),
    "missing dependency omitted actionable error",
  );

  fs.writeFileSync(
    path.join(evidenceDir, "result.json"),
    `${JSON.stringify(
      {
        schema_version: "ntpro.local_delivery_acceptance.v1",
        status: "pass",
        package_manifest: "ntpro.local_delivery_manifest.v1",
        single_entrypoint: true,
        production_browser: true,
        duplicate_launch_rejected: true,
        normal_stop_clean: true,
        same_workspace_restart: true,
        backtest_persisted: true,
        abnormal_exit_recovered: true,
        occupied_port_rejected: true,
        missing_dependency_rejected: true,
        external_venue_connection: false,
        real_orders_submitted: false,
      },
      null,
      2,
    )}\n`,
  );
  console.log(`local_delivery_acceptance=pass evidence=${evidenceDir}`);
} catch (error) {
  failure = error;
  if (page) {
    await page
      .screenshot({ path: path.join(evidenceDir, "failure.png"), fullPage: true })
      .catch(() => {});
  }
  fs.writeFileSync(
    path.join(evidenceDir, "result.json"),
    `${JSON.stringify(
      {
        schema_version: "ntpro.local_delivery_acceptance.v1",
        status: "fail",
        error: redact(error.message),
        external_venue_connection: false,
        real_orders_submitted: false,
      },
      null,
      2,
    )}\n`,
  );
} finally {
  if (currentLaunch && !exited(currentLaunch.child)) await stopGracefully(currentLaunch);
  for (const child of activeChildren) {
    if (!exited(child)) {
      try {
        process.kill(-child.pid, "SIGKILL");
      } catch {
        // 验收失败后尽力清理测试进程。
      }
    }
  }
  if (browser) await browser.close();
  const logs = evidenceLog
    .map(({ label, pid, chunks }) => `--- ${label} pid=${pid} ---\n${chunks.join("")}`)
    .join("\n");
  fs.writeFileSync(path.join(evidenceDir, "local-delivery.log"), redact(logs));
  fs.rmSync(root, { recursive: true, force: true });
}

if (failure) throw failure;
