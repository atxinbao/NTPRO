use super::*;

#[test]
fn trader_terminal_v28_embedded_sources_project_required_fields() -> Result<(), String> {
    let contract: Value =
        serde_json::from_str(API_CONTRACT_SOURCE).map_err(|error| error.to_string())?;
    let cases = [
        (
            project_backend_closure_status(BACKEND_CLOSURE_SOURCE)
                .map_err(|error| format!("{error:?}"))?,
            "backend_closure_status",
        ),
        (
            project_provenance_drilldown(PROVENANCE_SOURCE)
                .map_err(|error| format!("{error:?}"))?,
            "provenance_drilldown",
        ),
        (
            project_audit_entries(AUDIT_SOURCE).map_err(|error| format!("{error:?}"))?,
            "audit_entries",
        ),
        (
            project_telemetry_health(TELEMETRY_SOURCE).map_err(|error| format!("{error:?}"))?,
            "telemetry_health",
        ),
        (
            project_permission_snapshot(PERMISSION_SOURCE).map_err(|error| format!("{error:?}"))?,
            "permission_snapshot",
        ),
        (
            project_deployment_state(DEPLOYMENT_SOURCE).map_err(|error| format!("{error:?}"))?,
            "deployment_state",
        ),
    ];

    for (response, contract_key) in cases {
        let Some(required_fields) =
            contract["api_contracts"][contract_key]["response_schema"]["required_fields"]
                .as_array()
        else {
            return Err(format!(
                "{contract_key} contract required_fields must be an array"
            ));
        };
        for field in required_fields {
            let Some(field) = field.as_str() else {
                return Err(format!(
                    "{contract_key} contract required field must be a string"
                ));
            };
            assert!(
                response.get(field).is_some(),
                "{contract_key} response missing required field {field}: {response}"
            );
        }
        assert_eq!(response["read_only"], true);
        assert_eq!(response["operation_controls_enabled"], false);
        assert_eq!(response["trading_controls_enabled"], false);
        assert_eq!(response["order_ticket_enabled"], false);
        assert_eq!(response["manual_operation_submit_allowed"], false);
    }
    Ok(())
}

#[test]
fn trader_terminal_v28_malformed_source_fails_closed() {
    let error = project_backend_closure_status("{not-json").unwrap_err();
    assert_eq!(error.kind, ProjectionErrorKind::Malformed);
}

#[test]
fn trader_terminal_v28_unredacted_source_fails_closed() -> Result<(), String> {
    let mut source: Value =
        serde_json::from_str(TELEMETRY_SOURCE).map_err(|error| error.to_string())?;
    source["payload_policy"]["redaction_status"] = json!("raw");
    let error = project_telemetry_health(&source.to_string()).unwrap_err();
    assert_eq!(error.kind, ProjectionErrorKind::Unredacted);
    assert_eq!(error.field, "/payload_policy/redaction_status");
    Ok(())
}

#[test]
fn trader_terminal_v28_forbidden_control_fails_closed() -> Result<(), String> {
    let mut source: Value =
        serde_json::from_str(PERMISSION_SOURCE).map_err(|error| error.to_string())?;
    source["required_false_permissions"]["submit_order"] = json!(true);
    let error = project_permission_snapshot(&source.to_string()).unwrap_err();
    assert_eq!(error.kind, ProjectionErrorKind::ForbiddenControls);
    assert_eq!(error.field, "/required_false_permissions/submit_order");
    Ok(())
}
