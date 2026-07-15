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
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use serde_json::{Map, Value};

const SCHEMA_VERSION: &str = "golden-trace-v1";
const RELEASE_SCOPE_SCHEMA_VERSION: &str = "golden-trace-release-scope-v1";
const EXECUTABLE_DECISION: &str = "included_in_final_replay_scope";
const VALIDATOR_EXECUTABLE_DECISION: &str = "validator_executable_scope_recorded";
const SCHEMA_ONLY_DECISION: &str = "schema_only_scope_recorded";
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
    "adapter_payload",
    "backtest_live",
    "cache_msgbus",
    "execution",
    "market_data",
    "order_lifecycle",
    "portfolio_pnl",
    "position",
    "read_model",
    "release_governance",
    "risk",
];

/// Counts the release-scope classifications validated by the Rust tool.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReleaseScopeCounts {
    pub executable_replay: usize,
    pub validator_executable_replay: usize,
    pub schema_only_scoped: usize,
}

impl ReleaseScopeCounts {
    #[must_use]
    pub const fn total(self) -> usize {
        self.executable_replay + self.validator_executable_replay + self.schema_only_scoped
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TraceCase {
    trace: String,
    category: String,
}

/// Loads and validates one golden trace JSONL file.
///
/// # Errors
///
/// Returns an error when the file cannot be read, contains malformed JSON, or
/// violates the golden trace row and event contract.
pub fn validate_trace(path: &Path) -> Result<Vec<Value>> {
    let rows = load_jsonl(path)?;
    let mut errors = Vec::new();
    let mut seen_case_ids = BTreeSet::new();

    if rows.is_empty() {
        bail!("trace must contain at least one row");
    }

    for (index, row) in rows.iter().enumerate() {
        let row_path = format!("row {index}");
        let Some(object) = row.as_object() else {
            errors.push(format!("{row_path}: row must be an object"));
            continue;
        };

        for field in REQUIRED_ROW_FIELDS {
            if !object.contains_key(*field) {
                errors.push(format!("{row_path}: missing {field}"));
            }
        }
        if object.get("schema_version").and_then(Value::as_str) != Some(SCHEMA_VERSION) {
            errors.push(format!(
                "{row_path}.schema_version: expected {SCHEMA_VERSION}"
            ));
        }

        match non_empty_string(object.get("case_id")) {
            Some(case_id) if !seen_case_ids.insert(case_id.to_string()) => {
                errors.push(format!("{row_path}.case_id: duplicate case_id {case_id}"));
            }
            Some(_) => {}
            None => errors.push(format!("{row_path}.case_id: must be a non-empty string")),
        }

        let category = object.get("category").and_then(Value::as_str);
        if !category.is_some_and(|value| VALID_CATEGORIES.contains(&value)) {
            errors.push(format!(
                "{row_path}.category: expected one of {}",
                VALID_CATEGORIES.join(", ")
            ));
        }
        if non_empty_string(object.get("description")).is_none() {
            errors.push(format!(
                "{row_path}.description: must be a non-empty string"
            ));
        }

        validate_event_section(
            object.get("input"),
            &format!("{row_path}.input"),
            &mut errors,
        );
        validate_event_section(
            object.get("expected"),
            &format!("{row_path}.expected"),
            &mut errors,
        );
        if !object.get("tolerances").is_some_and(Value::is_object) {
            errors.push(format!("{row_path}.tolerances: must be an object"));
        }
    }

    if !errors.is_empty() {
        bail!(errors.join("\n"));
    }
    Ok(rows)
}

/// Replays validated rows through a command and compares normalized JSON output.
///
/// # Errors
///
/// Returns an error when the command fails, output JSONL is malformed, or
/// actual cases differ from the expected cases.
pub fn replay_trace(trace: &Path, command_template: &str, rows: &[Value]) -> Result<()> {
    let directory = tempfile::tempdir().context("failed to create replay temporary directory")?;
    let actual_path = directory.path().join("actual.jsonl");
    let has_placeholder =
        command_template.contains("{trace}") || command_template.contains("{actual}");
    let mut command = command_template
        .replace("{trace}", &shell_quote(trace))
        .replace("{actual}", &shell_quote(&actual_path));
    if !has_placeholder {
        command.push(' ');
        command.push_str(&shell_quote(trace));
    }

    let output = shell_command(&command)
        .output()
        .with_context(|| format!("failed to run replay command: {command}"))?;
    if !output.status.success() {
        bail!(
            "replay command failed ({})\nSTDOUT:\n{}\nSTDERR:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let actual_rows = if actual_path.exists() {
        load_jsonl(&actual_path)?
    } else {
        load_replay_stdout(
            &String::from_utf8(output.stdout).context("replay stdout is not UTF-8")?,
        )?
    };

    let mut expected = BTreeMap::new();
    for row in rows {
        let object = row.as_object().context("validated row is not an object")?;
        let case_id = required_string(object, "case_id", "validated row")?;
        expected.insert(
            case_id.to_string(),
            object.get("expected").cloned().unwrap_or(Value::Null),
        );
    }

    let mut actual = BTreeMap::new();
    for row in actual_rows {
        let object = row
            .as_object()
            .context("replay output row must be an object")?;
        let case_id = required_string(object, "case_id", "replay output row")?;
        let value = object
            .get("actual")
            .or_else(|| object.get("output"))
            .or_else(|| object.get("expected"))
            .cloned()
            .unwrap_or(Value::Null);
        actual.insert(case_id.to_string(), value);
    }

    let mut errors = Vec::new();
    for (case_id, expected_value) in &expected {
        match actual.get(case_id) {
            Some(actual_value) if actual_value == expected_value => {}
            Some(actual_value) => errors.push(format!(
                "case {case_id}: expected {} got {}",
                normalized(expected_value),
                normalized(actual_value)
            )),
            None => errors.push(format!("case {case_id}: missing actual output")),
        }
    }
    let extra: Vec<_> = actual
        .keys()
        .filter(|case_id| !expected.contains_key(*case_id))
        .cloned()
        .collect();
    if !extra.is_empty() {
        errors.push(format!("unexpected actual cases: {}", extra.join(", ")));
    }
    if !errors.is_empty() {
        bail!("golden trace replay mismatch:\n{}", errors.join("\n"));
    }
    Ok(())
}

/// Validates the release replay/scope manifest against its golden trace cases.
///
/// # Errors
///
/// Returns an error when the manifest, trace glob, trace cases, classification
/// fields, or release decisions violate the release-scope contract.
pub fn validate_release_scope(
    manifest_path: &Path,
    trace_glob: &str,
) -> Result<ReleaseScopeCounts> {
    let manifest = load_json_object(manifest_path)?;
    let trace_cases = collect_trace_cases(trace_glob, manifest_trace_paths(&manifest))?;
    validate_manifest(&manifest, &trace_cases)
}

fn load_jsonl(path: &Path) -> Result<Vec<Value>> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    load_jsonl_text(&text, &path.display().to_string())
}

fn load_jsonl_text(text: &str, source: &str) -> Result<Vec<Value>> {
    let mut rows = Vec::new();
    for (index, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let value = serde_json::from_str(line)
            .with_context(|| format!("{source}:{}: invalid JSON", index + 1))?;
        rows.push(value);
    }
    Ok(rows)
}

fn load_replay_stdout(text: &str) -> Result<Vec<Value>> {
    let mut rows = Vec::new();
    for (index, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        rows.push(
            serde_json::from_str(line)
                .with_context(|| format!("replay stdout:{}: invalid JSON", index + 1))?,
        );
    }
    Ok(rows)
}

fn validate_event_section(section: Option<&Value>, path: &str, errors: &mut Vec<String>) {
    let Some(object) = section.and_then(Value::as_object) else {
        errors.push(format!("{path}: must be an object"));
        return;
    };
    let Some(events) = object.get("events").and_then(Value::as_array) else {
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
    for field in REQUIRED_EVENT_FIELDS {
        if !object.contains_key(*field) {
            errors.push(format!("{path}: missing event field {field}"));
        }
    }
    if non_empty_string(object.get("event_type")).is_none() {
        errors.push(format!("{path}.event_type: must be a non-empty string"));
    }
    for field in ["ts_event", "ts_init"] {
        if let Some(value) = object.get(field)
            && !is_timestamp(value)
        {
            errors.push(format!(
                "{path}.{field}: must be an integer or decimal string nanosecond timestamp"
            ));
        }
    }
    if !object.get("payload").is_some_and(Value::is_object) {
        errors.push(format!("{path}.payload: must be an object"));
    }
}

fn is_timestamp(value: &Value) -> bool {
    value.as_i64().is_some()
        || value.as_u64().is_some()
        || value
            .as_str()
            .is_some_and(|text| !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit()))
}

fn non_empty_string(value: Option<&Value>) -> Option<&str> {
    value
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
}

fn non_blank_string(value: Option<&Value>) -> Option<&str> {
    value
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
}

fn required_string<'a>(object: &'a Map<String, Value>, key: &str, path: &str) -> Result<&'a str> {
    non_empty_string(object.get(key))
        .with_context(|| format!("{path}.{key}: must be a non-empty string"))
}

fn normalized(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<invalid JSON>".to_string())
}

fn load_json_object(path: &Path) -> Result<Map<String, Value>> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let value: Value =
        serde_json::from_str(&text).with_context(|| format!("{}: invalid JSON", path.display()))?;
    value
        .as_object()
        .cloned()
        .with_context(|| format!("{}: manifest must be a JSON object", path.display()))
}

fn manifest_trace_paths(manifest: &Map<String, Value>) -> Vec<PathBuf> {
    manifest
        .get("cases")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.get("trace").and_then(Value::as_str))
        .filter(|trace| !trace.trim().is_empty())
        .map(PathBuf::from)
        .collect()
}

fn collect_trace_cases(
    trace_glob: &str,
    manifest_paths: Vec<PathBuf>,
) -> Result<BTreeMap<String, TraceCase>> {
    let mut paths = BTreeMap::new();
    for entry in
        glob::glob(trace_glob).with_context(|| format!("invalid TRACE_GLOB={trace_glob}"))?
    {
        let path = entry.with_context(|| format!("failed to expand TRACE_GLOB={trace_glob}"))?;
        paths.insert(path.to_string_lossy().into_owned(), path);
    }
    for path in manifest_paths {
        paths
            .entry(path.to_string_lossy().into_owned())
            .or_insert(path);
    }
    if paths.is_empty() {
        bail!("no golden trace files found for TRACE_GLOB={trace_glob}");
    }

    let mut cases = BTreeMap::new();
    for (trace_name, trace_path) in paths {
        if !trace_path.exists() {
            bail!(
                "{}: trace file referenced by release scope does not exist",
                trace_path.display()
            );
        }
        for row in load_jsonl(&trace_path)? {
            let object = row
                .as_object()
                .with_context(|| format!("{}: row must be a JSON object", trace_path.display()))?;
            let case_id = required_string(object, "case_id", &trace_name)?;
            let category = required_string(object, "category", case_id)?;
            if cases
                .insert(
                    case_id.to_string(),
                    TraceCase {
                        trace: trace_name.clone(),
                        category: category.to_string(),
                    },
                )
                .is_some()
            {
                bail!("duplicate case_id in tests/golden: {case_id}");
            }
        }
    }
    Ok(cases)
}

fn validate_manifest(
    manifest: &Map<String, Value>,
    trace_cases: &BTreeMap<String, TraceCase>,
) -> Result<ReleaseScopeCounts> {
    let mut errors = Vec::new();
    if manifest.get("schema_version").and_then(Value::as_str) != Some(RELEASE_SCOPE_SCHEMA_VERSION)
    {
        errors.push(format!(
            "manifest.schema_version must be {RELEASE_SCOPE_SCHEMA_VERSION}"
        ));
    }
    if non_blank_string(manifest.get("owner_signoff")).is_none() {
        errors
            .push("manifest.owner_signoff must record the current owner signoff state".to_string());
    }

    let entries = match manifest.get("cases").and_then(Value::as_array) {
        Some(entries) if !entries.is_empty() => entries.as_slice(),
        _ => {
            errors.push("manifest.cases must be a non-empty array".to_string());
            &[]
        }
    };
    let mut manifest_case_ids = BTreeSet::new();
    let mut counts = ReleaseScopeCounts::default();

    for (index, entry) in entries.iter().enumerate() {
        let path = format!("manifest.cases[{index}]");
        let Some(object) = entry.as_object() else {
            errors.push(format!("{path}: entry must be an object"));
            continue;
        };
        let Some(case_id) = non_blank_string(object.get("case_id")) else {
            errors.push(format!("{path}.case_id must be a non-empty string"));
            continue;
        };
        if !manifest_case_ids.insert(case_id.to_string()) {
            errors.push(format!("{path}.case_id duplicates {case_id}"));
            continue;
        }

        let Some(status) = object.get("status").and_then(Value::as_str) else {
            errors.push(format!("{case_id}.status must be one of executable_replay, validator_executable_replay, schema_only_scoped"));
            continue;
        };
        match status {
            "executable_replay" => {
                counts.executable_replay += 1;
                validate_executable(object, case_id, &mut errors);
            }
            "validator_executable_replay" => {
                counts.validator_executable_replay += 1;
                validate_validator_executable(object, case_id, &mut errors);
            }
            "schema_only_scoped" => {
                counts.schema_only_scoped += 1;
                validate_schema_only(object, case_id, &mut errors);
            }
            _ => {
                errors.push(format!("{case_id}.status must be one of executable_replay, validator_executable_replay, schema_only_scoped"));
                continue;
            }
        }

        match trace_cases.get(case_id) {
            Some(actual) => {
                if object.get("trace").and_then(Value::as_str) != Some(actual.trace.as_str()) {
                    errors.push(format!(
                        "{case_id}.trace expected {} got {}",
                        actual.trace,
                        object.get("trace").map_or("null".to_string(), normalized)
                    ));
                }
                if object.get("category").and_then(Value::as_str) != Some(actual.category.as_str())
                {
                    errors.push(format!(
                        "{case_id}.category expected {} got {}",
                        actual.category,
                        object
                            .get("category")
                            .map_or("null".to_string(), normalized)
                    ));
                }
            }
            None => errors.push(format!(
                "{case_id}: manifest entry has no matching tests/golden case"
            )),
        }
    }

    let missing: Vec<_> = trace_cases
        .keys()
        .filter(|case_id| !manifest_case_ids.contains(*case_id))
        .cloned()
        .collect();
    let extra: Vec<_> = manifest_case_ids
        .iter()
        .filter(|case_id| !trace_cases.contains_key(*case_id))
        .cloned()
        .collect();
    if !missing.is_empty() {
        errors.push(format!(
            "manifest missing trace cases: {}",
            missing.join(", ")
        ));
    }
    if !extra.is_empty() {
        errors.push(format!(
            "manifest has extra trace cases: {}",
            extra.join(", ")
        ));
    }
    if !errors.is_empty() {
        bail!(
            "golden trace release scope validation failed:\n{}",
            errors.join("\n")
        );
    }
    Ok(counts)
}

fn validate_executable(object: &Map<String, Value>, case_id: &str, errors: &mut Vec<String>) {
    require_fields(
        object,
        case_id,
        &["evidence_id", "harness", "rust_entrypoint"],
        "executable_replay",
        errors,
    );
    require_decision(object, case_id, EXECUTABLE_DECISION, errors);
}

fn validate_validator_executable(
    object: &Map<String, Value>,
    case_id: &str,
    errors: &mut Vec<String>,
) {
    require_fields(
        object,
        case_id,
        &["evidence_id", "harness", "validator_entrypoint"],
        "validator_executable_replay",
        errors,
    );
    if object.contains_key("rust_entrypoint") {
        errors.push(format!(
            "{case_id}: validator_executable_replay must not claim rust_entrypoint"
        ));
    }
    require_decision(object, case_id, VALIDATOR_EXECUTABLE_DECISION, errors);
    if object.get("runtime_adapter_integration") != Some(&Value::Bool(false)) {
        errors.push(format!(
            "{case_id}.runtime_adapter_integration must be false for validator_executable_replay"
        ));
    }
}

fn validate_schema_only(object: &Map<String, Value>, case_id: &str, errors: &mut Vec<String>) {
    require_fields(
        object,
        case_id,
        &["scope_owner", "reason", "follow_up"],
        "schema_only_scoped",
        errors,
    );
    require_decision(object, case_id, SCHEMA_ONLY_DECISION, errors);
    let forbidden: Vec<_> = ["evidence_id", "harness", "rust_entrypoint"]
        .into_iter()
        .filter(|field| object.contains_key(*field))
        .collect();
    if !forbidden.is_empty() {
        errors.push(format!(
            "{case_id}: schema_only_scoped must not claim executable fields {forbidden:?}"
        ));
    }
}

fn require_fields(
    object: &Map<String, Value>,
    case_id: &str,
    fields: &[&str],
    status: &str,
    errors: &mut Vec<String>,
) {
    for field in fields {
        if non_blank_string(object.get(*field)).is_none() {
            errors.push(format!("{case_id}.{field} is required for {status}"));
        }
    }
}

fn require_decision(
    object: &Map<String, Value>,
    case_id: &str,
    expected: &str,
    errors: &mut Vec<String>,
) {
    if object.get("release_decision").and_then(Value::as_str) != Some(expected) {
        errors.push(format!("{case_id}.release_decision must be {expected}"));
    }
}

#[cfg(unix)]
fn shell_command(command: &str) -> Command {
    let mut process = Command::new("sh");
    process.arg("-c").arg(command);
    process
}

#[cfg(windows)]
fn shell_command(command: &str) -> Command {
    let mut process = Command::new("cmd");
    process.arg("/C").arg(command);
    process
}

fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::json;

    use super::*;

    fn valid_row(case_id: &str) -> Value {
        json!({
            "schema_version": SCHEMA_VERSION,
            "case_id": case_id,
            "category": "risk",
            "description": "valid fixture",
            "input": {"events": [{"event_type": "RiskInput", "ts_event": 1, "payload": {}}]},
            "expected": {"events": [{"event_type": "RiskOutput", "ts_event": "2", "payload": {}}]},
            "tolerances": {}
        })
    }

    fn write_jsonl(path: &Path, rows: &[Value]) {
        let text = rows
            .iter()
            .map(|row| serde_json::to_string(row).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(path, format!("{text}\n")).unwrap();
    }

    #[test]
    fn validates_trace_and_replay() {
        let directory = tempfile::tempdir().unwrap();
        let trace = directory.path().join("trace.jsonl");
        write_jsonl(&trace, &[valid_row("case.001")]);

        let rows = validate_trace(&trace).unwrap();
        assert_eq!(rows.len(), 1);
        replay_trace(&trace, "cat {trace}", &rows).unwrap();
        replay_trace(&trace, "cat", &rows).unwrap();
        replay_trace(&trace, "cp {trace} {actual}", &rows).unwrap();
    }

    #[test]
    fn rejects_duplicate_and_invalid_rows() {
        let directory = tempfile::tempdir().unwrap();
        let trace = directory.path().join("trace.jsonl");
        let mut invalid = valid_row("case.001");
        invalid["category"] = json!("unknown");
        invalid["input"]["events"][0]["ts_event"] = json!(false);
        write_jsonl(&trace, &[valid_row("case.001"), invalid]);

        let error = validate_trace(&trace).unwrap_err().to_string();
        assert!(error.contains("duplicate case_id case.001"));
        assert!(error.contains("expected one of"));
        assert!(error.contains("nanosecond timestamp"));
    }

    #[test]
    fn rejects_replay_mismatch() {
        let directory = tempfile::tempdir().unwrap();
        let trace = directory.path().join("trace.jsonl");
        write_jsonl(&trace, &[valid_row("case.001")]);
        let rows = validate_trace(&trace).unwrap();

        let error = replay_trace(
            &trace,
            "test -f {trace} && printf '%s\\n' '{\"case_id\":\"case.001\",\"actual\":{}}'",
            &rows,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("golden trace replay mismatch"));
    }

    #[test]
    fn rejects_comments_in_replay_stdout() {
        let directory = tempfile::tempdir().unwrap();
        let trace = directory.path().join("trace.jsonl");
        write_jsonl(&trace, &[valid_row("case.001")]);
        let rows = validate_trace(&trace).unwrap();

        let error = replay_trace(&trace, "test -f {trace} && printf '# comment\\n'", &rows)
            .unwrap_err()
            .to_string();
        assert!(error.contains("replay stdout:1: invalid JSON"));
    }

    #[test]
    fn validates_release_scope_and_rejects_missing_case() {
        let directory = tempfile::tempdir().unwrap();
        let trace = directory.path().join("trace.jsonl");
        write_jsonl(&trace, &[valid_row("case.001")]);
        let trace_name = trace.to_string_lossy();
        let manifest = directory.path().join("scope.json");
        fs::write(
            &manifest,
            serde_json::to_vec_pretty(&json!({
                "schema_version": RELEASE_SCOPE_SCHEMA_VERSION,
                "owner_signoff": "approved",
                "cases": [{
                    "case_id": "case.001",
                    "category": "risk",
                    "trace": trace_name,
                    "status": "executable_replay",
                    "evidence_id": "evidence",
                    "harness": "harness",
                    "rust_entrypoint": "entrypoint",
                    "release_decision": EXECUTABLE_DECISION
                }]
            }))
            .unwrap(),
        )
        .unwrap();

        let counts = validate_release_scope(&manifest, &trace.to_string_lossy()).unwrap();
        assert_eq!(counts.executable_replay, 1);

        fs::write(
            &manifest,
            serde_json::to_vec_pretty(&json!({
                "schema_version": RELEASE_SCOPE_SCHEMA_VERSION,
                "owner_signoff": "approved",
                "cases": []
            }))
            .unwrap(),
        )
        .unwrap();
        let error = validate_release_scope(&manifest, &trace.to_string_lossy())
            .unwrap_err()
            .to_string();
        assert!(error.contains("manifest.cases must be a non-empty array"));
    }

    #[test]
    fn rejects_blank_release_scope_evidence_fields() {
        let directory = tempfile::tempdir().unwrap();
        let trace = directory.path().join("trace.jsonl");
        write_jsonl(&trace, &[valid_row("case.001")]);
        let manifest = directory.path().join("scope.json");
        fs::write(
            &manifest,
            serde_json::to_vec_pretty(&json!({
                "schema_version": RELEASE_SCOPE_SCHEMA_VERSION,
                "owner_signoff": " ",
                "cases": [{
                    "case_id": "case.001",
                    "category": "risk",
                    "trace": trace.to_string_lossy(),
                    "status": "executable_replay",
                    "evidence_id": " ",
                    "harness": "harness",
                    "rust_entrypoint": "entrypoint",
                    "release_decision": EXECUTABLE_DECISION
                }]
            }))
            .unwrap(),
        )
        .unwrap();

        let error = validate_release_scope(&manifest, &trace.to_string_lossy())
            .unwrap_err()
            .to_string();
        assert!(error.contains("manifest.owner_signoff"));
        assert!(error.contains("case.001.evidence_id is required"));
    }
}
