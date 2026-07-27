use std::{net::SocketAddr, path::PathBuf};

use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::dashboard_router;

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
        "/api/event-store/raw",
        "/api/redb",
        "/api/runs/example/events",
        "/event_store/trader-001/run.redb",
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
        assert!(
            response.contains("HTTP/1.1 404 Not Found"),
            "raw Event Store path {path} must remain unmounted, got:\n{response}"
        );
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
    assert!(
        response.contains("HTTP/1.1 200 OK"),
        "{context} expected HTTP 200 OK, got:\n{response}"
    );
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
