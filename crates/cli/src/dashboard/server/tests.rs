use std::{net::SocketAddr, path::PathBuf};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
    middleware,
    response::Response,
    routing::any,
};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tower::ServiceExt;

use super::{
    INSTITUTION_ACCESS_COOKIE, OPERATOR_ACCESS_COOKIE, PORTAL_ACCESS_ERROR_SCHEMA_VERSION,
    dashboard_router, dashboard_router_with_access, reject_raw_event_store_paths,
};

const INSTITUTION_TOKEN: &str = "test-institution-access-token";
const OPERATOR_TOKEN: &str = "test-operator-access-token";

#[tokio::test]
async fn trader_terminal_v28_http_routes_serve_read_only_contracts() {
    let listener_result = tokio::net::TcpListener::bind("127.0.0.1:0").await;
    assert!(
        listener_result.is_ok(),
        "test listener must bind: {:?}",
        listener_result.as_ref().err()
    );
    let Ok(listener) = listener_result else {
        return;
    };
    let addr_result = listener.local_addr();
    assert!(
        addr_result.is_ok(),
        "test listener must expose its local address: {:?}",
        addr_result.as_ref().err()
    );
    let Ok(addr) = addr_result else {
        return;
    };
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            dashboard_router(
                PathBuf::from("missing-v28-registry.json"),
                PathBuf::from("missing-ntpro-node"),
            ),
        )
        .await
    });

    for (path, schema_version) in [
        (
            "/api/v28/backend-closure/status",
            "ntpro.v280.trader_terminal.backend_closure_status.response.v1",
        ),
        (
            "/api/v28/provenance/drilldown",
            "ntpro.v280.trader_terminal.provenance_drilldown.response.v1",
        ),
        (
            "/api/v28/audit/entries",
            "ntpro.v280.trader_terminal.audit_entries.response.v1",
        ),
        (
            "/api/v28/telemetry/health",
            "ntpro.v280.trader_terminal.telemetry_health.response.v1",
        ),
        (
            "/api/v28/permissions/snapshot",
            "ntpro.v280.trader_terminal.permission_snapshot.response.v1",
        ),
        (
            "/api/v28/deployment/state",
            "ntpro.v280.trader_terminal.deployment_state.response.v1",
        ),
    ] {
        let response_result = http_request(addr, "GET", path).await;
        assert!(
            response_result.is_ok(),
            "GET {path} must complete: {:?}",
            response_result.as_ref().err()
        );
        let Ok(response) = response_result else {
            continue;
        };
        assert_http_ok(&response, path);
        let value_result = serde_json::from_str::<Value>(response_body(&response));
        assert!(
            value_result.is_ok(),
            "{path} must return valid JSON: {:?}",
            value_result.as_ref().err()
        );
        let Ok(value) = value_result else {
            continue;
        };
        assert_eq!(value["schema_version"], schema_version, "{path}");
        assert_eq!(
            value["contract_version"], "ntpro.v280.trader_terminal_backend_api_contract_handoff.v1",
            "{path}"
        );
        assert_eq!(value["read_only"], true, "{path}");
        assert_eq!(value["operation_controls_enabled"], false, "{path}");
        assert_eq!(value["trading_controls_enabled"], false, "{path}");
        assert_eq!(value["order_ticket_enabled"], false, "{path}");
        assert_eq!(value["manual_operation_submit_allowed"], false, "{path}");
        assert_forbidden_keys_absent(&value);

        for method in ["POST", "PUT", "PATCH", "DELETE"] {
            let response_result = http_request(addr, method, path).await;
            assert!(
                response_result.is_ok(),
                "{method} {path} must complete: {:?}",
                response_result.as_ref().err()
            );
            let Ok(response) = response_result else {
                continue;
            };
            assert!(
                response.contains("HTTP/1.1 405 Method Not Allowed"),
                "{method} {path} must be rejected, got:\n{response}"
            );
        }
    }

    for path in [
        "/api/event-store",
        "/api/event-store/entries",
        "/api/event_store/raw",
        "/api/REDB/status",
        "/api/runs/example/events",
        "/api/runs/example/raw-events",
        "/event_store/trader-001/run.redb",
        "/downloads/run.REDB",
    ] {
        for method in ["GET", "POST", "PUT", "PATCH", "DELETE"] {
            let response_result = http_request(addr, method, path).await;
            assert!(
                response_result.is_ok(),
                "{method} {path} must complete: {:?}",
                response_result.as_ref().err()
            );
            let Ok(response) = response_result else {
                continue;
            };
            assert_eq!(
                response_status_line(&response),
                "HTTP/1.1 404 Not Found",
                "raw Event Store path {method} {path} must remain unavailable"
            );
        }
    }

    server.abort();
}

#[tokio::test]
async fn mvp_shared_status_route_is_get_only() {
    let root =
        std::env::temp_dir().join(format!("ntpro-mvp-005-http-method-{}", std::process::id()));
    let router = dashboard_router(
        root.join("supervisor/registry.json"),
        PathBuf::from("missing-ntpro-node"),
    );

    let (status, body) = router_request(&router, Method::GET, "/api/mvp/v1/status").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    let body: Value =
        serde_json::from_slice(&body).expect("GET error response should be valid JSON");
    assert_eq!(
        body["schema_version"],
        "ntpro.mvp_shared_status_api.error.v1"
    );
    assert_eq!(body["order_submission_allowed"], false);

    for method in [
        Method::HEAD,
        Method::POST,
        Method::PUT,
        Method::PATCH,
        Method::DELETE,
        Method::OPTIONS,
        Method::CONNECT,
        Method::TRACE,
    ] {
        let (status, _) = router_request(&router, method.clone(), "/api/mvp/v1/status").await;
        assert_eq!(
            status,
            StatusCode::METHOD_NOT_ALLOWED,
            "{method} must be rejected"
        );
    }
}

#[tokio::test]
async fn mvp_event_correlation_route_is_get_only() {
    let root = std::env::temp_dir().join(format!(
        "ntpro-mvp-008-event-correlation-http-method-{}",
        std::process::id()
    ));
    let router = dashboard_router(
        root.join("supervisor/registry.json"),
        PathBuf::from("missing-ntpro-node"),
    );
    let path = "/api/mvp/v1/event-correlation";

    let (status, body) = router_request(&router, Method::GET, path).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    let body: Value =
        serde_json::from_slice(&body).expect("GET error response should be valid JSON");
    assert_eq!(
        body["schema_version"],
        "ntpro.mvp_shared_status_api.error.v1"
    );
    assert_eq!(body["order_submission_allowed"], false);

    for method in [
        Method::HEAD,
        Method::POST,
        Method::PUT,
        Method::PATCH,
        Method::DELETE,
        Method::OPTIONS,
        Method::CONNECT,
        Method::TRACE,
    ] {
        let (status, _) = router_request(&router, method.clone(), path).await;
        assert_eq!(
            status,
            StatusCode::METHOD_NOT_ALLOWED,
            "{method} {path} must be rejected"
        );
    }
}

#[tokio::test]
async fn institution_workbench_route_serves_read_only_shell_and_assets() {
    let root = std::env::temp_dir().join(format!(
        "ntpro-mvp-006-institution-workbench-{}",
        std::process::id()
    ));
    let router = dashboard_router(
        root.join("supervisor/registry.json"),
        PathBuf::from("missing-ntpro-node"),
    );

    for (path, marker) in [
        ("/institution-workbench", "<title>NTPRO 机构工作台</title>"),
        (
            "/assets/institution-workbench.css",
            ".app-shell { display: grid;",
        ),
        (
            "/assets/institution-workbench.js",
            "const SHARED_STATUS_URL = \"/api/mvp/v1/status\";",
        ),
    ] {
        let (status, body) = router_request(&router, Method::GET, path).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        assert!(
            String::from_utf8_lossy(&body).contains(marker),
            "{path} missing {marker}"
        );
        for method in [
            Method::HEAD,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
            Method::CONNECT,
            Method::TRACE,
        ] {
            let (status, _) = router_request(&router, method.clone(), path).await;
            assert_eq!(
                status,
                StatusCode::METHOD_NOT_ALLOWED,
                "{method} {path} must be rejected",
            );
        }
    }
}

#[tokio::test]
async fn control_center_route_serves_read_only_shell_and_assets() {
    let root = std::env::temp_dir().join(format!(
        "ntpro-mvp-007-control-center-{}",
        std::process::id()
    ));
    let router = dashboard_router(
        root.join("supervisor/registry.json"),
        PathBuf::from("missing-ntpro-node"),
    );

    for (path, marker) in [
        ("/control-center", "<title>NTPRO 控制中心</title>"),
        ("/assets/control-center.css", ".app-shell { display: grid;"),
        (
            "/assets/control-center.js",
            "const SHARED_STATUS_URL = \"/api/mvp/v1/status\";",
        ),
    ] {
        let (status, body) = router_request(&router, Method::GET, path).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        assert!(
            String::from_utf8_lossy(&body).contains(marker),
            "{path} missing {marker}"
        );
        for method in [
            Method::HEAD,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
            Method::CONNECT,
            Method::TRACE,
        ] {
            let (status, _) = router_request(&router, method.clone(), path).await;
            assert_eq!(
                status,
                StatusCode::METHOD_NOT_ALLOWED,
                "{method} {path} must be rejected",
            );
        }
    }

    let path = "/api/mvp/v1/control-center";
    let (status, body) = router_request(&router, Method::GET, path).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(!String::from_utf8_lossy(&body).contains("missing-ntpro-node"));
    for method in [
        Method::HEAD,
        Method::POST,
        Method::PUT,
        Method::PATCH,
        Method::DELETE,
        Method::OPTIONS,
        Method::CONNECT,
        Method::TRACE,
    ] {
        let (status, _) = router_request(&router, method.clone(), path).await;
        assert_eq!(
            status,
            StatusCode::METHOD_NOT_ALLOWED,
            "{method} {path} must be rejected",
        );
    }
}

#[tokio::test]
async fn portal_access_bootstrap_redirects_to_clean_url_and_sets_private_cookie() {
    let router = dashboard_router_with_access(
        PathBuf::from("missing-mvp-role-registry.json"),
        PathBuf::from("missing-ntpro-node"),
        INSTITUTION_TOKEN,
        OPERATOR_TOKEN,
    );

    let response = router_response(
        &router,
        Method::GET,
        &format!("/institution-workbench?event_id=event%3A1&access_token={INSTITUTION_TOKEN}"),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        response.headers()[header::LOCATION],
        "/institution-workbench?event_id=event%3A1"
    );
    let cookie = response.headers()[header::SET_COOKIE].to_str().ok();
    assert_eq!(
        cookie,
        Some(
            format!(
                "{INSTITUTION_ACCESS_COOKIE}={INSTITUTION_TOKEN}; HttpOnly; SameSite=Strict; Path=/"
            )
            .as_str()
        )
    );
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(response.headers()[header::REFERRER_POLICY], "no-referrer");

    let response = router_response(
        &router,
        Method::GET,
        &format!("/control-center?access_token={OPERATOR_TOKEN}"),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers()[header::LOCATION], "/control-center");
    assert!(
        response.headers()[header::SET_COOKIE]
            .to_str()
            .is_ok_and(|cookie| cookie.starts_with(OPERATOR_ACCESS_COOKIE))
    );
}

#[tokio::test]
async fn portal_access_rejects_missing_wrong_empty_and_duplicate_bootstrap_credentials() {
    let router = dashboard_router_with_access(
        PathBuf::from("missing-mvp-role-registry.json"),
        PathBuf::from("missing-ntpro-node"),
        INSTITUTION_TOKEN,
        OPERATOR_TOKEN,
    );
    let institution_cookie = format!("{INSTITUTION_ACCESS_COOKIE}={INSTITUTION_TOKEN}");

    for (path, cookie) in [
        ("/institution-workbench", None),
        ("/institution-workbench?access_token=", None),
        ("/institution-workbench?access_token=forged", None),
        (
            "/institution-workbench?access_token=forged&access_token=forged",
            None,
        ),
        (
            "/institution-workbench?access_token=forged",
            Some(institution_cookie.as_str()),
        ),
        (
            "/institution-workbench?access_token=test-institution-access-token&access_token=test-institution-access-token",
            Some(institution_cookie.as_str()),
        ),
        (
            &format!("/institution-workbench?access_token={OPERATOR_TOKEN}"),
            None,
        ),
    ] {
        let response = router_response(&router, Method::GET, path, cookie).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{path}");
        let body_result = to_bytes(response.into_body(), 2 * 1024 * 1024).await;
        assert!(
            body_result.is_ok(),
            "{path} access error body must be readable"
        );
        let body = body_result.unwrap_or_default();
        let value_result = serde_json::from_slice::<Value>(&body);
        assert!(
            value_result.is_ok(),
            "{path} access error must be valid JSON"
        );
        let Ok(value) = value_result else {
            continue;
        };
        assert_eq!(value["schema_version"], PORTAL_ACCESS_ERROR_SCHEMA_VERSION);
        assert_eq!(value["error_code"], "portal_role_access_required");
        assert_eq!(value["order_submission_allowed"], false);
        assert_eq!(value["supervisor_actions_allowed"], false);
        assert!(!String::from_utf8_lossy(&body).contains(INSTITUTION_TOKEN));
        assert!(!String::from_utf8_lossy(&body).contains(OPERATOR_TOKEN));
    }
}

#[tokio::test]
async fn portal_access_enforces_server_side_role_matrix_without_api_bypass() {
    let router = dashboard_router_with_access(
        PathBuf::from("missing-mvp-role-registry.json"),
        PathBuf::from("missing-ntpro-node"),
        INSTITUTION_TOKEN,
        OPERATOR_TOKEN,
    );
    let institution_cookie = format!("{INSTITUTION_ACCESS_COOKIE}={INSTITUTION_TOKEN}");
    let operator_cookie = format!("{OPERATOR_ACCESS_COOKIE}={OPERATOR_TOKEN}");

    for (method, path) in [
        (Method::GET, "/institution-workbench"),
        (Method::GET, "/control-center"),
        (Method::GET, "/dashboard"),
        (Method::GET, "/api/mvp/v1/status"),
        (Method::GET, "/api/mvp/v1/event-correlation"),
        (Method::GET, "/api/mvp/v1/control-center"),
        (Method::GET, "/api/server"),
        (Method::GET, "/api/snapshot"),
        (Method::GET, "/api/v28/telemetry/health"),
        (Method::GET, "/api/nodes"),
        (Method::POST, "/api/nodes/mvp-node-001/actions/start"),
    ] {
        let response = router_response(&router, method, path, None).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{path}");
    }

    for path in ["/institution-workbench", "/assets/institution-workbench.js"] {
        let response = router_response(&router, Method::GET, path, Some(&institution_cookie)).await;
        assert_eq!(response.status(), StatusCode::OK, "{path}");
    }
    for path in ["/api/mvp/v1/status", "/api/mvp/v1/event-correlation"] {
        let response = router_response(&router, Method::GET, path, Some(&institution_cookie)).await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE, "{path}");
    }
    for (method, path) in [
        (Method::GET, "/control-center"),
        (Method::GET, "/dashboard"),
        (Method::GET, "/api/mvp/v1/control-center"),
        (Method::GET, "/api/server"),
        (Method::GET, "/api/snapshot"),
        (Method::GET, "/api/v28/telemetry/health"),
        (Method::GET, "/api/nodes"),
        (Method::POST, "/api/nodes/mvp-node-001/actions/start"),
    ] {
        let response = router_response(&router, method, path, Some(&institution_cookie)).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "{path}");
    }

    for path in ["/control-center", "/dashboard"] {
        let response = router_response(&router, Method::GET, path, Some(&operator_cookie)).await;
        assert_eq!(response.status(), StatusCode::OK, "{path}");
    }
    for path in [
        "/api/mvp/v1/status",
        "/api/mvp/v1/event-correlation",
        "/api/mvp/v1/control-center",
        "/api/server",
        "/api/snapshot",
        "/api/v28/telemetry/health",
        "/api/nodes",
    ] {
        let response = router_response(&router, Method::GET, path, Some(&operator_cookie)).await;
        assert_ne!(response.status(), StatusCode::FORBIDDEN, "{path}");
    }
    let response = router_response(
        &router,
        Method::GET,
        "/institution-workbench",
        Some(&operator_cookie),
    )
    .await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let both_cookies = format!("{institution_cookie}; {operator_cookie}");
    for path in ["/institution-workbench", "/control-center"] {
        let response = router_response(&router, Method::GET, path, Some(&both_cookies)).await;
        assert_eq!(response.status(), StatusCode::OK, "{path}");
    }

    for (path, cookie) in [
        ("/institution-workbench", institution_cookie.as_str()),
        ("/api/mvp/v1/status", institution_cookie.as_str()),
        ("/api/mvp/v1/event-correlation", institution_cookie.as_str()),
        ("/control-center", operator_cookie.as_str()),
        ("/dashboard", operator_cookie.as_str()),
        ("/api/mvp/v1/control-center", operator_cookie.as_str()),
        ("/api/server", operator_cookie.as_str()),
        ("/api/snapshot", operator_cookie.as_str()),
        ("/api/v28/telemetry/health", operator_cookie.as_str()),
        ("/api/nodes", operator_cookie.as_str()),
    ] {
        let response = router_response(&router, Method::HEAD, path, Some(cookie)).await;
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED, "{path}");
    }
}

async fn router_request(router: &Router, method: Method, path: &str) -> (StatusCode, Vec<u8>) {
    let response = router_response(router, method, path, None).await;
    let status = response.status();
    let body = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .expect("router response body should be readable")
        .to_vec();
    (status, body)
}

async fn router_response(
    router: &Router,
    method: Method,
    path: &str,
    cookie: Option<&str>,
) -> Response {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(cookie) = cookie {
        request = request.header(header::COOKIE, cookie);
    }
    router
        .clone()
        .oneshot(
            request
                .body(Body::empty())
                .expect("router request should build"),
        )
        .await
        .expect("router request should complete")
}

#[tokio::test]
async fn raw_event_store_guard_blocks_registered_routes_without_hiding_node_ids() {
    let listener_result = tokio::net::TcpListener::bind("127.0.0.1:0").await;
    assert!(
        listener_result.is_ok(),
        "test listener must bind: {:?}",
        listener_result.as_ref().err()
    );
    let Ok(listener) = listener_result else {
        return;
    };
    let addr_result = listener.local_addr();
    assert!(
        addr_result.is_ok(),
        "test listener must expose its local address: {:?}",
        addr_result.as_ref().err()
    );
    let Ok(addr) = addr_result else {
        return;
    };
    let router = Router::new()
        .route("/api/event-store/probe", any(|| async { StatusCode::OK }))
        .route("/api/nodes/redb", any(|| async { StatusCode::OK }))
        .route("/api/nodes/run.redb", any(|| async { StatusCode::OK }))
        .layer(middleware::from_fn(reject_raw_event_store_paths));
    let server = tokio::spawn(async move { axum::serve(listener, router).await });

    for method in ["GET", "POST", "PUT", "PATCH", "DELETE"] {
        let blocked_result = http_request(addr, method, "/api/event-store/probe").await;
        assert!(
            blocked_result.is_ok(),
            "{method} forbidden probe must complete: {:?}",
            blocked_result.as_ref().err()
        );
        let Ok(blocked) = blocked_result else {
            continue;
        };
        assert_eq!(
            response_status_line(&blocked),
            "HTTP/1.1 404 Not Found",
            "{method} registered raw Event Store route must be blocked"
        );

        for allowed_path in ["/api/nodes/redb", "/api/nodes/run.redb"] {
            let allowed_result = http_request(addr, method, allowed_path).await;
            assert!(
                allowed_result.is_ok(),
                "{method} {allowed_path} must complete: {:?}",
                allowed_result.as_ref().err()
            );
            let Ok(allowed) = allowed_result else {
                continue;
            };
            assert_eq!(
                response_status_line(&allowed),
                "HTTP/1.1 200 OK",
                "guard must not interpret a node ID as a storage namespace"
            );
        }
    }

    server.abort();
}

async fn http_request(addr: SocketAddr, method: &str, path: &str) -> std::io::Result<String> {
    let mut stream = tokio::net::TcpStream::connect(addr).await?;
    let request = format!("{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await?;
    let mut response = String::new();
    stream.read_to_string(&mut response).await?;
    Ok(response)
}

fn assert_http_ok(response: &str, context: &str) {
    assert_eq!(
        response_status_line(response),
        "HTTP/1.1 200 OK",
        "{context} expected HTTP 200 OK"
    );
}

fn response_status_line(response: &str) -> &str {
    response.lines().next().map_or("", str::trim)
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
                    "forbidden Trader Terminal key exposed: {key}"
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
