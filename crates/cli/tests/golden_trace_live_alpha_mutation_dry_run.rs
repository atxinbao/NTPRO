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

const TRACE_FILE: &str = "live_alpha_mutation_dry_run_schema.jsonl";
const RISK_PREFLIGHT_INPUT_SCHEMA_VERSION: &str = "ntpro.v140_live_alpha_risk_preflight_input.v1";
const SYNTHETIC_API_KEY_ENV: &str = "NTPRO_V150006_API_KEY";
const SYNTHETIC_API_SECRET_ENV: &str = "NTPRO_V150006_API_SECRET";
const SYNTHETIC_API_KEY: &str = "ntpro_v150006_synthetic_api_key_value";
const SYNTHETIC_API_SECRET: &str = "ntpro_v150006_synthetic_api_secret_value";

#[test]
fn rust_cli_replays_live_alpha_mutation_dry_run_golden_traces() -> Result<(), Box<dyn Error>> {
    let cases = load_cases(TRACE_FILE)?;
    if cases.len() != 9 {
        return Err(
            "V150-006 must keep the nine scoped mutation dry-run scenarios executable".into(),
        );
    }

    let bin = nautilus_bin();
    let root = temporary_root()?;
    for case in cases {
        let expected = events(&case, "expected")?;
        let actual = replay_case(&bin, &root, &case)?;
        assert_no_production_mutation_proof(&actual)?;
        if actual != expected {
            return Err(format!(
                "live-alpha mutation dry-run replay must match {}\nactual={actual:#?}\nexpected={expected:#?}",
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

    let artifact = match scenario {
        "dashboard_control_attempted" => {
            let order_gate = run_order_gate(bin, &case_dir, scenario, false)?;
            ("order_gate", read_json(&order_gate)?)
        }
        "approval_expired" => {
            let order_gate = run_order_gate(bin, &case_dir, scenario, true)?;
            let approval =
                run_manual_approval(bin, &case_dir, scenario, "expired", 1_718_400_070_000)?;
            let request_preview =
                run_request_preview(bin, &case_dir, scenario, &order_gate, &approval)?;
            ("request_preview", read_json(&request_preview)?)
        }
        "kill_switch_active" | "missing_approval" | "network_disabled" => {
            let order_gate = run_order_gate(bin, &case_dir, scenario, true)?;
            let risk_preflight = run_risk_preflight(bin, &case_dir, scenario, &order_gate)?;
            let approval =
                run_manual_approval(bin, &case_dir, scenario, "approved", 1_718_400_000_000)?;
            let request_preview =
                run_request_preview(bin, &case_dir, scenario, &order_gate, &approval)?;
            let (kill_switch_active, approval_state) = match scenario {
                "kill_switch_active" => (true, "approved"),
                "missing_approval" => (false, "pending"),
                "network_disabled" => (false, "approved"),
                _ => unreachable!("scenario was matched above"),
            };
            let kill_switch_approval = run_kill_switch_approval(
                bin,
                &case_dir,
                scenario,
                kill_switch_active,
                approval_state,
            )?;
            let runtime_gate = run_kill_switch_runtime_gate(
                bin,
                &case_dir,
                scenario,
                &kill_switch_approval,
                &risk_preflight,
                &request_preview,
            )?;
            if scenario == "network_disabled" {
                let execution = run_execution_dry_run(
                    bin,
                    &case_dir,
                    scenario,
                    &order_gate,
                    &risk_preflight,
                    &request_preview,
                    &runtime_gate,
                )?;
                ("execution_dry_run", read_json(&execution)?)
            } else {
                ("kill_switch_runtime_gate", read_json(&runtime_gate)?)
            }
        }
        "symbol_not_allowlisted"
        | "notional_too_large"
        | "order_state_stale"
        | "account_unreadable" => {
            let order_gate = run_order_gate(bin, &case_dir, scenario, true)?;
            let risk_preflight = run_risk_preflight(bin, &case_dir, scenario, &order_gate)?;
            ("risk_preflight", read_json(&risk_preflight)?)
        }
        other => return Err(format!("unsupported mutation dry-run scenario: {other}").into()),
    };

    Ok(vec![normalized_mutation_event(
        &input_event,
        scenario,
        artifact.0,
        &artifact.1,
    )?])
}

fn run_order_gate(
    bin: &Path,
    case_dir: &Path,
    scenario: &str,
    confirm_dashboard_disabled: bool,
) -> Result<PathBuf, Box<dyn Error>> {
    let output = case_dir.join("live-alpha-dry-run-order-gate.json");
    let mut command = Command::new(bin);
    command
        .arg("live")
        .arg("production-live-alpha-dry-run-order-gate")
        .arg("--run-id")
        .arg(format!("v150-mutation-gate-{scenario}"))
        .arg("--session-id")
        .arg(format!("session-{scenario}"))
        .arg("--strategy-id")
        .arg("ema_cross_btcusdt_v1")
        .arg("--symbol")
        .arg(order_symbol(scenario))
        .arg("--side")
        .arg("BUY")
        .arg("--order-type")
        .arg("LIMIT")
        .arg("--quantity")
        .arg("0.001")
        .arg("--notional")
        .arg(order_notional(scenario))
        .arg("--output")
        .arg(&output)
        .arg("--allow-production-live-alpha-dry-run")
        .arg("--confirm-owner-approved-dry-run")
        .arg("--confirm-no-production-order-submission")
        .arg("--confirm-no-production-order-mutation")
        .arg("--confirm-no-execution-adapter-call")
        .arg("--confirm-no-listen-key-lifecycle")
        .arg("--confirm-no-real-funds");
    if confirm_dashboard_disabled {
        command.arg("--confirm-dashboard-order-controls-disabled");
    }
    run_checked(&mut command)?;
    Ok(output)
}

fn run_risk_preflight(
    bin: &Path,
    case_dir: &Path,
    scenario: &str,
    order_gate: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    let input = case_dir.join("risk-preflight-input.json");
    write_risk_preflight_input(&input, scenario)?;
    let output = case_dir.join("risk-preflight-report.json");
    run_checked(
        Command::new(bin)
            .arg("live")
            .arg("production-live-alpha-risk-preflight")
            .arg("--run-id")
            .arg(format!("v150-mutation-risk-{scenario}"))
            .arg("--order-gate")
            .arg(order_gate)
            .arg("--input")
            .arg(&input)
            .arg("--output")
            .arg(&output)
            .arg("--confirm-hypothetical-dry-run-only")
            .arg("--confirm-no-execution-adapter-call")
            .arg("--confirm-no-production-order-submission")
            .arg("--confirm-no-production-order-mutation")
            .arg("--confirm-dashboard-order-controls-disabled"),
    )?;
    Ok(output)
}

fn run_manual_approval(
    bin: &Path,
    case_dir: &Path,
    scenario: &str,
    approval_state: &str,
    now_unix_ms: u64,
) -> Result<PathBuf, Box<dyn Error>> {
    let output = case_dir.join(format!("manual-approval-{approval_state}.json"));
    let mut command = Command::new(bin);
    command
        .arg("live")
        .arg("production-live-alpha-manual-approval-lifecycle")
        .arg("--run-id")
        .arg(request_run_id(scenario))
        .arg("--strategy-id")
        .arg("ema_cross_btcusdt_v1")
        .arg("--symbol")
        .arg(order_symbol(scenario))
        .arg("--notional")
        .arg(order_notional(scenario))
        .arg("--approval-state")
        .arg(approval_state)
        .arg("--now-unix-ms")
        .arg(now_unix_ms.to_string())
        .arg("--expires-at-unix-ms")
        .arg("1718400060000")
        .arg("--output")
        .arg(&output)
        .arg("--confirm-dry-run-request-preview-only")
        .arg("--confirm-one-time-approval")
        .arg("--confirm-no-production-mutation")
        .arg("--confirm-dashboard-order-controls-disabled");
    if approval_state != "pending" {
        command
            .arg("--manual-approval-id")
            .arg(format!("owner-approval-v150-006-{scenario}"))
            .arg("--approved-by")
            .arg("owner");
    }
    run_checked(&mut command)?;
    Ok(output)
}

fn run_request_preview(
    bin: &Path,
    case_dir: &Path,
    scenario: &str,
    order_gate: &Path,
    approval: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    let output = case_dir.join("request-preview.json");
    let mut command = Command::new(bin);
    command
        .env(SYNTHETIC_API_KEY_ENV, SYNTHETIC_API_KEY)
        .env(SYNTHETIC_API_SECRET_ENV, SYNTHETIC_API_SECRET)
        .arg("live")
        .arg("production-live-alpha-order-request-preview")
        .arg("--run-id")
        .arg(request_run_id(scenario))
        .arg("--order-gate")
        .arg(order_gate)
        .arg("--manual-approval-lifecycle")
        .arg(approval)
        .arg("--endpoint-path")
        .arg("/api/v3/order")
        .arg("--price")
        .arg("10000.00")
        .arg("--time-in-force")
        .arg("GTC")
        .arg("--timestamp-ms")
        .arg("1718400000000")
        .arg("--recv-window-ms")
        .arg("5000")
        .arg("--api-key-env")
        .arg(SYNTHETIC_API_KEY_ENV)
        .arg("--api-secret-env")
        .arg(SYNTHETIC_API_SECRET_ENV)
        .arg("--output")
        .arg(&output)
        .arg("--allow-production-live-alpha-request-preview")
        .arg("--confirm-owner-approved-request-preview")
        .arg("--confirm-memory-only-signature")
        .arg("--confirm-no-production-order-submission")
        .arg("--confirm-no-production-order-mutation")
        .arg("--confirm-no-execution-adapter-call")
        .arg("--confirm-no-network")
        .arg("--confirm-no-listen-key-lifecycle")
        .arg("--confirm-dashboard-order-controls-disabled")
        .arg("--confirm-no-real-funds");
    run_checked(&mut command)?;
    Ok(output)
}

fn run_kill_switch_approval(
    bin: &Path,
    case_dir: &Path,
    scenario: &str,
    kill_switch_active: bool,
    approval_state: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    let output = case_dir.join("kill-switch-approval.json");
    let mut command = Command::new(bin);
    command
        .arg("live")
        .arg("production-kill-switch-approval-artifact")
        .arg("--run-id")
        .arg(format!("v150-mutation-kill-switch-{scenario}"))
        .arg("--session-id")
        .arg(format!("session-{scenario}"))
        .arg("--strategy-id")
        .arg("ema_cross_btcusdt_v1")
        .arg("--output")
        .arg(&output)
        .arg("--kill-switch-active")
        .arg(kill_switch_active.to_string())
        .arg("--approval-state")
        .arg(approval_state)
        .arg("--confirm-dry-run-only")
        .arg("--confirm-no-production-mutation")
        .arg("--confirm-dashboard-order-controls-disabled");
    if approval_state == "approved" {
        command
            .arg("--manual-approval-id")
            .arg(format!("owner-kill-switch-approval-v150-006-{scenario}"))
            .arg("--approved-by")
            .arg("owner");
    }
    run_checked(&mut command)?;
    Ok(output)
}

fn run_kill_switch_runtime_gate(
    bin: &Path,
    case_dir: &Path,
    scenario: &str,
    kill_switch_approval: &Path,
    risk_preflight: &Path,
    request_preview: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    let output = case_dir.join("kill-switch-runtime-gate.json");
    run_checked(
        Command::new(bin)
            .arg("live")
            .arg("production-live-alpha-kill-switch-runtime-gate")
            .arg("--run-id")
            .arg(format!("v150-mutation-runtime-gate-{scenario}"))
            .arg("--kill-switch-approval")
            .arg(kill_switch_approval)
            .arg("--risk-preflight")
            .arg(risk_preflight)
            .arg("--request-preview")
            .arg(request_preview)
            .arg("--output")
            .arg(&output)
            .arg("--allow-production-live-alpha-kill-switch-runtime-gate")
            .arg("--confirm-owner-approved-runtime-gate")
            .arg("--confirm-no-production-order-submission")
            .arg("--confirm-no-production-order-mutation")
            .arg("--confirm-no-network")
            .arg("--confirm-no-listen-key-lifecycle")
            .arg("--confirm-dashboard-order-controls-disabled")
            .arg("--confirm-no-real-funds"),
    )?;
    Ok(output)
}

fn run_execution_dry_run(
    bin: &Path,
    case_dir: &Path,
    scenario: &str,
    order_gate: &Path,
    risk_preflight: &Path,
    request_preview: &Path,
    runtime_gate: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    let output = case_dir.join("execution-dry-run.json");
    run_checked(
        Command::new(bin)
            .arg("live")
            .arg("production-live-alpha-execution-dry-run")
            .arg("--run-id")
            .arg(format!("v150-mutation-execution-{scenario}"))
            .arg("--order-gate")
            .arg(order_gate)
            .arg("--risk-preflight")
            .arg(risk_preflight)
            .arg("--request-preview")
            .arg(request_preview)
            .arg("--kill-switch-runtime-gate")
            .arg(runtime_gate)
            .arg("--output")
            .arg(&output)
            .arg("--allow-production-live-alpha-execution-dry-run")
            .arg("--confirm-owner-approved-execution-dry-run")
            .arg("--confirm-dry-run-adapter-only")
            .arg("--confirm-no-production-adapter")
            .arg("--confirm-no-production-order-submission")
            .arg("--confirm-no-production-order-mutation")
            .arg("--confirm-no-network")
            .arg("--confirm-no-listen-key-lifecycle")
            .arg("--confirm-dashboard-order-controls-disabled")
            .arg("--confirm-no-real-funds"),
    )?;
    Ok(output)
}

fn write_risk_preflight_input(path: &Path, scenario: &str) -> Result<(), Box<dyn Error>> {
    let account = if scenario == "account_unreadable" {
        json!({"readable": false, "account_id": ""})
    } else {
        json!({"readable": true, "account_id": "BINANCE-001"})
    };
    let order_state = if scenario == "order_state_stale" {
        json!({
            "readable": true,
            "open_order_count": 0,
            "last_read_at_unix_ms": 1000,
            "now_unix_ms": 5000,
            "max_age_ms": 100
        })
    } else {
        json!({
            "readable": true,
            "open_order_count": 0,
            "last_read_at_unix_ms": 1400,
            "now_unix_ms": 1500,
            "max_age_ms": 1000
        })
    };
    let allowed_symbols = if scenario == "symbol_not_allowlisted" {
        json!(["ETHUSDT"])
    } else {
        json!([order_symbol(scenario)])
    };
    let input = json!({
        "schema_version": RISK_PREFLIGHT_INPUT_SCHEMA_VERSION,
        "session": {"state": "running"},
        "market": {
            "symbol": order_symbol(scenario),
            "last_event_at_unix_ms": 1000,
            "now_unix_ms": 1500,
            "max_age_ms": 1000
        },
        "account": account,
        "order_state": order_state,
        "risk": {
            "kill_switch_active": false,
            "allowed_symbols": allowed_symbols
        },
        "order": {
            "symbol": order_symbol(scenario),
            "side": "BUY",
            "order_type": "LIMIT",
            "quantity": "0.001",
            "notional": order_notional(scenario)
        },
        "limits": {
            "max_order_notional": "25.00",
            "current_position_notional": "50.00",
            "max_position_notional": "100.00",
            "max_open_orders": 5,
            "max_clock_skew_ms": 100,
            "observed_clock_skew_ms": 25
        }
    });
    fs::write(path, serde_json::to_string_pretty(&input)?)?;
    Ok(())
}

fn normalized_mutation_event(
    input_event: &Value,
    scenario: &str,
    source_artifact: &str,
    artifact: &Value,
) -> Result<Value, Box<dyn Error>> {
    let mut payload = Map::new();
    payload.insert("scenario".to_string(), json!(scenario));
    payload.insert("source_artifact".to_string(), json!(source_artifact));
    payload.insert(
        "status".to_string(),
        json!(required_string(artifact, "status")?),
    );
    payload.insert(
        "decision".to_string(),
        json!(artifact_decision(source_artifact, artifact)?),
    );
    payload.insert(
        "blocking_reasons".to_string(),
        json!(blocking_reasons(scenario, artifact)?),
    );
    payload.insert(
        "missing_cli_flags".to_string(),
        json!(string_array_or_empty(artifact, "missing_cli_flags")?),
    );

    for key in [
        "request_preview_built",
        "dry_run_execution_adapter_called",
        "production_order_submission_allowed",
        "production_order_mutation_allowed",
        "network_attempted",
        "dashboard_order_controls_enabled",
        "request_sent",
        "production_adapter_called",
    ] {
        payload.insert(key.to_string(), json!(bool_or_false(artifact, key)));
    }
    payload.insert(
        "runtime_gate_open".to_string(),
        json!(
            bool_or_false(artifact, "runtime_gate_open")
                || bool_or_false(artifact, "kill_switch_runtime_gate_open")
        ),
    );
    payload.insert(
        "production_orders_submitted".to_string(),
        json!(u64_or_zero(artifact, "production_orders_submitted")),
    );
    payload.insert(
        "production_order_mutations_attempted".to_string(),
        json!(u64_or_zero(
            artifact,
            "production_order_mutations_attempted"
        )),
    );
    payload.insert(
        "execution_adapter_called".to_string(),
        json!(production_execution_adapter_called(artifact)),
    );

    Ok(json!({
        "event_type": "execution.live_alpha_mutation_dry_run.preflight",
        "ts_event": "0",
        "ts_init": "0",
        "instrument_id": string_field(input_event, "instrument_id")?,
        "venue": string_field(input_event, "venue")?,
        "correlation_id": string_field(input_event, "correlation_id")?,
        "payload": Value::Object(payload),
    }))
}

fn artifact_decision(source_artifact: &str, artifact: &Value) -> Result<String, Box<dyn Error>> {
    let field = match source_artifact {
        "risk_preflight" => "risk_decision",
        "kill_switch_runtime_gate" => "runtime_gate_decision",
        "execution_dry_run" => "execution_decision",
        "order_gate" | "request_preview" => "status",
        other => return Err(format!("unsupported source artifact: {other}").into()),
    };
    required_string(artifact, field)
}

fn blocking_reasons(scenario: &str, artifact: &Value) -> Result<Vec<String>, Box<dyn Error>> {
    let source = string_array_or_empty(artifact, "reasons")?
        .into_iter()
        .chain(string_array_or_empty(artifact, "runtime_gate_reasons")?)
        .chain(string_array_or_empty(
            artifact,
            "manual_approval_lifecycle_issues",
        )?)
        .chain(string_array_or_empty(artifact, "source_artifact_issues")?)
        .chain(string_array_or_empty(artifact, "missing_cli_flags")?)
        .collect::<Vec<_>>();

    let required = match scenario {
        "kill_switch_active" => Some("kill_switch_active"),
        "missing_approval" => Some("manual_approval_missing_or_not_approved"),
        "approval_expired" => Some("manual_approval_expired"),
        "symbol_not_allowlisted" => Some("symbol_not_allowlisted"),
        "notional_too_large" => Some("notional_limit_exceeded"),
        "order_state_stale" => Some("order_state_stale"),
        "account_unreadable" => Some("account_read_failed"),
        "dashboard_control_attempted" => Some("--confirm-dashboard-order-controls-disabled"),
        "network_disabled" => None,
        other => return Err(format!("unsupported blocking scenario: {other}").into()),
    };
    if let Some(required_reason) = required {
        if !source.iter().any(|reason| reason == required_reason) {
            return Err(format!(
                "{scenario} must emit blocking reason {required_reason}, got {source:?}"
            )
            .into());
        }
        Ok(vec![required_reason.to_string()])
    } else {
        Ok(Vec::new())
    }
}

fn assert_no_production_mutation_proof(events: &[Value]) -> Result<(), Box<dyn Error>> {
    for event in events {
        let payload = payload(event)?;
        if u64_field(payload, "production_orders_submitted")? != 0 {
            return Err("golden trace recorded production_orders_submitted != 0".into());
        }
        if u64_field(payload, "production_order_mutations_attempted")? != 0 {
            return Err("golden trace recorded production_order_mutations_attempted != 0".into());
        }
        if bool_field(payload, "network_attempted")? {
            return Err("golden trace recorded network_attempted=true".into());
        }
        if bool_field(payload, "execution_adapter_called")? {
            return Err("golden trace recorded execution_adapter_called=true".into());
        }
    }
    Ok(())
}

fn request_run_id(scenario: &str) -> String {
    format!("v150-mutation-request-{scenario}")
}

fn order_symbol(_scenario: &str) -> &'static str {
    "BTCUSDT"
}

fn order_notional(scenario: &str) -> &'static str {
    if scenario == "notional_too_large" {
        "30.00"
    } else {
        "10.00"
    }
}

fn production_execution_adapter_called(artifact: &Value) -> bool {
    bool_or_false(artifact, "execution_adapter_called")
        || bool_or_false(artifact, "real_execution_adapter_called")
        || bool_or_false(artifact, "production_adapter_called")
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

fn read_json(path: &Path) -> Result<Value, Box<dyn Error>> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn string_array_or_empty(value: &Value, key: &str) -> Result<Vec<String>, Box<dyn Error>> {
    match value.get(key) {
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(ToString::to_string)
                    .ok_or_else(|| format!("{key} entries must be strings").into())
            })
            .collect(),
        Some(_) => Err(format!("{key} must be an array").into()),
        None => Ok(Vec::new()),
    }
}

fn required_string(value: &Value, key: &str) -> Result<String, Box<dyn Error>> {
    Ok(value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{key} must be a string"))?
        .to_string())
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
        .ok_or_else(|| format!("{key} must be a u64").into())
}

fn bool_or_false(value: &Value, key: &str) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn u64_or_zero(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
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
        "ntpro-v150-live-alpha-mutation-dry-run-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    Ok(root)
}
