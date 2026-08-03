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

//! Loopback dashboard HTTP transport and route wiring.
//!
//! This module owns server startup, static asset responses, and request
//! adaptation. Snapshot projection and fail-closed control decisions remain in
//! the parent `dashboard` module.

use axum::{
    Router,
    extract::{Path as AxumPath, Request, State},
    http::{StatusCode, header::CONTENT_TYPE},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};

use crate::opt::{DashboardCommand, DashboardOpt, DashboardServeOpt};

use super::mvp_status_api::mvp_shared_status_api;
use super::trader_terminal_api::{
    audit_entries_api, backend_closure_status_api, deployment_state_api, permission_snapshot_api,
    provenance_drilldown_api, telemetry_health_api,
};
use super::*;

#[cfg(test)]
mod tests;

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
        "dashboard server is local-only; use a loopback bind address"
    );

    let registry_path = opt.registry;
    let workflow_root = opt.workflow_root;
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
        "dashboard.serve status=ok bind={} registry={} workflow_root={} dashboard_url=http://{}/dashboard institution_workbench_url=http://{}/institution-workbench",
        local_addr,
        registry_path.display(),
        workflow_root
            .as_ref()
            .map_or_else(|| "auto".to_string(), |path| path.display().to_string()),
        local_addr,
        local_addr
    );
    axum::serve(
        listener,
        dashboard_router_with_workflow_root(registry_path, ntpro_node_bin, workflow_root),
    )
    .await
    .context("dashboard HTTP server exited with an error")?;
    Ok(())
}

#[cfg(test)]
pub(super) fn dashboard_router(registry_path: PathBuf, ntpro_node_bin: PathBuf) -> Router {
    dashboard_router_with_workflow_root(registry_path, ntpro_node_bin, None)
}

fn dashboard_router_with_workflow_root(
    registry_path: PathBuf,
    ntpro_node_bin: PathBuf,
    workflow_root: Option<PathBuf>,
) -> Router {
    let state = DashboardServerState {
        registry_path,
        workflow_root,
        ntpro_node_bin,
    };
    Router::new()
        .route("/", get(dashboard_shell))
        .route("/dashboard", get(dashboard_shell))
        .route("/assets/dashboard.css", get(dashboard_css))
        .route("/assets/dashboard.js", get(dashboard_js))
        .route(
            "/institution-workbench",
            get(institution_workbench_shell).head(reject_non_get),
        )
        .route(
            "/assets/institution-workbench.css",
            get(institution_workbench_css).head(reject_non_get),
        )
        .route(
            "/assets/institution-workbench.js",
            get(institution_workbench_js).head(reject_non_get),
        )
        .route("/api/server", get(server_metadata_api))
        .route("/api/snapshot", get(snapshot_api))
        .route(
            "/api/mvp/v1/status",
            get(mvp_shared_status_api).head(reject_non_get),
        )
        .route(
            "/api/v28/backend-closure/status",
            get(backend_closure_status_api),
        )
        .route(
            "/api/v28/provenance/drilldown",
            get(provenance_drilldown_api),
        )
        .route("/api/v28/audit/entries", get(audit_entries_api))
        .route("/api/v28/telemetry/health", get(telemetry_health_api))
        .route(
            "/api/v28/permissions/snapshot",
            get(permission_snapshot_api),
        )
        .route("/api/v28/deployment/state", get(deployment_state_api))
        .route("/api/nodes", get(nodes_api))
        .route("/api/nodes/{node_id}", get(node_detail_api))
        .route("/api/nodes/{node_id}/metrics", get(node_metrics_api))
        .route("/api/nodes/{node_id}/logs", get(node_logs_api))
        .route("/api/nodes/{node_id}/actions/start", post(start_action_api))
        .route("/api/nodes/{node_id}/actions/stop", post(stop_action_api))
        .route("/api/nodes/{node_id}/actions/pause", post(pause_action_api))
        .route(
            "/api/nodes/{node_id}/actions/resume",
            post(resume_action_api),
        )
        .route(
            "/api/nodes/{node_id}/actions/reconnect_data",
            post(reconnect_data_action_api),
        )
        .route(
            "/api/nodes/{node_id}/actions/reconnect_execution",
            post(reconnect_execution_action_api),
        )
        .with_state(state)
        .layer(middleware::from_fn(reject_raw_event_store_paths))
}

async fn reject_raw_event_store_paths(request: Request, next: Next) -> Response {
    if is_forbidden_raw_event_store_path(request.uri().path()) {
        return StatusCode::NOT_FOUND.into_response();
    }
    next.run(request).await
}

fn is_forbidden_raw_event_store_path(path: &str) -> bool {
    let normalized = path.to_ascii_lowercase();
    let segments = normalized
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    matches!(
        segments.as_slice(),
        ["api", "event-store" | "event_store" | "redb", ..]
            | ["event-store" | "event_store" | "redb", ..]
    ) || matches!(
        segments.as_slice(),
        ["api", "runs", _, "events" | "raw-events" | "raw_events", ..]
    ) || matches!(segments.as_slice(), [file] if file.ends_with(".redb"))
        || matches!(
            segments.as_slice(),
            ["downloads", path @ ..] if path.iter().any(|segment| segment.ends_with(".redb"))
        )
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

async fn institution_workbench_shell() -> Html<&'static str> {
    Html(INSTITUTION_WORKBENCH_HTML)
}

async fn institution_workbench_css() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "text/css; charset=utf-8")],
        INSTITUTION_WORKBENCH_CSS,
    )
}

async fn institution_workbench_js() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "application/javascript; charset=utf-8")],
        INSTITUTION_WORKBENCH_JS,
    )
}

async fn reject_non_get() -> StatusCode {
    StatusCode::METHOD_NOT_ALLOWED
}

async fn server_metadata_api(
    State(state): State<DashboardServerState>,
) -> Json<DashboardServerMetadata> {
    Json(DashboardServerMetadata {
        registry_path: state.registry_path.display().to_string(),
        workflow_root: state
            .workflow_root
            .as_ref()
            .map(|path| path.display().to_string()),
        local_only: true,
    })
}

async fn snapshot_api(State(state): State<DashboardServerState>) -> ApiResult<DashboardSnapshot> {
    load_dashboard_snapshot(&state).map(Json)
}

async fn nodes_api(
    State(state): State<DashboardServerState>,
) -> ApiResult<Vec<DashboardNodeSummary>> {
    let snapshot = load_dashboard_snapshot(&state)?;
    Ok(Json(snapshot.nodes))
}

async fn node_detail_api(
    State(state): State<DashboardServerState>,
    AxumPath(node_id): AxumPath<String>,
) -> ApiResult<DashboardNodeSummary> {
    let snapshot = load_dashboard_snapshot(&state)?;
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
    let snapshot = load_dashboard_snapshot(&state)?;
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
    let snapshot = load_dashboard_snapshot(&state)?;
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

async fn pause_action_api(
    State(state): State<DashboardServerState>,
    AxumPath(node_id): AxumPath<String>,
) -> ApiStatusResult<ControlActionResponse> {
    control_action_response(&state, &node_id, "pause")
}

async fn resume_action_api(
    State(state): State<DashboardServerState>,
    AxumPath(node_id): AxumPath<String>,
) -> ApiStatusResult<ControlActionResponse> {
    control_action_response(&state, &node_id, "resume")
}

async fn reconnect_data_action_api(
    State(state): State<DashboardServerState>,
    AxumPath(node_id): AxumPath<String>,
) -> ApiStatusResult<ControlActionResponse> {
    control_action_response(&state, &node_id, "reconnect_data")
}

async fn reconnect_execution_action_api(
    State(state): State<DashboardServerState>,
    AxumPath(node_id): AxumPath<String>,
) -> ApiStatusResult<ControlActionResponse> {
    control_action_response(&state, &node_id, "reconnect_execution")
}
