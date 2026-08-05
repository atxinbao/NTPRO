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

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use aws_lc_rs::{
    constant_time,
    rand::{SecureRandom, SystemRandom},
};
use axum::{
    Extension, Router,
    extract::{Path as AxumPath, Request, State},
    http::{
        HeaderMap, HeaderValue, Method, StatusCode, Uri,
        header::{CACHE_CONTROL, CONTENT_TYPE, COOKIE, LOCATION, REFERRER_POLICY, SET_COOKIE},
    },
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use tower_http::services::{ServeDir, ServeFile};

use crate::opt::{DashboardCommand, DashboardOpt, DashboardServeOpt};

use super::mvp_status_api::{mvp_event_correlation_api, mvp_shared_status_api};
use super::trader_terminal_api::{
    audit_entries_api, backend_closure_status_api, deployment_state_api, permission_snapshot_api,
    provenance_drilldown_api, telemetry_health_api,
};
use super::*;

#[cfg(test)]
mod tests;

const ACCESS_TOKEN_QUERY: &str = "access_token";
const INSTITUTION_ACCESS_COOKIE: &str = "ntpro_mvp_institution_access";
const OPERATOR_ACCESS_COOKIE: &str = "ntpro_mvp_operator_access";
const PORTAL_ACCESS_ERROR_SCHEMA_VERSION: &str = "ntpro.mvp_portal_access.error.v1";

#[derive(Clone)]
struct PortalAccess {
    institution_token: Arc<str>,
    operator_token: Arc<str>,
    enforced: bool,
}

#[derive(Clone, Copy)]
enum PortalRole {
    InstitutionUser,
    OperationsOperator,
    SharedRead,
}

impl PortalRole {
    const fn label(self) -> &'static str {
        match self {
            Self::InstitutionUser => "institution_user",
            Self::OperationsOperator => "operations_operator",
            Self::SharedRead => "institution_user_or_operations_operator",
        }
    }
}

impl PortalAccess {
    fn generate() -> anyhow::Result<Self> {
        Ok(Self {
            institution_token: generate_access_token("institution_user")?.into(),
            operator_token: generate_access_token("operations_operator")?.into(),
            enforced: true,
        })
    }

    #[cfg(test)]
    fn disabled_for_existing_tests() -> Self {
        Self {
            institution_token: Arc::from("test-institution-access"),
            operator_token: Arc::from("test-operator-access"),
            enforced: false,
        }
    }

    #[cfg(test)]
    fn enforced_for_test(institution_token: &str, operator_token: &str) -> Self {
        Self {
            institution_token: Arc::from(institution_token),
            operator_token: Arc::from(operator_token),
            enforced: true,
        }
    }

    fn token(&self, role: PortalRole) -> Option<&str> {
        match role {
            PortalRole::InstitutionUser => Some(&self.institution_token),
            PortalRole::OperationsOperator => Some(&self.operator_token),
            PortalRole::SharedRead => None,
        }
    }

    fn authorizes(&self, headers: &HeaderMap, role: PortalRole) -> bool {
        if !self.enforced {
            return true;
        }
        match role {
            PortalRole::InstitutionUser => {
                request_cookie_matches(headers, INSTITUTION_ACCESS_COOKIE, &self.institution_token)
            }
            PortalRole::OperationsOperator => {
                request_cookie_matches(headers, OPERATOR_ACCESS_COOKIE, &self.operator_token)
            }
            PortalRole::SharedRead => {
                request_cookie_matches(headers, INSTITUTION_ACCESS_COOKIE, &self.institution_token)
                    || request_cookie_matches(headers, OPERATOR_ACCESS_COOKIE, &self.operator_token)
            }
        }
    }
}

fn generate_access_token(role: &str) -> anyhow::Result<String> {
    let mut bytes = [0_u8; 32];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| anyhow::anyhow!("failed to generate {role} portal access token"))?;
    let mut token = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        token.push_str(&format!("{byte:02x}"));
    }
    Ok(token)
}

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

    validate_strategy_workbench_dist(&opt.strategy_workbench_dist)?;
    let access = PortalAccess::generate()?;
    let registry_path = opt.registry;
    let workflow_root = opt.workflow_root;
    let strategy_workbench_dist = opt.strategy_workbench_dist;
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
        "dashboard.serve status=ok bind={} registry={} workflow_root={} dashboard_url=http://{}/dashboard?access_token={} strategy_workbench_url=http://{}/strategy-workbench?access_token={} institution_workbench_url=http://{}/institution-workbench?access_token={} control_center_url=http://{}/control-center?access_token={} portal_access=local_bootstrap external_identity_provider=false",
        local_addr,
        registry_path.display(),
        workflow_root
            .as_ref()
            .map_or_else(|| "auto".to_string(), |path| path.display().to_string()),
        local_addr,
        access.operator_token,
        local_addr,
        access.institution_token,
        local_addr,
        access.institution_token,
        local_addr,
        access.operator_token,
    );
    axum::serve(
        listener,
        dashboard_router_with_workflow_root(
            registry_path,
            ntpro_node_bin,
            workflow_root,
            &strategy_workbench_dist,
            access,
        ),
    )
    .await
    .context("dashboard HTTP server exited with an error")?;
    Ok(())
}

#[cfg(test)]
pub(super) fn dashboard_router(registry_path: PathBuf, ntpro_node_bin: PathBuf) -> Router {
    dashboard_router_with_workflow_root(
        registry_path,
        ntpro_node_bin,
        None,
        &strategy_workbench_test_dist(),
        PortalAccess::disabled_for_existing_tests(),
    )
}

#[cfg(test)]
pub(super) fn dashboard_router_with_access(
    registry_path: PathBuf,
    ntpro_node_bin: PathBuf,
    institution_token: &str,
    operator_token: &str,
) -> Router {
    dashboard_router_with_workflow_root(
        registry_path,
        ntpro_node_bin,
        None,
        &strategy_workbench_test_dist(),
        PortalAccess::enforced_for_test(institution_token, operator_token),
    )
}

#[cfg(test)]
fn strategy_workbench_test_dist() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/strategy-workbench")
}

fn dashboard_router_with_workflow_root(
    registry_path: PathBuf,
    ntpro_node_bin: PathBuf,
    workflow_root: Option<PathBuf>,
    strategy_workbench_dist: &Path,
    access: PortalAccess,
) -> Router {
    let state = DashboardServerState {
        registry_path,
        workflow_root,
        ntpro_node_bin,
        lifecycle_action_lock: Arc::new(std::sync::Mutex::new(())),
    };
    let strategy_workbench_routes = strategy_workbench_routes(strategy_workbench_dist);
    let public_routes = Router::new()
        .route("/", get(dashboard_shell).head(reject_non_get))
        .route("/dashboard", get(dashboard_shell).head(reject_non_get))
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
        .route(
            "/control-center",
            get(control_center_shell).head(reject_non_get),
        )
        .route(
            "/assets/control-center.css",
            get(control_center_css).head(reject_non_get),
        )
        .route(
            "/assets/control-center.js",
            get(control_center_js).head(reject_non_get),
        )
        .merge(strategy_workbench_routes);
    let shared_read_routes = Router::new()
        .route(
            "/api/mvp/v1/status",
            get(mvp_shared_status_api).head(reject_non_get),
        )
        .route(
            "/api/mvp/v1/event-correlation",
            get(mvp_event_correlation_api).head(reject_non_get),
        )
        .route_layer(middleware::from_fn(require_shared_read_access));
    let operator_routes = Router::new()
        .route("/api/server", get(server_metadata_api).head(reject_non_get))
        .route("/api/snapshot", get(snapshot_api).head(reject_non_get))
        .route(
            "/api/mvp/v1/control-center",
            get(control_center_operational_api).head(reject_non_get),
        )
        .route(
            "/api/mvp/v1/control-center/nodes/{node_id}/actions/start",
            post(control_center_start_action_api),
        )
        .route(
            "/api/mvp/v1/control-center/nodes/{node_id}/actions/stop",
            post(control_center_stop_action_api),
        )
        .route(
            "/api/v28/backend-closure/status",
            get(backend_closure_status_api).head(reject_non_get),
        )
        .route(
            "/api/v28/provenance/drilldown",
            get(provenance_drilldown_api).head(reject_non_get),
        )
        .route(
            "/api/v28/audit/entries",
            get(audit_entries_api).head(reject_non_get),
        )
        .route(
            "/api/v28/telemetry/health",
            get(telemetry_health_api).head(reject_non_get),
        )
        .route(
            "/api/v28/permissions/snapshot",
            get(permission_snapshot_api).head(reject_non_get),
        )
        .route(
            "/api/v28/deployment/state",
            get(deployment_state_api).head(reject_non_get),
        )
        .route("/api/nodes", get(nodes_api).head(reject_non_get))
        .route(
            "/api/nodes/{node_id}",
            get(node_detail_api).head(reject_non_get),
        )
        .route(
            "/api/nodes/{node_id}/metrics",
            get(node_metrics_api).head(reject_non_get),
        )
        .route(
            "/api/nodes/{node_id}/logs",
            get(node_logs_api).head(reject_non_get),
        )
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
        .route_layer(middleware::from_fn(require_operator_access));
    Router::new()
        .merge(public_routes)
        .merge(shared_read_routes)
        .merge(operator_routes)
        .with_state(state)
        .layer(Extension(access))
        .layer(middleware::from_fn(reject_raw_event_store_paths))
}

async fn require_shared_read_access(
    Extension(access): Extension<PortalAccess>,
    request: Request,
    next: Next,
) -> Response {
    require_role_access(access, PortalRole::SharedRead, request, next).await
}

async fn require_operator_access(
    Extension(access): Extension<PortalAccess>,
    request: Request,
    next: Next,
) -> Response {
    require_role_access(access, PortalRole::OperationsOperator, request, next).await
}

async fn require_role_access(
    access: PortalAccess,
    role: PortalRole,
    request: Request,
    next: Next,
) -> Response {
    if access.authorizes(request.headers(), role) {
        let mut response = next.run(request).await;
        add_private_response_headers(response.headers_mut());
        response
    } else {
        portal_access_denied(role)
    }
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

pub(crate) fn validate_strategy_workbench_dist(dist_path: &Path) -> anyhow::Result<()> {
    ensure!(
        dist_path.is_dir(),
        "strategy workbench bundle directory '{}' does not exist; run 'npm ci && npm run build' in apps/strategy-workbench or pass --strategy-workbench-dist",
        dist_path.display(),
    );
    let index_path = dist_path.join("index.html");
    let index = fs::read_to_string(&index_path).with_context(|| {
        format!(
            "failed to read strategy workbench entrypoint '{}'",
            index_path.display()
        )
    })?;
    ensure!(
        index.contains("<div id=\"root\"></div>"),
        "strategy workbench entrypoint '{}' does not contain the React root",
        index_path.display(),
    );

    let assets_path = dist_path.join("assets");
    ensure!(
        assets_path.is_dir(),
        "strategy workbench assets directory '{}' does not exist",
        assets_path.display(),
    );
    let references = strategy_workbench_asset_references(&index);
    ensure!(
        !references.is_empty(),
        "strategy workbench entrypoint '{}' does not reference any assets under /strategy-workbench/assets/",
        index_path.display(),
    );
    let mut has_hashed_js = false;
    let mut has_hashed_css = false;
    for reference in references {
        let relative_path = Path::new(reference);
        ensure!(
            relative_path
                .components()
                .all(|component| matches!(component, std::path::Component::Normal(_))),
            "strategy workbench entrypoint '{}' contains invalid asset path '{reference}'",
            index_path.display(),
        );
        let asset_path = assets_path.join(relative_path);
        ensure!(
            asset_path.is_file(),
            "strategy workbench entrypoint '{}' references missing asset '{}'",
            index_path.display(),
            asset_path.display(),
        );

        let extension = relative_path.extension().and_then(|value| value.to_str());
        if let Some(extension @ ("js" | "css")) = extension {
            let file_name = relative_path
                .file_name()
                .and_then(|value| value.to_str())
                .with_context(|| format!("invalid strategy workbench asset '{reference}'"))?;
            ensure!(
                is_hashed_strategy_workbench_asset(file_name, extension),
                "strategy workbench entrypoint '{}' must reference only hashed .{} assets under /strategy-workbench/assets/",
                index_path.display(),
                extension,
            );
            has_hashed_js |= extension == "js";
            has_hashed_css |= extension == "css";
        }
    }
    ensure!(
        has_hashed_js && has_hashed_css,
        "strategy workbench entrypoint '{}' must reference hashed .js and .css assets under /strategy-workbench/assets/",
        index_path.display(),
    );
    Ok(())
}

fn strategy_workbench_asset_references(index: &str) -> Vec<&str> {
    const PREFIX: &str = "/strategy-workbench/assets/";
    let mut references = Vec::new();
    let mut remaining = index;
    while let Some(position) = remaining.find(PREFIX) {
        remaining = &remaining[position + PREFIX.len()..];
        let end = remaining
            .find(|character: char| {
                character.is_ascii_whitespace()
                    || matches!(character, '"' | '\'' | '<' | '>' | '?' | '#')
            })
            .unwrap_or(remaining.len());
        if end > 0 {
            references.push(&remaining[..end]);
        }
        remaining = &remaining[end..];
    }
    references
}

fn is_hashed_strategy_workbench_asset(file_name: &str, extension: &str) -> bool {
    let Some((_, hash_and_extension)) = file_name.split_once('-') else {
        return false;
    };
    let Some(hash) = hash_and_extension.strip_suffix(&format!(".{extension}")) else {
        return false;
    };
    hash.len() >= 6
        && hash
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn strategy_workbench_routes(dist_path: &Path) -> Router<DashboardServerState> {
    let index_path = dist_path.join("index.html");
    let static_routes = Router::new()
        .route_service("/", ServeFile::new(index_path.clone()))
        .nest_service("/assets", ServeDir::new(dist_path.join("assets")))
        .fallback_service(ServeFile::new(index_path));

    Router::new()
        .nest("/strategy-workbench", static_routes)
        .layer(middleware::from_fn(require_strategy_workbench_access))
}

async fn require_strategy_workbench_access(
    Extension(access): Extension<PortalAccess>,
    request: Request,
    next: Next,
) -> Response {
    if request.method() != Method::GET {
        let mut response = StatusCode::METHOD_NOT_ALLOWED.into_response();
        add_private_response_headers(response.headers_mut());
        return response;
    }

    let uri = request.uri().clone();
    let bootstrap_tokens = query_values(&uri, ACCESS_TOKEN_QUERY);
    if bootstrap_tokens.len() > 1 || bootstrap_tokens.first().is_some_and(String::is_empty) {
        return portal_access_denied(PortalRole::InstitutionUser);
    }
    if let Some(token) = bootstrap_tokens.first() {
        let expected = access
            .token(PortalRole::InstitutionUser)
            .expect("institution access token must exist");
        if !access_tokens_equal(expected, token) {
            return portal_access_denied(PortalRole::InstitutionUser);
        }
        return portal_bootstrap_redirect(PortalRole::InstitutionUser, token, &uri, uri.path());
    }
    if !access.authorizes(request.headers(), PortalRole::InstitutionUser) {
        return portal_access_denied(PortalRole::InstitutionUser);
    }

    let mut response = next.run(request).await;
    add_private_response_headers(response.headers_mut());
    response
}

async fn dashboard_shell(
    Extension(access): Extension<PortalAccess>,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    portal_shell_response(
        &access,
        PortalRole::OperationsOperator,
        &headers,
        &uri,
        "/dashboard",
        DASHBOARD_HTML,
    )
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

async fn institution_workbench_shell(
    Extension(access): Extension<PortalAccess>,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    portal_shell_response(
        &access,
        PortalRole::InstitutionUser,
        &headers,
        &uri,
        "/institution-workbench",
        INSTITUTION_WORKBENCH_HTML,
    )
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

async fn control_center_shell(
    Extension(access): Extension<PortalAccess>,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    portal_shell_response(
        &access,
        PortalRole::OperationsOperator,
        &headers,
        &uri,
        "/control-center",
        CONTROL_CENTER_HTML,
    )
}

async fn control_center_css() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "text/css; charset=utf-8")],
        CONTROL_CENTER_CSS,
    )
}

async fn control_center_js() -> impl IntoResponse {
    (
        [(CONTENT_TYPE, "application/javascript; charset=utf-8")],
        CONTROL_CENTER_JS,
    )
}

async fn reject_non_get() -> StatusCode {
    StatusCode::METHOD_NOT_ALLOWED
}

fn portal_shell_response(
    access: &PortalAccess,
    role: PortalRole,
    headers: &HeaderMap,
    uri: &Uri,
    canonical_path: &str,
    html: &'static str,
) -> Response {
    if !access.enforced {
        return Html(html).into_response();
    }

    let bootstrap_tokens = query_values(uri, ACCESS_TOKEN_QUERY);
    if bootstrap_tokens.len() > 1 || bootstrap_tokens.first().is_some_and(String::is_empty) {
        return portal_access_denied(role);
    }
    if let Some(token) = bootstrap_tokens.first() {
        let Some(expected) = access.token(role) else {
            return portal_access_denied(role);
        };
        if !access_tokens_equal(expected, token) {
            return portal_access_denied(role);
        }
        return portal_bootstrap_redirect(role, token, uri, canonical_path);
    }
    if !access.authorizes(headers, role) {
        return portal_access_denied(role);
    }

    let mut response = Html(html).into_response();
    add_private_response_headers(response.headers_mut());
    response
}

fn query_values(uri: &Uri, key: &str) -> Vec<String> {
    uri.query()
        .into_iter()
        .flat_map(|query| url::form_urlencoded::parse(query.as_bytes()))
        .filter_map(|(candidate, value)| (candidate == key).then(|| value.into_owned()))
        .collect()
}

fn portal_bootstrap_redirect(
    role: PortalRole,
    token: &str,
    uri: &Uri,
    canonical_path: &str,
) -> Response {
    let mut query = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in uri
        .query()
        .into_iter()
        .flat_map(|value| url::form_urlencoded::parse(value.as_bytes()))
    {
        if key != ACCESS_TOKEN_QUERY {
            query.append_pair(&key, &value);
        }
    }
    let query = query.finish();
    let location = if query.is_empty() {
        canonical_path.to_string()
    } else {
        format!("{canonical_path}?{query}")
    };
    let cookie_name = match role {
        PortalRole::InstitutionUser => INSTITUTION_ACCESS_COOKIE,
        PortalRole::OperationsOperator => OPERATOR_ACCESS_COOKIE,
        PortalRole::SharedRead => return portal_access_denied(role),
    };
    let cookie = format!("{cookie_name}={token}; HttpOnly; SameSite=Strict; Path=/");
    let Ok(location) = HeaderValue::from_str(&location) else {
        return portal_access_denied(role);
    };
    let Ok(cookie) = HeaderValue::from_str(&cookie) else {
        return portal_access_denied(role);
    };
    let mut response = StatusCode::SEE_OTHER.into_response();
    response.headers_mut().insert(LOCATION, location);
    response.headers_mut().insert(SET_COOKIE, cookie);
    add_private_response_headers(response.headers_mut());
    response
}

fn portal_access_denied(role: PortalRole) -> Response {
    let mut response = (
        StatusCode::FORBIDDEN,
        Json(json!({
            "schema_version": PORTAL_ACCESS_ERROR_SCHEMA_VERSION,
            "error_code": "portal_role_access_required",
            "required_role": role.label(),
            "read_only": true,
            "order_submission_allowed": false,
            "supervisor_actions_allowed": false,
            "external_venue_connection_allowed": false,
            "retry_allowed": false,
            "automatic_remediation_allowed": false,
        })),
    )
        .into_response();
    add_private_response_headers(response.headers_mut());
    response
}

fn add_private_response_headers(headers: &mut HeaderMap) {
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(REFERRER_POLICY, HeaderValue::from_static("no-referrer"));
}

fn request_cookie_matches(headers: &HeaderMap, name: &str, expected: &str) -> bool {
    headers.get_all(COOKIE).iter().any(|header| {
        header.to_str().is_ok_and(|cookies| {
            cookies.split(';').any(|cookie| {
                cookie
                    .trim()
                    .split_once('=')
                    .is_some_and(|(candidate, value)| {
                        candidate == name && access_tokens_equal(expected, value)
                    })
            })
        })
    })
}

fn access_tokens_equal(expected: &str, actual: &str) -> bool {
    constant_time::verify_slices_are_equal(expected.as_bytes(), actual.as_bytes()).is_ok()
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

async fn control_center_operational_api(
    State(state): State<DashboardServerState>,
) -> ApiResult<ControlCenterOperationalSnapshot> {
    let snapshot = load_dashboard_snapshot(&state)
        .map_err(|_| control_center_operational_error("snapshot_unavailable"))?;
    project_control_center_snapshot(&state.registry_path, snapshot)
        .map(Json)
        .map_err(control_center_operational_error)
}

fn control_center_operational_error(error_code: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({
            "schema_version": "ntpro.mvp_control_center_snapshot.error.v1",
            "error_code": error_code,
            "message": "控制中心运维投影不可用",
            "read_only": true,
            "supervisor_actions_exposed": false,
            "raw_errors_exposed": false,
        })),
    )
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

async fn control_center_start_action_api(
    State(state): State<DashboardServerState>,
    AxumPath(node_id): AxumPath<String>,
) -> (StatusCode, Json<ControlCenterLifecycleActionEnvelope>) {
    control_center_lifecycle_action_response(&state, &node_id, "start")
}

async fn control_center_stop_action_api(
    State(state): State<DashboardServerState>,
    AxumPath(node_id): AxumPath<String>,
) -> (StatusCode, Json<ControlCenterLifecycleActionEnvelope>) {
    control_center_lifecycle_action_response(&state, &node_id, "stop")
}

fn control_center_lifecycle_action_response(
    state: &DashboardServerState,
    node_id: &str,
    action: &str,
) -> (StatusCode, Json<ControlCenterLifecycleActionEnvelope>) {
    let Ok(_action_guard) = state.lifecycle_action_lock.lock() else {
        return control_center_lifecycle_action_error(
            StatusCode::SERVICE_UNAVAILABLE,
            node_id,
            action,
            "control_action_lock_unavailable",
            "本地控制动作串行锁不可用，动作未执行",
        );
    };
    let snapshot = match load_dashboard_snapshot(state) {
        Ok(snapshot) => snapshot,
        Err(_) => {
            return control_center_lifecycle_action_error(
                StatusCode::SERVICE_UNAVAILABLE,
                node_id,
                action,
                "lifecycle_snapshot_unavailable",
                "节点生命周期状态不可用，动作未执行",
            );
        }
    };
    match validate_control_center_action_scope(&snapshot, node_id) {
        Ok(_) => {}
        Err("action_target_node_mismatch") => {
            return control_center_lifecycle_action_error(
                StatusCode::NOT_FOUND,
                node_id,
                action,
                "node_not_found",
                "单节点 sandbox 合同中没有找到目标节点，动作未执行",
            );
        }
        Err(_) => {
            return control_center_lifecycle_action_error(
                StatusCode::SERVICE_UNAVAILABLE,
                node_id,
                action,
                "control_center_scope_violation",
                "单节点 sandbox 生命周期边界未通过，动作未执行",
            );
        }
    }

    let (status, Json(result)) =
        control_action_response_for_snapshot_locked(state, &snapshot, node_id, action);
    (
        status,
        Json(project_control_center_lifecycle_action(
            node_id, action, result,
        )),
    )
}

fn control_center_lifecycle_action_error(
    status: StatusCode,
    node_id: &str,
    action: &str,
    error_code: &str,
    message: &str,
) -> (StatusCode, Json<ControlCenterLifecycleActionEnvelope>) {
    let result = action_response(ControlActionResponseParts {
        action,
        node_id,
        status: ControlActionStatus::Failed,
        previous_state: LifecycleStatus::Unknown,
        current_state: LifecycleStatus::Unknown,
        started_at: generated_at_now(),
        error_code: DashboardValue::available(error_code.to_string()),
        message: DashboardValue::available(message.to_string()),
    });
    (
        status,
        Json(project_control_center_lifecycle_action(
            node_id, action, result,
        )),
    )
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
