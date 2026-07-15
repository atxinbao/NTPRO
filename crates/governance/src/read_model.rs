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

use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result, anyhow, bail, ensure};
use jsonschema::Validator;
use serde_json::{Map, Value, json};

const REQUIRED_BOUNDARY_FLAGS: &[&str] = &[
    "new_submit_capability",
    "production_order_submission_allowed",
    "production_order_mutation_allowed",
    "dashboard_order_controls_enabled",
    "dashboard_approval_controls_enabled",
    "dashboard_cancel_controls_enabled",
    "dashboard_retry_controls_enabled",
    "retry_replace_amend_flatten_allowed",
    "product_grade_trading_terminal_claim",
];
const DASHBOARD_BOUNDARY_FLAGS: &[&str] = &[
    "dashboard_submit_controls_enabled",
    "dashboard_replace_controls_enabled",
    "dashboard_amend_controls_enabled",
    "dashboard_flatten_controls_enabled",
    "trader_terminal_order_ticket_enabled",
    "trader_terminal_live_trading_claim",
];
const REQUIRED_CATEGORIES: &[&str] = &[
    "source_provenance",
    "snapshot_redaction",
    "component_redaction",
    "capability_boundary",
    "lineage",
    "freshness",
    "component",
    "component_data",
];

type Snapshots = BTreeMap<String, (String, Value)>;

/// Validates the v0.21 read-model schema, all golden snapshots, and eight
/// fail-closed negative mutations.
///
/// # Errors
///
/// Returns an error when the schema is invalid, its boundary strategy drifts,
/// a read-model snapshot violates the schema, or a negative mutation passes.
pub fn validate_read_model_schema(schema_path: &Path, trace_glob: &str) -> Result<usize> {
    let schema = load_json_object(schema_path)?;
    jsonschema::draft202012::meta::validate(&schema)
        .map_err(|error| anyhow!("read-model schema is not valid Draft 2020-12: {error}"))?;
    validate_schema_strategy(&schema)?;
    let contract_version = schema
        .pointer("/properties/contract_version/const")
        .and_then(Value::as_str)
        .context("read-model schema must declare properties.contract_version.const")?;
    let validator = jsonschema::draft202012::new(&schema)
        .map_err(|error| anyhow!("failed to build Draft 2020-12 read-model validator: {error}"))?;
    let snapshots = collect_read_model_snapshots(trace_glob, contract_version)?;
    validate_all_snapshots(&validator, &snapshots)?;
    run_negative_mutations(&validator, &snapshots)?;
    Ok(snapshots.len())
}

fn load_json_object(path: &Path) -> Result<Value> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let value: Value =
        serde_json::from_str(&text).with_context(|| format!("{}: invalid JSON", path.display()))?;
    ensure!(
        value.is_object(),
        "{}: root must be a JSON object",
        path.display()
    );
    Ok(value)
}

fn collect_read_model_snapshots(trace_glob: &str, contract_version: &str) -> Result<Snapshots> {
    let mut snapshots = BTreeMap::new();
    let entries = glob::glob(trace_glob)
        .with_context(|| format!("invalid read-model trace glob: {trace_glob}"))?;
    for entry in entries {
        let path = entry.with_context(|| format!("failed to expand trace glob: {trace_glob}"))?;
        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        for (index, raw_line) in text.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let row: Value = serde_json::from_str(line)
                .with_context(|| format!("{}:{}: invalid JSON", path.display(), index + 1))?;
            let object = row.as_object().with_context(|| {
                format!("{}:{}: row must be an object", path.display(), index + 1)
            })?;
            if object.get("category").and_then(Value::as_str) != Some("read_model") {
                continue;
            }
            let case_id = object
                .get("case_id")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .with_context(|| {
                    format!("{}: read_model row has invalid case_id", path.display())
                })?;
            let events = object
                .get("input")
                .and_then(|value| value.get("events"))
                .and_then(Value::as_array)
                .with_context(|| {
                    format!(
                        "{}:{case_id}: input.events must be a non-empty array",
                        path.display()
                    )
                })?;
            ensure!(
                !events.is_empty(),
                "{}:{case_id}: input.events must be a non-empty array",
                path.display()
            );
            let Some(snapshot) = events[0].pointer("/payload/snapshot") else {
                continue;
            };
            ensure!(
                snapshot.is_object(),
                "{}:{case_id}: payload.snapshot must be an object",
                path.display()
            );
            if snapshot.get("contract_version").and_then(Value::as_str) != Some(contract_version) {
                continue;
            }
            ensure!(
                snapshots
                    .insert(
                        case_id.to_string(),
                        (path.display().to_string(), snapshot.clone())
                    )
                    .is_none(),
                "duplicate read_model case_id {case_id}"
            );
        }
    }
    ensure!(
        !snapshots.is_empty(),
        "no {contract_version} read_model snapshots found for {trace_glob}"
    );
    Ok(snapshots)
}

fn validate_schema_strategy(schema: &Value) -> Result<()> {
    ensure!(
        schema.get("additionalProperties") == Some(&Value::Bool(false)),
        "snapshot schema must fail closed on undeclared top-level fields"
    );
    let definitions = schema
        .get("$defs")
        .and_then(Value::as_object)
        .context("schema missing $defs")?;
    for name in REQUIRED_CATEGORIES {
        let definition = definitions
            .get(*name)
            .and_then(Value::as_object)
            .with_context(|| format!("schema missing $defs.{name}"))?;
        ensure!(
            definition.get("additionalProperties") == Some(&Value::Bool(false)),
            "$defs.{name} must set additionalProperties=false"
        );
    }

    let boundary = definitions["capability_boundary"]
        .as_object()
        .context("$defs.capability_boundary must be an object")?;
    let properties = boundary
        .get("properties")
        .and_then(Value::as_object)
        .context("$defs.capability_boundary.properties must be an object")?;
    for flag in REQUIRED_BOUNDARY_FLAGS
        .iter()
        .chain(DASHBOARD_BOUNDARY_FLAGS)
    {
        let definition = properties
            .get(*flag)
            .and_then(Value::as_object)
            .with_context(|| format!("capability boundary missing flag: {flag}"))?;
        ensure!(
            definition.get("const") == Some(&Value::Bool(false)),
            "{flag} must be constrained to false"
        );
    }

    let source = definitions["source_provenance"]
        .as_object()
        .context("$defs.source_provenance must be an object")?;
    let source_constraints = source
        .get("allOf")
        .and_then(Value::as_array)
        .filter(|constraints| !constraints.is_empty())
        .context("source_provenance must include exchange truth / adapter runtime constraints")?;
    ensure!(
        source_constraints.iter().all(Value::is_object),
        "source_provenance constraints must be objects"
    );
    Ok(())
}

fn validate_all_snapshots(validator: &Validator, snapshots: &Snapshots) -> Result<()> {
    let mut failures = Vec::new();
    for (case_id, (path, snapshot)) in snapshots {
        let errors = validation_errors(validator, snapshot);
        if !errors.is_empty() {
            failures.push(format!("{path}:{case_id}\n{}", errors.join("\n")));
        }
    }
    if !failures.is_empty() {
        bail!(
            "read_model JSON Schema validation failed:\n{}",
            failures.join("\n\n")
        );
    }
    Ok(())
}

fn validation_errors(validator: &Validator, instance: &Value) -> Vec<String> {
    let mut errors: Vec<_> = validator
        .iter_errors(instance)
        .map(|error| {
            let path = error.instance_path().to_string();
            let location = if path.is_empty() { "<root>" } else { &path };
            format!("{location}: {error}")
        })
        .collect();
    errors.sort();
    errors
}

fn expect_invalid(
    validator: &Validator,
    label: &str,
    snapshot: &Value,
    expected_marker: &str,
) -> Result<()> {
    let errors = validation_errors(validator, snapshot);
    ensure!(
        !errors.is_empty(),
        "negative schema mutation unexpectedly passed: {label}"
    );
    ensure!(
        errors.iter().any(|error| error.contains(expected_marker)),
        "negative schema mutation failed for an unexpected reason: {label}\nexpected marker: {expected_marker}\n{}",
        errors.join("\n")
    );
    Ok(())
}

fn run_negative_mutations(validator: &Validator, snapshots: &Snapshots) -> Result<()> {
    let mut account = snapshot(snapshots, "read_model.account_snapshot.fresh.001")?.clone();
    account["snapshot_kind"] = json!("unified_snapshot");
    account["health_status"] = json!("healthy");
    account["blocking_reasons"] = json!([]);
    expect_invalid(
        validator,
        "partial component snapshot masquerades as unified healthy",
        &account,
        "/components",
    )?;

    let mut undeclared_root =
        snapshot(snapshots, "read_model.contract.healthy_minimal.001")?.clone();
    object_mut(&mut undeclared_root, "<root>")?
        .insert("raw_exchange_response".to_string(), json!({"leak": true}));
    expect_invalid(
        validator,
        "undeclared top-level raw exchange response",
        &undeclared_root,
        "raw_exchange_response",
    )?;

    let mut sensitive_data = snapshot(snapshots, "read_model.account_snapshot.fresh.001")?.clone();
    object_at_mut(&mut sensitive_data, "/components/account/data")?
        .insert("api_secret".to_string(), json!("not-allowed"));
    expect_invalid(
        validator,
        "sensitive component data field",
        &sensitive_data,
        "/components/account/data",
    )?;

    let mut unauthorized =
        snapshot(snapshots, "read_model.dashboard.readonly_complete.001")?.clone();
    object_at_mut(&mut unauthorized, "/capability_boundary")?.insert(
        "dashboard_force_submit_enabled".to_string(),
        Value::Bool(false),
    );
    expect_invalid(
        validator,
        "undeclared dashboard boundary flag",
        &unauthorized,
        "/capability_boundary",
    )?;

    let mut missing_flag =
        snapshot(snapshots, "read_model.dashboard.readonly_complete.001")?.clone();
    object_at_mut(&mut missing_flag, "/capability_boundary")?
        .remove("dashboard_submit_controls_enabled");
    expect_invalid(
        validator,
        "dashboard submit flag omitted",
        &missing_flag,
        "/capability_boundary",
    )?;

    let mut exchange_truth =
        snapshot(snapshots, "read_model.contract.healthy_minimal.001")?.clone();
    let provenance = object_at_mut(&mut exchange_truth, "/source_provenance")?;
    provenance.insert("source_type".to_string(), json!("fixture"));
    provenance.insert("exchange_truth".to_string(), Value::Bool(true));
    provenance.insert("adapter_runtime_integrated".to_string(), Value::Bool(false));
    expect_invalid(
        validator,
        "fixture claims exchange truth",
        &exchange_truth,
        "/source_provenance/exchange_truth",
    )?;

    let mut adapter_runtime =
        snapshot(snapshots, "read_model.contract.healthy_minimal.001")?.clone();
    let provenance = object_at_mut(&mut adapter_runtime, "/source_provenance")?;
    provenance.insert("source_type".to_string(), json!("fixture"));
    provenance.insert("adapter_runtime_integrated".to_string(), Value::Bool(true));
    expect_invalid(
        validator,
        "fixture claims adapter runtime integration",
        &adapter_runtime,
        "/source_provenance/adapter_runtime_integrated",
    )?;

    let mut redaction = snapshot(snapshots, "read_model.contract.healthy_minimal.001")?.clone();
    object_at_mut(&mut redaction, "/redaction")?
        .insert("signed_url".to_string(), json!("not-allowed"));
    expect_invalid(
        validator,
        "redaction object undeclared signed URL",
        &redaction,
        "/redaction",
    )?;
    Ok(())
}

fn snapshot<'a>(snapshots: &'a Snapshots, case_id: &str) -> Result<&'a Value> {
    snapshots
        .get(case_id)
        .map(|(_, snapshot)| snapshot)
        .with_context(|| format!("missing required read-model mutation fixture: {case_id}"))
}

fn object_at_mut<'a>(value: &'a mut Value, pointer: &str) -> Result<&'a mut Map<String, Value>> {
    let target = value
        .pointer_mut(pointer)
        .with_context(|| format!("missing mutation target: {pointer}"))?;
    object_mut(target, pointer)
}

fn object_mut<'a>(value: &'a mut Value, path: &str) -> Result<&'a mut Map<String, Value>> {
    value
        .as_object_mut()
        .with_context(|| format!("mutation target must be an object: {path}"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn schema_strategy_fixture() -> Value {
        let mut properties = Map::new();
        for flag in REQUIRED_BOUNDARY_FLAGS
            .iter()
            .chain(DASHBOARD_BOUNDARY_FLAGS)
        {
            properties.insert((*flag).to_string(), json!({"const": false}));
        }
        let definitions: Map<_, _> = REQUIRED_CATEGORIES
            .iter()
            .map(|name| {
                let mut definition = json!({"additionalProperties": false});
                if *name == "capability_boundary" {
                    definition["properties"] = Value::Object(properties.clone());
                }
                if *name == "source_provenance" {
                    definition["allOf"] = json!([{"if": {}, "then": {}}]);
                }
                ((*name).to_string(), definition)
            })
            .collect();
        json!({
            "additionalProperties": false,
            "$defs": definitions,
        })
    }

    #[test]
    fn schema_strategy_rejects_boundary_capability_drift() {
        let mut schema = schema_strategy_fixture();
        schema["$defs"]["capability_boundary"]["properties"]["new_submit_capability"] =
            json!({"const": true});

        let error = validate_schema_strategy(&schema).unwrap_err().to_string();
        assert!(error.contains("new_submit_capability must be constrained to false"));
    }

    #[test]
    fn schema_strategy_rejects_empty_source_constraints() {
        let mut schema = schema_strategy_fixture();
        schema["$defs"]["source_provenance"]["allOf"] = json!([]);

        let error = validate_schema_strategy(&schema).unwrap_err().to_string();
        assert!(error.contains("source_provenance must include"));
    }

    #[test]
    fn collects_only_matching_contract_snapshots() {
        let directory = tempfile::tempdir().unwrap();
        let trace = directory.path().join("trace.jsonl");
        let rows = [
            json!({
                "category": "read_model",
                "case_id": "matching.001",
                "input": {"events": [{"payload": {"snapshot": {
                    "contract_version": "target.v1"
                }}}]}
            }),
            json!({
                "category": "read_model",
                "case_id": "other.001",
                "input": {"events": [{"payload": {"snapshot": {
                    "contract_version": "other.v1"
                }}}]}
            }),
            json!({
                "category": "read_model",
                "case_id": "event-only.001",
                "input": {"events": [{"payload": {"status": "ready"}}]}
            }),
        ];
        fs::write(
            &trace,
            rows.iter()
                .map(|row| serde_json::to_string(row).unwrap())
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .unwrap();

        let snapshots =
            collect_read_model_snapshots(&trace.to_string_lossy(), "target.v1").unwrap();
        assert_eq!(snapshots.len(), 1);
        assert!(snapshots.contains_key("matching.001"));
    }

    #[test]
    fn rejects_non_object_snapshot_payload() {
        let directory = tempfile::tempdir().unwrap();
        let trace = directory.path().join("trace.jsonl");
        fs::write(
            &trace,
            serde_json::to_string(&json!({
                "category": "read_model",
                "case_id": "malformed.001",
                "input": {"events": [{"payload": {"snapshot": "invalid"}}]}
            }))
            .unwrap(),
        )
        .unwrap();

        let error = collect_read_model_snapshots(&trace.to_string_lossy(), "target.v1")
            .unwrap_err()
            .to_string();
        assert!(error.contains("payload.snapshot must be an object"));
    }

    #[test]
    fn rejects_empty_read_model_event_arrays() {
        let directory = tempfile::tempdir().unwrap();
        let trace = directory.path().join("trace.jsonl");
        fs::write(
            &trace,
            serde_json::to_string(&json!({
                "category": "read_model",
                "case_id": "malformed.empty-events.001",
                "input": {"events": []}
            }))
            .unwrap(),
        )
        .unwrap();

        let error = collect_read_model_snapshots(&trace.to_string_lossy(), "target.v1")
            .unwrap_err()
            .to_string();
        assert!(error.contains("input.events must be a non-empty array"));
    }
}
