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
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::{
    backtest::validate_minimal_backtest_config_file,
    data::validate_data_catalog_config_file,
    opt::{ConfigCommand, ConfigKind, ConfigOpt, ConfigValidateOpt},
    sandbox::validate_minimal_sandbox_config_file,
};

pub(crate) fn run_config_command(opt: ConfigOpt) -> anyhow::Result<()> {
    match opt.command {
        ConfigCommand::Validate(validate) => run_config_validate(&validate),
    }
}

fn run_config_validate(opt: &ConfigValidateOpt) -> anyhow::Result<()> {
    validate_workflow_config(opt.kind, &opt.config)?;

    if let Some(output_dir) = &opt.output {
        write_config_validation_artifact(opt.kind, &opt.config, output_dir)?;
    }

    println!(
        "config.validate status=ok kind={} config={}",
        config_kind_label(opt.kind),
        opt.config.display()
    );
    Ok(())
}

fn validate_workflow_config(kind: ConfigKind, path: &Path) -> anyhow::Result<()> {
    match kind {
        ConfigKind::Backtest => validate_minimal_backtest_config_file(path),
        ConfigKind::Sandbox => validate_minimal_sandbox_config_file(path),
        ConfigKind::Live => validate_live_smoke_config(path),
        ConfigKind::Data => validate_data_catalog_config_file(path),
    }
}

fn validate_live_smoke_config(path: &Path) -> anyhow::Result<()> {
    let value = parse_toml_config(path, "live")?;
    let run = required_table(&value, "run")?;
    validate_required_string(run, "run.id")?;
    validate_one_of(run, "run.mode", &["live", "live-init-smoke"])?;
    validate_required_string(run, "run.environment")?;

    let system = required_table(&value, "system")?;
    validate_required_string(system, "system.trader_id")?;
    validate_any_string(system, &["system.instance_id", "system.node_name"])?;

    if value.get("adapter").is_none() && value.get("venues").is_none() {
        anyhow::bail!("live config must declare either [adapter] or [[venues]]");
    }
    if let Some(adapter) = value.get("adapter").and_then(toml::Value::as_table) {
        validate_required_string(adapter, "adapter.name")?;
        validate_required_string(adapter, "adapter.kind")?;
        validate_required_string(adapter, "adapter.venue")?;
    }
    if let Some(venues) = value.get("venues").and_then(toml::Value::as_array) {
        if venues.is_empty() {
            anyhow::bail!("venues must contain at least one live venue");
        }
        for (index, venue) in venues.iter().enumerate() {
            let table = venue
                .as_table()
                .with_context(|| format!("venues[{index}] must be a table"))?;
            validate_required_string(table, &format!("venues[{index}].name"))?;
            validate_required_string(table, &format!("venues[{index}].adapter"))?;
        }
    }

    let execution = required_table(&value, "execution")?;
    if let Some(order_submission) = execution.get("order_submission") {
        validate_string_value("execution.order_submission", order_submission, "disabled")?;
    }
    if let Some(allow_order_submission) = execution.get("allow_order_submission") {
        validate_bool_value(
            "execution.allow_order_submission",
            allow_order_submission,
            false,
        )?;
    }
    if let Some(external_connection) = execution.get("external_venue_connection") {
        validate_bool_value(
            "execution.external_venue_connection",
            external_connection,
            false,
        )?;
    }

    required_table(&value, "shutdown")?;
    Ok(())
}

fn parse_toml_config(path: &Path, kind: &str) -> anyhow::Result<toml::Value> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read {kind} config '{}'", path.display()))?;
    let value: toml::Value = toml::from_str(&raw)
        .with_context(|| format!("failed to parse {kind} config '{}'", path.display()))?;
    if value.as_table().is_none_or(toml::value::Table::is_empty) {
        anyhow::bail!("{kind} config must contain at least one TOML table");
    }
    Ok(value)
}

fn required_table<'a>(
    value: &'a toml::Value,
    name: &str,
) -> anyhow::Result<&'a toml::value::Table> {
    value
        .get(name)
        .and_then(toml::Value::as_table)
        .with_context(|| format!("{name} section is required"))
}

fn validate_required_string(table: &toml::value::Table, field: &str) -> anyhow::Result<()> {
    let (_, key) = field
        .rsplit_once('.')
        .with_context(|| format!("invalid field path '{field}'"))?;
    let value = table
        .get(key)
        .and_then(toml::Value::as_str)
        .with_context(|| format!("{field} must be a string"))?;
    if value.trim().is_empty() {
        anyhow::bail!("{field} must not be empty");
    }
    Ok(())
}

fn validate_any_string(table: &toml::value::Table, fields: &[&str]) -> anyhow::Result<()> {
    for field in fields {
        let Some((_, key)) = field.rsplit_once('.') else {
            continue;
        };
        if table
            .get(key)
            .and_then(toml::Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
        {
            return Ok(());
        }
    }
    anyhow::bail!("one of {} must be a non-empty string", fields.join(", "))
}

fn validate_one_of(
    table: &toml::value::Table,
    field: &str,
    allowed: &[&str],
) -> anyhow::Result<()> {
    let (_, key) = field
        .rsplit_once('.')
        .with_context(|| format!("invalid field path '{field}'"))?;
    let value = table
        .get(key)
        .and_then(toml::Value::as_str)
        .with_context(|| format!("{field} must be a string"))?;
    if !allowed.contains(&value) {
        anyhow::bail!(
            "{field} must be one of {}, got '{value}'",
            allowed.join(", ")
        );
    }
    Ok(())
}

fn validate_string_value(field: &str, value: &toml::Value, expected: &str) -> anyhow::Result<()> {
    let actual = value
        .as_str()
        .with_context(|| format!("{field} must be a string"))?;
    if actual != expected {
        anyhow::bail!("{field} must be '{expected}', got '{actual}'");
    }
    Ok(())
}

fn validate_bool_value(field: &str, value: &toml::Value, expected: bool) -> anyhow::Result<()> {
    let actual = value
        .as_bool()
        .with_context(|| format!("{field} must be a bool"))?;
    if actual != expected {
        anyhow::bail!("{field} must be {expected}, got {actual}");
    }
    Ok(())
}

fn write_config_validation_artifact(
    kind: ConfigKind,
    config: &Path,
    output_dir: &PathBuf,
) -> anyhow::Result<()> {
    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create output dir '{}'", output_dir.display()))?;
    let path = output_dir.join("validation.txt");
    let summary = format!(
        "command=config.validate\nstatus=ok\nkind={}\nconfig={}\n",
        config_kind_label(kind),
        config.display()
    );
    fs::write(&path, summary)
        .with_context(|| format!("failed to write validation artifact '{}'", path.display()))?;
    Ok(())
}

fn config_kind_label(kind: ConfigKind) -> &'static str {
    match kind {
        ConfigKind::Backtest => "backtest",
        ConfigKind::Sandbox => "sandbox",
        ConfigKind::Live => "live",
        ConfigKind::Data => "data",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ntpro-gh-155-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_config(dir: &Path, content: &str) -> PathBuf {
        let path = dir.join("config.toml");
        fs::write(&path, content).unwrap();
        path
    }

    fn minimal_backtest_config(output_dir: &Path) -> String {
        format!(
            r#"[run]
id = "minimal-backtest-dry-run"
mode = "dry-run"

[data]
source = "synthetic-quotes"
instrument_id = "AUD/USD.SIM"
quotes = 3

[strategy]
name = "no-op"

[output]
dir = "{}"
"#,
            output_dir.display()
        )
    }

    fn minimal_live_config() -> &'static str {
        r#"[run]
id = "live-init-smoke"
mode = "live-init-smoke"
environment = "sandbox"

[system]
trader_id = "LIVE-INIT-001"
node_name = "LiveInitSmoke"

[adapter]
name = "SANDBOX"
kind = "sandbox-simulated-execution"
venue = "SANDBOX"

[execution]
order_submission = "disabled"
external_venue_connection = false

[shutdown]
mode = "start-stop"
"#
    }

    fn minimal_data_config() -> &'static str {
        r#"[run]
id = "catalog-audit"
mode = "inspect"

[catalog]
path = "catalog/backtests/ema-cross"
protocol = "file"

[[queries]]
data_type = "QuoteTick"
instrument_id = "AUD/USD.SIM"
"#
    }

    #[test]
    fn config_validate_backtest_writes_artifact() {
        let dir = temp_dir("backtest");
        let output_dir = dir.join("validation");
        let config = write_config(&dir, &minimal_backtest_config(&dir.join("runs")));

        run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::Backtest,
                config: config.clone(),
                output: Some(output_dir.clone()),
            }),
        })
        .unwrap();

        let artifact = fs::read_to_string(output_dir.join("validation.txt")).unwrap();
        assert!(artifact.contains("command=config.validate"));
        assert!(artifact.contains("status=ok"));
        assert!(artifact.contains("kind=backtest"));
        assert!(artifact.contains(&format!("config={}", config.display())));
    }

    #[test]
    fn config_validate_live_accepts_smoke_config() {
        let dir = temp_dir("live");
        let config = write_config(&dir, minimal_live_config());

        run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::Live,
                config,
                output: None,
            }),
        })
        .unwrap();
    }

    #[test]
    fn config_validate_data_accepts_query_config() {
        let dir = temp_dir("data");
        let config = write_config(&dir, minimal_data_config());

        run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::Data,
                config,
                output: None,
            }),
        })
        .unwrap();
    }

    #[test]
    fn config_validate_missing_file_reports_read_error() {
        let dir = temp_dir("missing");
        let config = dir.join("missing.toml");

        let error = run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::Data,
                config,
                output: None,
            }),
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("failed to read data config"));
    }

    #[test]
    fn config_validate_data_rejects_missing_queries() {
        let dir = temp_dir("data-missing-queries");
        let config = write_config(
            &dir,
            r#"[run]
id = "catalog-audit"
mode = "inspect"

[catalog]
path = "catalog/backtests/ema-cross"
protocol = "file"
"#,
        );

        let error = run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::Data,
                config,
                output: None,
            }),
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("queries must contain at least one data query"));
    }
}
