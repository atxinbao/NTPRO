use std::{net::SocketAddr, path::PathBuf};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
    middleware,
    routing::any,
};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tower::ServiceExt;

use super::{dashboard_router, reject_raw_event_store_paths};

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

async fn router_request(router: &Router, method: Method, path: &str) -> (StatusCode, Vec<u8>) {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .body(Body::empty())
                .expect("router request should build"),
        )
        .await
        .expect("router request should complete");
    let status = response.status();
    let body = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .expect("router response body should be readable")
        .to_vec();
    (status, body)
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
