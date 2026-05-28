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
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

const SCHEMA_VERSION: &str = "golden-trace-v1";
const REQUIRED_ROW_FIELDS: &[&str] = &[
    "schema_version",
    "case_id",
    "category",
    "description",
    "input",
    "expected",
    "tolerances",
];
const REQUIRED_EVENT_FIELDS: &[&str] = &["event_type", "ts_event", "payload"];
const VALID_CATEGORIES: &[&str] = &[
    "market_data",
    "order_lifecycle",
    "risk",
    "execution",
    "position",
    "portfolio_pnl",
    "cache_msgbus",
    "backtest_live",
    "adapter_payload",
];

#[test]
fn golden_trace_jsonl_files_match_rust_schema_contract() {
    let trace_dir = repository_root().join("tests/golden");
    let traces = golden_trace_files(&trace_dir);

    assert!(
        !traces.is_empty(),
        "no golden trace files found in {}",
        trace_dir.display()
    );

    for trace in traces {
        let rows = load_jsonl(&trace);
        let errors = validate_rows(&rows);
        assert!(
            errors.is_empty(),
            "{} failed Rust golden trace validation:\n{}",
            trace.display(),
            errors.join("\n")
        );
    }
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root should resolve from crates/testkit")
}

fn golden_trace_files(trace_dir: &Path) -> Vec<PathBuf> {
    let mut traces = fs::read_dir(trace_dir)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", trace_dir.display()))
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.extension().and_then(|ext| ext.to_str()) == Some("jsonl")).then_some(path)
        })
        .collect::<Vec<_>>();
    traces.sort();
    traces
}

fn load_jsonl(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
        .lines()
        .enumerate()
        .filter_map(|(line_index, line)| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }

            Some(serde_json::from_str::<Value>(line).unwrap_or_else(|err| {
                panic!("{}:{} invalid JSON: {err}", path.display(), line_index + 1)
            }))
        })
        .collect()
}

fn validate_rows(rows: &[Value]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut seen_case_ids = HashSet::new();

    if rows.is_empty() {
        errors.push("trace must contain at least one row".to_string());
        return errors;
    }

    for (index, row) in rows.iter().enumerate() {
        let path = format!("row {index}");
        let Some(object) = row.as_object() else {
            errors.push(format!("{path}: row must be an object"));
            continue;
        };

        for key in REQUIRED_ROW_FIELDS {
            if !object.contains_key(*key) {
                errors.push(format!("{path}: missing {key}"));
            }
        }

        if row.get("schema_version") != Some(&Value::String(SCHEMA_VERSION.to_string())) {
            errors.push(format!("{path}.schema_version: expected {SCHEMA_VERSION}"));
        }

        match row.get("case_id").and_then(Value::as_str) {
            Some(case_id) if !case_id.is_empty() && seen_case_ids.insert(case_id.to_string()) => {}
            Some(case_id) if case_id.is_empty() => {
                errors.push(format!("{path}.case_id: must be a non-empty string"));
            }
            Some(case_id) => {
                errors.push(format!("{path}.case_id: duplicate case_id {case_id}"));
            }
            None => {
                errors.push(format!("{path}.case_id: must be a non-empty string"));
            }
        }

        match row.get("category").and_then(Value::as_str) {
            Some(category) if VALID_CATEGORIES.contains(&category) => {}
            _ => errors.push(format!(
                "{path}.category: expected one of {}",
                VALID_CATEGORIES.join(", ")
            )),
        }

        match row.get("description").and_then(Value::as_str) {
            Some(description) if !description.is_empty() => {}
            _ => errors.push(format!("{path}.description: must be a non-empty string")),
        }

        validate_event_section(row.get("input"), &format!("{path}.input"), &mut errors);
        validate_event_section(
            row.get("expected"),
            &format!("{path}.expected"),
            &mut errors,
        );

        if !row.get("tolerances").is_some_and(Value::is_object) {
            errors.push(format!("{path}.tolerances: must be an object"));
        }
    }

    errors
}

fn validate_event_section(section: Option<&Value>, path: &str, errors: &mut Vec<String>) {
    let Some(section) = section else {
        errors.push(format!("{path}: must be an object"));
        return;
    };
    if !section.is_object() {
        errors.push(format!("{path}: must be an object"));
        return;
    }

    let Some(events) = section.get("events").and_then(Value::as_array) else {
        errors.push(format!("{path}.events: must be an array"));
        return;
    };

    for (index, event) in events.iter().enumerate() {
        validate_event(event, &format!("{path}.events[{index}]"), errors);
    }
}

fn validate_event(event: &Value, path: &str, errors: &mut Vec<String>) {
    let Some(object) = event.as_object() else {
        errors.push(format!("{path}: event must be an object"));
        return;
    };

    for key in REQUIRED_EVENT_FIELDS {
        if !object.contains_key(*key) {
            errors.push(format!("{path}: missing event field {key}"));
        }
    }

    match event.get("event_type").and_then(Value::as_str) {
        Some(event_type) if !event_type.is_empty() => {}
        _ => errors.push(format!("{path}.event_type: must be a non-empty string")),
    }

    for key in ["ts_event", "ts_init"] {
        if let Some(timestamp) = event.get(key)
            && !is_timestamp(timestamp)
        {
            errors.push(format!(
                "{path}.{key}: must be an integer or decimal string nanosecond timestamp"
            ));
        }
    }

    if !event.get("payload").is_some_and(Value::is_object) {
        errors.push(format!("{path}.payload: must be an object"));
    }
}

fn is_timestamp(value: &Value) -> bool {
    value.as_u64().is_some()
        || value
            .as_str()
            .is_some_and(|value| value.chars().all(|ch| ch.is_ascii_digit()))
}
