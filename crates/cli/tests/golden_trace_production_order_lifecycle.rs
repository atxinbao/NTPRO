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

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

const TRACE_FILE: &str = "production_order_lifecycle_schema.jsonl";
const CONTRACT_VERSION: &str = "ntpro.v200_order_lifecycle_golden_fixture.v1";
const REQUIRED_REFS: &[&str] = &[
    "candidate_ref",
    "response_ref",
    "readback_ref",
    "failure_ref",
    "audit_ref",
    "dashboard_ref",
    "provenance_ref",
];
const FORBIDDEN_FALSE_FIELDS: &[&str] = &[
    "credential_plaintext_recorded",
    "raw_response_recorded",
    "raw_readback_body_recorded",
    "credential_material_recorded",
    "signature_material_recorded",
    "token_value_recorded",
    "signed_query_recorded",
    "signed_url_recorded",
    "retry_attempted",
    "duplicate_submit_attempted",
    "second_submit_attempted",
    "replace_attempted",
    "amend_attempted",
    "flatten_attempted",
    "automatic_cancel_attempted",
    "automatic_remediation_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_approval_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "dashboard_retry_controls_enabled",
    "network_replay_required",
    "live_broker_required",
];
const FORBIDDEN_RAW_TOKENS: &[&str] = &[
    "X-MBX-APIKEY",
    "apiKey",
    "api_key",
    "apiSecret",
    "api_secret",
    "secretKey",
    "signature=",
    "signedQuery=",
    "signedUrl=",
    "prod-secret",
    "real-secret",
    "actual-api-key",
    "sk_live",
];
const REQUIRED_SCENARIOS: &[ScenarioExpectation] = &[
    ScenarioExpectation {
        scenario: "pre_submit_blocked_missing_approval",
        outcome: "pre_submit_blocked",
        status: "v200_order_lifecycle_pre_submit_blocked",
        response_state: "not_attempted",
        readback_state: "not_required",
        failure_category: "approval_failed",
        audit_state: "audit_risk_visible",
        risk_visibility: "risk_visible",
    },
    ScenarioExpectation {
        scenario: "accepted_readback_matched_audit_closed",
        outcome: "accepted_readback_matched",
        status: "v200_order_lifecycle_audit_closed",
        response_state: "accepted",
        readback_state: "matched",
        failure_category: "none",
        audit_state: "audit_closed",
        risk_visibility: "clear",
    },
    ScenarioExpectation {
        scenario: "venue_rejected_failure_no_retry",
        outcome: "venue_rejected",
        status: "v200_order_lifecycle_failure_venue_rejected",
        response_state: "rejected",
        readback_state: "not_required",
        failure_category: "venue_rejected",
        audit_state: "audit_risk_visible",
        risk_visibility: "risk_visible",
    },
    ScenarioExpectation {
        scenario: "unknown_response_failure_no_retry",
        outcome: "response_unknown",
        status: "v200_order_lifecycle_failure_response_unknown",
        response_state: "unknown",
        readback_state: "unknown",
        failure_category: "response_unknown",
        audit_state: "audit_risk_visible",
        risk_visibility: "risk_visible",
    },
    ScenarioExpectation {
        scenario: "readback_mismatch_failure_no_retry",
        outcome: "readback_mismatch",
        status: "v200_order_lifecycle_failure_readback_mismatch",
        response_state: "accepted",
        readback_state: "mismatch",
        failure_category: "readback_mismatch",
        audit_state: "audit_risk_visible",
        risk_visibility: "risk_visible",
    },
    ScenarioExpectation {
        scenario: "readback_missing_failure_no_retry",
        outcome: "readback_missing",
        status: "v200_order_lifecycle_failure_readback_missing",
        response_state: "accepted",
        readback_state: "missing",
        failure_category: "readback_missing",
        audit_state: "audit_risk_visible",
        risk_visibility: "risk_visible",
    },
];

#[derive(Clone, Copy)]
struct ScenarioExpectation {
    scenario: &'static str,
    outcome: &'static str,
    status: &'static str,
    response_state: &'static str,
    readback_state: &'static str,
    failure_category: &'static str,
    audit_state: &'static str,
    risk_visibility: &'static str,
}

#[test]
fn rust_cli_production_order_lifecycle_golden_traces_cover_v200_required_paths()
-> Result<(), Box<dyn Error>> {
    let cases = load_cases(TRACE_FILE)?;
    if cases.len() != REQUIRED_SCENARIOS.len() {
        return Err(format!(
            "V200-011 must keep {} production order lifecycle scenarios, got {}",
            REQUIRED_SCENARIOS.len(),
            cases.len()
        )
        .into());
    }

    let by_scenario = cases_by_scenario(&cases)?;
    let mut response_states = BTreeSet::new();
    let mut readback_states = BTreeSet::new();
    let mut failure_categories = BTreeSet::new();
    let mut audit_states = BTreeSet::new();
    let mut risk_visibility = BTreeSet::new();

    for expected in REQUIRED_SCENARIOS {
        let case = by_scenario.get(expected.scenario).ok_or_else(|| {
            format!(
                "missing production order lifecycle golden trace scenario {}",
                expected.scenario
            )
        })?;
        let input_event = single_event(case, "input", expected.scenario)?;
        let expected_event = single_event(case, "expected", expected.scenario)?;
        let input_payload = payload(input_event)?;
        let expected_payload = payload(expected_event)?;

        assert_input_contract(input_payload, expected.scenario)?;
        assert_expected_fields(expected_payload, expected)?;
        assert_required_refs(input_payload, expected.scenario, "input")?;
        assert_required_refs(expected_payload, expected.scenario, "expected")?;
        assert_shared_refs(input_payload, expected_payload, expected.scenario)?;
        assert_false_boundary(expected_payload, expected.scenario)?;
        assert_dashboard_audit_boundary(expected_payload, expected.scenario)?;
        assert_no_forbidden_raw_tokens(case, expected.scenario)?;

        response_states.insert(expected.response_state);
        readback_states.insert(expected.readback_state);
        failure_categories.insert(expected.failure_category);
        audit_states.insert(expected.audit_state);
        risk_visibility.insert(expected.risk_visibility);
    }

    assert_contains_all(
        &response_states,
        &["accepted", "rejected", "unknown", "not_attempted"],
        "response_state",
    )?;
    assert_contains_all(
        &readback_states,
        &["matched", "mismatch", "missing", "unknown", "not_required"],
        "readback_state",
    )?;
    assert_contains_all(
        &failure_categories,
        &[
            "none",
            "approval_failed",
            "venue_rejected",
            "response_unknown",
            "readback_mismatch",
            "readback_missing",
        ],
        "failure_category",
    )?;
    assert_contains_all(
        &audit_states,
        &["audit_closed", "audit_risk_visible"],
        "audit_state",
    )?;
    assert_contains_all(
        &risk_visibility,
        &["clear", "risk_visible"],
        "risk_visibility",
    )?;

    Ok(())
}

fn assert_input_contract(payload: &Value, scenario: &str) -> Result<(), Box<dyn Error>> {
    if string_field(payload, "contract_version")? != CONTRACT_VERSION {
        return Err(format!("{scenario} must pin the V200-011 fixture contract").into());
    }
    if string_field(payload, "fixture_family")? != "production_order_lifecycle_raw_adapter" {
        return Err(format!("{scenario} must stay in the raw/adapter fixture family").into());
    }
    Ok(())
}

fn assert_expected_fields(
    payload: &Value,
    expected: &ScenarioExpectation,
) -> Result<(), Box<dyn Error>> {
    for (field, value) in [
        ("outcome", expected.outcome),
        ("status", expected.status),
        ("response_state", expected.response_state),
        ("readback_state", expected.readback_state),
        ("failure_category", expected.failure_category),
        ("audit_state", expected.audit_state),
        ("risk_visibility", expected.risk_visibility),
    ] {
        if string_field(payload, field)? != value {
            return Err(
                format!("{} {field} must stay stable as {value}", expected.scenario).into(),
            );
        }
    }

    if !bool_field(payload, "no_implicit_retry")? {
        return Err(format!("{} must prove no implicit retry", expected.scenario).into());
    }
    if bool_field(payload, "retry_allowed")? {
        return Err(format!("{} must keep retry_allowed=false", expected.scenario).into());
    }
    if u64_field(payload, "retry_attempts")? != 0 {
        return Err(format!("{} must keep retry_attempts=0", expected.scenario).into());
    }

    if expected.failure_category == "none" {
        if bool_field(payload, "failure_recorded")? {
            return Err(format!("{} must not record failure evidence", expected.scenario).into());
        }
        if string_field(payload, "terminal_action")? != "audit_closed_no_action" {
            return Err(format!(
                "{} must close without terminal failure action",
                expected.scenario
            )
            .into());
        }
    } else {
        if !bool_field(payload, "failure_recorded")? {
            return Err(format!("{} must record failure evidence", expected.scenario).into());
        }
        if string_field(payload, "terminal_action")? != "write_evidence_and_stop" {
            return Err(format!("{} must write evidence and stop", expected.scenario).into());
        }
    }

    Ok(())
}

fn assert_dashboard_audit_boundary(payload: &Value, scenario: &str) -> Result<(), Box<dyn Error>> {
    for field in [
        "dashboard_audit_recorded",
        "dashboard_read_only_consumable",
        "release_gate_consumable",
    ] {
        if !bool_field(payload, field)? {
            return Err(format!("{scenario} must keep {field}=true").into());
        }
    }
    Ok(())
}

fn cases_by_scenario(cases: &[Value]) -> Result<BTreeMap<String, &Value>, Box<dyn Error>> {
    let mut by_scenario = BTreeMap::new();
    for case in cases {
        let input_event = single_event(case, "input", "unknown")?;
        let scenario = string_field(payload(input_event)?, "scenario")?.to_string();
        if by_scenario.insert(scenario.clone(), case).is_some() {
            return Err(format!("duplicate production order lifecycle scenario {scenario}").into());
        }
    }
    Ok(by_scenario)
}

fn single_event<'a>(
    case: &'a Value,
    section: &str,
    scenario: &str,
) -> Result<&'a Value, Box<dyn Error>> {
    let events = case
        .get(section)
        .and_then(|value| value.get("events"))
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{scenario} {section}.events must be an array"))?;
    if events.len() != 1 {
        return Err(format!("{scenario} {section}.events must contain one event").into());
    }
    Ok(&events[0])
}

fn payload(event: &Value) -> Result<&Value, Box<dyn Error>> {
    event
        .get("payload")
        .and_then(Value::as_object)
        .map(|_| &event["payload"])
        .ok_or_else(|| "event payload must be an object".into())
}

fn assert_required_refs(
    payload: &Value,
    scenario: &str,
    section: &str,
) -> Result<(), Box<dyn Error>> {
    let refs = object_field(payload, "refs")?;
    for key in REQUIRED_REFS {
        let value = string_field(refs, key)?;
        if value.trim().is_empty() {
            return Err(format!("{scenario} {section}.{key} must be non-empty").into());
        }
        if !value.starts_with("production_order_lifecycle_") {
            return Err(format!(
                "{scenario} {section}.{key} must be a production order lifecycle ref"
            )
            .into());
        }
    }
    Ok(())
}

fn assert_shared_refs(
    input_payload: &Value,
    expected_payload: &Value,
    scenario: &str,
) -> Result<(), Box<dyn Error>> {
    let input_refs = object_field(input_payload, "refs")?;
    let expected_refs = object_field(expected_payload, "refs")?;
    for key in REQUIRED_REFS {
        if input_refs.get(*key) != expected_refs.get(*key) {
            return Err(format!("{scenario} expected ref {key} must match input ref").into());
        }
    }
    Ok(())
}

fn assert_false_boundary(payload: &Value, scenario: &str) -> Result<(), Box<dyn Error>> {
    for field in FORBIDDEN_FALSE_FIELDS {
        if bool_field(payload, field)? {
            return Err(format!("{scenario} must keep {field}=false").into());
        }
    }
    Ok(())
}

fn assert_no_forbidden_raw_tokens(case: &Value, scenario: &str) -> Result<(), Box<dyn Error>> {
    let body = serde_json::to_string(case)?;
    for token in FORBIDDEN_RAW_TOKENS {
        if body.contains(token) {
            return Err(format!("{scenario} leaked forbidden raw token {token}").into());
        }
    }
    Ok(())
}

fn assert_contains_all(
    seen: &BTreeSet<&str>,
    required: &[&str],
    field: &str,
) -> Result<(), Box<dyn Error>> {
    for value in required {
        if !seen.contains(value) {
            return Err(
                format!("missing required production order lifecycle {field}={value}").into(),
            );
        }
    }
    Ok(())
}

fn object_field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_object)
        .map(|_| &value[key])
        .ok_or_else(|| format!("{key} must be an object").into())
}

fn string_field<'a>(value: &'a Value, key: &str) -> Result<&'a str, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{key} must be a string").into())
}

fn bool_field(value: &Value, key: &str) -> Result<bool, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("{key} must be a bool").into())
}

fn u64_field(value: &Value, key: &str) -> Result<u64, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{key} must be an unsigned integer").into())
}

fn load_cases(file_name: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    let trace = repository_root().join("tests/golden").join(file_name);
    fs::read_to_string(&trace)?
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
        .map(|line| Ok(serde_json::from_str::<Value>(line)?))
        .collect()
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|err| panic!("repository root should resolve from crates/cli: {err}"))
}
