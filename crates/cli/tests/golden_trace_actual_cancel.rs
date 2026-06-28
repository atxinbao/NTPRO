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

const TRACE_FILE: &str = "actual_cancel_schema.jsonl";
const CONTRACT_VERSION: &str = "ntpro.v190_actual_cancel_golden_fixture.v1";
const REQUIRED_REFS: &[&str] = &[
    "request_ref",
    "response_ref",
    "readback_ref",
    "audit_ref",
    "provenance_ref",
];
const REQUIRED_SCENARIOS: &[(&str, &str, &str)] = &[
    (
        "success",
        "cancel_confirmed",
        "ready_actual_cancel_failure_recovered_cancel_confirmed",
    ),
    (
        "approval_missing",
        "approval_missing",
        "blocked_source_artifact",
    ),
    (
        "approval_reused",
        "approval_reused",
        "blocked_source_artifact",
    ),
    ("risk_mismatch", "risk_mismatch", "blocked_source_artifact"),
    (
        "adapter_unsupported",
        "adapter_unsupported",
        "blocked_source_artifact",
    ),
    (
        "cancel_rejected",
        "rejected",
        "ready_actual_cancel_failure_rejected",
    ),
    ("timeout", "timeout", "ready_actual_cancel_failure_timeout"),
    ("unknown", "unknown", "ready_actual_cancel_failure_unknown"),
    (
        "already_cancelled",
        "already_cancelled",
        "ready_actual_cancel_failure_recovered_already_cancelled",
    ),
    (
        "partial_fill",
        "partial_fill",
        "ready_actual_cancel_partial_success_partial_fill",
    ),
];
const FORBIDDEN_RAW_TOKENS: &[&str] = &[
    "123456789",
    "owner-approved-v160-single-shot",
    "X-MBX-APIKEY",
    "apiSecret",
    "signature=",
    "signedQuery=",
    "signedUrl=",
];

#[test]
fn rust_cli_actual_cancel_golden_traces_cover_required_outcomes() -> Result<(), Box<dyn Error>> {
    let cases = load_cases(TRACE_FILE)?;
    if cases.len() != REQUIRED_SCENARIOS.len() {
        return Err(format!(
            "V190-009 must keep {} actual-cancel scenarios, got {}",
            REQUIRED_SCENARIOS.len(),
            cases.len()
        )
        .into());
    }

    let by_scenario = cases_by_scenario(&cases)?;
    let mut seen_outcomes = BTreeSet::new();

    for (scenario, outcome, status) in REQUIRED_SCENARIOS {
        let case = by_scenario
            .get(*scenario)
            .ok_or_else(|| format!("missing actual-cancel golden trace scenario {scenario}"))?;
        let input_event = single_event(case, "input", scenario)?;
        let expected_event = single_event(case, "expected", scenario)?;
        let input_payload = payload(input_event)?;
        let expected_payload = payload(expected_event)?;

        if string_field(input_payload, "contract_version")? != CONTRACT_VERSION {
            return Err(format!("{scenario} must pin the V190-009 fixture contract").into());
        }
        if string_field(expected_payload, "outcome")? != *outcome {
            return Err(format!("{scenario} outcome must stay stable as {outcome}").into());
        }
        if string_field(expected_payload, "status")? != *status {
            return Err(format!("{scenario} status must stay stable as {status}").into());
        }
        assert_required_refs(input_payload, scenario, "input")?;
        assert_required_refs(expected_payload, scenario, "expected")?;
        assert_shared_refs(input_payload, expected_payload, scenario)?;
        assert_false_boundary(expected_payload, scenario)?;
        assert_no_forbidden_raw_tokens(case, scenario)?;
        seen_outcomes.insert((*outcome).to_string());
    }

    for required in [
        "cancel_confirmed",
        "approval_missing",
        "approval_reused",
        "risk_mismatch",
        "adapter_unsupported",
        "rejected",
        "timeout",
        "unknown",
        "already_cancelled",
        "partial_fill",
    ] {
        if !seen_outcomes.contains(required) {
            return Err(format!("missing required actual-cancel outcome {required}").into());
        }
    }

    let partial = payload(single_event(
        by_scenario
            .get("partial_fill")
            .ok_or("missing partial_fill scenario")?,
        "expected",
        "partial_fill",
    )?)?;
    assert_decimal_strings(partial, "partial_fill")?;

    Ok(())
}

fn cases_by_scenario(cases: &[Value]) -> Result<BTreeMap<String, &Value>, Box<dyn Error>> {
    let mut by_scenario = BTreeMap::new();
    for case in cases {
        let input_event = single_event(case, "input", "unknown")?;
        let scenario = string_field(payload(input_event)?, "scenario")?.to_string();
        if by_scenario.insert(scenario.clone(), case).is_some() {
            return Err(format!("duplicate actual-cancel scenario {scenario}").into());
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
        if !value.starts_with("actual_cancel_") {
            return Err(format!("{scenario} {section}.{key} must be an actual-cancel ref").into());
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
    for field in [
        "retry_attempted",
        "second_cancel_attempted",
        "remediation_attempted",
        "dashboard_cancel_controls_enabled",
    ] {
        if bool_field(payload, field)? {
            return Err(format!("{scenario} must keep {field}=false").into());
        }
    }
    Ok(())
}

fn assert_decimal_strings(payload: &Value, scenario: &str) -> Result<(), Box<dyn Error>> {
    let fields = object_field(payload, "quantity_fields")?;
    for key in ["orig_qty", "executed_qty", "remaining_qty"] {
        let value = string_field(fields, key)?;
        if !value.chars().all(|ch| ch.is_ascii_digit() || ch == '.') {
            return Err(format!("{scenario} {key} must stay a decimal string").into());
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
