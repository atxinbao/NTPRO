import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { spawn, spawnSync } from "node:child_process";

const nautilusBin = path.resolve(process.env.NTPRO_NAUTILUS_BIN || "target/debug/nautilus");
const nodeBin = path.resolve(process.env.NTPRO_NODE_BIN || "target/debug/ntpro-node");
const nodeConfig = path.resolve(process.env.NTPRO_MVP_CONFIG || "configs/nodes/btc-ema-shadow.toml");
const root = fs.mkdtempSync(path.join(os.tmpdir(), "ntpro-mvp-012-fault-matrix-"));
const evidenceDir = path.resolve(
  process.env.NTPRO_MVP_FAULT_EVIDENCE_DIR
    || fs.mkdtempSync(path.join(os.tmpdir(), "ntpro-mvp-012-evidence-")),
);
const resultPath = path.join(evidenceDir, "result.json");
const logPath = path.join(evidenceDir, "mvp-fault-matrix.log");
const sessions = [];
const noAutomaticRecoveryObservationMs = 6_000;
const statusFalseFields = [
  "http_success_implies_technical_health",
  "process_alive_implies_technical_health",
  "backtest_reference_implies_research_accepted",
  "backtest_complete_implies_trading_readiness",
  "external_venue_connection",
  "order_submission_allowed",
  "order_mutation_allowed",
  "automatic_retry_allowed",
  "automatic_remediation_allowed",
  "real_orders_submitted",
];
const sharedFalseFields = [
  "http_success_implies_technical_health",
  "process_alive_implies_technical_health",
  "backtest_reference_implies_research_accepted",
  "backtest_complete_implies_trading_readiness",
  "raw_event_store_exposed",
  "raw_venue_payload_exposed",
  "external_venue_connection",
  "order_submission_allowed",
  "order_mutation_allowed",
  "automatic_retry_allowed",
  "automatic_remediation_allowed",
  "real_orders_submitted",
];
const errorFalseFields = [
  "raw_event_store_exposed",
  "raw_venue_payload_exposed",
  "external_venue_connection",
  "order_submission_allowed",
  "order_mutation_allowed",
  "automatic_retry_allowed",
  "automatic_remediation_allowed",
  "real_orders_submitted",
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

fs.mkdirSync(evidenceDir, { recursive: true });

const assert = (condition, message) => {
  if (!condition) throw new Error(message);
};
const sleep = (millis) => new Promise((resolve) => setTimeout(resolve, millis));
const redact = (value) => String(value)
  .replace(/(access_token=)[^\s&]+/g, "$1[REDACTED]")
  .replace(/(ntpro_mvp_(?:institution|operator)_access=)[^;\s]+/g, "$1[REDACTED]");
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
const readJson = (filePath) => JSON.parse(fs.readFileSync(filePath, "utf8"));
const writeJson = (filePath, value) => {
  const temporary = `${filePath}.fault-matrix.tmp`;
  fs.writeFileSync(temporary, `${JSON.stringify(value, null, 2)}\n`);
  fs.renameSync(temporary, filePath);
};
const processIsAlive = (pid) => {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    if (error?.code === "ESRCH") return false;
    throw error;
  }
};
const freePort = () => new Promise((resolve, reject) => {
  const listener = net.createServer();
  listener.once("error", reject);
  listener.listen(0, "127.0.0.1", () => {
    const address = listener.address();
    listener.close((error) => error ? reject(error) : resolve(address.port));
  });
});
const bootstrapCookie = async (accessUrl, expectedPath, expectedCookie) => {
  const response = await fetch(accessUrl, {
    redirect: "manual",
    signal: AbortSignal.timeout(5_000),
  });
  assert(response.status === 303, `${expectedPath} bootstrap expected 303, got ${response.status}`);
  assert(response.headers.get("location") === expectedPath, `${expectedPath} bootstrap location mismatch`);
  const setCookie = response.headers.get("set-cookie");
  assert(setCookie?.includes("HttpOnly"), `${expectedPath} cookie is not HttpOnly`);
  assert(setCookie?.includes("SameSite=Strict"), `${expectedPath} cookie is not SameSite=Strict`);
  const cookie = setCookie.split(";", 1)[0];
  assert(cookie.startsWith(`${expectedCookie}=`), `${expectedPath} set wrong role cookie`);
  return cookie;
};
const fetchJson = async (url, cookie, options = {}) => {
  const response = await fetch(url, {
    ...options,
    headers: { ...(options.headers || {}), ...(cookie ? { cookie } : {}) },
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
const assertFalseFields = (value, label, fields) => {
  assert(value && typeof value === "object", `${label} is missing`);
  for (const field of fields) {
    assert(Object.hasOwn(value, field), `${label}.${field} is required`);
    assert(value[field] === false, `${label}.${field} must be false`);
  }
};
const assertSharedBoundaries = (body, label) => {
  assert(body.status?.trading_readiness?.status === "blocked", `${label} trading readiness opened`);
  assert(body.boundaries?.read_only === true, `${label} shared API is not read-only`);
  assert(body.status?.boundaries?.read_only_product_contract === true, `${label} status is not read-only`);
  assertFalseFields(body.boundaries, `${label}.shared`, sharedFalseFields);
  assertFalseFields(body.status.boundaries, `${label}.status`, statusFalseFields);
};
const assertSharedError = (response, expectedStatus, expectedCode, label) => {
  assert(response.status === expectedStatus, `${label} expected ${expectedStatus}, got ${response.status}`);
  assert(response.body.schema_version === "ntpro.mvp_shared_status_api.error.v1", `${label} error schema mismatch`);
  assert(response.body.error_code === expectedCode, `${label} error code mismatch`);
  assert(response.body.read_only === true, `${label} error envelope is not read-only`);
  assertFalseFields(response.body, `${label}.error`, errorFalseFields);
};
const assertNotHealthy = (body, label, expectedRuntimeStatuses = ["running", "unknown"]) => {
  assertSharedBoundaries(body, label);
  assert(
    expectedRuntimeStatuses.includes(body.status?.runtime?.status),
    `${label} runtime status ${body.status?.runtime?.status} is unexpected`,
  );
  assert(body.status?.technical_health?.status !== "healthy", `${label} reported healthy`);
};
const registryPath = (session) => path.join(session.workspace, "supervisor/registry.json");
const identityPath = (session) => path.join(session.workspace, "mvp/identity_contract.json");
const statusContractPath = (session) => path.join(session.workspace, "mvp/status_contract.json");
const nodeRecord = (session) => {
  const record = readJson(registryPath(session)).nodes?.["mvp-node-001"];
  const pid = record?.process?.pid?.value;
  if (Number.isInteger(pid) && pid > 0) session.observedNodePids.add(pid);
  return record;
};
const nodePid = (session) => {
  const pid = nodeRecord(session)?.process?.pid?.value;
  assert(Number.isInteger(pid) && pid > 0, `${session.name} registry omitted node PID`);
  return pid;
};
const nodePaths = (session) => {
  const record = nodeRecord(session);
  assert(record, `${session.name} registry omitted node record`);
  return { status: record.status_path, metrics: record.metrics_path };
};

const startSession = async (name) => {
  const workspace = path.join(root, name);
  assert(!fs.existsSync(workspace), `${name} workspace must start absent`);
  const port = await freePort();
  const baseUrl = `http://127.0.0.1:${port}`;
  const session = {
    name,
    workspace,
    baseUrl,
    serverLog: "",
    server: undefined,
    observedNodePids: new Set(),
  };
  const server = spawn(
    nautilusBin,
    [
      "mvp", "serve",
      "--config", nodeConfig,
      "--workspace", workspace,
      "--bind", `127.0.0.1:${port}`,
      "--strategy-workbench-dist", "crates/cli/tests/fixtures/strategy-workbench",
      "--ntpro-node-bin", nodeBin,
      "--startup-timeout-ms", "10000",
      "--node-max-runtime-ms", "180000",
      "--node-heartbeat-interval-ms", "1000",
      "--node-shutdown-timeout-ms", "5000",
    ],
    { stdio: ["ignore", "pipe", "pipe"], env: { ...process.env, NO_COLOR: "1" } },
  );
  session.server = server;
  sessions.push(session);
  server.stdout.on("data", (chunk) => { session.serverLog += chunk.toString(); });
  server.stderr.on("data", (chunk) => { session.serverLog += chunk.toString(); });

  const access = await waitFor(`${name} role bootstrap URLs`, 30_000, () => {
    const institution = session.serverLog.match(/institution_workbench_url=(\S+)/)?.[1];
    const operator = session.serverLog.match(/control_center_url=(\S+)/)?.[1];
    return institution && operator ? { institution, operator } : undefined;
  });
  if (process.env.NTPRO_MVP_FAULT_FORCE_TOKEN_FAILURE === "1") {
    throw new Error(`forced token-bearing failure: ${access.operator}`);
  }
  session.institutionCookie = await bootstrapCookie(
    access.institution,
    "/institution-workbench",
    "ntpro_mvp_institution_access",
  );
  session.operatorCookie = await bootstrapCookie(
    access.operator,
    "/control-center",
    "ntpro_mvp_operator_access",
  );
  await waitHealthy(session);
  nodePid(session);
  return session;
};

const waitShared = (session, description, predicate) => waitFor(description, 12_000, async () => {
  const response = await fetchJson(`${session.baseUrl}/api/mvp/v1/status`, session.institutionCookie);
  if (predicate(response)) return response;
  throw new Error(
    `status=${response.status} runtime=${response.body.status?.runtime?.status} process_reasons=${JSON.stringify(response.body.status?.runtime?.reasons || [])} technical=${response.body.status?.technical_health?.status} freshness=${response.body.status?.technical_health?.freshness} error_code=${response.body.error_code || "none"}`,
  );
});
const waitHealthy = (session) => waitShared(session, `${session.name} healthy projection`, (response) => {
  if (response.status !== 200) {
    throw new Error(`${session.name} shared status returned ${response.status} ${response.body?.error_code}`);
  }
  const body = response.body;
  const healthy = body.status?.runtime?.status === "running"
    && body.status?.technical_health?.status === "healthy"
    && body.status?.technical_health?.freshness === "fresh"
    && body.status?.trading_readiness?.status === "blocked";
  if (!healthy) {
    throw new Error(
      `${session.name} runtime=${body.status?.runtime?.status} technical=${body.status?.technical_health?.status} freshness=${body.status?.technical_health?.freshness} reasons=${JSON.stringify(body.status?.technical_health?.reasons || [])}`,
    );
  }
  return true;
}).then((response) => {
  assertSharedBoundaries(response.body, `${session.name}.healthy`);
  return response;
});
const stopSession = async (session) => {
  if (serverExited(session.server)) return;
  session.server.kill("SIGINT");
  assert(await waitForServerExit(session.server, 12_000), `${session.name} server did not stop after SIGINT`);
  assert(session.server.exitCode === 0, `${session.name} server exited with code ${session.server.exitCode}`);
};
const cleanupSession = async (session) => {
  try {
    nodeRecord(session);
  } catch {
    session.serverLog += "\ncleanup_registry=unavailable\n";
  }
  if (!serverExited(session.server)) {
    session.server.kill("SIGINT");
    if (!await waitForServerExit(session.server, 12_000)) {
      session.server.kill("SIGKILL");
      await waitForServerExit(session.server, 5_000);
    }
  }

  for (const pid of session.observedNodePids) {
    if (!processIsAlive(pid)) continue;
    for (const signal of ["SIGCONT", "SIGTERM"]) {
      try {
        process.kill(pid, signal);
      } catch (error) {
        if (error?.code !== "ESRCH") throw error;
      }
    }
    const stopped = await waitFor(`cleanup of ${session.name} node ${pid}`, 5_000, () => (
      !processIsAlive(pid) ? true : undefined
    )).catch(() => false);
    if (!stopped && processIsAlive(pid)) {
      process.kill(pid, "SIGKILL");
      await waitFor(`forced cleanup of ${session.name} node ${pid}`, 5_000, () => (
        !processIsAlive(pid) ? true : undefined
      ));
    }
  }
  const leakedPids = [...session.observedNodePids].filter(processIsAlive);
  assert(leakedPids.length === 0, `${session.name} leaked node PIDs: ${leakedPids.join(",")}`);
  session.cleanupVerified = true;
};
const actionUrl = (session, action) => (
  `${session.baseUrl}/api/mvp/v1/control-center/nodes/mvp-node-001/actions/${action}`
);

const observeNoAutomaticRecovery = async (
  session,
  label,
  expectedProcessState,
  expectedPid,
  verifyShared,
) => {
  const initialRecord = nodeRecord(session);
  const eventsPath = initialRecord?.events_log_path;
  const initialEvents = eventsPath && fs.existsSync(eventsPath) ? fs.readFileSync(eventsPath, "utf8") : "";
  const startedAt = Date.now();
  let samples = 0;

  while (Date.now() - startedAt < noAutomaticRecoveryObservationMs) {
    const record = nodeRecord(session);
    assert(record?.process?.state === expectedProcessState, `${label} process state changed automatically`);
    assert(record?.process?.pid?.value === expectedPid, `${label} process PID changed automatically`);
    const response = await fetchJson(`${session.baseUrl}/api/mvp/v1/status`, session.institutionCookie);
    verifyShared(response);
    samples += 1;
    await sleep(500);
  }

  const finalEvents = eventsPath && fs.existsSync(eventsPath) ? fs.readFileSync(eventsPath, "utf8") : "";
  assert(finalEvents === initialEvents, `${label} lifecycle events changed without operator action`);
  assert(samples >= 10, `${label} observation did not span enough heartbeat samples`);
  return {
    duration_ms: Date.now() - startedAt,
    heartbeat_interval_ms: 1_000,
    samples,
    expected_process_state: expectedProcessState,
    expected_pid: expectedPid ?? null,
    observed_pids: [...session.observedNodePids],
    lifecycle_events_unchanged: true,
    automatic_action_observed: false,
  };
};

const runFrozenArtifactFault = async (session, name, inject, observe) => {
  await waitHealthy(session);
  const pid = nodePid(session);
  const paths = nodePaths(session);
  const originals = {
    status: fs.readFileSync(paths.status, "utf8"),
    metrics: fs.readFileSync(paths.metrics, "utf8"),
    identity: fs.readFileSync(identityPath(session), "utf8"),
  };
  process.kill(pid, "SIGSTOP");
  let resumed = false;
  try {
    await sleep(300);
    await inject({ session, paths, originals });
    await observe({ session, paths, originals });
  } finally {
    fs.writeFileSync(paths.status, originals.status);
    fs.writeFileSync(paths.metrics, originals.metrics);
    fs.writeFileSync(identityPath(session), originals.identity);
    if (processIsAlive(pid)) {
      process.kill(pid, "SIGCONT");
      resumed = true;
    }
  }
  assert(resumed, `${name} node exited while artifact fault was active`);
  await waitHealthy(session);
  return { fault: name, fail_closed: true, restored: true };
};

const artifactMatrix = async () => {
  const session = await startSession("artifact-faults");
  const results = [];

  results.push(await runFrozenArtifactFault(
    session,
    "status_missing",
    async ({ paths }) => fs.rmSync(paths.status),
    async ({ session: current }) => {
      const response = await waitShared(current, "missing status fail-closed", (candidate) => (
        candidate.status === 200
        && candidate.body.status?.runtime?.availability === "missing"
        && candidate.body.status?.technical_health?.status === "degraded"
      ));
      assertNotHealthy(response.body, "status_missing", ["unknown"]);
    },
  ));
  results.push(await runFrozenArtifactFault(
    session,
    "status_invalid",
    async ({ paths }) => fs.writeFileSync(paths.status, "not-json\n"),
    async ({ session: current }) => {
      const response = await waitShared(current, "invalid status fail-closed", (candidate) => (
        candidate.status === 200
        && candidate.body.status?.runtime?.availability === "error"
        && candidate.body.status?.technical_health?.status === "unhealthy"
      ));
      assertNotHealthy(response.body, "status_invalid", ["unknown"]);
    },
  ));
  results.push(await runFrozenArtifactFault(
    session,
    "metrics_missing",
    async ({ paths }) => fs.rmSync(paths.metrics),
    async ({ session: current }) => {
      const response = await waitShared(current, "missing metrics fail-closed", (candidate) => (
        candidate.status === 200
        && candidate.body.status?.technical_health?.availability === "missing"
        && candidate.body.status?.technical_health?.status === "degraded"
      ));
      assertNotHealthy(response.body, "metrics_missing", ["running"]);
    },
  ));
  results.push(await runFrozenArtifactFault(
    session,
    "metrics_invalid",
    async ({ paths }) => fs.writeFileSync(paths.metrics, "not-json\n"),
    async ({ session: current }) => {
      const response = await waitShared(current, "invalid metrics fail-closed", (candidate) => (
        candidate.status === 200
        && candidate.body.status?.technical_health?.availability === "error"
        && candidate.body.status?.technical_health?.status === "unhealthy"
      ));
      assertNotHealthy(response.body, "metrics_invalid", ["running"]);
    },
  ));
  results.push(await runFrozenArtifactFault(
    session,
    "generation_stale",
    async ({ paths }) => {
      const status = readJson(paths.status);
      const metrics = readJson(paths.metrics);
      status.generated_at = { availability: "available", value: "1" };
      metrics.generated_at = { availability: "available", value: "1" };
      writeJson(paths.status, status);
      writeJson(paths.metrics, metrics);
    },
    async ({ session: current }) => {
      const response = await waitShared(current, "stale generation fail-closed", (candidate) => (
        candidate.status === 200
        && candidate.body.status?.technical_health?.freshness === "stale"
        && candidate.body.status?.technical_health?.status === "degraded"
      ));
      assertNotHealthy(response.body, "generation_stale", ["running"]);
      const reasons = response.body.status.technical_health.reasons || [];
      assert(reasons.includes("node_status_timestamp_stale"), "stale status reason missing");
      assert(reasons.includes("node_metrics_timestamp_stale"), "stale metrics reason missing");
    },
  ));
  results.push(await runFrozenArtifactFault(
    session,
    "generation_mismatch",
    async ({ paths }) => {
      const status = readJson(paths.status);
      const metrics = readJson(paths.metrics);
      const statusGeneration = Number(status.generated_at?.value);
      assert(Number.isFinite(statusGeneration) && statusGeneration > 1, "status generation is invalid");
      metrics.generated_at = { availability: "available", value: String(statusGeneration - 1) };
      writeJson(paths.metrics, metrics);
    },
    async ({ session: current }) => {
      const response = await waitShared(current, "generation mismatch fail-closed", (candidate) => (
        candidate.status === 200
        && candidate.body.status?.technical_health?.reasons?.includes("status_metrics_generation_mismatch")
      ));
      assertNotHealthy(response.body, "generation_mismatch", ["running"]);
    },
  ));
  results.push(await runFrozenArtifactFault(
    session,
    "identity_missing",
    async ({ session: current }) => fs.rmSync(identityPath(current)),
    async ({ session: current }) => {
      const response = await waitShared(current, "missing identity fail-closed", (candidate) => candidate.status === 503);
      assertSharedError(response, 503, "mvp_status_source_unavailable", "identity_missing");
      await waitFor("missing identity local unhealthy contract", 5_000, () => {
        const status = readJson(statusContractPath(current));
        return status.provenance?.identity_contract_available === false
          && status.technical_health?.status === "unhealthy"
          && status.trading_readiness?.status === "blocked";
      });
    },
  ));
  results.push(await runFrozenArtifactFault(
    session,
    "identity_invalid",
    async ({ session: current }) => fs.writeFileSync(identityPath(current), "not-json\n"),
    async ({ session: current }) => {
      const response = await waitShared(current, "invalid identity fail-closed", (candidate) => candidate.status === 500);
      assertSharedError(response, 500, "mvp_status_source_invalid", "identity_invalid");
    },
  ));
  results.push(await runFrozenArtifactFault(
    session,
    "identity_mismatch",
    async ({ session: current }) => {
      const identity = readJson(identityPath(current));
      identity.identities.strategy_version = "fault-matrix-mismatch";
      writeJson(identityPath(current), identity);
    },
    async ({ session: current }) => {
      const response = await waitShared(current, "identity mismatch fail-closed", (candidate) => candidate.status === 409);
      assertSharedError(response, 409, "mvp_status_identity_mismatch", "identity_mismatch");
    },
  ));

  await stopSession(session);
  return results;
};

const gracefulExitMatrix = async () => {
  const session = await startSession("external-sigterm");
  const originalPid = nodePid(session);
  process.kill(originalPid, "SIGTERM");
  const stopped = await waitShared(session, "external SIGTERM stopped projection", (response) => (
    response.status === 200
    && response.body.status?.runtime?.status === "stopped"
    && response.body.status?.technical_health?.status === "not_running"
  ));
  assertSharedBoundaries(stopped.body, "external_sigterm.stopped");
  const stoppedRecord = nodeRecord(session);
  assert(stoppedRecord?.process?.state === "stopped", "SIGTERM registry process is not stopped");
  assert(stoppedRecord?.process?.pid?.value === undefined, "SIGTERM registry retained a PID");

  const snapshot = await waitFor("external SIGTERM operational stopped projection", 10_000, async () => {
    const response = await fetchJson(
      `${session.baseUrl}/api/mvp/v1/control-center`,
      session.operatorCookie,
    );
    return response.status === 200 && response.body.node?.lifecycle_state === "stopped"
      ? response
      : undefined;
  });
  assertFalseFields(snapshot.body.boundaries, "external_sigterm.control_center", operationalFalseFields);

  const noAutomaticRecovery = await observeNoAutomaticRecovery(
    session,
    "external SIGTERM",
    "stopped",
    undefined,
    (response) => {
      assert(response.status === 200, `external SIGTERM observation returned ${response.status}`);
      assert(response.body.status?.runtime?.status === "stopped", "external SIGTERM runtime changed");
      assert(
        response.body.status?.technical_health?.status === "not_running",
        "external SIGTERM technical health changed",
      );
      assertSharedBoundaries(response.body, "external_sigterm.observation");
    },
  );

  const start = await fetchJson(actionUrl(session, "start"), session.operatorCookie, { method: "POST" });
  assert(start.status === 200, `SIGTERM recovery start expected 200, got ${start.status}`);
  assert(start.body.result?.status === "succeeded", "SIGTERM recovery start did not succeed");
  assert(start.body.result?.previous_state === "stopped", "SIGTERM recovery previous state mismatch");
  assert(start.body.result?.current_state === "running", "SIGTERM recovery current state mismatch");
  assertFalseFields(start.body.boundaries, "external_sigterm.start", actionFalseFields);
  await waitHealthy(session);
  const recoveredPid = nodePid(session);
  assert(recoveredPid !== originalPid, "SIGTERM recovery reused the exited PID");
  await stopSession(session);
  return {
    signal: "SIGTERM",
    detected_state: "stopped",
    automatic_restart: false,
    no_automatic_recovery_observation: noAutomaticRecovery,
    recovery: "operator_start",
    recovered_state: "running",
  };
};

const hardKillMatrix = async () => {
  const session = await startSession("hard-kill");
  const originalPid = nodePid(session);
  process.kill(originalPid, "SIGKILL");
  await waitFor("hard-kill stale registry", 10_000, () => {
    const record = nodeRecord(session);
    return record?.process?.state === "stale" ? record : undefined;
  });
  const stale = await waitShared(session, "hard-kill fail-closed projection", (response) => (
    response.status === 200
    && response.body.status?.runtime?.status === "unknown"
    && ["degraded", "unhealthy"].includes(response.body.status?.technical_health?.status)
  ));
  assertNotHealthy(stale.body, "hard_kill", ["unknown"]);
  assert(stale.body.status.runtime.reasons?.includes("supervisor_process_state_stale"), "hard-kill stale reason missing");

  const noAutomaticRecovery = await observeNoAutomaticRecovery(
    session,
    "hard-kill",
    "stale",
    originalPid,
    (response) => {
      assert(response.status === 200, `hard-kill observation returned ${response.status}`);
      assertNotHealthy(response.body, "hard_kill.observation", ["unknown"]);
      assert(
        response.body.status.runtime.reasons?.includes("supervisor_process_state_stale"),
        "hard-kill observation lost stale reason",
      );
    },
  );

  const unauthenticated = await fetchJson(actionUrl(session, "stop"), undefined, { method: "POST" });
  assert(unauthenticated.status === 403, `hard-kill unauthenticated action expected 403, got ${unauthenticated.status}`);
  const institution = await fetchJson(actionUrl(session, "stop"), session.institutionCookie, { method: "POST" });
  assert(institution.status === 403, `hard-kill institution action expected 403, got ${institution.status}`);
  const lifecycleGet = await fetch(actionUrl(session, "stop"), {
    headers: { cookie: session.operatorCookie },
    redirect: "manual",
    signal: AbortSignal.timeout(5_000),
  });
  assert(lifecycleGet.status === 405, `hard-kill lifecycle GET expected 405, got ${lifecycleGet.status}`);
  const failedStop = await fetchJson(actionUrl(session, "stop"), session.operatorCookie, { method: "POST" });
  assert(failedStop.status === 500, `hard-kill stop expected 500, got ${failedStop.status}`);
  assert(failedStop.body.result?.status === "failed", "hard-kill stop was not reported as failed");
  assertFalseFields(failedStop.body.boundaries, "hard_kill.failed_stop", actionFalseFields);

  const start = spawnSync(
    nautilusBin,
    [
      "supervisor", "start",
      "--registry", registryPath(session),
      "--node-id", "mvp-node-001",
      "--ntpro-node-bin", nodeBin,
      "--startup-timeout-ms", "10000",
      "--node-max-runtime-ms", "180000",
      "--node-heartbeat-interval-ms", "1000",
      "--node-parent-pid", String(session.server.pid),
      "--node-shutdown-timeout-ms", "5000",
    ],
    { encoding: "utf8", timeout: 30_000, env: { ...process.env, NO_COLOR: "1" } },
  );
  assert(!start.error, `manual supervisor start failed to launch: ${start.error?.message}`);
  assert(start.status === 0, `manual supervisor start exited ${start.status}: ${redact(start.stderr || start.stdout)}`);
  assert(start.stdout.includes("supervisor.start status=ok"), "manual supervisor start omitted success evidence");
  assert(start.stdout.includes("external_venue_connection=false"), "manual start opened external venue");
  assert(start.stdout.includes("real_orders_submitted=false"), "manual start submitted real orders");
  await waitHealthy(session);
  const recoveredPid = nodePid(session);
  assert(recoveredPid !== originalPid, "hard-kill recovery reused the exited PID");
  await stopSession(session);
  return {
    signal: "SIGKILL",
    detected_process_state: "stale",
    detected_runtime_state: "unknown",
    automatic_restart: false,
    no_automatic_recovery_observation: noAutomaticRecovery,
    failed_portal_stop_status: 500,
    recovery: "explicit_supervisor_start",
    recovered_state: "running",
  };
};

let failure;
let passResult;

try {
  assert(process.platform !== "win32", "MVP fault matrix requires Unix process signals");
  requireFile(nautilusBin, "nautilus binary");
  requireFile(nodeBin, "ntpro-node binary");
  requireFile(nodeConfig, "MVP node config");
  const redactionProbe = redact(
    "http://127.0.0.1/?access_token=fault-probe ntpro_mvp_operator_access=fault-cookie;",
  );
  assert(!redactionProbe.includes("fault-probe"), "access token redaction self-test failed");
  assert(!redactionProbe.includes("fault-cookie"), "cookie redaction self-test failed");

  const artifacts = await artifactMatrix();
  const gracefulExit = await gracefulExitMatrix();
  const hardKill = await hardKillMatrix();
  const automaticActionObserved = [gracefulExit, hardKill].some(
    (entry) => entry.no_automatic_recovery_observation.automatic_action_observed,
  );
  assert(!automaticActionObserved, "process matrix observed an automatic recovery action");
  passResult = {
    schema_version: "ntpro.mvp_fault_matrix_evidence.v1",
    status: "pass",
    matrix: {
      artifact_faults: artifacts,
      external_graceful_exit: gracefulExit,
      hard_kill: hardKill,
    },
    summary: {
      fault_cases: artifacts.length + 2,
      artifact_fault_cases: artifacts.length,
      process_fault_cases: 2,
      fail_closed: true,
      manual_recovery_only: !automaticActionObserved,
      automatic_retry_allowed: automaticActionObserved,
      automatic_remediation_allowed: automaticActionObserved,
      automatic_recovery_allowed: automaticActionObserved,
    },
    boundaries: {
      single_supervisor: true,
      single_node: true,
      sandbox_only: true,
      external_venue_connection: false,
      external_network_attempted: false,
      order_submission_allowed: false,
      order_mutation_allowed: false,
      real_orders_submitted: false,
    },
  };
} catch (error) {
  failure = error instanceof Error ? error : new Error(String(error));
} finally {
  for (const session of sessions) {
    try {
      const record = nodeRecord(session);
      for (const [label, filePath] of [
        ["node_stdout", record?.stdout_log_path],
        ["node_stderr", record?.stderr_log_path],
      ]) {
        if (filePath && fs.existsSync(filePath)) {
          session.serverLog += `\n${label}:\n${fs.readFileSync(filePath, "utf8")}`;
        }
      }
    } catch {
      session.serverLog += "\nnode_diagnostics=unavailable\n";
    }
  }
  for (const session of sessions) {
    try {
      await cleanupSession(session);
    } catch (error) {
      failure ||= error instanceof Error ? error : new Error(String(error));
    }
  }
  const cleanupComplete = sessions.every((session) => session.cleanupVerified === true);
  if (!cleanupComplete) {
    failure ||= new Error("one or more fault-matrix sessions did not prove node cleanup");
  }
  const logs = sessions
    .map((session) => `== ${session.name} ==\n${session.serverLog}`)
    .join("\n");
  const sanitizedLog = redact(logs);
  assert(!/access_token=(?!\[REDACTED\])/.test(sanitizedLog), "fault log contains an access token");
  assert(
    !/ntpro_mvp_(institution|operator)_access=(?!\[REDACTED\])/.test(sanitizedLog),
    "fault log contains a role cookie",
  );
  fs.writeFileSync(logPath, sanitizedLog);
  const result = failure
    ? { schema_version: "ntpro.mvp_fault_matrix_evidence.v1", status: "fail", error: redact(failure.message) }
    : passResult;
  const serialized = `${JSON.stringify(result, null, 2)}\n`;
  assert(!/access_token=(?!\[REDACTED\])/.test(serialized), "result.json contains an access token");
  assert(
    !/ntpro_mvp_(institution|operator)_access=(?!\[REDACTED\])/.test(serialized),
    "result.json contains a role cookie",
  );
  fs.writeFileSync(resultPath, serialized);
  if (process.env.NTPRO_MVP_FAULT_KEEP_ROOT !== "1" && cleanupComplete) {
    try {
      fs.rmSync(root, { recursive: true, force: true, maxRetries: 10, retryDelay: 100 });
    } catch (error) {
      if (!failure) throw error;
    }
  } else {
    console.error(`mvp_fault_matrix_workspace=${root}`);
  }
}

if (failure) {
  console.error(`mvp_fault_matrix=fail error=${redact(failure.message)}`);
  process.exitCode = 1;
} else {
  console.log(
    `mvp_fault_matrix=pass cases=${passResult.summary.fault_cases} artifact_faults=${passResult.summary.artifact_fault_cases} process_faults=${passResult.summary.process_fault_cases} fail_closed=pass manual_recovery_only=pass`,
  );
}
