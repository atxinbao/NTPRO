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
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{Map, Value, json};

const TRACE_FILE: &str = "live_alpha_reconciliation_schema.jsonl";
const RISK_PREFLIGHT_INPUT_SCHEMA_VERSION: &str = "ntpro.v140_live_alpha_risk_preflight_input.v1";

#[test]
fn rust_cli_replays_live_alpha_reconciliation_golden_traces() -> Result<(), Box<dyn Error>> {
    let cases = load_cases(TRACE_FILE)?;
    if cases.len() != 7 {
        return Err(
            "V140-005 must keep the seven scoped reconciliation scenarios executable".into(),
        );
    }

    let bin = nautilus_bin();
    let root = temporary_root()?;
    for case in cases {
        let expected = events(&case, "expected")?;
        let actual = replay_case(&bin, &root, &case)?;
        if actual != expected {
            return Err(format!(
                "live-alpha reconciliation replay must match {}\nactual={actual:#?}\nexpected={expected:#?}",
                string_field(&case, "case_id")?
            )
            .into());
        }
    }

    fs::remove_dir_all(root)?;
    Ok(())
}

fn replay_case(bin: &Path, root: &Path, case: &Value) -> Result<Vec<Value>, Box<dyn Error>> {
    let input_event = events(case, "input")?
        .into_iter()
        .next()
        .ok_or("input.events must contain one scenario event")?;
    let payload = payload(&input_event)?;
    let scenario = string_field(payload, "scenario")?;
    let case_dir = root.join(scenario);
    fs::create_dir_all(&case_dir)?;

    let order = object_field(payload, "order")?;
    let order_gate_path = case_dir.join("live-alpha-dry-run-order-gate.json");
    run_checked(
        Command::new(bin)
            .arg("live")
            .arg("production-live-alpha-dry-run-order-gate")
            .arg("--run-id")
            .arg(format!("v140-recon-gate-{scenario}"))
            .arg("--session-id")
            .arg(format!("session-{scenario}"))
            .arg("--strategy-id")
            .arg("ema_cross_btcusdt_v1")
            .arg("--symbol")
            .arg(string_field(order, "symbol")?)
            .arg("--side")
            .arg(string_field(order, "side")?)
            .arg("--order-type")
            .arg(string_field(order, "order_type")?)
            .arg("--quantity")
            .arg(string_field(order, "quantity")?)
            .arg("--notional")
            .arg(string_field(order, "notional")?)
            .arg("--output")
            .arg(&order_gate_path)
            .arg("--allow-production-live-alpha-dry-run")
            .arg("--confirm-owner-approved-dry-run")
            .arg("--confirm-no-production-order-submission")
            .arg("--confirm-no-production-order-mutation")
            .arg("--confirm-no-execution-adapter-call")
            .arg("--confirm-no-listen-key-lifecycle")
            .arg("--confirm-dashboard-order-controls-disabled")
            .arg("--confirm-no-real-funds"),
    )?;

    let input_path = case_dir.join("risk-preflight-input.json");
    write_risk_preflight_input(&input_path, payload)?;
    let report_path = case_dir.join("risk-preflight-report.json");
    run_checked(
        Command::new(bin)
            .arg("live")
            .arg("production-live-alpha-risk-preflight")
            .arg("--run-id")
            .arg(format!("v140-recon-risk-{scenario}"))
            .arg("--order-gate")
            .arg(&order_gate_path)
            .arg("--input")
            .arg(&input_path)
            .arg("--output")
            .arg(&report_path)
            .arg("--confirm-hypothetical-dry-run-only")
            .arg("--confirm-no-execution-adapter-call")
            .arg("--confirm-no-production-order-submission")
            .arg("--confirm-no-production-order-mutation")
            .arg("--confirm-dashboard-order-controls-disabled"),
    )?;

    let report = read_json(&report_path)?;
    Ok(vec![normalized_reconciliation_event(
        &input_event,
        payload,
        &report,
    )?])
}

fn write_risk_preflight_input(path: &Path, payload: &Value) -> Result<(), Box<dyn Error>> {
    let input = json!({
        "schema_version": RISK_PREFLIGHT_INPUT_SCHEMA_VERSION,
        "session": object_field(payload, "session")?,
        "market": object_field(payload, "market")?,
        "account": object_field(payload, "account")?,
        "order_state": object_field(payload, "order_state")?,
        "risk": object_field(payload, "risk")?,
        "order": object_field(payload, "order")?,
        "limits": object_field(payload, "limits")?,
    });
    fs::write(path, serde_json::to_string_pretty(&input)?)?;
    Ok(())
}

fn normalized_reconciliation_event(
    input_event: &Value,
    input_payload: &Value,
    report: &Value,
) -> Result<Value, Box<dyn Error>> {
    let mut payload = Map::new();
    payload.insert(
        "scenario".to_string(),
        json!(string_field(input_payload, "scenario")?),
    );
    for key in [
        "status",
        "risk_decision",
        "reasons",
        "order_gate_ready",
        "order_state_age_ms",
        "max_order_state_age_ms",
        "open_order_count",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "production_order_state_reads_allowed",
        "production_order_submissions_attempted",
        "production_orders_submitted",
        "production_order_mutations_attempted",
        "production_order_state_reads_attempted",
        "execution_adapter_called",
        "order_endpoint_access_attempted",
        "matching_engine_submission",
        "actual_submission_count",
        "automatic_correction_orders_submitted",
        "dashboard_order_controls_enabled",
        "network_attempted",
        "real_orders_submitted",
        "real_funds",
        "production_trading_enabled",
        "values_are_exchange_truth",
    ] {
        payload.insert(key.to_string(), report_field(report, key)?);
    }

    Ok(json!({
        "event_type": "execution.reconciliation.live_alpha_risk_preflight",
        "ts_event": "0",
        "ts_init": "0",
        "instrument_id": string_field(input_event, "instrument_id")?,
        "venue": string_field(input_event, "venue")?,
        "correlation_id": string_field(input_event, "correlation_id")?,
        "payload": Value::Object(payload),
    }))
}

fn load_cases(file_name: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    let trace = repository_root().join("tests/golden").join(file_name);
    fs::read_to_string(&trace)?
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
        .map(|line| Ok(serde_json::from_str::<Value>(line)?))
        .collect()
}

fn events(case: &Value, section: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    Ok(case
        .get(section)
        .and_then(|value| value.get("events"))
        .and_then(Value::as_array)
        .ok_or("trace section events must be an array")?
        .clone())
}

fn payload(event: &Value) -> Result<&Value, Box<dyn Error>> {
    event
        .get("payload")
        .and_then(Value::as_object)
        .map(|_| &event["payload"])
        .ok_or_else(|| "event payload must be an object".into())
}

fn object_field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_object)
        .map(|_| &value[key])
        .ok_or_else(|| format!("{key} must be an object").into())
}

fn report_field(report: &Value, key: &str) -> Result<Value, Box<dyn Error>> {
    report
        .get(key)
        .cloned()
        .ok_or_else(|| format!("report missing {key}").into())
}

fn string_field<'a>(value: &'a Value, key: &str) -> Result<&'a str, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{key} must be a string").into())
}

fn read_json(path: &Path) -> Result<Value, Box<dyn Error>> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn run_checked(command: &mut Command) -> Result<(), Box<dyn Error>> {
    let output = command.output()?;
    if output.status.success() {
        return Ok(());
    }
    Err(command_error(&output).into())
}

fn command_error(output: &Output) -> String {
    format!(
        "command failed with status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn nautilus_bin() -> PathBuf {
    option_env!("CARGO_BIN_EXE_nautilus")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let mut path = repository_root().join("target/debug/nautilus");
            if cfg!(windows) {
                path.set_extension("exe");
            }
            path
        })
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap_or_else(|err| panic!("repository root should resolve from crates/cli: {err}"))
}

fn temporary_root() -> Result<PathBuf, Box<dyn Error>> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!(
        "ntpro-v140-live-alpha-reconciliation-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    Ok(root)
}
