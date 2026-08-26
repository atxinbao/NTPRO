import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
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
const signalStopWaitMs = 75_000;

fs.mkdirSync(evidenceDir, { recursive: true });

const assert = (condition, message) => {
  if (!condition) throw new Error(message);
};
const sleep = (millis) => new Promise((resolve) => setTimeout(resolve, millis));
const sha256File = (filePath) =>
  createHash("sha256").update(fs.readFileSync(filePath)).digest("hex");
const sha256Tree = (treeRoot, excluded = undefined) => {
  const files = [];
  const visit = (directory) => {
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const absolute = path.join(directory, entry.name);
      if (entry.isDirectory()) visit(absolute);
      else if (entry.isFile()) files.push(path.relative(treeRoot, absolute));
    }
  };
  visit(treeRoot);
  const canonical = files
    .filter((relative) => relative !== excluded)
    .sort()
    .map((relative) => `${sha256File(path.join(treeRoot, relative))}  ${relative}\n`)
    .join("");
  return createHash("sha256").update(canonical).digest("hex");
};
const processAlive = (pid) => {
  if (!Number.isInteger(pid) || pid <= 0) return false;
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
};
const waitFor = async (description, timeoutMs, check) => {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const value = await check();
    if (value) return value;
    await sleep(100);
  }
  throw new Error(`timed out waiting for ${description}`);
};
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
const launcherEnv = (workspacePath, port, overrides = {}) => ({
  ...process.env,
  HOME: path.join(root, "home"),
  XDG_DATA_HOME: path.join(root, "xdg-data"),
  NTPRO_WORKSPACE: workspacePath,
  NTPRO_BIND: `127.0.0.1:${port}`,
  NTPRO_NODE_MAX_RUNTIME_MS: "120000",
  NTPRO_STARTUP_TIMEOUT_MS: "10000",
  NTPRO_NODE_SHUTDOWN_TIMEOUT_MS: "10000",
  NO_COLOR: "1",
  ...overrides,
});
const startLauncher = (workspacePath, port, label, options = {}) => {
  const chunks = [];
  const child = spawn(options.launcher ?? launcher, [], {
    cwd: options.cwd ?? packageDir,
    env: launcherEnv(workspacePath, port, options.env),
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
const lockPath = (workspacePath = workspace) =>
  path.join(workspacePath, ".local-delivery.lock");
const guardianPidFor = (workspacePath = workspace) => readPid(lockPath(workspacePath));
const servicePidFor = (workspacePath = workspace) => {
  const ownerPid = guardianPidFor(workspacePath);
  return ownerPid
    ? readPid(path.join(workspacePath, `.local-delivery.child.${ownerPid}`))
    : undefined;
};
const killAbnormally = async (launch) => {
  const guardianPid = guardianPidFor();
  const servicePid = servicePidFor();
  assert(guardianPid && processAlive(guardianPid), "group kill omitted live guardian PID");
  assert(servicePid && processAlive(servicePid), "group kill omitted live service PID");
  if (!exited(launch.child)) process.kill(-launch.child.pid, "SIGKILL");
  if (processAlive(guardianPid)) process.kill(-guardianPid, "SIGKILL");
  assert(await waitForExit(launch.child, 5_000), "abnormal process group did not exit");
  await waitFor("abnormal guardian exit", 5_000, () => !processAlive(guardianPid));
  await waitFor("abnormal service exit", 5_000, () => !processAlive(servicePid));
  return servicePid;
};
const readPid = (filePath) => {
  if (!fs.existsSync(filePath)) return undefined;
  const pid = Number(fs.readFileSync(filePath, "utf8").trim());
  return Number.isInteger(pid) && pid > 0 ? pid : undefined;
};
const stopWithSignal = async (launch, signal, processGroup = false) => {
  const servicePid = servicePidFor();
  assert(servicePid && processAlive(servicePid), `${signal} test omitted live service PID`);
  if (processGroup) process.kill(-launch.child.pid, signal);
  else launch.child.kill(signal);
  assert(await waitForExit(launch.child, signalStopWaitMs), `${signal} did not stop launcher`);
  assert(
    launch.child.exitCode === 0 && launch.child.signalCode === null,
    `${signal} launcher exit was not clean: code=${launch.child.exitCode} signal=${launch.child.signalCode}`,
  );
  await waitFor(`${signal} service exit`, 5_000, () => !processAlive(servicePid));
  await waitFor(`${signal} lock cleanup`, 5_000, () =>
    !fs.existsSync(lockPath())
  );
  const log = combinedLog(launch);
  assert(log.includes("mvp.serve status=stopped"), `${signal} omitted MVP stopped evidence`);
  assert(log.includes("NTPRO 已安全停止"), `${signal} omitted launcher safe-stop evidence`);
  return servicePid;
};
const verifyForcedStopFailsClosed = async () => {
  const forcedPackage = path.join(root, "forced-stop-delivery");
  const forcedLauncher = path.join(forcedPackage, "start-ntpro");
  const fakeService = path.join(forcedPackage, "bin", "nautilus");
  const forcedWorkspace = path.join(root, "forced-stop-workspace");
  const fakeReady = path.join(root, "forced-stop-ready");
  const fakeSignalled = path.join(root, "forced-stop-signalled");
  for (const directory of [
    path.join(forcedPackage, "bin"),
    path.join(forcedPackage, "configs", "nodes"),
    path.join(forcedPackage, "configs", "backtests"),
    path.join(forcedPackage, "apps", "strategy-workbench", "dist"),
  ]) {
    fs.mkdirSync(directory, { recursive: true });
  }
  fs.copyFileSync(launcher, forcedLauncher);
  fs.chmodSync(forcedLauncher, 0o755);
  fs.symlinkSync(
    path.join(packageDir, "bin", "ntpro-node"),
    path.join(forcedPackage, "bin", "ntpro-node"),
  );
  for (const [source, destination] of [
    ["configs/nodes/btc-ema-shadow.toml", "configs/nodes/btc-ema-shadow.toml"],
    [
      "configs/backtests/ema-cross-btcusdt-product.toml",
      "configs/backtests/ema-cross-btcusdt-product.toml",
    ],
    ["apps/strategy-workbench/dist/index.html", "apps/strategy-workbench/dist/index.html"],
  ]) {
    fs.copyFileSync(path.join(packageDir, source), path.join(forcedPackage, destination));
  }
  fs.writeFileSync(
    fakeService,
    `#!/usr/bin/env node
const fs = require("node:fs");
process.on("SIGINT", () => {
  fs.writeFileSync(process.env.NTPRO_FORCED_STOP_SIGNALLED, "signalled\\n");
});
fs.writeFileSync(process.env.NTPRO_FORCED_STOP_READY, "ready\\n");
setInterval(() => {}, 1000);
`,
    { mode: 0o755 },
  );

  const launch = startLauncher(forcedWorkspace, await freePort(), "forced-stop", {
    launcher: forcedLauncher,
    cwd: forcedPackage,
    env: {
      NTPRO_NODE_SHUTDOWN_TIMEOUT_MS: "1",
      NTPRO_SERVICE_STOP_TIMEOUT_MS: "1500",
      NTPRO_FORCED_STOP_READY: fakeReady,
      NTPRO_FORCED_STOP_SIGNALLED: fakeSignalled,
    },
  });
  const guardianPid = await waitFor("forced-stop guardian", 5_000, () =>
    guardianPidFor(forcedWorkspace)
  );
  const servicePid = await waitFor("forced-stop service", 5_000, () => {
    const pid = servicePidFor(forcedWorkspace);
    return pid && processAlive(pid) && fs.existsSync(fakeReady) ? pid : undefined;
  });
  launch.child.kill("SIGTERM");
  await waitFor("forced-stop SIGINT delivery", 1_000, () => fs.existsSync(fakeSignalled));
  assert(processAlive(servicePid), "forced-stop fixture did not ignore SIGINT");
  assert(
    readPid(lockPath(forcedWorkspace)) === guardianPid,
    "forced-stop path released guardian lock before service exit",
  );
  assert(await waitForExit(launch.child, 5_000), "forced-stop launcher did not exit");
  assert(
    launch.child.exitCode !== 0 && launch.child.signalCode === null,
    `forced-stop launcher reported success: code=${launch.child.exitCode} signal=${launch.child.signalCode}`,
  );
  await waitFor("forced-stop service exit", 3_000, () => !processAlive(servicePid));
  await waitFor("forced-stop lock cleanup", 3_000, () =>
    !fs.existsSync(lockPath(forcedWorkspace))
  );
  const log = combinedLog(launch);
  assert(log.includes("已强制终止"), "forced-stop path omitted forced termination error");
  assert(!log.includes("NTPRO 已安全停止"), "forced-stop path masqueraded as safe stop");
};
const killLauncherDuringStartup = async (workspacePath, port) => {
  const launch = startLauncher(workspacePath, port, "startup-launcher-kill");
  const guardianPid = await waitFor("startup guardian ownership", 10_000, () => {
    const ownerPid = guardianPidFor(workspacePath);
    return ownerPid && ownerPid !== launch.child.pid && processAlive(ownerPid)
      ? ownerPid
      : undefined;
  });
  let observedServicePid = servicePidFor(workspacePath);
  launch.child.kill("SIGKILL");
  assert(await waitForExit(launch.child, 5_000), "startup launcher SIGKILL did not exit");
  await waitFor("startup guardian cleanup", 60_000, () => {
    observedServicePid ||= servicePidFor(workspacePath);
    return !processAlive(guardianPid) && !fs.existsSync(lockPath(workspacePath));
  });
  if (observedServicePid) {
    assert(!processAlive(observedServicePid), "startup launcher kill orphaned service");
  }
};
const killLauncherOnly = async (launch) => {
  const servicePid = servicePidFor();
  assert(servicePid && processAlive(servicePid), "launcher-only kill omitted live service PID");
  launch.child.kill("SIGKILL");
  assert(await waitForExit(launch.child, 5_000), "launcher-only SIGKILL did not exit launcher");
  await waitFor("guardian service cleanup", 20_000, () => !processAlive(servicePid));
  await waitFor("guardian lock cleanup", 5_000, () =>
    !fs.existsSync(lockPath())
  );
  return servicePid;
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
const createDemo = async (port, cookie) => {
  const response = await fetch(`http://127.0.0.1:${port}/api/product/v1/demo-runs`, {
    method: "POST",
    headers: { cookie, "content-type": "application/json" },
    body: JSON.stringify({
      strategy_id: "ema_cross_btcusdt_v1",
      strategy_version_id: "ema_cross_btcusdt_v1@v1",
      environment: "sandbox",
      supervisor_node_id: "mvp-node-001",
      account_ref: "account://sandbox/SANDBOX-001",
      venue_ref: "venue://sandbox/BINANCE_TESTNET",
      user_confirmed: true,
    }),
    signal: AbortSignal.timeout(10_000),
  });
  const body = await response.json();
  assert(response.status === 201, `package Demo creation failed: ${JSON.stringify(body)}`);
  assert(body.data?.run_id?.startsWith("demo-"), "package Demo run ID drifted");
  return body.data.run_id;
};
const actOnDemo = async (port, cookie, runId, action) => {
  const response = await fetch(
    `http://127.0.0.1:${port}/api/product/v1/demo-runs/${runId}/actions`,
    {
      method: "POST",
      headers: { cookie, "content-type": "application/json" },
      body: JSON.stringify({ run_id: runId, action, user_confirmed: true }),
      signal: AbortSignal.timeout(15_000),
    },
  );
  const body = await response.json();
  assert(response.status === 200, `Demo ${action} failed: ${JSON.stringify(body)}`);
};
const readRun = async (port, cookie, runId) => {
  const response = await fetch(
    `http://127.0.0.1:${port}/api/product/v1/runs/${runId}`,
    { headers: { cookie }, signal: AbortSignal.timeout(5_000) },
  );
  const body = await response.json();
  assert(response.status === 200, `persisted Run readback failed: ${JSON.stringify(body)}`);
  assert(body.data?.run_id === runId, "persisted Run identity drifted");
  return body.data;
};
const waitForRunLifecycle = async (port, cookie, runId, lifecycle) =>
  waitFor(`Run ${runId} lifecycle ${lifecycle}`, 15_000, async () => {
    const run = await readRun(port, cookie, runId);
    return run.lifecycle === lifecycle ? run : undefined;
  });

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
        NTPRO_LOCAL_DELIVERY_OUTPUT: packageDir,
      },
      encoding: "utf8",
      timeout: 600_000,
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
    path.join(packageDir, "LICENSE"),
  ]) {
    assert(fs.existsSync(required), `delivery package omitted ${required}`);
  }
  const manifest = JSON.parse(
    fs.readFileSync(path.join(packageDir, "delivery-manifest.json"), "utf8"),
  );
  assert(/^([a-f0-9]{40})$/.test(manifest.source_sha), "manifest source SHA is invalid");
  assert(
    ["git_head_clean_workspace_build", "git_head_dirty_workspace_build"].includes(
      manifest.source_binding,
    ),
    "manifest source binding is missing",
  );
  assert(manifest.platform?.os && manifest.platform?.arch, "manifest platform is missing");
  assert(manifest.platform?.rust_target, "manifest Rust target is missing");
  const sourceSha = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: repoRoot,
    encoding: "utf8",
  }).stdout.trim();
  const sourceDirty = Boolean(
    spawnSync("git", ["status", "--porcelain", "--untracked-files=normal"], {
      cwd: repoRoot,
      encoding: "utf8",
    }).stdout.trim(),
  );
  assert(manifest.source_sha === sourceSha, "manifest source SHA does not match checkout");
  assert(manifest.source_tree_dirty === sourceDirty, "manifest dirty state does not match checkout");
  assert(
    manifest.source_binding ===
      (sourceDirty ? "git_head_dirty_workspace_build" : "git_head_clean_workspace_build"),
    "manifest source binding contradicts checkout state",
  );
  const components = manifest.components ?? {};
  const componentPaths = {
    nautilus_sha256: path.join(packageDir, "bin", "nautilus"),
    ntpro_node_sha256: path.join(packageDir, "bin", "ntpro-node"),
    strategy_workbench_index_sha256: path.join(
      packageDir,
      "apps",
      "strategy-workbench",
      "dist",
      "index.html",
    ),
    launcher_sha256: launcher,
    node_config_sha256: path.join(packageDir, "configs", "nodes", "btc-ema-shadow.toml"),
    backtest_config_sha256: path.join(
      packageDir,
      "configs",
      "backtests",
      "ema-cross-btcusdt-product.toml",
    ),
  };
  for (const [field, filePath] of Object.entries(componentPaths)) {
    assert(components[field] === sha256File(filePath), `manifest hash mismatch: ${field}`);
  }
  const frontendRoot = path.join(packageDir, "apps", "strategy-workbench", "dist");
  assert(
    components.strategy_workbench_tree_sha256 === sha256Tree(frontendRoot),
    "manifest frontend tree hash mismatch",
  );
  assert(
    components.delivery_payload_tree_sha256 ===
      sha256Tree(packageDir, "delivery-manifest.json"),
    "manifest delivery payload tree hash mismatch",
  );

  const startupKillPort = await freePort();
  const startupKillWorkspace = path.join(root, "startup-kill-workspace");
  await killLauncherDuringStartup(startupKillWorkspace, startupKillPort);

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
  const demoRunId = await createDemo(port, initial.cookie);
  await actOnDemo(port, initial.cookie, demoRunId, "start");
  await waitForRunLifecycle(port, initial.cookie, demoRunId, "running");
  const demoNodePid = await waitFor("active Demo node PID", 10_000, () => {
    const pidPath = path.join(workspace, "nodes", "mvp-node-001", "pid.json");
    if (!fs.existsSync(pidPath)) return undefined;
    const artifact = JSON.parse(fs.readFileSync(pidPath, "utf8"));
    return processAlive(artifact.pid) ? artifact.pid : undefined;
  });
  const retentionProbe = path.join(workspace, "local-delivery-retention-probe.txt");
  fs.writeFileSync(retentionProbe, `${createdRunId}\n`);
  await stopWithSignal(currentLaunch, "SIGTERM");
  assert(!processAlive(demoNodePid), "SIGTERM left the active Demo node running");

  currentLaunch = startLauncher(workspace, port, "term-restart");
  const restarted = await waitForReady(currentLaunch);
  assert(fs.readFileSync(retentionProbe, "utf8").trim() === createdRunId, "workspace data was lost");
  await readRun(port, restarted.cookie, createdRunId);
  await waitForRunLifecycle(port, restarted.cookie, demoRunId, "stopped");
  await stopWithSignal(currentLaunch, "SIGHUP");

  currentLaunch = startLauncher(workspace, port, "hup-restart");
  const guardianSession = await waitForReady(currentLaunch);
  const guardianDemoRunId = await createDemo(port, guardianSession.cookie);
  await actOnDemo(port, guardianSession.cookie, guardianDemoRunId, "start");
  await waitForRunLifecycle(port, guardianSession.cookie, guardianDemoRunId, "running");
  const guardianDemoNodePid = await waitFor("guardian Demo node PID", 10_000, () => {
    const pidPath = path.join(workspace, "nodes", "mvp-node-001", "pid.json");
    if (!fs.existsSync(pidPath)) return undefined;
    const artifact = JSON.parse(fs.readFileSync(pidPath, "utf8"));
    return processAlive(artifact.pid) ? artifact.pid : undefined;
  });
  await killLauncherOnly(currentLaunch);
  assert(!processAlive(guardianDemoNodePid), "guardian left active Demo node running");

  currentLaunch = startLauncher(workspace, port, "guardian-restart");
  const guardianRestarted = await waitForReady(currentLaunch);
  await waitForRunLifecycle(port, guardianRestarted.cookie, guardianDemoRunId, "stopped");
  await killAbnormally(currentLaunch);
  await sleep(250);
  assert(fs.existsSync(lockPath()), "abnormal exit did not exercise stale lock");
  assert(fs.statSync(lockPath()).isFile(), "runtime lock is not an atomic owner file");

  const competingPort = await freePort();
  const recoveryA = startLauncher(workspace, port, "abnormal-recovery-a");
  const recoveryB = startLauncher(workspace, competingPort, "abnormal-recovery-b");
  const recoveries = await Promise.allSettled([
    waitForReady(recoveryA),
    waitForReady(recoveryB),
  ]);
  const winners = recoveries
    .map((result, index) => ({ result, launch: index === 0 ? recoveryA : recoveryB }))
    .filter(({ result }) => result.status === "fulfilled");
  const losers = recoveries
    .map((result, index) => ({ result, launch: index === 0 ? recoveryA : recoveryB }))
    .filter(({ result }) => result.status === "rejected");
  assert(winners.length === 1 && losers.length === 1, "stale-lock race did not produce one winner");
  assert(
    [73, 75].includes(losers[0].launch.child.exitCode),
    `stale-lock loser exited ${losers[0].launch.child.exitCode}`,
  );
  currentLaunch = winners[0].launch;
  await stopWithSignal(currentLaunch, "SIGINT", true);

  await verifyForcedStopFailsClosed();

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
    !fs.existsSync(lockPath(occupiedWorkspace)),
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
        package_hashes_recomputed: true,
        source_binding_verified: true,
        startup_launcher_kill_guarded: true,
        terminal_process_group_ctrl_c: true,
        single_entrypoint: true,
        production_browser: true,
        duplicate_launch_rejected: true,
        normal_stop_clean: true,
        term_and_hup_mapped_to_ctrl_c: true,
        active_demo_stopped_on_term: true,
        launcher_only_kill_guarded: true,
        process_tree_exit_verified: true,
        concurrent_stale_lock_single_winner: true,
        same_workspace_restart: true,
        backtest_persisted: true,
        abnormal_exit_recovered: true,
        forced_stop_fail_closed: true,
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
