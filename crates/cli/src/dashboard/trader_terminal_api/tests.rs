use super::*;

#[test]
fn trader_terminal_v28_embedded_sources_project_required_fields() {
    let contract_result = serde_json::from_str::<Value>(API_CONTRACT_SOURCE);
    assert!(
        contract_result.is_ok(),
        "v0.28 API contract source must be valid JSON: {:?}",
        contract_result.as_ref().err()
    );
    let Ok(contract) = contract_result else {
        return;
    };
    let cases = [
        (
            project_backend_closure_status(BACKEND_CLOSURE_SOURCE),
            "backend_closure_status",
        ),
        (
            project_provenance_drilldown(PROVENANCE_SOURCE),
            "provenance_drilldown",
        ),
        (project_audit_entries(AUDIT_SOURCE), "audit_entries"),
        (
            project_telemetry_health(TELEMETRY_SOURCE),
            "telemetry_health",
        ),
        (
            project_permission_snapshot(PERMISSION_SOURCE),
            "permission_snapshot",
        ),
        (
            project_deployment_state(DEPLOYMENT_SOURCE),
            "deployment_state",
        ),
    ];

    for (response_result, contract_key) in cases {
        assert!(
            response_result.is_ok(),
            "{contract_key} embedded source projection must succeed: {:?}",
            response_result.as_ref().err()
        );
        let Ok(response) = response_result else {
            continue;
        };

        let required_fields =
            contract["api_contracts"][contract_key]["response_schema"]["required_fields"]
                .as_array();
        assert!(
            required_fields.is_some(),
            "{contract_key} contract required_fields must be an array"
        );
        let Some(required_fields) = required_fields else {
            continue;
        };
        for field in required_fields {
            let field_name = field.as_str();
            assert!(
                field_name.is_some(),
                "{contract_key} contract required field must be a string"
            );
            let Some(field) = field_name else {
                continue;
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
}

#[test]
fn trader_terminal_v28_malformed_source_fails_closed() {
    let error_result = project_backend_closure_status("{not-json");
    assert!(error_result.is_err(), "malformed source must fail closed");
    let Err(error) = error_result else {
        return;
    };
    assert_eq!(error.kind, ProjectionErrorKind::Malformed);
}

#[test]
fn trader_terminal_v28_unredacted_source_fails_closed() {
    let source_result = serde_json::from_str::<Value>(TELEMETRY_SOURCE);
    assert!(
        source_result.is_ok(),
        "telemetry source must be valid JSON: {:?}",
        source_result.as_ref().err()
    );
    let Ok(mut source) = source_result else {
        return;
    };
    source["payload_policy"]["redaction_status"] = json!("raw");
    let error_result = project_telemetry_health(&source.to_string());
    assert!(
        error_result.is_err(),
        "unredacted telemetry source must fail closed"
    );
    let Err(error) = error_result else {
        return;
    };
    assert_eq!(error.kind, ProjectionErrorKind::Unredacted);
    assert_eq!(error.field, "/payload_policy/redaction_status");
}

#[test]
fn trader_terminal_v28_forbidden_control_fails_closed() {
    let source_result = serde_json::from_str::<Value>(PERMISSION_SOURCE);
    assert!(
        source_result.is_ok(),
        "permission source must be valid JSON: {:?}",
        source_result.as_ref().err()
    );
    let Ok(mut source) = source_result else {
        return;
    };
    source["required_false_permissions"]["submit_order"] = json!(true);
    let error_result = project_permission_snapshot(&source.to_string());
    assert!(
        error_result.is_err(),
        "forbidden operation control must fail closed"
    );
    let Err(error) = error_result else {
        return;
    };
    assert_eq!(error.kind, ProjectionErrorKind::ForbiddenControls);
    assert_eq!(error.field, "/required_false_permissions/submit_order");
}
