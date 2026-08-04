import crypto from "node:crypto";
import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { spawn, spawnSync } from "node:child_process";

const nautilusBin = path.resolve(process.env.NTPRO_NAUTILUS_BIN || "target/debug/nautilus");
const nodeBin = path.resolve(process.env.NTPRO_NODE_BIN || "target/debug/ntpro-node");
const cargoBin = process.env.NTPRO_CARGO_BIN || "cargo";
const nodeConfig = path.resolve(process.env.NTPRO_MVP_CONFIG || "configs/nodes/btc-ema-shadow.toml");
const backtestConfigArg = process.env.NTPRO_MVP_BACKTEST_CONFIG
  || "examples/rust/backtest/minimal_engine_smoke.toml";
const backtestConfig = path.resolve(backtestConfigArg);
const root = fs.mkdtempSync(path.join(os.tmpdir(), "ntpro-mvp-011-acceptance-"));
const evidenceDir = path.resolve(
  process.env.NTPRO_MVP_ACCEPTANCE_EVIDENCE_DIR
    || fs.mkdtempSync(path.join(os.tmpdir(), "ntpro-mvp-011-evidence-")),
);
const workspace = path.join(root, "workspace");
const backtestA = path.join(root, "backtest-a");
const backtestB = path.join(root, "backtest-b");
const resultPath = path.join(evidenceDir, "result.json");
const serverLogPath = path.join(evidenceDir, "mvp-server.log");
const identityFalseFields = [
  "external_venue_connection",
  "order_submission_allowed",
  "order_mutation_allowed",
  "automatic_retry_allowed",
  "automatic_remediation_allowed",
  "real_orders_submitted",
];
const statusFalseFields = [
  "http_success_implies_technical_health",
  "process_alive_implies_technical_health",
  "backtest_reference_implies_research_accepted",
  "backtest_complete_implies_trading_readiness",
  ...identityFalseFields,
];
const sharedFalseFields = [
  "http_success_implies_technical_health",
  "process_alive_implies_technical_health",
  "backtest_reference_implies_research_accepted",
  "backtest_complete_implies_trading_readiness",
  "raw_event_store_exposed",
  "raw_venue_payload_exposed",
  ...identityFalseFields,
];
const eventFalseFields = [
  "raw_event_store_exposed",
  "raw_event_payload_exposed",
  "raw_errors_exposed",
  "supervisor_actions_exposed",
  "trading_controls_exposed",
];
const operationalFalseFields = [
  "external_venue_connection",
  "production_venue_connection",
  "testnet_public_network_connection",
  "external_network_attempted",
  "real_orders_submitted",
  "unsupported_supervisor_actions_exposed",
  "trading_controls_exposed",
  "automatic_retry_allowed",
  "automatic_remediation_allowed",
  "raw_errors_exposed",
];
const actionFalseFields = [
  "external_venue_connection",
  "production_venue_connection",
  "external_network_attempted",
  "order_submission_allowed",
  "order_mutation_allowed",
  "automatic_retry_allowed",
  "automatic_remediation_allowed",
  "real_orders_submitted",
];
const expectedStrategyResultSha256 = "5d1e0903a1060f75b28b30241d2f086c6d2dec1faf171ddf0fb40bf09d369e1c";

fs.mkdirSync(evidenceDir, { recursive: true });

const assert = (condition, message) => {
  if (!condition) throw new Error(message);
};
const redact = (value) => String(value)
  .replace(/(access_token=)[^\s&]+/g, "$1[REDACTED]")
  .replace(/(ntpro_mvp_(?:institution|operator)_access=)[^;\s]+/g, "$1[REDACTED]");
const sha256 = (value) => crypto.createHash("sha256").update(value).digest("hex");
const sleep = (millis) => new Promise((resolve) => setTimeout(resolve, millis));
const serverExited = (server) => server.exitCode !== null || server.signalCode !== null;
const waitForServerExit = (server, timeoutMs) => {
  if (serverExited(server)) return Promise.resolve(true);
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
const waitFor = async (description, timeoutMs, check) => {
  const deadline = Date.now() + timeoutMs;
  let lastError;
  while (Date.now() < deadline) {
    try {
      const value = await check();
      if (value) return value;
    } catch (error) {
      lastError = error;
    }
    await sleep(100);
  }
  const suffix = lastError ? `: ${redact(lastError.message)}` : "";
  throw new Error(`timed out waiting for ${description}${suffix}`);
};
const requireFile = (filePath, label) => {
  assert(fs.existsSync(filePath), `${label} is missing: ${filePath}`);
  assert(fs.statSync(filePath).isFile(), `${label} is not a file: ${filePath}`);
};
const run = (args) => {
  const result = spawnSync(nautilusBin, args, {
    encoding: "utf8",
    timeout: 120_000,
    env: { ...process.env, NO_COLOR: "1" },
  });
  assert(!result.error, `command failed to start: ${result.error?.message}`);
  assert(
    result.status === 0,
    `command exited ${result.status}: ${redact(result.stderr || result.stdout)}`,
  );
  return result;
};
const runStrategyGolden = () => {
  const result = spawnSync(
    cargoBin,
    [
      "test", "--quiet", "-p", "nautilus-backtest", "--test", "golden_trace_backtest",
      "rust_backtest_engine_replays_mvp_ema_strategy_canonical_result", "--", "--nocapture",
    ],
    {
      encoding: "utf8",
      timeout: 180_000,
      maxBuffer: 10 * 1024 * 1024,
      env: { ...process.env, NO_COLOR: "1" },
    },
  );
  assert(!result.error, `strategy golden test failed to start: ${result.error?.message}`);
  assert(
    result.status === 0,
    `strategy golden test exited ${result.status}: ${redact(result.stderr || result.stdout)}`,
  );
  const output = `${result.stdout}\n${result.stderr}`;
  const canonical = output.match(/mvp_ema_canonical_result=(\{.*\})/)?.[1];
  assert(canonical, "strategy golden test omitted canonical result");
  const parsed = JSON.parse(canonical);
  assert(parsed.total_events > 0, "strategy golden result must contain events");
  assert(parsed.total_orders > 0, "strategy golden result must contain orders");
  assert(parsed.total_positions > 0, "strategy golden result must contain positions");
  assert(Object.keys(parsed.pnl_stats || {}).length > 0, "strategy golden result omitted PnL stats");
  return { canonical, parsed };
};
const bootstrapCookie = async (accessUrl, expectedPath, expectedCookie) => {
  const response = await fetch(accessUrl, { redirect: "manual", signal: AbortSignal.timeout(5_000) });
  assert(response.status === 303, `${expectedPath} bootstrap expected 303, got ${response.status}`);
  const location = response.headers.get("location");
  assert(location === expectedPath, `${expectedPath} bootstrap returned unsafe location ${location}`);
  assert(!location.includes("access_token"), `${expectedPath} redirect retained access token`);
  const setCookie = response.headers.get("set-cookie");
  assert(setCookie, `${expectedPath} bootstrap omitted cookie`);
  const cookie = setCookie.split(";", 1)[0];
  assert(cookie.startsWith(`${expectedCookie}=`), `${expectedPath} set wrong role cookie`);
  assert(setCookie.includes("HttpOnly"), `${expectedPath} cookie is not HttpOnly`);
  assert(setCookie.includes("SameSite=Strict"), `${expectedPath} cookie is not SameSite=Strict`);
  return cookie;
};
const fetchJson = async (url, cookie, options = {}) => {
  const response = await fetch(url, {
    ...options,
    headers: { ...(options.headers || {}), cookie },
    redirect: "manual",
    signal: AbortSignal.timeout(5_000),
  });
  const text = await response.text();
  let body;
  try {
    body = JSON.parse(text);
  } catch {
    throw new Error(`${options.method || "GET"} ${url} returned non-JSON status=${response.status}`);
  }
  return { status: response.status, body };
};
const assertFalseBoundaries = (boundaries, label, fields) => {
  assert(boundaries && typeof boundaries === "object", `${label} boundaries are missing`);
  for (const field of fields) {
    assert(Object.hasOwn(boundaries, field), `${label}.${field} is required`);
    assert(boundaries[field] === false, `${label}.${field} must be false`);
  }
};
const expectFailure = (description, check) => {
  try {
    check();
  } catch {
    return;
  }
  throw new Error(`${description} unexpectedly passed`);
};
expectFailure(
  "missing required-false boundary self-test",
  () => assertFalseBoundaries({ first: false }, "self_test", ["first", "second"]),
);
expectFailure(
  "true required-false boundary self-test",
  () => assertFalseBoundaries({ first: true }, "self_test", ["first"]),
);
const redactionProbe = redact(
  "http://127.0.0.1/?access_token=probe-token ntpro_mvp_operator_access=probe-cookie;",
);
assert(!redactionProbe.includes("probe-token"), "access token redaction self-test failed");
assert(!redactionProbe.includes("probe-cookie"), "role cookie redaction self-test failed");
const assertSharedStatus = (body, expectedRuntime) => {
  assert(body.schema_version === "ntpro.mvp_shared_status_api.response.v1", "shared schema mismatch");
  assert(body.contract_version === "ntpro.mvp_shared_status_api.v1", "shared contract mismatch");
  assert(body.identity?.identities?.node_id === "mvp-node-001", "shared node identity mismatch");
  assert(body.identity?.identities?.environment === "sandbox", "shared environment is not sandbox");
  assert(body.status?.runtime?.status === expectedRuntime, `shared runtime expected ${expectedRuntime}`);
  assert(body.status?.trading_readiness?.status === "blocked", "trading readiness must remain blocked");
  assert(body.boundaries?.read_only === true, "shared status must remain read-only");
  assert(body.identity?.boundaries?.read_only_product_contract === true, "identity must remain read-only");
  assert(body.status?.boundaries?.read_only_product_contract === true, "status contract must remain read-only");
  assertFalseBoundaries(body.boundaries, "shared", sharedFalseFields);
  assertFalseBoundaries(body.identity?.boundaries, "identity", identityFalseFields);
  assertFalseBoundaries(body.status?.boundaries, "status", statusFalseFields);
};
const assertOperationalSnapshot = (body, expectedLifecycle) => {
  assert(body.schema_version === "ntpro.mvp_control_center_snapshot.v2", "control center schema mismatch");
  assert(body.local_only === true, "control center must remain local-only");
  assert(body.overview?.node_count === 1, "control center must expose exactly one node");
  assert(body.node?.node_id === "mvp-node-001", "control center node identity mismatch");
  assert(body.node?.lifecycle_state === expectedLifecycle, `node lifecycle expected ${expectedLifecycle}`);
  assert(body.overview?.sandbox_only === true, "control center must remain sandbox-only");
  assert(body.boundaries?.read_only === true, "control center projection must remain read-only");
  assert(body.boundaries?.supervisor_actions_exposed === true, "lifecycle actions must be explicit");
  assertFalseBoundaries(body.boundaries, "control_center", operationalFalseFields);
};
const assertAction = (body, action, previousState, currentState) => {
  assert(
    body.schema_version === "ntpro.mvp_control_center_lifecycle_action.response.v1",
    `${action} schema mismatch`,
  );
  assert(
    body.contract_version === "ntpro.mvp_control_center_lifecycle_action.v1",
    `${action} contract mismatch`,
  );
  assert(body.target_node_id === "mvp-node-001", `${action} target mismatch`);
  assert(body.action_name === action, `${action} name mismatch`);
  assert(body.result?.status === "succeeded", `${action} did not succeed`);
  assert(body.result?.previous_state === previousState, `${action} previous state mismatch`);
  assert(body.result?.current_state === currentState, `${action} current state mismatch`);
  assert(body.boundaries?.supervisor_lifecycle_action === true, `${action} lifecycle boundary missing`);
  assertFalseBoundaries(body.boundaries, action, actionFalseFields);
};

let server;
let serverLog = "";
let failure;
let passResult;

try {
  requireFile(nautilusBin, "nautilus binary");
  requireFile(nodeBin, "ntpro-node binary");
  requireFile(nodeConfig, "MVP node config");
  requireFile(backtestConfig, "deterministic backtest config");
  assert(!fs.existsSync(workspace), "MVP workspace must not exist before acceptance starts");

  const runId = "mvp-011-deterministic-engine-smoke";
  run(["backtest", "run", "--config", backtestConfigArg, "--run-id", runId, "--output", backtestA]);
  run(["backtest", "run", "--config", backtestConfigArg, "--run-id", runId, "--output", backtestB]);
  const summaryA = fs.readFileSync(path.join(backtestA, "summary.txt"));
  const summaryB = fs.readFileSync(path.join(backtestB, "summary.txt"));
  assert(summaryA.equals(summaryB), "deterministic backtest summaries differ");
  const summaryText = summaryA.toString("utf8");
  for (const expected of [
    "mode=engine-smoke",
    `run_id=${runId}`,
    "quotes_loaded=120",
    "engine_started=true",
    "runtime_status=completed",
  ]) {
    assert(summaryText.includes(expected), `backtest summary omitted ${expected}`);
  }
  const strategyGoldenA = runStrategyGolden();
  const strategyGoldenB = runStrategyGolden();
  assert(
    strategyGoldenA.canonical === strategyGoldenB.canonical,
    "strategy golden canonical results differ",
  );
  const strategyResultSha256 = sha256(strategyGoldenA.canonical);
  assert(
    strategyResultSha256 === expectedStrategyResultSha256,
    `strategy golden digest drifted: expected ${expectedStrategyResultSha256}, got ${strategyResultSha256}`,
  );

  const port = await new Promise((resolve, reject) => {
    const listener = net.createServer();
    listener.once("error", reject);
    listener.listen(0, "127.0.0.1", () => {
      const address = listener.address();
      listener.close((error) => error ? reject(error) : resolve(address.port));
    });
  });
  const baseUrl = `http://127.0.0.1:${port}`;
  server = spawn(
    nautilusBin,
    [
      "mvp", "serve",
      "--config", nodeConfig,
      "--workspace", workspace,
      "--bind", `127.0.0.1:${port}`,
      "--ntpro-node-bin", nodeBin,
      "--startup-timeout-ms", "10000",
      "--node-max-runtime-ms", "120000",
    ],
    { stdio: ["ignore", "pipe", "pipe"], env: { ...process.env, NO_COLOR: "1" } },
  );
  server.stdout.on("data", (chunk) => { serverLog += chunk.toString(); });
  server.stderr.on("data", (chunk) => { serverLog += chunk.toString(); });

  const access = await waitFor("MVP role bootstrap URLs", 30_000, () => {
    const institution = serverLog.match(/institution_workbench_url=(\S+)/)?.[1];
    const operator = serverLog.match(/control_center_url=(\S+)/)?.[1];
    return institution && operator ? { institution, operator } : undefined;
  });
  if (process.env.NTPRO_MVP_ACCEPTANCE_FORCE_TOKEN_FAILURE === "1") {
    throw new Error(`forced token-bearing failure: ${access.institution}`);
  }
  const institutionCookie = await bootstrapCookie(
    access.institution,
    "/institution-workbench",
    "ntpro_mvp_institution_access",
  );
  const operatorCookie = await bootstrapCookie(
    access.operator,
    "/control-center",
    "ntpro_mvp_operator_access",
  );

  const sharedRunning = await fetchJson(`${baseUrl}/api/mvp/v1/status`, institutionCookie);
  assert(sharedRunning.status === 200, `institution shared status expected 200, got ${sharedRunning.status}`);
  assertSharedStatus(sharedRunning.body, "running");
  const correlation = await fetchJson(`${baseUrl}/api/mvp/v1/event-correlation`, institutionCookie);
  assert(correlation.status === 200, `institution event correlation expected 200, got ${correlation.status}`);
  assert(correlation.body.event?.node_id === "mvp-node-001", "event node identity mismatch");
  assert(correlation.body.boundaries?.read_only === true, "event correlation must remain read-only");
  assertFalseBoundaries(correlation.body.boundaries, "event", eventFalseFields);

  const operationalRunning = await fetchJson(`${baseUrl}/api/mvp/v1/control-center`, operatorCookie);
  assert(operationalRunning.status === 200, `operator snapshot expected 200, got ${operationalRunning.status}`);
  assertOperationalSnapshot(operationalRunning.body, "running");

  const actionBase = `${baseUrl}/api/mvp/v1/control-center/nodes/mvp-node-001/actions`;
  const unauthorized = await fetch(`${actionBase}/stop`, {
    method: "POST",
    redirect: "manual",
    signal: AbortSignal.timeout(5_000),
  });
  assert(unauthorized.status === 403, `unauthorized action expected 403, got ${unauthorized.status}`);
  const institutionAction = await fetch(`${actionBase}/stop`, {
    method: "POST",
    headers: { cookie: institutionCookie },
    redirect: "manual",
    signal: AbortSignal.timeout(5_000),
  });
  assert(institutionAction.status === 403, `institution action expected 403, got ${institutionAction.status}`);
  const lifecycleGet = await fetch(`${actionBase}/stop`, {
    headers: { cookie: operatorCookie },
    redirect: "manual",
    signal: AbortSignal.timeout(5_000),
  });
  assert(lifecycleGet.status === 405, `lifecycle GET expected 405, got ${lifecycleGet.status}`);

  const stop = await fetchJson(`${actionBase}/stop`, operatorCookie, { method: "POST" });
  assert(stop.status === 200, `operator stop expected 200, got ${stop.status}`);
  assertAction(stop.body, "stop", "running", "stopped");
  await waitFor("stopped operational projection", 10_000, async () => {
    const snapshot = await fetchJson(`${baseUrl}/api/mvp/v1/control-center`, operatorCookie);
    if (snapshot.status !== 200 || snapshot.body.node?.lifecycle_state !== "stopped") return undefined;
    assertOperationalSnapshot(snapshot.body, "stopped");
    return true;
  });

  const start = await fetchJson(`${actionBase}/start`, operatorCookie, { method: "POST" });
  assert(start.status === 200, `operator start expected 200, got ${start.status}`);
  assertAction(start.body, "start", "stopped", "running");
  await waitFor("restarted shared and operational projections", 10_000, async () => {
    const [shared, snapshot] = await Promise.all([
      fetchJson(`${baseUrl}/api/mvp/v1/status`, institutionCookie),
      fetchJson(`${baseUrl}/api/mvp/v1/control-center`, operatorCookie),
    ]);
    if (
      shared.status !== 200
      || snapshot.status !== 200
      || shared.body.status?.runtime?.status !== "running"
      || snapshot.body.node?.lifecycle_state !== "running"
    ) return undefined;
    assertSharedStatus(shared.body, "running");
    assertOperationalSnapshot(snapshot.body, "running");
    return true;
  });

  server.kill("SIGINT");
  assert(await waitForServerExit(server, 10_000), "MVP server did not stop after SIGINT");
  assert(server.exitCode === 0, `MVP server exited with code ${server.exitCode}`);
  assert(serverLog.includes("mvp.serve status=stopped"), "MVP server omitted stopped evidence");

  const registry = JSON.parse(fs.readFileSync(path.join(workspace, "supervisor/registry.json"), "utf8"));
  assert(Object.keys(registry.nodes || {}).length === 1, "final registry must contain exactly one node");
  const finalNode = registry.nodes?.["mvp-node-001"];
  assert(finalNode?.process?.state === "stopped", "final registry process is not stopped");
  assert(finalNode?.last_known_status?.lifecycle_state === "stopped", "final registry lifecycle is not stopped");
  assert(finalNode?.last_known_status?.external_venue_connection === false, "final registry opened external venue");
  assert(finalNode?.last_known_status?.real_orders_submitted === false, "final registry submitted real orders");
  const finalStatus = JSON.parse(fs.readFileSync(path.join(workspace, "mvp/status_contract.json"), "utf8"));
  assert(finalStatus.runtime?.status === "stopped", "final MVP status contract is not stopped");
  assert(finalStatus.trading_readiness?.status === "blocked", "final trading readiness is not blocked");
  assert(finalStatus.boundaries?.read_only_product_contract === true, "final status must remain read-only");
  assertFalseBoundaries(finalStatus.boundaries, "final_status", statusFalseFields);

  passResult = {
    schema_version: "ntpro.mvp_acceptance_evidence.v1",
    status: "pass",
    deterministic_backtest: {
      runs: 2,
      byte_equal: true,
      summary_sha256: sha256(summaryA),
      quotes_loaded: 120,
      runtime_status: "completed",
      strategy_result_runs: 2,
      strategy_result_sha256: strategyResultSha256,
      total_events: strategyGoldenA.parsed.total_events,
      total_orders: strategyGoldenA.parsed.total_orders,
      total_positions: strategyGoldenA.parsed.total_positions,
      pnl_currencies: Object.keys(strategyGoldenA.parsed.pnl_stats),
    },
    clean_workspace: { initial_state: "absent", isolated_temp_root: true },
    runtime: {
      supervisor_count: 1,
      node_count: 1,
      node_id: "mvp-node-001",
      environment: "sandbox",
      transitions: ["running", "stopped", "running", "stopped"],
    },
    access: {
      institution_read_only: true,
      institution_lifecycle_action_status: 403,
      unauthenticated_lifecycle_action_status: 403,
      lifecycle_get_status: 405,
      operator_stop_start: true,
      bootstrap_url_clean: true,
      boundary_validator_negative_cases: 2,
      diagnostic_redaction_selftest: true,
    },
    shutdown: { graceful: true, process_state: "stopped", lifecycle_state: "stopped" },
    boundaries: {
      external_venue_connection: false,
      real_orders_submitted: false,
      order_submission_allowed: false,
      order_mutation_allowed: false,
      automatic_retry_allowed: false,
      automatic_remediation_allowed: false,
      trading_readiness: "blocked",
    },
  };
} catch (error) {
  failure = error instanceof Error ? error : new Error(String(error));
} finally {
  if (server && !serverExited(server)) {
    server.kill("SIGINT");
    if (!await waitForServerExit(server, 5_000)) {
      server.kill("SIGKILL");
      await waitForServerExit(server, 5_000);
    }
  }
  const sanitizedLog = redact(serverLog);
  assert(!/access_token=(?!\[REDACTED\])/.test(sanitizedLog), "MVP log contains an access token");
  assert(
    !/ntpro_mvp_(institution|operator)_access=(?!\[REDACTED\])/.test(sanitizedLog),
    "MVP log contains a role cookie",
  );
  fs.writeFileSync(serverLogPath, sanitizedLog);
  const result = failure
    ? { schema_version: "ntpro.mvp_acceptance_evidence.v1", status: "fail", error: redact(failure.message) }
    : passResult;
  const serialized = `${JSON.stringify(result, null, 2)}\n`;
  assert(!/access_token=(?!\[REDACTED\])/.test(serialized), "result.json contains an access token");
  assert(
    !/ntpro_mvp_(institution|operator)_access=(?!\[REDACTED\])/.test(serialized),
    "result.json contains a role cookie",
  );
  fs.writeFileSync(resultPath, serialized);
  fs.rmSync(root, { recursive: true, force: true });
}

if (failure) {
  console.error(`mvp_acceptance=fail error=${redact(failure.message)}`);
  process.exitCode = 1;
} else {
  console.log(
    `mvp_acceptance=pass deterministic_runs=2 summary_sha256=${passResult.deterministic_backtest.summary_sha256} node_count=1 transitions=running,stopped,running,stopped role_boundary=pass graceful_shutdown=pass`,
  );
}
