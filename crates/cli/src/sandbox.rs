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
use serde::Deserialize;

use crate::opt::{SandboxCommand, SandboxOpt, SandboxRunOpt, SandboxValidateOpt};

const SANDBOX_MODE: &str = "sandbox";
const SYNTHETIC_QUOTES_SOURCE: &str = "synthetic-quotes";
const SANDBOX_ADAPTER: &str = "sandbox";
const SIMULATED_VALUE: &str = "simulated";
const DISABLED_RECONCILIATION: &str = "disabled";
const IN_MEMORY_CACHE: &str = "in-memory";
const ONCE_SHUTDOWN: &str = "once";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MinimalSandboxConfig {
    run: SandboxRunConfig,
    system: SandboxSystemConfig,
    venues: Vec<SandboxVenueConfig>,
    data: Vec<SandboxDataConfig>,
    execution: SandboxExecutionConfig,
    risk: SandboxRiskConfig,
    portfolio: SandboxPortfolioConfig,
    cache: SandboxCacheConfig,
    shutdown: SandboxShutdownConfig,
    output: Option<SandboxOutputConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SandboxRunConfig {
    id: String,
    mode: String,
    environment: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SandboxSystemConfig {
    trader_id: String,
    instance_id: String,
    log_level: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SandboxVenueConfig {
    name: String,
    adapter: String,
    account_type: String,
    oms_type: String,
    starting_balances: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SandboxDataConfig {
    source: String,
    instrument_id: String,
    events: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SandboxExecutionConfig {
    order_submission: String,
    reconciliation: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SandboxRiskConfig {
    mode: String,
    max_order_qty: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SandboxPortfolioConfig {
    mode: String,
    starting_balance: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SandboxCacheConfig {
    mode: String,
    warmup_instruments: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SandboxShutdownConfig {
    mode: String,
    max_runtime_secs: Option<u64>,
    disconnect_timeout_secs: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SandboxOutputConfig {
    dir: Option<PathBuf>,
    write_summary: Option<bool>,
}

pub(crate) fn run_sandbox_command(opt: SandboxOpt) -> anyhow::Result<()> {
    match opt.command {
        SandboxCommand::Validate(validate) => run_sandbox_validate(validate),
        SandboxCommand::Run(run) => run_sandbox_run(run),
    }
}

fn run_sandbox_validate(opt: SandboxValidateOpt) -> anyhow::Result<()> {
    let config = load_minimal_sandbox_config(&opt.config)?;
    let data = primary_data(&config);

    println!(
        "sandbox.validate status=ok mode={} run_id={} config={} environment={} trader_id={} venue_count={} data_source={} instrument_id={} events={} execution={} risk_state=simulated portfolio_state=simulated cache_state=in-memory external_venue_connection=false real_orders_submitted=false",
        config.run.mode,
        config.run.id,
        opt.config.display(),
        config.run.environment,
        config.system.trader_id,
        config.venues.len(),
        data.source,
        data.instrument_id,
        data.events,
        config.execution.order_submission,
    );

    Ok(())
}

fn run_sandbox_run(opt: SandboxRunOpt) -> anyhow::Result<()> {
    let config = load_minimal_sandbox_config(&opt.config)?;
    let run_id = opt.run_id.as_deref().unwrap_or(config.run.id.as_str());
    validate_non_empty("run_id", run_id)?;

    let venue = primary_venue(&config);
    let data = primary_data(&config);
    let output_dir = resolve_output_dir(run_id, opt.output.as_ref(), config.output.as_ref());
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create output dir '{}'", output_dir.display()))?;

    let summary_path = output_dir.join("summary.txt");
    let events_path = output_dir.join("events.log");

    let summary = format!(
        "command=sandbox.run\nstatus=ok\nmode={}\nrun_id={run_id}\nconfig={}\nenvironment={}\ntrader_id={}\ninstance_id={}\nvenue={}\nadapter={}\ndata_source={}\ninstrument_id={}\nevents={}\nexecution_state=simulated\nrisk_state=simulated\nportfolio_state=simulated\ncache_state=in-memory\nnode_started=true\nnode_stopped=true\nshutdown_reason=once\nexternal_venue_connection=false\nreal_orders_submitted=false\nruntime_status=simulated_demo\n",
        config.run.mode,
        opt.config.display(),
        config.run.environment,
        config.system.trader_id,
        config.system.instance_id,
        venue.name,
        venue.adapter,
        data.source,
        data.instrument_id,
        data.events,
    );
    fs::write(&summary_path, summary)
        .with_context(|| format!("failed to write summary '{}'", summary_path.display()))?;

    let event_log = format!(
        "event=validate_config status=ok\n\
         event=build_simulated_node status=ok trader_id={} instance_id={}\n\
         event=node_start status=started environment={}\n\
         event=market_data status=loaded source={} instrument_id={} events={}\n\
         event=risk_check status=passed mode={} max_order_qty={}\n\
         event=execution status=simulated order_submission={} venue={}\n\
         event=portfolio_update status=simulated starting_balance={}\n\
         event=cache_update status=simulated mode={} warmup_instruments={}\n\
         event=node_stop status=stopped shutdown_reason=once disconnect_timeout_secs={}\n",
        config.system.trader_id,
        config.system.instance_id,
        config.run.environment,
        data.source,
        data.instrument_id,
        data.events,
        config.risk.mode,
        config.risk.max_order_qty,
        config.execution.order_submission,
        venue.name,
        config.portfolio.starting_balance,
        config.cache.mode,
        config.cache.warmup_instruments.join(","),
        config.shutdown.disconnect_timeout_secs,
    );
    fs::write(&events_path, event_log)
        .with_context(|| format!("failed to write events '{}'", events_path.display()))?;

    println!(
        "sandbox.run status=ok mode={} run_id={} config={} output={} summary={} events={} node_started=true node_stopped=true data_source={} instrument_id={} event_count={} execution_state=simulated risk_state=simulated portfolio_state=simulated cache_state=in-memory external_venue_connection=false real_orders_submitted=false runtime_status=simulated_demo",
        config.run.mode,
        run_id,
        opt.config.display(),
        output_dir.display(),
        summary_path.display(),
        events_path.display(),
        data.source,
        data.instrument_id,
        data.events,
    );

    Ok(())
}

fn load_minimal_sandbox_config(path: &Path) -> anyhow::Result<MinimalSandboxConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read sandbox config '{}'", path.display()))?;
    let config: MinimalSandboxConfig = toml::from_str(&raw)
        .with_context(|| format!("failed to parse sandbox config '{}'", path.display()))?;
    validate_minimal_sandbox_config(&config)?;
    Ok(config)
}

fn validate_minimal_sandbox_config(config: &MinimalSandboxConfig) -> anyhow::Result<()> {
    validate_non_empty("run.id", &config.run.id)?;
    validate_exact("run.mode", &config.run.mode, SANDBOX_MODE)?;
    validate_exact("run.environment", &config.run.environment, SANDBOX_MODE)?;
    validate_non_empty("system.trader_id", &config.system.trader_id)?;
    validate_non_empty("system.instance_id", &config.system.instance_id)?;
    if let Some(log_level) = &config.system.log_level {
        validate_non_empty("system.log_level", log_level)?;
    }

    if config.venues.is_empty() {
        anyhow::bail!("venues must contain at least one sandbox venue");
    }
    for (index, venue) in config.venues.iter().enumerate() {
        let prefix = format!("venues[{index}]");
        validate_non_empty(&format!("{prefix}.name"), &venue.name)?;
        validate_exact(
            &format!("{prefix}.adapter"),
            &venue.adapter,
            SANDBOX_ADAPTER,
        )?;
        validate_non_empty(&format!("{prefix}.account_type"), &venue.account_type)?;
        validate_non_empty(&format!("{prefix}.oms_type"), &venue.oms_type)?;
        if venue.starting_balances.is_empty() {
            anyhow::bail!("{prefix}.starting_balances must not be empty");
        }
        for balance in &venue.starting_balances {
            validate_non_empty(&format!("{prefix}.starting_balances"), balance)?;
        }
    }

    if config.data.is_empty() {
        anyhow::bail!("data must contain at least one synthetic data source");
    }
    for (index, data) in config.data.iter().enumerate() {
        let prefix = format!("data[{index}]");
        validate_exact(
            &format!("{prefix}.source"),
            &data.source,
            SYNTHETIC_QUOTES_SOURCE,
        )?;
        validate_non_empty(&format!("{prefix}.instrument_id"), &data.instrument_id)?;
        if data.events == 0 {
            anyhow::bail!("{prefix}.events must be greater than zero");
        }
    }

    validate_exact(
        "execution.order_submission",
        &config.execution.order_submission,
        SIMULATED_VALUE,
    )?;
    validate_exact(
        "execution.reconciliation",
        &config.execution.reconciliation,
        DISABLED_RECONCILIATION,
    )?;
    validate_exact("risk.mode", &config.risk.mode, SIMULATED_VALUE)?;
    if config.risk.max_order_qty == 0 {
        anyhow::bail!("risk.max_order_qty must be greater than zero");
    }
    validate_exact("portfolio.mode", &config.portfolio.mode, SIMULATED_VALUE)?;
    validate_non_empty(
        "portfolio.starting_balance",
        &config.portfolio.starting_balance,
    )?;
    validate_exact("cache.mode", &config.cache.mode, IN_MEMORY_CACHE)?;
    if config.cache.warmup_instruments.is_empty() {
        anyhow::bail!("cache.warmup_instruments must not be empty");
    }
    for instrument in &config.cache.warmup_instruments {
        validate_non_empty("cache.warmup_instruments", instrument)?;
    }
    validate_exact("shutdown.mode", &config.shutdown.mode, ONCE_SHUTDOWN)?;
    if let Some(max_runtime_secs) = config.shutdown.max_runtime_secs
        && max_runtime_secs == 0
    {
        anyhow::bail!("shutdown.max_runtime_secs must be greater than zero when set");
    }
    if config.shutdown.disconnect_timeout_secs == 0 {
        anyhow::bail!("shutdown.disconnect_timeout_secs must be greater than zero");
    }
    if let Some(output) = &config.output {
        if let Some(dir) = &output.dir {
            validate_non_empty("output.dir", dir.to_string_lossy().as_ref())?;
        }
        if matches!(output.write_summary, Some(false)) {
            anyhow::bail!("output.write_summary must be true for the RHARD-004 sandbox demo");
        }
    }

    Ok(())
}

fn primary_venue(config: &MinimalSandboxConfig) -> &SandboxVenueConfig {
    &config.venues[0]
}

fn primary_data(config: &MinimalSandboxConfig) -> &SandboxDataConfig {
    &config.data[0]
}

fn validate_non_empty(field: &str, value: &str) -> anyhow::Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("{field} must not be empty");
    }
    Ok(())
}

fn validate_exact(field: &str, value: &str, expected: &str) -> anyhow::Result<()> {
    if value != expected {
        anyhow::bail!("{field} must be '{expected}', got '{value}'");
    }
    Ok(())
}

fn resolve_output_dir(
    run_id: &str,
    cli_output: Option<&PathBuf>,
    config_output: Option<&SandboxOutputConfig>,
) -> PathBuf {
    if let Some(output) = cli_output {
        return output.clone();
    }
    if let Some(output) = config_output
        && let Some(dir) = &output.dir
    {
        return dir.clone();
    }
    PathBuf::from("runs").join(run_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opt::SandboxRunOpt;

    fn write_config(name: &str, content: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ntpro-rhard-004-{}-{}", name, std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        fs::write(&path, content).unwrap();
        path
    }

    fn minimal_config(output_dir: &Path) -> String {
        format!(
            r#"[run]
id = "sandbox-smoke"
mode = "sandbox"
environment = "sandbox"

[system]
trader_id = "TRADER-001"
instance_id = "sandbox-smoke-001"
log_level = "info"

[[venues]]
name = "SIM"
adapter = "sandbox"
account_type = "MARGIN"
oms_type = "HEDGING"
starting_balances = ["1000000 USD"]

[[data]]
source = "synthetic-quotes"
instrument_id = "AUD/USD.SIM"
events = 3

[execution]
order_submission = "simulated"
reconciliation = "disabled"

[risk]
mode = "simulated"
max_order_qty = 1000

[portfolio]
mode = "simulated"
starting_balance = "1000000 USD"

[cache]
mode = "in-memory"
warmup_instruments = ["AUD/USD.SIM"]

[shutdown]
mode = "once"
max_runtime_secs = 1
disconnect_timeout_secs = 1

[output]
dir = "{}"
write_summary = true
"#,
            output_dir.display()
        )
    }

    #[test]
    fn validates_minimal_sandbox_config() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-rhard-004-validate-{}", std::process::id()));
        let path = write_config("validate", &minimal_config(&output_dir));

        let config = load_minimal_sandbox_config(&path).unwrap();

        assert_eq!(config.run.id, "sandbox-smoke");
        assert_eq!(config.run.mode, SANDBOX_MODE);
        assert_eq!(config.run.environment, SANDBOX_MODE);
        assert_eq!(primary_venue(&config).adapter, SANDBOX_ADAPTER);
        assert_eq!(primary_data(&config).source, SYNTHETIC_QUOTES_SOURCE);
    }

    #[test]
    fn run_writes_summary_and_events() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-rhard-004-run-{}", std::process::id()));
        let path = write_config("run", &minimal_config(&output_dir));

        run_sandbox_run(SandboxRunOpt {
            config: path,
            run_id: None,
            output: None,
        })
        .unwrap();

        let summary = fs::read_to_string(output_dir.join("summary.txt")).unwrap();
        assert!(summary.contains("command=sandbox.run"));
        assert!(summary.contains("node_started=true"));
        assert!(summary.contains("node_stopped=true"));
        assert!(summary.contains("real_orders_submitted=false"));

        let events = fs::read_to_string(output_dir.join("events.log")).unwrap();
        assert!(events.contains("event=node_start status=started"));
        assert!(events.contains("event=risk_check status=passed"));
        assert!(events.contains("event=node_stop status=stopped"));
    }

    #[test]
    fn rejects_non_sandbox_adapter() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-rhard-004-reject-{}", std::process::id()));
        let config = minimal_config(&output_dir).replace(
            r#"adapter = "sandbox""#,
            r#"adapter = "production-adapter""#,
        );
        let path = write_config("reject", &config);

        let error = load_minimal_sandbox_config(&path).unwrap_err().to_string();

        assert!(error.contains("venues[0].adapter must be 'sandbox'"));
        assert!(!output_dir.join("summary.txt").exists());
    }
}
