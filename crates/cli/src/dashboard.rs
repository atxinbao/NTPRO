// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! Dashboard read-model DTOs and local HTTP server for the local v0.3 MVP.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path as FsPath, PathBuf},
    time::Duration,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, ensure};
use axum::{
    Json, Router,
    extract::{Path as AxumPath, State},
    http::{StatusCode, header::CONTENT_TYPE},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use nautilus_live::status::{
    ConnectionStatus, HealthStatus, LifecycleStatus, NodeStatus, ProcessMode, RiskTradingState,
    SnapshotAvailability, SnapshotValue,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    opt::{DashboardCommand, DashboardOpt, DashboardServeOpt},
    supervisor::{
        NodeMetrics, RegistryArtifactState, StartNodeRequest, StopNodeRequest,
        SupervisorNodeRecord, SupervisorProcessState, SupervisorRegistry, SupervisorRegistryStore,
    },
};

pub const DASHBOARD_SNAPSHOT_SCHEMA_VERSION: &str = "ntpro.dashboard_snapshot.v1";

const DASHBOARD_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>NTPRO Dashboard</title>
  <link rel="stylesheet" href="/assets/dashboard.css">
</head>
<body>
  <header class="topbar">
    <div>
      <h1>NTPRO Dashboard</h1>
      <p>Local supervisor artifact view. No external venue connection is opened by this page.</p>
    </div>
    <button id="refresh" type="button">Refresh</button>
  </header>
  <main>
    <section class="band">
      <h2>Overview</h2>
      <div id="overview" class="grid"></div>
    </section>
    <section class="band">
      <h2>Nodes</h2>
      <div id="nodes" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>Controls</h2>
      <div id="controls" class="table-wrap"></div>
      <div id="control-result" class="list"></div>
    </section>
    <section class="band">
      <h2>Data Sources</h2>
      <div id="data-sources" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>Execution Gateways</h2>
      <div id="execution-gateways" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>Risk Engine</h2>
      <div id="risk" class="grid"></div>
    </section>
    <section class="band">
      <h2>Runtime Modules</h2>
      <div id="runtime-modules" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>Logs / Metrics</h2>
      <div id="logs-metrics" class="table-wrap"></div>
    </section>
    <section class="band">
      <h2>Alerts</h2>
      <div id="alerts" class="list"></div>
    </section>
    <section class="band">
      <h2>Gaps</h2>
      <div id="gaps" class="list"></div>
    </section>
  </main>
  <script src="/assets/dashboard.js"></script>
</body>
</html>
"#;

const DASHBOARD_CSS: &str = r#":root {
  color-scheme: light;
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: #f6f7f9;
  color: #1f2933;
}

body {
  margin: 0;
}

*,
*::before,
*::after {
  box-sizing: border-box;
}

.topbar {
  display: flex;
  justify-content: space-between;
  gap: 24px;
  align-items: center;
  padding: 24px 32px;
  background: #111827;
  color: #ffffff;
}

.topbar h1 {
  margin: 0 0 6px;
  font-size: 28px;
  letter-spacing: 0;
}

.topbar p {
  margin: 0;
  color: #cbd5e1;
}

button {
  border: 1px solid #94a3b8;
  background: #ffffff;
  color: #111827;
  border-radius: 6px;
  padding: 8px 12px;
  font-weight: 600;
  cursor: pointer;
}

main {
  width: min(1180px, calc(100vw - 32px));
  margin: 24px auto 48px;
}

.band {
  margin: 0 0 28px;
}

.band h2 {
  font-size: 18px;
  margin: 0 0 12px;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
  gap: 12px;
}

.tile,
.row {
  background: #ffffff;
  border: 1px solid #d8dee6;
  border-radius: 6px;
  padding: 12px;
}

.label {
  color: #64748b;
  font-size: 12px;
  text-transform: uppercase;
}

.value {
  margin-top: 6px;
  font-size: 18px;
  font-weight: 700;
  overflow-wrap: anywhere;
}

.table-wrap {
  overflow-x: auto;
  border: 1px solid #d8dee6;
  border-radius: 6px;
  background: #ffffff;
}

table {
  width: 100%;
  border-collapse: collapse;
  min-width: 980px;
}

th,
td {
  padding: 10px 12px;
  border-bottom: 1px solid #e5eaf0;
  text-align: left;
  vertical-align: top;
}

th {
  background: #eef2f6;
  color: #334155;
  font-size: 12px;
  text-transform: uppercase;
}

td {
  font-size: 13px;
}

.path {
  max-width: 260px;
  overflow-wrap: anywhere;
}

.muted {
  color: #64748b;
}

.list {
  display: grid;
  gap: 8px;
}

.row {
  display: grid;
  gap: 4px;
}

.status-healthy {
  color: #166534;
}

.status-error,
.status-stale {
  color: #991b1b;
}

.status-degraded,
.status-unknown {
  color: #92400e;
}

@media (max-width: 720px) {
  main {
    width: min(100%, calc(100vw - 24px));
    margin-top: 18px;
  }

  .topbar {
    align-items: flex-start;
    flex-direction: column;
    padding: 20px 16px;
  }

  .topbar h1 {
    font-size: 24px;
  }

  .topbar button,
  td button {
    max-width: 100%;
    white-space: normal;
  }

  .table-wrap {
    overflow-x: visible;
  }

  table,
  tbody,
  tr,
  td {
    display: block;
    min-width: 0;
    width: 100%;
  }

  table {
    min-width: 0;
  }

  thead {
    display: none;
  }

  tr {
    border-bottom: 1px solid #e5eaf0;
    padding: 8px 0;
  }

  tr:last-child {
    border-bottom: 0;
  }

  td {
    display: grid;
    grid-template-columns: 116px minmax(0, 1fr);
    gap: 10px;
    border-bottom: 0;
    padding: 6px 12px;
  }

  td::before {
    content: attr(data-label);
    color: #475569;
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
  }

  .path {
    max-width: none;
  }
}
"#;

const DASHBOARD_JS: &str = r#"const renderTile = (label, value, extraClass = "") =>
  `<div class="tile ${extraClass}"><div class="label">${text(label)}</div><div class="value">${text(value)}</div></div>`;

const safe = (value) => value === null || value === undefined ? "unknown" : String(value);

const text = (value) => safe(value)
  .replaceAll("&", "&amp;")
  .replaceAll("<", "&lt;")
  .replaceAll(">", "&gt;")
  .replaceAll("\"", "&quot;")
  .replaceAll("'", "&#39;");

const snapshotValue = (value) => {
  if (!value || typeof value !== "object") return "unknown";
  return value.value ?? value.availability ?? "unknown";
};

const availability = (value) => value && typeof value === "object" ? value.availability : "unknown";

const redactedError = (value) => {
  const present = value && typeof value === "string" && value.trim().length > 0;
  return present ? "present (redacted)" : "none";
};

const redactedDashboardValue = (value) => {
  if (!value || typeof value !== "object") return "unknown";
  if (value.availability === "redacted") return "redacted";
  if (value.value !== null && value.value !== undefined) return "present (redacted)";
  return value.availability ?? "unknown";
};

const dashboardErrorValue = (value) => {
  if (!value || typeof value !== "object") return "unknown";
  if (value.value !== null && value.value !== undefined) return "present (redacted)";
  return value.availability ?? "unknown";
};

const emptyTable = (message) => `<div class="tile"><div class="value">${text(message)}</div></div>`;

async function loadSnapshot() {
  const [metaResponse, snapshotResponse] = await Promise.all([
    fetch("/api/server"),
    fetch("/api/snapshot"),
  ]);
  if (!metaResponse.ok) {
    throw new Error(`server metadata request failed: ${metaResponse.status}`);
  }
  if (!snapshotResponse.ok) {
    throw new Error(`snapshot request failed: ${snapshotResponse.status}`);
  }
  return {
    metadata: await metaResponse.json(),
    snapshot: await snapshotResponse.json(),
  };
}

function render(payload) {
  const metadata = payload.metadata || {};
  const snapshot = payload.snapshot || {};
  const overview = snapshot.overview || {};
  const nodes = snapshot.nodes || [];
  const staleNodes = nodes.filter((node) => node.health === "stale").length;
  document.getElementById("overview").innerHTML = [
    renderTile("Registry", safe(metadata.registry_path)),
    renderTile("Nodes", safe(overview.node_count)),
    renderTile("Running", safe(overview.running_nodes), "status-healthy"),
    renderTile("Stopped", safe(overview.stopped_nodes)),
    renderTile("Errors", safe(overview.error_nodes), "status-error"),
    renderTile("Stale", safe(staleNodes), "status-stale"),
    renderTile("Unknown", safe(overview.unknown_nodes), "status-unknown"),
    renderTile("Health", safe(overview.health), `status-${safe(overview.health)}`),
    renderTile("Sandbox Only", safe(overview.sandbox_only)),
    renderTile("External Venue", safe(overview.external_venue_connection)),
    renderTile("Real Orders", safe(overview.real_orders_submitted)),
    renderTile("Latest Transition", snapshotValue(overview.latest_transition_at)),
    renderTile("Latest Error", redactedError(overview.latest_error), overview.latest_error ? "status-error" : ""),
    renderTile("Generated", snapshotValue(snapshot.generated_at)),
  ].join("");

  document.getElementById("nodes").innerHTML = nodes.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>Node</th>
          <th>Lifecycle</th>
          <th>Process</th>
          <th>PID</th>
          <th>Config</th>
          <th>Artifacts</th>
          <th>Started</th>
          <th>Stopped</th>
          <th>Last Transition</th>
          <th>Last Error</th>
        </tr>
      </thead>
      <tbody>
        ${nodes.map((node) => `
          <tr>
            <td data-label="Node"><strong>${text(node.node_id)}</strong><div class="muted">${text(node.process_mode)}</div></td>
            <td data-label="Lifecycle"><span class="status-${safe(node.health)}">${text(node.lifecycle_state)}</span></td>
            <td data-label="Process">${text(node.process_state)}</td>
            <td data-label="PID">${text(snapshotValue(node.pid))}<div class="muted">${text(availability(node.pid))}</div></td>
            <td data-label="Config" class="path">${text(snapshotValue(node.config_path))}</td>
            <td data-label="Artifacts" class="path">${text(snapshotValue(node.artifact_root))}</td>
            <td data-label="Started">${text(snapshotValue(node.started_at))}</td>
            <td data-label="Stopped">${text(snapshotValue(node.stopped_at))}</td>
            <td data-label="Last Transition">${text(snapshotValue(node.last_transition_at))}</td>
            <td data-label="Last Error">${text(redactedError(node.last_error))}</td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : `<div class="tile"><div class="value">No registered nodes</div></div>`;

  renderDataSources(snapshot.data_sources || []);
  renderExecutionGateways(snapshot.execution_gateways || []);
  renderRisk(snapshot.risk || {});
  renderRuntimeModules(snapshot.runtime_modules || []);
  renderLogsMetrics(snapshot.logs || [], snapshot.metrics || []);
  renderControls(snapshot.controls || []);

  document.getElementById("alerts").innerHTML = ((snapshot.alerts || {}).active || []).map((alert) =>
    `<div class="row"><strong>${text(alert.severity)}: ${text(alert.source)}</strong><span>${text(alert.message)}</span></div>`
  ).join("") || `<div class="row">No active alerts</div>`;

  document.getElementById("gaps").innerHTML = (snapshot.gaps || []).map((gap) =>
    `<div class="row"><strong>${text(gap.field_path)}</strong><span>${text(gap.reason)} - ${text(snapshotValue(gap.notes))}</span></div>`
  ).join("") || `<div class="row">No dashboard gaps</div>`;
}

const controlLabel = (action) => {
  const name = safe(action).split(":")[0];
  return {
    start: "Start",
    stop: "Stop",
    pause: "Pause",
    resume: "Resume",
    reconnect_data: "Reconnect data",
    reconnect_execution: "Reconnect execution",
  }[name] || name;
};

const controlNodeId = (action) => safe(action).split(":").slice(1).join(":");
const controlActionName = (action) => safe(action).split(":")[0];

function renderControls(controls) {
  document.getElementById("controls").innerHTML = controls.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>Node</th>
          <th>Action</th>
          <th>Availability</th>
          <th>Enabled</th>
          <th>Reason</th>
          <th>Run</th>
        </tr>
      </thead>
      <tbody>
        ${controls.map((control) => {
          const action = controlActionName(control.action);
          const nodeId = controlNodeId(control.action);
          const runnable = control.enabled && (action === "start" || action === "stop");
          return `
            <tr>
              <td data-label="Node"><strong>${text(nodeId)}</strong></td>
              <td data-label="Action">${text(controlLabel(control.action))}</td>
              <td data-label="Availability">${text(control.availability)}</td>
              <td data-label="Enabled">${text(control.enabled)}</td>
              <td data-label="Reason">${text(snapshotValue(control.reason))}</td>
              <td data-label="Run"><button type="button" data-dashboard-action="${text(action)}" data-node-id="${text(nodeId)}" ${runnable ? "" : "disabled"}>${text(controlLabel(control.action))}</button></td>
            </tr>`;
        }).join("")}
      </tbody>
    </table>` : emptyTable("No controls reported");
}

function renderDataSources(dataSources) {
  document.getElementById("data-sources").innerHTML = dataSources.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>Source</th>
          <th>Kind</th>
          <th>Provider</th>
          <th>Connection</th>
          <th>Freshness</th>
          <th>Lag</th>
          <th>Health</th>
          <th>Last Error</th>
        </tr>
      </thead>
      <tbody>
        ${dataSources.map((source) => `
          <tr>
            <td data-label="Source"><strong>${text(source.source_id)}</strong></td>
            <td data-label="Kind">${text(snapshotValue(source.source_kind))}</td>
            <td data-label="Provider">${text(snapshotValue(source.provider))}</td>
            <td data-label="Connection">${text(source.connection)}</td>
            <td data-label="Freshness">${text(snapshotValue(source.freshness))}</td>
            <td data-label="Lag">${text(snapshotValue(source.lag_ms))}</td>
            <td data-label="Health"><span class="status-${safe(source.health)}">${text(source.health)}</span></td>
            <td data-label="Last Error">${text(dashboardErrorValue(source.last_error))}</td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("No data sources reported");
}

function renderExecutionGateways(gateways) {
  document.getElementById("execution-gateways").innerHTML = gateways.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>Gateway</th>
          <th>Venue</th>
          <th>Connection</th>
          <th>Started</th>
          <th>Account</th>
          <th>Orders</th>
          <th>Last Report</th>
          <th>Last Error</th>
        </tr>
      </thead>
      <tbody>
        ${gateways.map((gateway) => `
          <tr>
            <td data-label="Gateway"><strong>${text(gateway.gateway_id)}</strong></td>
            <td data-label="Venue">${text(snapshotValue(gateway.venue))}</td>
            <td data-label="Connection">${text(gateway.connection)}</td>
            <td data-label="Started">${text(snapshotValue(gateway.started))}</td>
            <td data-label="Account">${text(redactedDashboardValue(gateway.account_ref))}</td>
            <td data-label="Orders">open ${text(snapshotValue(gateway.order_counts?.open))} / in-flight ${text(snapshotValue(gateway.order_counts?.inflight))} / closed ${text(snapshotValue(gateway.order_counts?.closed))}</td>
            <td data-label="Last Report">${text(snapshotValue(gateway.last_report_at))}</td>
            <td data-label="Last Error">${text(dashboardErrorValue(gateway.last_error))}</td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("No execution gateways reported");
}

function renderRisk(risk) {
  const lastRejection = risk.last_rejection && risk.last_rejection.value ? "present (redacted)" : snapshotValue(risk.last_rejection);
  document.getElementById("risk").innerHTML = [
    renderTile("Trading State", safe(risk.trading_state)),
    renderTile("Health", safe(risk.health), `status-${safe(risk.health)}`),
    renderTile("Commands", snapshotValue(risk.command_count)),
    renderTile("Events", snapshotValue(risk.event_count)),
    renderTile("Rejections", snapshotValue(risk.rejections_total)),
    renderTile("Last Rejection", lastRejection),
    renderTile("Last Error", dashboardErrorValue(risk.last_error)),
  ].join("");
}

function renderRuntimeModules(modules) {
  document.getElementById("runtime-modules").innerHTML = modules.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>Module</th>
          <th>Status</th>
          <th>Health</th>
          <th>Last Seen</th>
          <th>Last Error</th>
          <th>Evidence</th>
        </tr>
      </thead>
      <tbody>
        ${modules.map((module) => `
          <tr>
            <td data-label="Module"><strong>${text(module.module_name)}</strong></td>
            <td data-label="Status">${text(snapshotValue(module.status))}<div class="muted">${text(availability(module.status))}</div></td>
            <td data-label="Health"><span class="status-${safe(module.health)}">${text(module.health)}</span></td>
            <td data-label="Last Seen">${text(snapshotValue(module.last_seen_at))}</td>
            <td data-label="Last Error">${text(dashboardErrorValue(module.last_error))}</td>
            <td data-label="Evidence" class="path">${text(snapshotValue(module.evidence_source))}<div class="muted">${text(availability(module.evidence_source))}</div></td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("No runtime modules reported");
}

function renderLogsMetrics(logs, metrics) {
  const rows = [
    ...logs.map((log) => ({
      kind: "log",
      id: log.log_id,
      node: snapshotValue(log.node_id),
      path: snapshotValue(log.path),
      availability: log.availability,
      value: snapshotValue(log.last_seen_at),
      lastError: dashboardErrorValue(log.last_error),
    })),
    ...metrics.map((metric) => ({
      kind: "metric",
      id: metric.metric_id,
      node: snapshotValue(metric.node_id),
      path: "metric artifact",
      availability: metric.availability,
      value: snapshotValue(metric.value),
      lastError: dashboardErrorValue(metric.last_error),
    })),
  ];
  document.getElementById("logs-metrics").innerHTML = rows.length > 0 ? `
    <table>
      <thead>
        <tr>
          <th>Kind</th>
          <th>ID</th>
          <th>Node</th>
          <th>Evidence</th>
          <th>Availability</th>
          <th>Value / Seen</th>
          <th>Last Error</th>
        </tr>
      </thead>
      <tbody>
        ${rows.map((row) => `
          <tr>
            <td data-label="Kind">${text(row.kind)}</td>
            <td data-label="ID"><strong>${text(row.id)}</strong></td>
            <td data-label="Node">${text(row.node)}</td>
            <td data-label="Evidence" class="path">${text(row.path)}</td>
            <td data-label="Availability">${text(row.availability)}</td>
            <td data-label="Value / Seen">${text(row.value)}</td>
            <td data-label="Last Error">${text(row.lastError)}</td>
          </tr>
        `).join("")}
      </tbody>
    </table>` : emptyTable("No logs or metrics reported");
}

async function refresh() {
  const snapshot = await loadSnapshot();
  render(snapshot);
}

document.getElementById("refresh").addEventListener("click", () => refresh().catch(console.error));
document.addEventListener("click", async (event) => {
  const button = event.target.closest("[data-dashboard-action]");
  if (!button || button.disabled) return;
  const action = button.getAttribute("data-dashboard-action");
  const nodeId = button.getAttribute("data-node-id");
  button.disabled = true;
  document.getElementById("control-result").innerHTML = `<div class="row">Running ${text(action)} for ${text(nodeId)}</div>`;
  try {
    const response = await fetch(`/api/nodes/${encodeURIComponent(nodeId)}/actions/${encodeURIComponent(action)}`, { method: "POST" });
    const payload = await response.json();
    document.getElementById("control-result").innerHTML = `<div class="row"><strong>${text(snapshotValue(payload.message))}</strong><span>${text(payload.status)} ${text(snapshotValue(payload.error_code))}</span></div>`;
    await refresh();
  } catch (error) {
    document.getElementById("control-result").innerHTML = `<div class="row"><strong>Control failed</strong><span>${text(error.message)}</span></div>`;
  }
});
refresh().catch((error) => {
  document.getElementById("overview").innerHTML = renderTile("Error", error.message, "status-error");
});
"#;

#[derive(Clone, Debug)]
struct DashboardServerState {
    registry_path: PathBuf,
    ntpro_node_bin: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct DashboardServerMetadata {
    registry_path: String,
    local_only: bool,
}

const DASHBOARD_ACTION_TIMEOUT_MS: u64 = 5_000;

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<Value>)>;
type ApiStatusResult<T> = Result<(StatusCode, Json<T>), (StatusCode, Json<Value>)>;
type SnapshotLoadResult = Result<DashboardSnapshot, (StatusCode, Json<Value>)>;

/// Runs local dashboard commands.
///
/// # Errors
///
/// Returns an error if the server cannot bind the requested loopback address or
/// if the HTTP server exits with an error.
pub(crate) async fn run_dashboard_command(opt: DashboardOpt) -> anyhow::Result<()> {
    match opt.command {
        DashboardCommand::Serve(serve) => serve_dashboard(serve).await?,
    }
    Ok(())
}

async fn serve_dashboard(opt: DashboardServeOpt) -> anyhow::Result<()> {
    ensure!(
        opt.bind.ip().is_loopback(),
        "dashboard server is local-only for v0.3; use a loopback bind address"
    );

    let registry_path = opt.registry;
    let ntpro_node_bin = opt
        .ntpro_node_bin
        .unwrap_or_else(default_ntpro_node_bin_path);
    let listener = tokio::net::TcpListener::bind(opt.bind)
        .await
        .with_context(|| format!("failed to bind dashboard server at {}", opt.bind))?;
    let local_addr = listener
        .local_addr()
        .context("failed to read dashboard server local address")?;
    println!(
        "dashboard.serve status=ok bind={} registry={} dashboard_url=http://{}/dashboard",
        local_addr,
        registry_path.display(),
        local_addr
    );
    axum::serve(listener, dashboard_router(registry_path, ntpro_node_bin))
        .await
        .context("dashboard HTTP server exited with an error")?;
    Ok(())
}

fn dashboard_router(registry_path: PathBuf, ntpro_node_bin: PathBuf) -> Router {
    let state = DashboardServerState {
        registry_path,
        ntpro_node_bin,
    };
    Router::new()
        .route("/", get(dashboard_shell))
        .route("/dashboard", get(dashboard_shell))
        .route("/assets/dashboard.css", get(dashboard_css))
        .route("/assets/dashboard.js", get(dashboard_js))
        .route("/api/server", get(server_metadata_api))
        .route("/api/snapshot", get(snapshot_api))
        .route("/api/nodes", get(nodes_api))
        .route("/api/nodes/{node_id}", get(node_detail_api))
        .route("/api/nodes/{node_id}/metrics", get(node_metrics_api))
        .route("/api/nodes/{node_id}/logs", get(node_logs_api))
        .route("/api/nodes/{node_id}/actions/start", post(start_action_api))
        .route("/api/nodes/{node_id}/actions/stop", post(stop_action_api))
        .with_state(state)
}

fn default_ntpro_node_bin_path() -> PathBuf {
    std::env::current_exe().map_or_else(
        |_| PathBuf::from("ntpro-node"),
        |path| {
            let file_name = if cfg!(windows) {
                "ntpro-node.exe"
            } else {
                "ntpro-node"
            };
            path.with_file_name(file_name)
        },
    )
}

async fn dashboard_shell() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

async fn dashboard_css() -> impl IntoResponse {
    ([(CONTENT_TYPE, "text/css; charset=utf-8")], DASHBOARD_CSS)
}

async fn dashboard_js() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "application/javascript; charset=utf-8")],
        DASHBOARD_JS,
    )
}

async fn server_metadata_api(
    State(state): State<DashboardServerState>,
) -> Json<DashboardServerMetadata> {
    Json(DashboardServerMetadata {
        registry_path: state.registry_path.display().to_string(),
        local_only: true,
    })
}

async fn snapshot_api(State(state): State<DashboardServerState>) -> ApiResult<DashboardSnapshot> {
    load_dashboard_snapshot(&state.registry_path).map(Json)
}

async fn nodes_api(
    State(state): State<DashboardServerState>,
) -> ApiResult<Vec<DashboardNodeSummary>> {
    let snapshot = load_dashboard_snapshot(&state.registry_path)?;
    Ok(Json(snapshot.nodes))
}

async fn node_detail_api(
    State(state): State<DashboardServerState>,
    AxumPath(node_id): AxumPath<String>,
) -> ApiResult<DashboardNodeSummary> {
    let snapshot = load_dashboard_snapshot(&state.registry_path)?;
    snapshot
        .nodes
        .into_iter()
        .find(|node| node.node_id == node_id)
        .map(Json)
        .ok_or_else(|| not_found_response("node_not_found", &node_id))
}

async fn node_metrics_api(
    State(state): State<DashboardServerState>,
    AxumPath(node_id): AxumPath<String>,
) -> ApiResult<Vec<MetricStatus>> {
    let snapshot = load_dashboard_snapshot(&state.registry_path)?;
    if !snapshot.nodes.iter().any(|node| node.node_id == node_id) {
        return Err(not_found_response("node_not_found", &node_id));
    }
    let metric_prefix = format!("{node_id}:");
    let metrics = snapshot
        .metrics
        .into_iter()
        .filter(|metric| {
            metric.node_id.value.as_deref() == Some(node_id.as_str())
                || metric.metric_id.starts_with(&metric_prefix)
        })
        .collect();
    Ok(Json(metrics))
}

async fn node_logs_api(
    State(state): State<DashboardServerState>,
    AxumPath(node_id): AxumPath<String>,
) -> ApiResult<Vec<LogStatus>> {
    let snapshot = load_dashboard_snapshot(&state.registry_path)?;
    if !snapshot.nodes.iter().any(|node| node.node_id == node_id) {
        return Err(not_found_response("node_not_found", &node_id));
    }
    let log_prefix = format!("{node_id}:");
    let logs = snapshot
        .logs
        .into_iter()
        .filter(|log| {
            log.node_id.value.as_deref() == Some(node_id.as_str())
                || log.log_id.starts_with(&log_prefix)
        })
        .collect();
    Ok(Json(logs))
}

async fn start_action_api(
    State(state): State<DashboardServerState>,
    AxumPath(node_id): AxumPath<String>,
) -> ApiStatusResult<ControlActionResponse> {
    control_action_response(&state, &node_id, "start")
}

async fn stop_action_api(
    State(state): State<DashboardServerState>,
    AxumPath(node_id): AxumPath<String>,
) -> ApiStatusResult<ControlActionResponse> {
    control_action_response(&state, &node_id, "stop")
}

fn control_action_response(
    state: &DashboardServerState,
    node_id: &str,
    action: &str,
) -> ApiStatusResult<ControlActionResponse> {
    let started_at = generated_at_now();
    let snapshot = load_dashboard_snapshot(&state.registry_path)?;
    let Some(node) = snapshot.nodes.iter().find(|node| node.node_id == node_id) else {
        return Ok((
            StatusCode::NOT_FOUND,
            Json(action_response(ControlActionResponseParts {
                action,
                node_id,
                status: ControlActionStatus::Rejected,
                previous_state: LifecycleStatus::Unknown,
                current_state: LifecycleStatus::Unknown,
                started_at,
                error_code: DashboardValue::available("node_not_found".to_string()),
                message: DashboardValue::available(
                    "node was not found in local supervisor registry".to_string(),
                ),
            })),
        ));
    };
    let previous_state = node.lifecycle_state;

    match action {
        "start" if previous_state != LifecycleStatus::Stopped => Ok((
            StatusCode::CONFLICT,
            Json(action_response(ControlActionResponseParts {
                action,
                node_id,
                status: ControlActionStatus::Rejected,
                previous_state,
                current_state: previous_state,
                started_at,
                error_code: DashboardValue::available("invalid_lifecycle_state".to_string()),
                message: DashboardValue::available(
                    "start is only available for stopped nodes".to_string(),
                ),
            })),
        )),
        "stop" if previous_state != LifecycleStatus::Running => Ok((
            StatusCode::CONFLICT,
            Json(action_response(ControlActionResponseParts {
                action,
                node_id,
                status: ControlActionStatus::Rejected,
                previous_state,
                current_state: previous_state,
                started_at,
                error_code: DashboardValue::available("invalid_lifecycle_state".to_string()),
                message: DashboardValue::available(
                    "stop is only available for running nodes".to_string(),
                ),
            })),
        )),
        "start" => Ok(run_start_action(state, node_id, previous_state, started_at)),
        "stop" => Ok(run_stop_action(state, node_id, previous_state, started_at)),
        _ => Ok((
            StatusCode::NOT_IMPLEMENTED,
            Json(action_response(ControlActionResponseParts {
                action,
                node_id,
                status: ControlActionStatus::Rejected,
                previous_state,
                current_state: previous_state,
                started_at,
                error_code: DashboardValue::available("unsupported_control_action".to_string()),
                message: DashboardValue::available(
                    "control action is not supported in v0.3".to_string(),
                ),
            })),
        )),
    }
}

fn run_start_action(
    state: &DashboardServerState,
    node_id: &str,
    previous_state: LifecycleStatus,
    started_at: String,
) -> (StatusCode, Json<ControlActionResponse>) {
    let store = SupervisorRegistryStore::new(state.registry_path.clone());
    let result = store.start_node_process(&StartNodeRequest {
        node_id: node_id.to_string(),
        ntpro_node_bin: state.ntpro_node_bin.clone(),
        startup_timeout: Duration::from_millis(DASHBOARD_ACTION_TIMEOUT_MS),
        node_max_runtime: Duration::from_millis(3_600_000),
        node_heartbeat_interval: Duration::from_millis(1_000),
        node_parent_pid: Some(std::process::id()),
        node_shutdown_timeout: Duration::from_millis(5_000),
    });
    match result {
        Ok(record) => (
            StatusCode::OK,
            Json(action_response(ControlActionResponseParts {
                action: "start",
                node_id,
                status: ControlActionStatus::Succeeded,
                previous_state,
                current_state: record.last_known_status.lifecycle_state,
                started_at,
                error_code: DashboardValue::unknown(),
                message: DashboardValue::available(
                    "start completed through local supervisor".to_string(),
                ),
            })),
        ),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(action_response(ControlActionResponseParts {
                action: "start",
                node_id,
                status: ControlActionStatus::Failed,
                previous_state,
                current_state: previous_state,
                started_at,
                error_code: DashboardValue::available(control_error_code(&error)),
                message: DashboardValue::available(
                    "start failed; details are redacted".to_string(),
                ),
            })),
        ),
    }
}

fn run_stop_action(
    state: &DashboardServerState,
    node_id: &str,
    previous_state: LifecycleStatus,
    started_at: String,
) -> (StatusCode, Json<ControlActionResponse>) {
    let store = SupervisorRegistryStore::new(state.registry_path.clone());
    let result = store.stop_node_process(&StopNodeRequest {
        node_id: node_id.to_string(),
        stop_timeout: Duration::from_millis(DASHBOARD_ACTION_TIMEOUT_MS),
    });
    match result {
        Ok(record) => (
            StatusCode::OK,
            Json(action_response(ControlActionResponseParts {
                action: "stop",
                node_id,
                status: ControlActionStatus::Succeeded,
                previous_state,
                current_state: record.last_known_status.lifecycle_state,
                started_at,
                error_code: DashboardValue::unknown(),
                message: DashboardValue::available(
                    "stop completed through local supervisor".to_string(),
                ),
            })),
        ),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(action_response(ControlActionResponseParts {
                action: "stop",
                node_id,
                status: ControlActionStatus::Failed,
                previous_state,
                current_state: previous_state,
                started_at,
                error_code: DashboardValue::available(control_error_code(&error)),
                message: DashboardValue::available("stop failed; details are redacted".to_string()),
            })),
        ),
    }
}

#[derive(Debug)]
struct ControlActionResponseParts<'a> {
    action: &'a str,
    node_id: &'a str,
    status: ControlActionStatus,
    previous_state: LifecycleStatus,
    current_state: LifecycleStatus,
    started_at: String,
    error_code: DashboardValue<String>,
    message: DashboardValue<String>,
}

fn action_response(parts: ControlActionResponseParts<'_>) -> ControlActionResponse {
    let finished_at = generated_at_now();
    ControlActionResponse {
        action_id: format!("{}:{}:{}", parts.action, parts.node_id, parts.started_at),
        action: format!("{}:{}", parts.action, parts.node_id),
        status: parts.status,
        previous_state: parts.previous_state,
        current_state: parts.current_state,
        started_at: DashboardValue::available(parts.started_at),
        finished_at: DashboardValue::available(finished_at),
        error_code: parts.error_code,
        message: parts.message,
        observability_ref: DashboardValue::available(format!("registry:{}", parts.node_id)),
    }
}

fn control_error_code(error: &anyhow::Error) -> String {
    let message = error.to_string();
    if message.contains("already running") || message.contains("not running") {
        "invalid_lifecycle_state".to_string()
    } else if message.contains("ntpro-node binary") {
        "ntpro_node_binary_unavailable".to_string()
    } else if message.contains("timed out") {
        "lifecycle_timeout".to_string()
    } else if message.contains("not registered") {
        "node_not_found".to_string()
    } else {
        "supervisor_action_failed".to_string()
    }
}

fn load_dashboard_snapshot(registry_path: &FsPath) -> SnapshotLoadResult {
    snapshot_from_supervisor_artifacts(registry_path, generated_at_now()).map_err(|error| {
        let message = error.to_string();
        server_error_response("snapshot_load_failed", &message)
    })
}

fn not_found_response(error_code: &str, node_id: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error_code": error_code,
            "node_id": node_id,
            "message": "dashboard node was not found in the local supervisor registry"
        })),
    )
}

fn server_error_response(error_code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "error_code": error_code,
            "message": message
        })),
    )
}

fn generated_at_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    format!("unix_seconds:{seconds}")
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardAvailability {
    Available,
    NotConfigured,
    NotSupported,
    Stale,
    Redacted,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardValue<T> {
    pub availability: DashboardAvailability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<T>,
}

impl<T> DashboardValue<T> {
    #[must_use]
    pub const fn available(value: T) -> Self {
        Self {
            availability: DashboardAvailability::Available,
            value: Some(value),
        }
    }

    #[must_use]
    pub const fn not_configured() -> Self {
        Self {
            availability: DashboardAvailability::NotConfigured,
            value: None,
        }
    }

    #[must_use]
    pub const fn not_supported() -> Self {
        Self {
            availability: DashboardAvailability::NotSupported,
            value: None,
        }
    }

    #[must_use]
    pub const fn stale() -> Self {
        Self {
            availability: DashboardAvailability::Stale,
            value: None,
        }
    }

    #[must_use]
    pub const fn redacted() -> Self {
        Self {
            availability: DashboardAvailability::Redacted,
            value: None,
        }
    }

    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            availability: DashboardAvailability::Unknown,
            value: None,
        }
    }
}

impl<T> Default for DashboardValue<T> {
    fn default() -> Self {
        Self::unknown()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub schema_version: String,
    pub generated_at: DashboardValue<String>,
    pub overview: DashboardOverview,
    pub nodes: Vec<DashboardNodeSummary>,
    pub data_sources: Vec<DataSourceStatus>,
    pub execution_gateways: Vec<ExecutionGatewayStatus>,
    pub risk: RiskStatus,
    pub runtime_modules: Vec<RuntimeModuleStatus>,
    pub logs: Vec<LogStatus>,
    pub metrics: Vec<MetricStatus>,
    pub alerts: AlertSummary,
    pub controls: Vec<ControlStatus>,
    pub gaps: Vec<DashboardGap>,
}

impl DashboardSnapshot {
    #[must_use]
    pub fn empty(generated_at: impl Into<String>) -> Self {
        Self {
            schema_version: DASHBOARD_SNAPSHOT_SCHEMA_VERSION.to_string(),
            generated_at: DashboardValue::available(generated_at.into()),
            overview: DashboardOverview::default(),
            nodes: Vec::new(),
            data_sources: Vec::new(),
            execution_gateways: Vec::new(),
            risk: RiskStatus::unknown(),
            runtime_modules: Vec::new(),
            logs: Vec::new(),
            metrics: Vec::new(),
            alerts: AlertSummary::default(),
            controls: Vec::new(),
            gaps: Vec::new(),
        }
    }

    #[must_use]
    pub fn from_nodes(generated_at: impl Into<String>, nodes: Vec<DashboardNodeSummary>) -> Self {
        let overview = DashboardOverview::from_nodes(&nodes);
        Self {
            overview,
            nodes,
            ..Self::empty(generated_at)
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardOverview {
    pub node_count: u64,
    pub running_nodes: u64,
    pub stopped_nodes: u64,
    pub error_nodes: u64,
    pub unknown_nodes: u64,
    pub health: HealthStatus,
    pub sandbox_only: bool,
    pub external_venue_connection: bool,
    pub real_orders_submitted: bool,
    pub latest_transition_at: DashboardValue<String>,
    pub latest_error: Option<String>,
}

impl DashboardOverview {
    #[must_use]
    pub fn from_nodes(nodes: &[DashboardNodeSummary]) -> Self {
        let mut overview = Self {
            node_count: nodes.len() as u64,
            sandbox_only: true,
            health: HealthStatus::Unknown,
            ..Self::default()
        };

        for node in nodes {
            match node.lifecycle_state {
                LifecycleStatus::Running => overview.running_nodes += 1,
                LifecycleStatus::Stopped => overview.stopped_nodes += 1,
                LifecycleStatus::Error => overview.error_nodes += 1,
                LifecycleStatus::Unknown => overview.unknown_nodes += 1,
                LifecycleStatus::Starting
                | LifecycleStatus::Pausing
                | LifecycleStatus::Paused
                | LifecycleStatus::Resuming
                | LifecycleStatus::Stopping => {}
            }
            overview.external_venue_connection |= node.external_venue_connection;
            overview.real_orders_submitted |= node.real_orders_submitted;
            if overview.latest_error.is_none() {
                overview.latest_error.clone_from(&node.last_error);
            }
        }

        overview.health = derive_overview_health(&overview);
        overview
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardNodeSummary {
    pub node_id: String,
    pub lifecycle_state: LifecycleStatus,
    pub process_mode: ProcessMode,
    pub process_state: SupervisorProcessState,
    pub pid: SnapshotValue<u32>,
    pub health: HealthStatus,
    pub config_path: SnapshotValue<String>,
    pub artifact_root: SnapshotValue<String>,
    pub generated_at: SnapshotValue<String>,
    pub started_at: SnapshotValue<String>,
    pub stopped_at: SnapshotValue<String>,
    pub last_transition_at: SnapshotValue<String>,
    pub last_error: Option<String>,
    pub external_venue_connection: bool,
    pub real_orders_submitted: bool,
    pub gaps: Vec<DashboardGap>,
}

impl DashboardNodeSummary {
    #[must_use]
    pub fn from_status(status: &NodeStatus) -> Self {
        Self {
            node_id: status.node_id.clone(),
            lifecycle_state: status.lifecycle_state,
            process_mode: status.process_mode,
            process_state: SupervisorProcessState::Unknown,
            pid: SnapshotValue::unknown(),
            health: derive_node_health(status),
            config_path: status.config_path.clone(),
            artifact_root: status.artifact_root.clone(),
            generated_at: status.generated_at.clone(),
            started_at: status.started_at.clone(),
            stopped_at: status.stopped_at.clone(),
            last_transition_at: status.last_transition_at.clone(),
            last_error: status.last_error.clone(),
            external_venue_connection: status.external_venue_connection,
            real_orders_submitted: status.real_orders_submitted,
            gaps: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataSourceStatus {
    pub source_id: String,
    pub source_kind: DashboardValue<String>,
    pub provider: DashboardValue<String>,
    pub connection: ConnectionStatus,
    pub freshness: DashboardValue<String>,
    pub lag_ms: DashboardValue<u64>,
    pub health: HealthStatus,
    pub last_error: DashboardValue<String>,
}

impl DataSourceStatus {
    #[must_use]
    pub fn unknown(source_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            source_kind: DashboardValue::unknown(),
            provider: DashboardValue::unknown(),
            connection: ConnectionStatus::Unknown,
            freshness: DashboardValue::unknown(),
            lag_ms: DashboardValue::unknown(),
            health: HealthStatus::Unknown,
            last_error: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderCountSummary {
    pub open: DashboardValue<u64>,
    pub inflight: DashboardValue<u64>,
    pub closed: DashboardValue<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionGatewayStatus {
    pub gateway_id: String,
    pub venue: DashboardValue<String>,
    pub connection: ConnectionStatus,
    pub started: DashboardValue<bool>,
    pub account_ref: DashboardValue<String>,
    pub order_counts: OrderCountSummary,
    pub last_report_at: DashboardValue<String>,
    pub last_error: DashboardValue<String>,
}

impl ExecutionGatewayStatus {
    #[must_use]
    pub fn unknown(gateway_id: impl Into<String>) -> Self {
        Self {
            gateway_id: gateway_id.into(),
            venue: DashboardValue::unknown(),
            connection: ConnectionStatus::Unknown,
            started: DashboardValue::unknown(),
            account_ref: DashboardValue::redacted(),
            order_counts: OrderCountSummary::default(),
            last_report_at: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectionSummary {
    pub reason: DashboardValue<String>,
    pub last_rejected_at: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RiskStatus {
    pub availability: DashboardAvailability,
    pub trading_state: RiskTradingState,
    pub health: HealthStatus,
    pub command_count: DashboardValue<u64>,
    pub event_count: DashboardValue<u64>,
    pub rejections_total: DashboardValue<u64>,
    pub last_rejection: DashboardValue<RejectionSummary>,
    pub last_error: DashboardValue<String>,
}

impl RiskStatus {
    #[must_use]
    pub fn unknown() -> Self {
        Self {
            availability: DashboardAvailability::Unknown,
            trading_state: RiskTradingState::Unknown,
            health: HealthStatus::Unknown,
            command_count: DashboardValue::unknown(),
            event_count: DashboardValue::unknown(),
            rejections_total: DashboardValue::unknown(),
            last_rejection: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeModuleStatus {
    pub module_name: String,
    pub status: DashboardValue<String>,
    pub health: HealthStatus,
    pub last_seen_at: DashboardValue<String>,
    pub last_error: DashboardValue<String>,
    pub evidence_source: DashboardValue<String>,
}

impl RuntimeModuleStatus {
    #[must_use]
    pub fn unknown(module_name: impl Into<String>) -> Self {
        Self {
            module_name: module_name.into(),
            status: DashboardValue::unknown(),
            health: HealthStatus::Unknown,
            last_seen_at: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
            evidence_source: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogStatus {
    pub log_id: String,
    pub node_id: DashboardValue<String>,
    pub path: DashboardValue<String>,
    pub availability: DashboardAvailability,
    pub last_seen_at: DashboardValue<String>,
    pub last_error: DashboardValue<String>,
}

impl LogStatus {
    #[must_use]
    pub fn unknown(log_id: impl Into<String>) -> Self {
        Self {
            log_id: log_id.into(),
            node_id: DashboardValue::unknown(),
            path: DashboardValue::unknown(),
            availability: DashboardAvailability::Unknown,
            last_seen_at: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricStatus {
    pub metric_id: String,
    pub node_id: DashboardValue<String>,
    pub value: DashboardValue<String>,
    pub availability: DashboardAvailability,
    pub last_seen_at: DashboardValue<String>,
    pub last_error: DashboardValue<String>,
}

impl MetricStatus {
    #[must_use]
    pub fn unknown(metric_id: impl Into<String>) -> Self {
        Self {
            metric_id: metric_id.into(),
            node_id: DashboardValue::unknown(),
            value: DashboardValue::unknown(),
            availability: DashboardAvailability::Unknown,
            last_seen_at: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlertSummary {
    pub active_count: u64,
    pub counts_by_severity: BTreeMap<String, u64>,
    pub active: Vec<DashboardAlert>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardAlert {
    pub alert_id: String,
    pub severity: String,
    pub source: String,
    pub message: String,
    pub first_seen_at: DashboardValue<String>,
    pub last_seen_at: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlStatus {
    pub action: String,
    pub availability: DashboardAvailability,
    pub enabled: bool,
    pub reason: DashboardValue<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlActionStatus {
    Accepted,
    Rejected,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    #[default]
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlActionResponse {
    pub action_id: String,
    pub action: String,
    pub status: ControlActionStatus,
    pub previous_state: LifecycleStatus,
    pub current_state: LifecycleStatus,
    pub started_at: DashboardValue<String>,
    pub finished_at: DashboardValue<String>,
    pub error_code: DashboardValue<String>,
    pub message: DashboardValue<String>,
    pub observability_ref: DashboardValue<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardGap {
    pub field_path: String,
    pub reason: DashboardAvailability,
    pub owner_task: DashboardValue<String>,
    pub notes: DashboardValue<String>,
}

impl DashboardGap {
    #[must_use]
    pub fn new(
        field_path: impl Into<String>,
        reason: DashboardAvailability,
        owner_task: impl Into<String>,
        notes: impl Into<String>,
    ) -> Self {
        Self {
            field_path: field_path.into(),
            reason,
            owner_task: DashboardValue::available(owner_task.into()),
            notes: DashboardValue::available(notes.into()),
        }
    }
}

/// Builds a dashboard snapshot from local supervisor registry and node artifacts.
///
/// # Errors
///
/// Returns an error if the registry file exists but cannot be read. Missing or
/// invalid supervisor artifacts are represented as explicit dashboard gaps so
/// callers can still render an owner-visible partial snapshot.
pub fn snapshot_from_supervisor_artifacts(
    registry_path: impl AsRef<FsPath>,
    generated_at: impl Into<String>,
) -> anyhow::Result<DashboardSnapshot> {
    let registry_path = registry_path.as_ref();
    let mut snapshot = DashboardSnapshot::empty(generated_at);

    if !registry_path.exists() {
        snapshot.gaps.push(DashboardGap::new(
            "supervisor.registry",
            DashboardAvailability::Unknown,
            "V03-004",
            format!(
                "supervisor registry artifact '{}' is missing",
                registry_path.display()
            ),
        ));
        return Ok(snapshot);
    }

    let raw = fs::read_to_string(registry_path).with_context(|| {
        format!(
            "failed to read supervisor registry '{}'",
            registry_path.display()
        )
    })?;
    let registry = match serde_json::from_str::<SupervisorRegistry>(&raw) {
        Ok(registry) => registry,
        Err(error) => {
            snapshot.gaps.push(DashboardGap::new(
                "supervisor.registry",
                DashboardAvailability::Unknown,
                "V03-004",
                format!("invalid supervisor registry artifact: {error}"),
            ));
            return Ok(snapshot);
        }
    };

    if registry.nodes.is_empty() {
        snapshot.gaps.push(DashboardGap::new(
            "nodes",
            DashboardAvailability::NotConfigured,
            "V03-004",
            "supervisor registry contains no nodes",
        ));
        return Ok(snapshot);
    }

    let mut nodes = Vec::with_capacity(registry.nodes.len());
    let mut statuses = Vec::with_capacity(registry.nodes.len());

    for record in registry.nodes.values() {
        let ArtifactStatus {
            status,
            status_availability,
            mut gaps,
        } = read_status_artifact(record);

        gaps.extend(process_gaps(record));
        let RuntimeModuleReadout {
            modules,
            gaps: module_gaps,
        } = runtime_modules_from_status(record, &status);
        gaps.extend(module_gaps);
        let mut node = DashboardNodeSummary::from_status(&status);
        node.node_id = record.node_id.clone();
        node.process_state = record.process.state;
        node.pid = record.process.pid.clone();
        node.gaps = gaps.clone();
        snapshot.gaps.extend(gaps);

        snapshot
            .data_sources
            .push(data_source_from_status(record, &status));
        snapshot
            .execution_gateways
            .push(execution_gateway_from_status(record, &status));
        snapshot.runtime_modules.extend(modules);
        snapshot
            .logs
            .extend(log_statuses_from_record(record, &status));
        snapshot
            .metrics
            .extend(metric_statuses_from_record(record, status_availability));

        nodes.push(node);
        statuses.push(status);
    }

    snapshot.overview = DashboardOverview::from_nodes(&nodes);
    snapshot.nodes = nodes;
    snapshot.risk = aggregate_risk_status(&statuses);
    snapshot.controls = control_statuses_from_nodes(&snapshot.nodes);
    snapshot.alerts = alert_summary_from_gaps(&snapshot.gaps);
    Ok(snapshot)
}

struct ArtifactStatus {
    status: NodeStatus,
    status_availability: DashboardAvailability,
    gaps: Vec<DashboardGap>,
}

fn read_status_artifact(record: &SupervisorNodeRecord) -> ArtifactStatus {
    let mut gaps = Vec::new();

    if !record.status_path.exists() {
        let mut status = record.last_known_status.clone();
        status.generated_at = SnapshotValue::stale();
        status.last_error = Some(format!(
            "status artifact '{}' is missing",
            record.status_path.display()
        ));
        gaps.push(DashboardGap::new(
            format!("nodes.{}.status", record.node_id),
            DashboardAvailability::Unknown,
            "V03-004",
            format!(
                "status artifact '{}' is missing",
                record.status_path.display()
            ),
        ));
        return ArtifactStatus {
            status,
            status_availability: DashboardAvailability::Unknown,
            gaps,
        };
    }

    let raw = match fs::read_to_string(&record.status_path) {
        Ok(raw) => raw,
        Err(error) => {
            let mut status = record.last_known_status.clone();
            status.generated_at = SnapshotValue::stale();
            status.last_error = Some(format!("failed to read status artifact: {error}"));
            gaps.push(DashboardGap::new(
                format!("nodes.{}.status", record.node_id),
                DashboardAvailability::Unknown,
                "V03-004",
                format!(
                    "failed to read status artifact '{}': {error}",
                    record.status_path.display()
                ),
            ));
            return ArtifactStatus {
                status,
                status_availability: DashboardAvailability::Unknown,
                gaps,
            };
        }
    };

    match serde_json::from_str::<NodeStatus>(&raw) {
        Ok(status) if status.node_id == record.node_id => {
            let mut availability = availability_from_registry_state(record.status_artifact);
            if status.generated_at.availability == SnapshotAvailability::Stale {
                availability = DashboardAvailability::Stale;
                gaps.push(DashboardGap::new(
                    format!("nodes.{}.status.generated_at", record.node_id),
                    DashboardAvailability::Stale,
                    "V03-004",
                    "status artifact generated_at is stale",
                ));
            }
            ArtifactStatus {
                status,
                status_availability: availability,
                gaps,
            }
        }
        Ok(status) => {
            let mut fallback = record.last_known_status.clone();
            fallback.node_id.clone_from(&record.node_id);
            fallback.generated_at = SnapshotValue::stale();
            fallback.last_error = Some(format!(
                "status node identity mismatch: registry node '{}' received runtime node '{}'",
                record.node_id, status.node_id
            ));
            gaps.push(DashboardGap::new(
                format!("nodes.{}.status.node_id", record.node_id),
                DashboardAvailability::Unknown,
                "P0-006",
                fallback.last_error.clone().unwrap(),
            ));
            ArtifactStatus {
                status: fallback,
                status_availability: DashboardAvailability::Unknown,
                gaps,
            }
        }
        Err(error) => {
            let mut status = record.last_known_status.clone();
            status.generated_at = SnapshotValue::stale();
            status.last_error = Some(format!("invalid status artifact: {error}"));
            gaps.push(DashboardGap::new(
                format!("nodes.{}.status", record.node_id),
                DashboardAvailability::Unknown,
                "V03-004",
                format!(
                    "invalid status artifact '{}': {error}",
                    record.status_path.display()
                ),
            ));
            ArtifactStatus {
                status,
                status_availability: DashboardAvailability::Unknown,
                gaps,
            }
        }
    }
}

fn process_gaps(record: &SupervisorNodeRecord) -> Vec<DashboardGap> {
    let mut gaps = Vec::new();
    if record.process.state == SupervisorProcessState::Stale {
        gaps.push(DashboardGap::new(
            format!("nodes.{}.process", record.node_id),
            DashboardAvailability::Stale,
            "V03-004",
            "supervisor process state is stale",
        ));
    }
    if record.status_artifact == RegistryArtifactState::Stale {
        gaps.push(DashboardGap::new(
            format!("nodes.{}.status", record.node_id),
            DashboardAvailability::Stale,
            "V03-004",
            "registry marks status artifact as stale",
        ));
    }
    if record.metrics_artifact == RegistryArtifactState::Stale {
        gaps.push(DashboardGap::new(
            format!("nodes.{}.metrics", record.node_id),
            DashboardAvailability::Stale,
            "V03-004",
            "registry marks metrics artifact as stale",
        ));
    }
    gaps
}

fn data_source_from_status(record: &SupervisorNodeRecord, status: &NodeStatus) -> DataSourceStatus {
    DataSourceStatus {
        source_id: format!("{}:data", record.node_id),
        source_kind: DashboardValue::available("supervisor_artifact".to_string()),
        provider: DashboardValue::available("local".to_string()),
        connection: status.data_connection,
        freshness: dashboard_value_from_snapshot(&status.generated_at),
        lag_ms: DashboardValue::unknown(),
        health: health_from_connection(status.data_connection),
        last_error: optional_dashboard_value(status.last_error.clone()),
    }
}

fn execution_gateway_from_status(
    record: &SupervisorNodeRecord,
    status: &NodeStatus,
) -> ExecutionGatewayStatus {
    ExecutionGatewayStatus {
        gateway_id: status
            .execution
            .gateway_id
            .value
            .clone()
            .unwrap_or_else(|| format!("{}:execution", record.node_id)),
        venue: DashboardValue::unknown(),
        connection: status.execution.connection,
        started: dashboard_value_from_snapshot(&status.execution.started),
        account_ref: DashboardValue::redacted(),
        order_counts: OrderCountSummary {
            open: dashboard_value_from_snapshot(&status.execution.orders_open),
            inflight: dashboard_value_from_snapshot(&status.execution.orders_inflight),
            closed: dashboard_value_from_snapshot(&status.execution.orders_closed),
        },
        last_report_at: dashboard_value_from_snapshot(&status.execution.last_report_at),
        last_error: optional_dashboard_value(status.execution.last_error.clone()),
    }
}

struct RuntimeModuleReadout {
    modules: Vec<RuntimeModuleStatus>,
    gaps: Vec<DashboardGap>,
}

fn runtime_modules_from_status(
    record: &SupervisorNodeRecord,
    status: &NodeStatus,
) -> RuntimeModuleReadout {
    let evidence_source = DashboardValue::available(record.status_path.display().to_string());
    let logging = logging_module_status(record, status);
    let metrics_writer = metrics_writer_module_status(record, status);
    let mut modules = vec![
        RuntimeModuleStatus {
            module_name: module_name(record, "LiveNode"),
            status: DashboardValue::available(json_label(&status.lifecycle_state)),
            health: derive_node_health(status),
            last_seen_at: dashboard_value_from_snapshot(&status.generated_at),
            last_error: redacted_optional_dashboard_error(status.last_error.as_deref()),
            evidence_source: evidence_source.clone(),
        },
        RuntimeModuleStatus {
            module_name: module_name(record, "DataEngine"),
            status: DashboardValue::available(json_label(&status.data_connection)),
            health: health_from_connection(status.data_connection),
            last_seen_at: dashboard_value_from_snapshot(&status.generated_at),
            last_error: DashboardValue::unknown(),
            evidence_source: evidence_source.clone(),
        },
        RuntimeModuleStatus {
            module_name: module_name(record, "ExecutionEngine"),
            status: DashboardValue::available(json_label(&status.execution.connection)),
            health: health_from_connection(status.execution.connection),
            last_seen_at: dashboard_value_from_snapshot(&status.generated_at),
            last_error: redacted_optional_dashboard_error(status.execution.last_error.as_deref()),
            evidence_source: evidence_source.clone(),
        },
        RuntimeModuleStatus {
            module_name: module_name(record, "RiskEngine"),
            status: DashboardValue::available(json_label(&status.risk.trading_state)),
            health: status.risk.health,
            last_seen_at: dashboard_value_from_snapshot(&status.generated_at),
            last_error: redacted_optional_dashboard_error(status.risk.last_error.as_deref()),
            evidence_source,
        },
        logging.clone(),
        metrics_writer.clone(),
        supervisor_module_status(record),
    ];

    let mut gaps = Vec::new();
    if logging.status.availability != DashboardAvailability::Available {
        gaps.push(DashboardGap::new(
            module_gap_path(record, "Logging"),
            logging.status.availability,
            "V03-008",
            "one or more log artifacts are unavailable",
        ));
    }
    if metrics_writer.status.availability != DashboardAvailability::Available {
        gaps.push(DashboardGap::new(
            module_gap_path(record, "Metrics writer"),
            metrics_writer.status.availability,
            "V03-008",
            "metrics writer artifact is unavailable",
        ));
    }
    for module in ["NautilusKernel", "Portfolio", "Cache", "MessageBus"] {
        let (status, gap) = unsupported_runtime_module(
            record,
            module,
            "supervisor artifacts do not expose this module detail yet",
        );
        modules.push(status);
        gaps.push(gap);
    }

    RuntimeModuleReadout { modules, gaps }
}

fn logging_module_status(
    record: &SupervisorNodeRecord,
    status: &NodeStatus,
) -> RuntimeModuleStatus {
    let log_paths = [
        &record.stdout_log_path,
        &record.stderr_log_path,
        &record.events_log_path,
    ];
    let all_logs_present = log_paths.iter().all(|path| path.exists());
    RuntimeModuleStatus {
        module_name: module_name(record, "Logging"),
        status: if all_logs_present {
            DashboardValue::available("logs_available".to_string())
        } else {
            DashboardValue::unknown()
        },
        health: if all_logs_present {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        },
        last_seen_at: if all_logs_present {
            dashboard_value_from_snapshot(&status.generated_at)
        } else {
            DashboardValue::unknown()
        },
        last_error: if all_logs_present {
            DashboardValue::unknown()
        } else {
            DashboardValue::available("present".to_string())
        },
        evidence_source: DashboardValue::available(
            record.artifact_root.join("logs").display().to_string(),
        ),
    }
}

fn metrics_writer_module_status(
    record: &SupervisorNodeRecord,
    status: &NodeStatus,
) -> RuntimeModuleStatus {
    let availability = availability_from_registry_state(record.metrics_artifact);
    RuntimeModuleStatus {
        module_name: module_name(record, "Metrics writer"),
        status: match availability {
            DashboardAvailability::Available => {
                DashboardValue::available("metrics_available".to_string())
            }
            DashboardAvailability::Stale => DashboardValue::stale(),
            DashboardAvailability::NotConfigured => DashboardValue::not_configured(),
            DashboardAvailability::NotSupported => DashboardValue::not_supported(),
            DashboardAvailability::Redacted | DashboardAvailability::Unknown => {
                DashboardValue::unknown()
            }
        },
        health: match availability {
            DashboardAvailability::Available => HealthStatus::Healthy,
            DashboardAvailability::Stale => HealthStatus::Stale,
            DashboardAvailability::Unknown
            | DashboardAvailability::NotConfigured
            | DashboardAvailability::NotSupported
            | DashboardAvailability::Redacted => HealthStatus::Unknown,
        },
        last_seen_at: if record.metrics_path.exists() {
            dashboard_value_from_snapshot(&status.generated_at)
        } else {
            DashboardValue::unknown()
        },
        last_error: if record.metrics_path.exists() {
            DashboardValue::unknown()
        } else {
            DashboardValue::available("present".to_string())
        },
        evidence_source: DashboardValue::available(record.metrics_path.display().to_string()),
    }
}

fn supervisor_module_status(record: &SupervisorNodeRecord) -> RuntimeModuleStatus {
    RuntimeModuleStatus {
        module_name: module_name(record, "Supervisor"),
        status: DashboardValue::available(json_label(&record.process.state)),
        health: health_from_process_state(record.process.state),
        last_seen_at: dashboard_value_from_snapshot(&record.process.updated_at),
        last_error: DashboardValue::unknown(),
        evidence_source: DashboardValue::available(record.pid_path.display().to_string()),
    }
}

fn unsupported_runtime_module(
    record: &SupervisorNodeRecord,
    module: &str,
    notes: &str,
) -> (RuntimeModuleStatus, DashboardGap) {
    (
        RuntimeModuleStatus {
            module_name: module_name(record, module),
            status: DashboardValue::not_supported(),
            health: HealthStatus::Unknown,
            last_seen_at: DashboardValue::not_supported(),
            last_error: DashboardValue::not_supported(),
            evidence_source: DashboardValue::not_supported(),
        },
        DashboardGap::new(
            module_gap_path(record, module),
            DashboardAvailability::NotSupported,
            "V03-008",
            notes,
        ),
    )
}

fn module_name(record: &SupervisorNodeRecord, module: &str) -> String {
    format!("{}:{module}", record.node_id)
}

fn module_gap_path(record: &SupervisorNodeRecord, module: &str) -> String {
    format!(
        "runtime_modules.{}.{}",
        record.node_id,
        module.to_ascii_lowercase().replace(' ', "_")
    )
}

fn log_statuses_from_record(record: &SupervisorNodeRecord, status: &NodeStatus) -> Vec<LogStatus> {
    [
        ("stdout", &record.stdout_log_path),
        ("stderr", &record.stderr_log_path),
        ("events", &record.events_log_path),
    ]
    .into_iter()
    .map(|(kind, path)| {
        let exists = path.exists();
        LogStatus {
            log_id: format!("{}:{kind}", record.node_id),
            node_id: DashboardValue::available(record.node_id.clone()),
            path: DashboardValue::available(path.display().to_string()),
            availability: if exists {
                DashboardAvailability::Available
            } else {
                DashboardAvailability::Unknown
            },
            last_seen_at: if exists {
                dashboard_value_from_snapshot(&status.generated_at)
            } else {
                DashboardValue::unknown()
            },
            last_error: if exists {
                DashboardValue::unknown()
            } else {
                DashboardValue::available(format!("log artifact '{}' is missing", path.display()))
            },
        }
    })
    .collect()
}

fn metric_statuses_from_record(
    record: &SupervisorNodeRecord,
    status_availability: DashboardAvailability,
) -> Vec<MetricStatus> {
    if !record.metrics_path.exists() {
        return vec![MetricStatus {
            metric_id: format!("{}:node-metrics", record.node_id),
            node_id: DashboardValue::available(record.node_id.clone()),
            value: DashboardValue::unknown(),
            availability: DashboardAvailability::Unknown,
            last_seen_at: DashboardValue::unknown(),
            last_error: DashboardValue::available(format!(
                "metrics artifact '{}' is missing",
                record.metrics_path.display()
            )),
        }];
    }

    let raw = match fs::read_to_string(&record.metrics_path) {
        Ok(raw) => raw,
        Err(error) => {
            return vec![MetricStatus {
                metric_id: format!("{}:node-metrics", record.node_id),
                node_id: DashboardValue::available(record.node_id.clone()),
                value: DashboardValue::unknown(),
                availability: DashboardAvailability::Unknown,
                last_seen_at: DashboardValue::unknown(),
                last_error: DashboardValue::available(format!(
                    "failed to read metrics artifact '{}': {error}",
                    record.metrics_path.display()
                )),
            }];
        }
    };

    let metrics = match serde_json::from_str::<NodeMetrics>(&raw) {
        Ok(metrics) if metrics.node_id == record.node_id => metrics,
        Ok(metrics) => {
            return vec![MetricStatus {
                metric_id: format!("{}:node-metrics", record.node_id),
                node_id: DashboardValue::available(record.node_id.clone()),
                value: DashboardValue::unknown(),
                availability: DashboardAvailability::Unknown,
                last_seen_at: DashboardValue::unknown(),
                last_error: DashboardValue::available(format!(
                    "metrics node identity mismatch: registry node '{}' received runtime node '{}'",
                    record.node_id, metrics.node_id
                )),
            }];
        }
        Err(error) => {
            return vec![MetricStatus {
                metric_id: format!("{}:node-metrics", record.node_id),
                node_id: DashboardValue::available(record.node_id.clone()),
                value: DashboardValue::unknown(),
                availability: DashboardAvailability::Unknown,
                last_seen_at: DashboardValue::unknown(),
                last_error: DashboardValue::available(format!(
                    "invalid metrics artifact '{}': {error}",
                    record.metrics_path.display()
                )),
            }];
        }
    };

    let availability = if metrics.generated_at.availability == SnapshotAvailability::Stale
        || record.metrics_artifact == RegistryArtifactState::Stale
        || status_availability == DashboardAvailability::Stale
    {
        DashboardAvailability::Stale
    } else {
        availability_from_registry_state(record.metrics_artifact)
    };

    [
        ("starts_total", metrics.starts_total.to_string()),
        ("stops_total", metrics.stops_total.to_string()),
        (
            "state_transitions_total",
            metrics.state_transitions_total.to_string(),
        ),
        (
            "uptime_ms",
            metrics
                .uptime_ms
                .value
                .map_or_else(|| "unknown".to_string(), |value| value.to_string()),
        ),
    ]
    .into_iter()
    .map(|(name, value)| MetricStatus {
        metric_id: format!("{}:{name}", record.node_id),
        node_id: DashboardValue::available(record.node_id.clone()),
        value: DashboardValue::available(value),
        availability,
        last_seen_at: dashboard_value_from_snapshot(&metrics.generated_at),
        last_error: optional_dashboard_value(metrics.last_error_summary.clone()),
    })
    .collect()
}

fn aggregate_risk_status(statuses: &[NodeStatus]) -> RiskStatus {
    if statuses.is_empty() {
        return RiskStatus::unknown();
    }

    let mut risk = RiskStatus {
        availability: DashboardAvailability::Available,
        trading_state: RiskTradingState::Unknown,
        health: HealthStatus::Unknown,
        command_count: DashboardValue::unknown(),
        event_count: DashboardValue::unknown(),
        rejections_total: DashboardValue::unknown(),
        last_rejection: DashboardValue::unknown(),
        last_error: DashboardValue::unknown(),
    };

    let mut commands = 0;
    let mut events = 0;
    let mut rejections = 0;
    let mut has_commands = false;
    let mut has_events = false;
    let mut has_rejections = false;

    for status in statuses {
        risk.trading_state = strongest_trading_state(risk.trading_state, status.risk.trading_state);
        risk.health = strongest_health(risk.health, status.risk.health);
        if let Some(value) = status.risk.command_count.value {
            commands += value;
            has_commands = true;
        }
        if let Some(value) = status.risk.event_count.value {
            events += value;
            has_events = true;
        }
        if let Some(value) = status.risk.rejections_total.value {
            rejections += value;
            has_rejections = true;
        }
        if risk.last_rejection.value.is_none()
            && let Some(reason) = status.risk.last_rejection.clone()
        {
            risk.last_rejection = DashboardValue::available(RejectionSummary {
                reason: DashboardValue::available(reason),
                last_rejected_at: dashboard_value_from_snapshot(&status.generated_at),
            });
        }
        if risk.last_error.value.is_none()
            && let Some(error) = status.risk.last_error.clone()
        {
            risk.last_error = DashboardValue::available(error);
        }
    }

    if has_commands {
        risk.command_count = DashboardValue::available(commands);
    }
    if has_events {
        risk.event_count = DashboardValue::available(events);
    }
    if has_rejections {
        risk.rejections_total = DashboardValue::available(rejections);
    }
    risk
}

fn control_statuses_from_nodes(nodes: &[DashboardNodeSummary]) -> Vec<ControlStatus> {
    let mut controls = Vec::with_capacity(nodes.len() * 6);
    for node in nodes {
        controls.push(ControlStatus {
            action: format!("start:{}", node.node_id),
            availability: DashboardAvailability::Available,
            enabled: node.lifecycle_state == LifecycleStatus::Stopped,
            reason: if node.lifecycle_state == LifecycleStatus::Running {
                DashboardValue::available("node is already running".to_string())
            } else if node.lifecycle_state == LifecycleStatus::Stopped {
                DashboardValue::available("node can be started by supervisor control".to_string())
            } else {
                DashboardValue::available("node is not stopped".to_string())
            },
        });
        controls.push(ControlStatus {
            action: format!("stop:{}", node.node_id),
            availability: DashboardAvailability::Available,
            enabled: node.lifecycle_state == LifecycleStatus::Running,
            reason: if node.lifecycle_state == LifecycleStatus::Running {
                DashboardValue::available("node can be stopped by supervisor control".to_string())
            } else {
                DashboardValue::available("node is not running".to_string())
            },
        });
        for (action, reason) in [
            (
                "pause",
                "pause is not supported by the v0.3 local dashboard MVP",
            ),
            (
                "resume",
                "resume is not supported by the v0.3 local dashboard MVP",
            ),
            (
                "reconnect_data",
                "data reconnect is not supported by the v0.3 local dashboard MVP",
            ),
            (
                "reconnect_execution",
                "execution reconnect is not supported by the v0.3 local dashboard MVP",
            ),
        ] {
            controls.push(ControlStatus {
                action: format!("{action}:{}", node.node_id),
                availability: DashboardAvailability::NotSupported,
                enabled: false,
                reason: DashboardValue::available(reason.to_string()),
            });
        }
    }
    controls
}

fn alert_summary_from_gaps(gaps: &[DashboardGap]) -> AlertSummary {
    let mut summary = AlertSummary::default();
    for (index, gap) in gaps.iter().enumerate() {
        let severity = match gap.reason {
            DashboardAvailability::Stale => "warning",
            DashboardAvailability::NotSupported | DashboardAvailability::NotConfigured => "info",
            DashboardAvailability::Available
            | DashboardAvailability::Redacted
            | DashboardAvailability::Unknown => "warning",
        };
        *summary
            .counts_by_severity
            .entry(severity.to_string())
            .or_insert(0) += 1;
        summary.active.push(DashboardAlert {
            alert_id: format!("gap-{index}"),
            severity: severity.to_string(),
            source: gap.field_path.clone(),
            message: gap
                .notes
                .value
                .clone()
                .unwrap_or_else(|| "dashboard gap detected".to_string()),
            first_seen_at: DashboardValue::unknown(),
            last_seen_at: DashboardValue::unknown(),
        });
    }
    summary.active_count = summary.active.len() as u64;
    summary
}

fn dashboard_value_from_snapshot<T: Clone>(value: &SnapshotValue<T>) -> DashboardValue<T> {
    match value.availability {
        SnapshotAvailability::Available => value
            .value
            .clone()
            .map_or_else(DashboardValue::unknown, DashboardValue::available),
        SnapshotAvailability::NotConfigured => DashboardValue::not_configured(),
        SnapshotAvailability::NotSupported => DashboardValue::not_supported(),
        SnapshotAvailability::Stale => DashboardValue::stale(),
        SnapshotAvailability::Unknown => DashboardValue::unknown(),
    }
}

fn optional_dashboard_value(value: Option<String>) -> DashboardValue<String> {
    value.map_or_else(DashboardValue::unknown, DashboardValue::available)
}

fn redacted_optional_dashboard_error(value: Option<&str>) -> DashboardValue<String> {
    if value.is_some() {
        DashboardValue::available("present".to_string())
    } else {
        DashboardValue::unknown()
    }
}

fn availability_from_registry_state(state: RegistryArtifactState) -> DashboardAvailability {
    match state {
        RegistryArtifactState::Available => DashboardAvailability::Available,
        RegistryArtifactState::Missing
        | RegistryArtifactState::Invalid
        | RegistryArtifactState::Unknown => DashboardAvailability::Unknown,
        RegistryArtifactState::Stale => DashboardAvailability::Stale,
    }
}

fn health_from_process_state(state: SupervisorProcessState) -> HealthStatus {
    match state {
        SupervisorProcessState::Running | SupervisorProcessState::Stopped => HealthStatus::Healthy,
        SupervisorProcessState::Stale => HealthStatus::Stale,
        SupervisorProcessState::NotStarted | SupervisorProcessState::Unknown => {
            HealthStatus::Unknown
        }
    }
}

fn health_from_connection(connection: ConnectionStatus) -> HealthStatus {
    match connection {
        ConnectionStatus::Connected | ConnectionStatus::NotConfigured => HealthStatus::Healthy,
        ConnectionStatus::Connecting | ConnectionStatus::Disconnecting => HealthStatus::Degraded,
        ConnectionStatus::Disconnected | ConnectionStatus::Stale => HealthStatus::Stale,
        ConnectionStatus::NotSupported | ConnectionStatus::Unknown => HealthStatus::Unknown,
    }
}

fn strongest_health(current: HealthStatus, next: HealthStatus) -> HealthStatus {
    match (current, next) {
        (HealthStatus::Error, _) | (_, HealthStatus::Error) => HealthStatus::Error,
        (HealthStatus::Stale, _) | (_, HealthStatus::Stale) => HealthStatus::Stale,
        (HealthStatus::Degraded, _) | (_, HealthStatus::Degraded) => HealthStatus::Degraded,
        (HealthStatus::Healthy, _) | (_, HealthStatus::Healthy) => HealthStatus::Healthy,
        _ => HealthStatus::Unknown,
    }
}

fn strongest_trading_state(current: RiskTradingState, next: RiskTradingState) -> RiskTradingState {
    match (current, next) {
        (RiskTradingState::Halted, _) | (_, RiskTradingState::Halted) => RiskTradingState::Halted,
        (RiskTradingState::Reducing, _) | (_, RiskTradingState::Reducing) => {
            RiskTradingState::Reducing
        }
        (RiskTradingState::Active, _) | (_, RiskTradingState::Active) => RiskTradingState::Active,
        _ => RiskTradingState::Unknown,
    }
}

fn json_label<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(ToString::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}

fn derive_node_health(status: &NodeStatus) -> HealthStatus {
    if status.last_error.is_some() {
        return HealthStatus::Error;
    }

    match status.lifecycle_state {
        LifecycleStatus::Running | LifecycleStatus::Stopped => HealthStatus::Healthy,
        LifecycleStatus::Error => HealthStatus::Error,
        LifecycleStatus::Unknown => HealthStatus::Unknown,
        LifecycleStatus::Starting
        | LifecycleStatus::Pausing
        | LifecycleStatus::Paused
        | LifecycleStatus::Resuming
        | LifecycleStatus::Stopping => HealthStatus::Degraded,
    }
}

fn derive_overview_health(overview: &DashboardOverview) -> HealthStatus {
    if overview.error_nodes > 0 || overview.latest_error.is_some() {
        HealthStatus::Error
    } else if overview.node_count == 0 || overview.unknown_nodes > 0 {
        HealthStatus::Unknown
    } else if overview.running_nodes > 0 {
        HealthStatus::Healthy
    } else {
        HealthStatus::Degraded
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        net::SocketAddr,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::supervisor::{
        NodeMetricArtifacts, NodeMetricCounts, NodeMetrics, RegisterNodeRequest,
        RegistryArtifactState, SupervisorNodeRecord, SupervisorProcessState, SupervisorRegistry,
        SupervisorRegistryStore, write_node_metrics_artifact,
    };
    use serde_json::{Value, json};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn empty_snapshot_serializes_stable_top_level_sections() {
        let snapshot = DashboardSnapshot::empty("2026-06-07T14:00:00Z");
        let value = serde_json::to_value(snapshot).unwrap();

        assert_eq!(value["schema_version"], DASHBOARD_SNAPSHOT_SCHEMA_VERSION);
        assert_eq!(
            value["generated_at"],
            json!({"availability": "available", "value": "2026-06-07T14:00:00Z"})
        );
        for key in [
            "overview",
            "nodes",
            "data_sources",
            "execution_gateways",
            "risk",
            "runtime_modules",
            "logs",
            "metrics",
            "alerts",
            "controls",
            "gaps",
        ] {
            assert!(value.get(key).is_some(), "missing dashboard key {key}");
        }
        assert_eq!(value["overview"]["node_count"], 0);
        assert_eq!(value["overview"]["health"], "unknown");
        assert_eq!(value["risk"]["availability"], "unknown");
    }

    #[test]
    fn dashboard_shell_includes_system_panel_mounts_and_redaction_helpers() {
        for mount_id in [
            "data-sources",
            "execution-gateways",
            "risk",
            "runtime-modules",
            "logs-metrics",
            "controls",
            "control-result",
        ] {
            assert!(
                DASHBOARD_HTML.contains(mount_id),
                "dashboard shell missing mount id {mount_id}"
            );
        }

        for js_symbol in [
            "renderDataSources",
            "renderExecutionGateways",
            "renderRisk",
            "renderRuntimeModules",
            "renderLogsMetrics",
            "renderControls",
            "redactedDashboardValue",
            "dashboardErrorValue",
            "No data sources reported",
            "No execution gateways reported",
            "No runtime modules reported",
            "No logs or metrics reported",
            "No controls reported",
        ] {
            assert!(
                DASHBOARD_JS.contains(js_symbol),
                "dashboard JS missing {js_symbol}"
            );
        }
    }

    #[test]
    fn one_node_snapshot_counts_running_node() {
        let status = NodeStatus {
            lifecycle_state: LifecycleStatus::Running,
            generated_at: SnapshotValue::available("2026-06-07T14:01:00Z".to_string()),
            ..NodeStatus::unknown("sandbox-a")
        };
        let node = DashboardNodeSummary::from_status(&status);
        let snapshot = DashboardSnapshot::from_nodes("2026-06-07T14:01:01Z", vec![node]);
        let value = serde_json::to_value(snapshot).unwrap();

        assert_eq!(value["overview"]["node_count"], 1);
        assert_eq!(value["overview"]["running_nodes"], 1);
        assert_eq!(value["overview"]["health"], "healthy");
        assert_eq!(value["nodes"][0]["node_id"], "sandbox-a");
        assert_eq!(value["nodes"][0]["lifecycle_state"], "running");
        assert_eq!(value["nodes"][0]["health"], "healthy");
    }

    #[test]
    fn two_node_snapshot_counts_running_and_stopped_nodes() {
        let running = DashboardNodeSummary::from_status(&NodeStatus {
            lifecycle_state: LifecycleStatus::Running,
            ..NodeStatus::unknown("sandbox-a")
        });
        let stopped = DashboardNodeSummary::from_status(&NodeStatus {
            lifecycle_state: LifecycleStatus::Stopped,
            ..NodeStatus::unknown("sandbox-b")
        });
        let snapshot =
            DashboardSnapshot::from_nodes("2026-06-07T14:02:00Z", vec![running, stopped]);
        let value = serde_json::to_value(snapshot).unwrap();

        assert_eq!(value["overview"]["node_count"], 2);
        assert_eq!(value["overview"]["running_nodes"], 1);
        assert_eq!(value["overview"]["stopped_nodes"], 1);
        assert_eq!(value["nodes"][1]["node_id"], "sandbox-b");
    }

    #[test]
    fn explicit_unavailable_states_survive_json_shape() {
        let mut snapshot = DashboardSnapshot::empty("2026-06-07T14:03:00Z");
        snapshot
            .data_sources
            .push(DataSourceStatus::unknown("sandbox-data"));
        snapshot
            .execution_gateways
            .push(ExecutionGatewayStatus::unknown("sandbox-exec"));
        snapshot
            .runtime_modules
            .push(RuntimeModuleStatus::unknown("MessageBus"));
        snapshot.logs.push(LogStatus::unknown("events"));
        snapshot.metrics.push(MetricStatus::unknown("node-metrics"));
        snapshot.gaps = vec![
            DashboardGap::new(
                "data_sources[0].last_event_at",
                DashboardAvailability::Unknown,
                "V03-004",
                "aggregator not implemented yet",
            ),
            DashboardGap::new(
                "execution_gateways",
                DashboardAvailability::NotConfigured,
                "V03-003",
                "no execution gateway configured",
            ),
            DashboardGap::new(
                "runtime_modules.cache",
                DashboardAvailability::NotSupported,
                "V03-008",
                "module detail is not supported yet",
            ),
            DashboardGap::new(
                "metrics.generated_at",
                DashboardAvailability::Stale,
                "V03-004",
                "metrics artifact is older than threshold",
            ),
            DashboardGap::new(
                "execution_gateways[0].account_ref",
                DashboardAvailability::Redacted,
                "V03-003",
                "account reference is intentionally hidden",
            ),
        ];
        snapshot.controls.push(ControlStatus {
            action: "pause_trading".to_string(),
            availability: DashboardAvailability::NotSupported,
            enabled: false,
            reason: DashboardValue::not_supported(),
        });

        let value = serde_json::to_value(snapshot).unwrap();
        let reasons: Vec<_> = value["gaps"]
            .as_array()
            .unwrap()
            .iter()
            .map(|gap| gap["reason"].as_str().unwrap())
            .collect();

        assert_eq!(
            reasons,
            [
                "unknown",
                "not_configured",
                "not_supported",
                "stale",
                "redacted"
            ]
        );
        assert_eq!(value["controls"][0]["availability"], "not_supported");
        assert_eq!(
            value["controls"][0]["reason"],
            json!({"availability": "not_supported"})
        );
        assert_eq!(value["data_sources"][0]["connection"], "unknown");
        assert_eq!(
            value["execution_gateways"][0]["account_ref"],
            json!({"availability": "redacted"})
        );
        assert_eq!(value["runtime_modules"][0]["module_name"], "MessageBus");
        assert_eq!(value["logs"][0]["availability"], "unknown");
        assert_eq!(value["metrics"][0]["availability"], "unknown");
        assert_eq!(value["risk"]["availability"], "unknown");
        assert_eq!(value["risk"]["trading_state"], "unknown");
    }

    #[test]
    fn detail_dtos_serialize_without_raw_or_secret_fields() {
        let mut snapshot = DashboardSnapshot::empty("2026-06-07T14:05:00Z");
        snapshot.data_sources.push(DataSourceStatus {
            source_id: "sandbox-data".to_string(),
            source_kind: DashboardValue::available("sandbox".to_string()),
            provider: DashboardValue::available("sandbox".to_string()),
            connection: ConnectionStatus::NotConfigured,
            freshness: DashboardValue::not_configured(),
            lag_ms: DashboardValue::not_configured(),
            health: HealthStatus::Unknown,
            last_error: DashboardValue::unknown(),
        });
        snapshot.execution_gateways.push(ExecutionGatewayStatus {
            gateway_id: "sandbox-exec".to_string(),
            venue: DashboardValue::available("SIM".to_string()),
            connection: ConnectionStatus::NotConfigured,
            started: DashboardValue::not_configured(),
            account_ref: DashboardValue::redacted(),
            order_counts: OrderCountSummary {
                open: DashboardValue::available(0),
                inflight: DashboardValue::available(0),
                closed: DashboardValue::available(0),
            },
            last_report_at: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
        });
        snapshot.risk = RiskStatus {
            availability: DashboardAvailability::Available,
            trading_state: RiskTradingState::Active,
            health: HealthStatus::Healthy,
            command_count: DashboardValue::available(0),
            event_count: DashboardValue::available(0),
            rejections_total: DashboardValue::available(0),
            last_rejection: DashboardValue::unknown(),
            last_error: DashboardValue::unknown(),
        };
        snapshot
            .runtime_modules
            .push(RuntimeModuleStatus::unknown("RiskEngine"));
        snapshot.controls.push(ControlStatus {
            action: "start".to_string(),
            availability: DashboardAvailability::Available,
            enabled: true,
            reason: DashboardValue::available("node is stopped".to_string()),
        });

        let response = ControlActionResponse {
            action_id: "action-001".to_string(),
            action: "start".to_string(),
            status: ControlActionStatus::Accepted,
            previous_state: LifecycleStatus::Stopped,
            current_state: LifecycleStatus::Starting,
            started_at: DashboardValue::available("2026-06-07T14:05:01Z".to_string()),
            finished_at: DashboardValue::unknown(),
            error_code: DashboardValue::unknown(),
            message: DashboardValue::available("start accepted".to_string()),
            observability_ref: DashboardValue::unknown(),
        };

        let snapshot_value = serde_json::to_value(snapshot).unwrap();
        let response_value = serde_json::to_value(response).unwrap();

        assert_eq!(
            snapshot_value["execution_gateways"][0]["account_ref"],
            json!({"availability": "redacted"})
        );
        assert_eq!(snapshot_value["risk"]["trading_state"], "active");
        assert_eq!(snapshot_value["controls"][0]["enabled"], true);
        assert_eq!(response_value["status"], "accepted");
        assert_eq!(response_value["previous_state"], "stopped");
        assert_eq!(response_value["current_state"], "starting");
        assert_forbidden_keys_absent(&snapshot_value);
        assert_forbidden_keys_absent(&response_value);
    }

    #[test]
    fn snapshot_shape_does_not_expose_forbidden_raw_or_secret_fields() {
        let snapshot = DashboardSnapshot::from_nodes(
            "2026-06-07T14:04:00Z",
            vec![DashboardNodeSummary::from_status(&NodeStatus::unknown(
                "sandbox-a",
            ))],
        );
        let value = serde_json::to_value(snapshot).unwrap();

        assert_forbidden_keys_absent(&value);
    }

    #[test]
    fn missing_supervisor_registry_records_gap() {
        let root = temp_root("missing-registry");
        let snapshot =
            snapshot_from_supervisor_artifacts(root.join("registry.json"), "2026-06-07T15:00:00Z")
                .unwrap();

        assert!(snapshot.nodes.is_empty());
        assert_eq!(snapshot.gaps.len(), 1);
        assert_eq!(snapshot.gaps[0].field_path, "supervisor.registry");
        assert_eq!(snapshot.gaps[0].reason, DashboardAvailability::Unknown);
        assert!(
            snapshot.gaps[0]
                .notes
                .value
                .as_deref()
                .unwrap()
                .contains("missing")
        );
    }

    #[test]
    fn empty_supervisor_registry_records_not_configured_gap() {
        let root = temp_root("empty-registry");
        let registry_path = root.join("registry.json");
        write_registry(&registry_path, []);

        let snapshot =
            snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:01:00Z").unwrap();

        assert!(snapshot.nodes.is_empty());
        assert_eq!(snapshot.overview.node_count, 0);
        assert_eq!(snapshot.gaps[0].field_path, "nodes");
        assert_eq!(
            snapshot.gaps[0].reason,
            DashboardAvailability::NotConfigured
        );
    }

    #[test]
    fn one_node_supervisor_artifacts_populate_dashboard_sections() {
        let root = temp_root("one-node");
        let registry_path = root.join("registry.json");
        let mut record = node_record(&root, "sandbox-a");
        let status = node_status_for_record(&record, LifecycleStatus::Running);
        write_status_artifact(&record, &status);
        write_metrics_artifact(&record, &status);
        write_log_artifacts(&record);
        record.status_artifact = RegistryArtifactState::Available;
        record.metrics_artifact = RegistryArtifactState::Available;
        record.process.state = SupervisorProcessState::Running;
        record.process.pid = SnapshotValue::available(42_001);
        write_registry(&registry_path, [record]);

        let snapshot =
            snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:02:00Z").unwrap();

        assert_eq!(snapshot.overview.node_count, 1);
        assert_eq!(snapshot.overview.running_nodes, 1);
        assert!(!snapshot.overview.external_venue_connection);
        assert!(!snapshot.overview.real_orders_submitted);
        assert_eq!(snapshot.nodes[0].node_id, "sandbox-a");
        assert_eq!(
            snapshot.nodes[0].process_state,
            SupervisorProcessState::Running
        );
        assert_eq!(snapshot.nodes[0].pid.value, Some(42_001));
        assert_eq!(snapshot.data_sources[0].source_id, "sandbox-a:data");
        assert_eq!(
            snapshot.execution_gateways[0].gateway_id,
            "sandbox-a:gateway"
        );
        assert_eq!(snapshot.risk.availability, DashboardAvailability::Available);
        assert_eq!(snapshot.logs.len(), 3);
        assert!(
            snapshot
                .logs
                .iter()
                .all(|log| log.availability == DashboardAvailability::Available)
        );
        assert!(snapshot.metrics.iter().any(|metric| {
            metric.metric_id == "sandbox-a:starts_total"
                && metric.value.value.as_deref() == Some("1")
        }));
        assert!(snapshot.runtime_modules.iter().any(|module| {
            module.module_name == "sandbox-a:NautilusKernel"
                && module.status.availability == DashboardAvailability::NotSupported
        }));
        assert!(snapshot.runtime_modules.iter().any(|module| {
            module.module_name == "sandbox-a:LiveNode"
                && module.status.value.as_deref() == Some("running")
                && module.health == HealthStatus::Healthy
        }));
        assert!(snapshot.runtime_modules.iter().any(|module| {
            module.module_name == "sandbox-a:Metrics writer"
                && module.status.availability == DashboardAvailability::Available
        }));
        assert_eq!(snapshot.runtime_modules.len(), 11);
        assert_eq!(snapshot.controls.len(), 6);
        assert!(snapshot.controls.iter().any(|control| {
            control.action == "start:sandbox-a"
                && !control.enabled
                && control.availability == DashboardAvailability::Available
        }));
        assert!(snapshot.controls.iter().any(|control| {
            control.action == "stop:sandbox-a"
                && control.enabled
                && control.availability == DashboardAvailability::Available
        }));
        assert!(snapshot.controls.iter().any(|control| {
            control.action == "reconnect_execution:sandbox-a"
                && !control.enabled
                && control.availability == DashboardAvailability::NotSupported
        }));
        assert!(snapshot.gaps.iter().any(|gap| {
            gap.field_path == "runtime_modules.sandbox-a.nautiluskernel"
                && gap.reason == DashboardAvailability::NotSupported
        }));
    }

    #[tokio::test]
    async fn dashboard_http_server_serves_shell_snapshot_and_rejects_invalid_action_state() {
        let root = temp_root("http-server");
        let registry_path = root.join("registry.json");
        let mut record = node_record(&root, "sandbox-a");
        let status = node_status_for_record(&record, LifecycleStatus::Running);
        write_status_artifact(&record, &status);
        write_metrics_artifact(&record, &status);
        write_log_artifacts(&record);
        record.status_artifact = RegistryArtifactState::Available;
        record.metrics_artifact = RegistryArtifactState::Available;
        write_registry(&registry_path, [record]);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let ntpro_node_bin = root.join("ntpro-node-missing");
        let server = tokio::spawn(async move {
            axum::serve(listener, dashboard_router(registry_path, ntpro_node_bin))
                .await
                .unwrap();
        });

        let shell = http_request(addr, "GET", "/dashboard").await;
        assert!(shell.contains("HTTP/1.1 200 OK"));
        assert!(shell.contains("NTPRO Dashboard"));

        let metadata = http_request(addr, "GET", "/api/server").await;
        assert!(metadata.contains("HTTP/1.1 200 OK"));
        let metadata_body = response_body(&metadata);
        let metadata_value: Value = serde_json::from_str(metadata_body).unwrap();
        assert_eq!(metadata_value["local_only"], true);
        assert!(
            metadata_value["registry_path"]
                .as_str()
                .unwrap()
                .ends_with("registry.json")
        );

        let snapshot = http_request(addr, "GET", "/api/snapshot").await;
        assert!(snapshot.contains("HTTP/1.1 200 OK"));
        let snapshot_body = response_body(&snapshot);
        let snapshot_value: Value = serde_json::from_str(snapshot_body).unwrap();
        assert_eq!(snapshot_value["nodes"][0]["node_id"], "sandbox-a");
        assert_eq!(snapshot_value["overview"]["running_nodes"], 1);
        assert_eq!(
            snapshot_value["data_sources"][0]["source_id"],
            "sandbox-a:data"
        );
        assert_eq!(
            snapshot_value["data_sources"][0]["source_kind"],
            json!({"availability": "available", "value": "supervisor_artifact"})
        );
        assert_eq!(
            snapshot_value["data_sources"][0]["provider"],
            json!({"availability": "available", "value": "local"})
        );
        assert_eq!(
            snapshot_value["execution_gateways"][0]["gateway_id"],
            "sandbox-a:gateway"
        );
        assert_eq!(
            snapshot_value["execution_gateways"][0]["account_ref"],
            json!({"availability": "redacted"})
        );
        assert_eq!(snapshot_value["risk"]["trading_state"], "active");
        assert_eq!(snapshot_value["risk"]["health"], "healthy");
        assert!(
            snapshot_value["runtime_modules"]
                .as_array()
                .unwrap()
                .iter()
                .any(|module| module["module_name"] == "sandbox-a:LiveNode")
        );
        assert!(
            snapshot_value["runtime_modules"]
                .as_array()
                .unwrap()
                .iter()
                .any(|module| module["module_name"] == "sandbox-a:MessageBus"
                    && module["status"]["availability"] == "not_supported")
        );
        assert_forbidden_keys_absent(&snapshot_value);

        let metrics = http_request(addr, "GET", "/api/nodes/sandbox-a/metrics").await;
        assert!(metrics.contains("HTTP/1.1 200 OK"));
        let metrics_body = response_body(&metrics);
        let metrics_value: Value = serde_json::from_str(metrics_body).unwrap();
        assert!(
            metrics_value
                .as_array()
                .unwrap()
                .iter()
                .any(|metric| metric["metric_id"] == "sandbox-a:starts_total")
        );

        let action = http_request(addr, "POST", "/api/nodes/sandbox-a/actions/start").await;
        assert!(action.contains("HTTP/1.1 409 Conflict"));
        let action_body = response_body(&action);
        let action_value: Value = serde_json::from_str(action_body).unwrap();
        assert_eq!(action_value["status"], "rejected");
        assert_eq!(
            action_value["error_code"],
            json!({"availability": "available", "value": "invalid_lifecycle_state"})
        );

        server.abort();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn dashboard_http_server_starts_and_stops_fixture_node() {
        let root = temp_root("http-control");
        let registry_path = root.join("registry.json");
        let config = write_config(&root, "sandbox-a");
        let fixture = write_fixture_node(&root);
        let store = SupervisorRegistryStore::new(registry_path.clone());
        store
            .register_node(RegisterNodeRequest {
                node_id: "sandbox-a".to_string(),
                config_path: config,
                artifact_root: Some(root.join("nodes").join("sandbox-a")),
            })
            .unwrap();

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, dashboard_router(registry_path, fixture))
                .await
                .unwrap();
        });

        let before = http_request(addr, "GET", "/api/snapshot").await;
        let before_value: Value = serde_json::from_str(response_body(&before)).unwrap();
        assert!(
            before_value["controls"]
                .as_array()
                .unwrap()
                .iter()
                .any(|control| control["action"] == "start:sandbox-a"
                    && control["enabled"] == true)
        );

        let started = http_request(addr, "POST", "/api/nodes/sandbox-a/actions/start").await;
        assert!(started.contains("HTTP/1.1 200 OK"));
        let started_value: Value = serde_json::from_str(response_body(&started)).unwrap();
        assert_eq!(started_value["status"], "succeeded");
        assert_eq!(started_value["previous_state"], "stopped");
        assert_eq!(started_value["current_state"], "running");
        assert_eq!(
            started_value["error_code"],
            json!({"availability": "unknown"})
        );

        let running = http_request(addr, "GET", "/api/snapshot").await;
        let running_value: Value = serde_json::from_str(response_body(&running)).unwrap();
        assert_eq!(running_value["overview"]["running_nodes"], 1);
        assert!(
            running_value["controls"]
                .as_array()
                .unwrap()
                .iter()
                .any(|control| control["action"] == "stop:sandbox-a" && control["enabled"] == true)
        );

        let stopped = http_request(addr, "POST", "/api/nodes/sandbox-a/actions/stop").await;
        assert!(stopped.contains("HTTP/1.1 200 OK"));
        let stopped_value: Value = serde_json::from_str(response_body(&stopped)).unwrap();
        assert_eq!(stopped_value["status"], "succeeded");
        assert_eq!(stopped_value["previous_state"], "running");
        assert_eq!(stopped_value["current_state"], "stopped");
        assert_eq!(
            stopped_value["error_code"],
            json!({"availability": "unknown"})
        );

        let after = http_request(addr, "GET", "/api/snapshot").await;
        let after_value: Value = serde_json::from_str(response_body(&after)).unwrap();
        assert_eq!(after_value["overview"]["stopped_nodes"], 1);
        assert!(
            after_value["controls"]
                .as_array()
                .unwrap()
                .iter()
                .any(|control| control["action"] == "start:sandbox-a"
                    && control["enabled"] == true)
        );

        server.abort();
    }

    #[test]
    fn two_node_supervisor_artifacts_aggregate_overview() {
        let root = temp_root("two-node");
        let registry_path = root.join("registry.json");
        let mut first = node_record(&root, "sandbox-a");
        let mut second = node_record(&root, "sandbox-b");
        let first_status = node_status_for_record(&first, LifecycleStatus::Running);
        let second_status = node_status_for_record(&second, LifecycleStatus::Stopped);

        for (record, status) in [(&mut first, &first_status), (&mut second, &second_status)] {
            write_status_artifact(record, status);
            write_metrics_artifact(record, status);
            write_log_artifacts(record);
            record.status_artifact = RegistryArtifactState::Available;
            record.metrics_artifact = RegistryArtifactState::Available;
        }
        write_registry(&registry_path, [first, second]);

        let snapshot =
            snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:03:00Z").unwrap();

        assert_eq!(snapshot.overview.node_count, 2);
        assert_eq!(snapshot.overview.running_nodes, 1);
        assert_eq!(snapshot.overview.stopped_nodes, 1);
        assert_eq!(snapshot.nodes.len(), 2);
        assert_eq!(snapshot.data_sources.len(), 2);
        assert_eq!(snapshot.execution_gateways.len(), 2);
        assert_eq!(snapshot.runtime_modules.len(), 22);
        assert!(!snapshot.overview.external_venue_connection);
        assert!(!snapshot.overview.real_orders_submitted);
    }

    #[test]
    fn missing_status_artifact_is_marked_explicitly() {
        let root = temp_root("missing-status");
        let registry_path = root.join("registry.json");
        let mut record = node_record(&root, "sandbox-a");
        record.status_artifact = RegistryArtifactState::Missing;
        write_registry(&registry_path, [record]);

        let snapshot =
            snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:04:00Z").unwrap();

        assert_eq!(
            snapshot.nodes[0].generated_at.availability,
            SnapshotAvailability::Stale
        );
        assert!(snapshot.gaps.iter().any(|gap| {
            gap.field_path == "nodes.sandbox-a.status"
                && gap.notes.value.as_deref().unwrap().contains("missing")
        }));
    }

    #[test]
    fn invalid_status_artifact_is_marked_explicitly() {
        let root = temp_root("invalid-status");
        let registry_path = root.join("registry.json");
        let mut record = node_record(&root, "sandbox-a");
        create_node_dirs(&record);
        fs::write(&record.status_path, "not-json").unwrap();
        record.status_artifact = RegistryArtifactState::Invalid;
        write_registry(&registry_path, [record]);

        let snapshot =
            snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:05:00Z").unwrap();

        assert!(
            snapshot.nodes[0]
                .last_error
                .as_deref()
                .unwrap()
                .contains("invalid status")
        );
        assert!(snapshot.gaps.iter().any(|gap| {
            gap.field_path == "nodes.sandbox-a.status"
                && gap.notes.value.as_deref().unwrap().contains("invalid")
        }));
    }

    #[test]
    fn mismatched_status_identity_is_marked_explicitly() {
        let root = temp_root("mismatched-status-identity");
        let registry_path = root.join("registry.json");
        let mut record = node_record(&root, "sandbox-a");
        let mut status = node_status_for_record(&record, LifecycleStatus::Running);
        status.node_id = "sandbox-b".to_string();
        write_status_artifact(&record, &status);
        record.status_artifact = RegistryArtifactState::Available;
        write_registry(&registry_path, [record]);

        let snapshot =
            snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:05:30Z").unwrap();

        assert_eq!(snapshot.overview.running_nodes, 0);
        assert!(
            snapshot.nodes[0]
                .last_error
                .as_deref()
                .unwrap()
                .contains("status node identity mismatch")
        );
        assert!(snapshot.gaps.iter().any(|gap| {
            gap.field_path == "nodes.sandbox-a.status.node_id"
                && gap
                    .notes
                    .value
                    .as_deref()
                    .unwrap()
                    .contains("registry node 'sandbox-a' received runtime node 'sandbox-b'")
        }));
    }

    #[test]
    fn missing_metrics_artifact_is_marked_explicitly() {
        let root = temp_root("missing-metrics");
        let registry_path = root.join("registry.json");
        let mut record = node_record(&root, "sandbox-a");
        let status = node_status_for_record(&record, LifecycleStatus::Running);
        write_status_artifact(&record, &status);
        record.status_artifact = RegistryArtifactState::Available;
        record.metrics_artifact = RegistryArtifactState::Missing;
        write_registry(&registry_path, [record]);

        let snapshot =
            snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:06:00Z").unwrap();

        assert_eq!(snapshot.metrics.len(), 1);
        assert_eq!(
            snapshot.metrics[0].availability,
            DashboardAvailability::Unknown
        );
        assert!(snapshot.runtime_modules.iter().any(|module| {
            module.module_name == "sandbox-a:Metrics writer"
                && module.status.availability == DashboardAvailability::Unknown
                && module.health == HealthStatus::Unknown
        }));
        assert!(snapshot.gaps.iter().any(|gap| {
            gap.field_path == "runtime_modules.sandbox-a.metrics_writer"
                && gap.reason == DashboardAvailability::Unknown
        }));
        assert!(
            snapshot.metrics[0]
                .last_error
                .value
                .as_deref()
                .unwrap()
                .contains("missing")
        );
    }

    #[test]
    fn mismatched_metrics_identity_is_marked_explicitly() {
        let root = temp_root("mismatched-metrics-identity");
        let registry_path = root.join("registry.json");
        let mut record = node_record(&root, "sandbox-a");
        let status = node_status_for_record(&record, LifecycleStatus::Running);
        write_status_artifact(&record, &status);
        let mut metrics = NodeMetrics::from_status(
            &status,
            &NodeMetricArtifacts::from_record(&record),
            NodeMetricCounts {
                uptime_ms: Some(100),
                starts_total: 1,
                stops_total: 0,
                state_transitions_total: 1,
            },
        );
        metrics.node_id = "sandbox-b".to_string();
        write_node_metrics_artifact(&record.metrics_path, &metrics).unwrap();
        record.status_artifact = RegistryArtifactState::Available;
        record.metrics_artifact = RegistryArtifactState::Available;
        write_registry(&registry_path, [record]);

        let snapshot =
            snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:06:30Z").unwrap();

        assert_eq!(snapshot.metrics.len(), 1);
        assert_eq!(
            snapshot.metrics[0].availability,
            DashboardAvailability::Unknown
        );
        assert!(
            snapshot.metrics[0]
                .last_error
                .value
                .as_deref()
                .unwrap()
                .contains(
                    "metrics node identity mismatch: registry node 'sandbox-a' received runtime node 'sandbox-b'"
                )
        );
    }

    #[test]
    fn stale_process_and_artifact_states_are_marked_explicitly() {
        let root = temp_root("stale");
        let registry_path = root.join("registry.json");
        let mut record = node_record(&root, "sandbox-a");
        let mut status = node_status_for_record(&record, LifecycleStatus::Running);
        status.generated_at = SnapshotValue::stale();
        write_status_artifact(&record, &status);
        let mut metrics = NodeMetrics::from_status(
            &status,
            &NodeMetricArtifacts::from_record(&record),
            NodeMetricCounts {
                uptime_ms: Some(100),
                starts_total: 1,
                stops_total: 0,
                state_transitions_total: 1,
            },
        );
        metrics.generated_at = SnapshotValue::stale();
        write_node_metrics_artifact(&record.metrics_path, &metrics).unwrap();
        record.process.state = SupervisorProcessState::Stale;
        record.status_artifact = RegistryArtifactState::Stale;
        record.metrics_artifact = RegistryArtifactState::Stale;
        write_registry(&registry_path, [record]);

        let snapshot =
            snapshot_from_supervisor_artifacts(&registry_path, "2026-06-07T15:07:00Z").unwrap();

        assert!(snapshot.gaps.iter().any(|gap| {
            gap.field_path == "nodes.sandbox-a.process"
                && gap.reason == DashboardAvailability::Stale
        }));
        assert!(snapshot.gaps.iter().any(|gap| {
            gap.field_path == "nodes.sandbox-a.status.generated_at"
                && gap.reason == DashboardAvailability::Stale
        }));
        assert!(
            snapshot
                .metrics
                .iter()
                .all(|metric| metric.availability == DashboardAvailability::Stale)
        );
        assert!(snapshot.alerts.active_count >= 3);
    }

    fn temp_root(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "ntpro-v03-004-{name}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_config(root: &Path, name: &str) -> PathBuf {
        let path = root.join(format!("{name}.toml"));
        fs::write(&path, "[run]\nid = \"dashboard-control-smoke\"\n").unwrap();
        path
    }

    fn node_record(root: &std::path::Path, node_id: &str) -> SupervisorNodeRecord {
        let config_path = root.join(format!("{node_id}.toml"));
        fs::write(&config_path, "environment = \"sandbox\"\n").unwrap();
        SupervisorNodeRecord::new(
            node_id.to_string(),
            config_path,
            root.join("nodes").join(node_id),
        )
    }

    fn node_status_for_record(
        record: &SupervisorNodeRecord,
        lifecycle_state: LifecycleStatus,
    ) -> NodeStatus {
        let mut status = NodeStatus::unknown(record.node_id.clone());
        status.process_mode = ProcessMode::TestHarness;
        status.config_path = SnapshotValue::available(record.config_path.display().to_string());
        status.artifact_root = SnapshotValue::available(record.artifact_root.display().to_string());
        status.lifecycle_state = lifecycle_state;
        status.previous_lifecycle_state = LifecycleStatus::Stopped;
        status.data_connection = ConnectionStatus::NotConfigured;
        status.execution_connection = ConnectionStatus::NotConfigured;
        status.execution.gateway_id =
            SnapshotValue::available(format!("{}:gateway", record.node_id));
        status.execution.connection = ConnectionStatus::NotConfigured;
        status.execution.started =
            SnapshotValue::available(lifecycle_state == LifecycleStatus::Running);
        status.execution.orders_open = SnapshotValue::available(0);
        status.execution.orders_inflight = SnapshotValue::available(0);
        status.execution.orders_closed = SnapshotValue::available(0);
        status.execution.last_report_at =
            SnapshotValue::available("2026-06-07T15:00:00Z".to_string());
        status.risk.trading_state = RiskTradingState::Active;
        status.risk.health = HealthStatus::Healthy;
        status.risk.command_count = SnapshotValue::available(0);
        status.risk.event_count = SnapshotValue::available(0);
        status.risk.rejections_total = SnapshotValue::available(0);
        status.generated_at = SnapshotValue::available("2026-06-07T15:00:00Z".to_string());
        status.last_transition_at = SnapshotValue::available("2026-06-07T15:00:00Z".to_string());
        status
    }

    fn write_registry(
        registry_path: &std::path::Path,
        records: impl IntoIterator<Item = SupervisorNodeRecord>,
    ) {
        let mut registry = SupervisorRegistry::default();
        for record in records {
            registry.nodes.insert(record.node_id.clone(), record);
        }
        registry.updated_at = SnapshotValue::available("2026-06-07T15:00:00Z".to_string());
        if let Some(parent) = registry_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let raw = serde_json::to_string_pretty(&registry).unwrap();
        fs::write(registry_path, format!("{raw}\n")).unwrap();
    }

    fn write_status_artifact(record: &SupervisorNodeRecord, status: &NodeStatus) {
        create_node_dirs(record);
        fs::write(
            &record.status_path,
            serde_json::to_string_pretty(status).unwrap(),
        )
        .unwrap();
    }

    fn write_metrics_artifact(record: &SupervisorNodeRecord, status: &NodeStatus) {
        let metrics = NodeMetrics::from_status(
            status,
            &NodeMetricArtifacts::from_record(record),
            NodeMetricCounts {
                uptime_ms: Some(100),
                starts_total: 1,
                stops_total: 0,
                state_transitions_total: 1,
            },
        );
        write_node_metrics_artifact(&record.metrics_path, &metrics).unwrap();
    }

    fn write_log_artifacts(record: &SupervisorNodeRecord) {
        create_node_dirs(record);
        fs::write(&record.stdout_log_path, "stdout\n").unwrap();
        fs::write(&record.stderr_log_path, "stderr\n").unwrap();
        fs::write(&record.events_log_path, "event=start status=ok\n").unwrap();
    }

    fn create_node_dirs(record: &SupervisorNodeRecord) {
        fs::create_dir_all(record.artifact_root.join("logs")).unwrap();
    }

    #[cfg(unix)]
    fn write_fixture_node(root: &Path) -> PathBuf {
        let path = root.join("fixture-ntpro-node.sh");
        fs::write(
            &path,
            r#"#!/bin/sh
set -eu
node_id=""
output=""
stop_file=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --run-id) node_id="$2"; shift 2 ;;
    --output) output="$2"; shift 2 ;;
    --stop-file) stop_file="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$output/logs"
echo "fixture stdout started node_id=$node_id"
echo "fixture stderr initialized node_id=$node_id" >&2
cat > "$output/logs/events.log" <<EOF
phase=start status=ok node_id=$node_id
EOF
cat > "$output/status.json" <<EOF
{
  "schema_version": "ntpro.node_status.v1",
  "node_id": "$node_id",
  "process_mode": "spawned_process",
  "config_path": {"availability": "available", "value": "fixture.toml"},
  "artifact_root": {"availability": "available", "value": "$output"},
  "lifecycle_state": "running",
  "previous_lifecycle_state": "starting",
  "data_connection": "not_configured",
  "execution_connection": "disconnected",
  "execution": {
    "gateway_id": {"availability": "available", "value": "SANDBOX"},
    "connection": "disconnected",
    "started": {"availability": "available", "value": true},
    "account_ref": {"availability": "available", "value": "configured"},
    "orders_open": {"availability": "unknown"},
    "orders_inflight": {"availability": "unknown"},
    "orders_closed": {"availability": "unknown"},
    "last_report_at": {"availability": "unknown"},
    "last_reconciliation_at": {"availability": "unknown"},
    "last_error": null
  },
  "risk": {
    "trading_state": "unknown",
    "health": "unknown",
    "command_count": {"availability": "unknown"},
    "event_count": {"availability": "unknown"},
    "rejections_total": {"availability": "unknown"},
    "last_rejection": null,
    "last_error": null
  },
  "generated_at": {"availability": "unknown"},
  "started_at": {"availability": "unknown"},
  "stopped_at": {"availability": "unknown"},
  "last_transition_at": {"availability": "unknown"},
  "last_error": null,
  "external_venue_connection": false,
  "real_orders_submitted": false
}
EOF
cat > "$output/metrics.json" <<EOF
{
  "schema_version": "ntpro.node_metrics.v1",
  "node_id": "$node_id",
  "lifecycle_state": "running",
  "previous_lifecycle_state": "starting",
  "process_mode": "spawned_process",
  "uptime_ms": {"availability": "available", "value": 0},
  "starts_total": 1,
  "stops_total": 0,
  "state_transitions_total": 1,
  "connection_counts": {
    "data_connected": 0,
    "data_disconnected": 0,
    "data_not_configured": 1,
    "execution_connected": 0,
    "execution_disconnected": 1,
    "execution_not_configured": 0
  },
  "last_error_summary": null,
  "generated_at": {"availability": "available", "value": "1"},
  "started_at": {"availability": "available", "value": "1"},
  "stopped_at": {"availability": "unknown"},
  "status_artifact_path": {"availability": "available", "value": "$output/status.json"},
  "stdout_log_path": {"availability": "available", "value": "$output/logs/stdout.log"},
  "stderr_log_path": {"availability": "available", "value": "$output/logs/stderr.log"},
  "events_log_path": {"availability": "available", "value": "$output/logs/events.log"},
  "external_venue_connection": false,
  "real_orders_submitted": false
}
EOF
while [ ! -f "$stop_file" ]; do
  sleep 0.05
done
cat >> "$output/logs/events.log" <<EOF
phase=stop status=ok node_id=$node_id
EOF
cat > "$output/status.json" <<EOF
{
  "schema_version": "ntpro.node_status.v1",
  "node_id": "$node_id",
  "process_mode": "spawned_process",
  "config_path": {"availability": "available", "value": "fixture.toml"},
  "artifact_root": {"availability": "available", "value": "$output"},
  "lifecycle_state": "stopped",
  "previous_lifecycle_state": "running",
  "data_connection": "not_configured",
  "execution_connection": "disconnected",
  "execution": {
    "gateway_id": {"availability": "available", "value": "SANDBOX"},
    "connection": "disconnected",
    "started": {"availability": "available", "value": false},
    "account_ref": {"availability": "available", "value": "configured"},
    "orders_open": {"availability": "unknown"},
    "orders_inflight": {"availability": "unknown"},
    "orders_closed": {"availability": "unknown"},
    "last_report_at": {"availability": "unknown"},
    "last_reconciliation_at": {"availability": "unknown"},
    "last_error": null
  },
  "risk": {
    "trading_state": "unknown",
    "health": "unknown",
    "command_count": {"availability": "unknown"},
    "event_count": {"availability": "unknown"},
    "rejections_total": {"availability": "unknown"},
    "last_rejection": null,
    "last_error": null
  },
  "generated_at": {"availability": "unknown"},
  "started_at": {"availability": "unknown"},
  "stopped_at": {"availability": "unknown"},
  "last_transition_at": {"availability": "unknown"},
  "last_error": null,
  "external_venue_connection": false,
  "real_orders_submitted": false
}
EOF
cat > "$output/metrics.json" <<EOF
{
  "schema_version": "ntpro.node_metrics.v1",
  "node_id": "$node_id",
  "lifecycle_state": "stopped",
  "previous_lifecycle_state": "running",
  "process_mode": "spawned_process",
  "uptime_ms": {"availability": "available", "value": 1},
  "starts_total": 1,
  "stops_total": 1,
  "state_transitions_total": 2,
  "connection_counts": {
    "data_connected": 0,
    "data_disconnected": 0,
    "data_not_configured": 1,
    "execution_connected": 0,
    "execution_disconnected": 1,
    "execution_not_configured": 0
  },
  "last_error_summary": null,
  "generated_at": {"availability": "available", "value": "2"},
  "started_at": {"availability": "available", "value": "1"},
  "stopped_at": {"availability": "available", "value": "2"},
  "status_artifact_path": {"availability": "available", "value": "$output/status.json"},
  "stdout_log_path": {"availability": "available", "value": "$output/logs/stdout.log"},
  "stderr_log_path": {"availability": "available", "value": "$output/logs/stderr.log"},
  "events_log_path": {"availability": "available", "value": "$output/logs/events.log"},
  "external_venue_connection": false,
  "real_orders_submitted": false
}
EOF
"#,
        )
        .unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        path
    }

    async fn http_request(addr: SocketAddr, method: &str, path: &str) -> String {
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        let request =
            format!("{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).await.unwrap();
        response
    }

    fn response_body(response: &str) -> &str {
        response
            .split_once("\r\n\r\n")
            .map_or("", |(_, body)| body.trim())
    }

    fn assert_forbidden_keys_absent(value: &Value) {
        match value {
            Value::Object(map) => {
                for key in map.keys() {
                    assert!(
                        !matches!(
                            key.as_str(),
                            "secret"
                                | "secrets"
                                | "credential"
                                | "credentials"
                                | "api_key"
                                | "token"
                                | "raw_order"
                                | "raw_orders"
                                | "raw_fill"
                                | "raw_fills"
                                | "raw_payload"
                                | "raw_venue_payload"
                                | "account_object"
                        ),
                        "forbidden dashboard key exposed: {key}"
                    );
                }
                for child in map.values() {
                    assert_forbidden_keys_absent(child);
                }
            }
            Value::Array(items) => {
                for child in items {
                    assert_forbidden_keys_absent(child);
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }
    }
}
