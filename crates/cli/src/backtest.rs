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
use nautilus_backtest::{
    config::{BacktestEngineConfig, SimulatedVenueConfig},
    engine::BacktestEngine,
};
use nautilus_model::{
    data::{Data, QuoteTick},
    enums::{AccountType, BookType, OmsType},
    identifiers::{InstrumentId, Venue},
    instruments::{Instrument, InstrumentAny, stubs::audusd_sim},
    types::{Money, Price, Quantity},
};
use nautilus_trading::examples::strategies::EmaCross;
use serde::Deserialize;

use crate::opt::{BacktestCommand, BacktestOpt, BacktestRunOpt, BacktestValidateOpt};

const DRY_RUN_MODE: &str = "dry-run";
const ENGINE_SMOKE_MODE: &str = "engine-smoke";
const SYNTHETIC_QUOTES_SOURCE: &str = "synthetic-quotes";
const NO_OP_STRATEGY: &str = "no-op";
const EMA_CROSS_STRATEGY: &str = "ema-cross";
const AUDUSD_SIM_INSTRUMENT_ID: &str = "AUD/USD.SIM";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MinimalBacktestConfig {
    run: MinimalRunConfig,
    data: MinimalDataConfig,
    strategy: MinimalStrategyConfig,
    output: Option<MinimalOutputConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MinimalRunConfig {
    id: String,
    mode: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MinimalDataConfig {
    source: String,
    instrument_id: String,
    quotes: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MinimalStrategyConfig {
    name: String,
    trade_size: Option<String>,
    fast_period: Option<usize>,
    slow_period: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MinimalOutputConfig {
    dir: Option<PathBuf>,
}

pub(crate) fn run_backtest_command(opt: BacktestOpt) -> anyhow::Result<()> {
    match opt.command {
        BacktestCommand::Validate(validate) => run_backtest_validate(&validate),
        BacktestCommand::Run(run) => run_backtest_run(&run),
    }
}

fn run_backtest_validate(opt: &BacktestValidateOpt) -> anyhow::Result<()> {
    let config = load_minimal_backtest_config(&opt.config)?;

    println!(
        "backtest.validate status=ok mode={} run_id={} config={} input={} instrument_id={} quotes={} strategy={}",
        config.run.mode,
        config.run.id,
        opt.config.display(),
        config.data.source,
        config.data.instrument_id,
        config.data.quotes,
        config.strategy.name,
    );

    Ok(())
}

pub(crate) fn validate_minimal_backtest_config_file(path: &Path) -> anyhow::Result<()> {
    load_minimal_backtest_config(path)?;
    Ok(())
}

fn run_backtest_run(opt: &BacktestRunOpt) -> anyhow::Result<()> {
    let config = load_minimal_backtest_config(&opt.config)?;

    if opt.dry_run {
        return run_backtest_dry_run(opt, &config);
    }

    run_backtest_engine_smoke(opt, &config)
}

fn run_backtest_dry_run(
    opt: &BacktestRunOpt,
    config: &MinimalBacktestConfig,
) -> anyhow::Result<()> {
    validate_exact("run.mode", &config.run.mode, DRY_RUN_MODE)?;

    let run_id = opt.run_id.as_deref().unwrap_or(config.run.id.as_str());
    validate_non_empty("run_id", run_id)?;
    let output_dir = resolve_output_dir(run_id, opt.output.as_ref(), config.output.as_ref());
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create output dir '{}'", output_dir.display()))?;

    let summary_path = output_dir.join("summary.txt");
    let summary = format!(
        "command=backtest.run\nstatus=ok\nmode={}\nrun_id={run_id}\nconfig={}\ninput={}\ninstrument_id={}\nquotes={}\nstrategy={}\nengine_started=false\nruntime_status=deferred\n",
        config.run.mode,
        opt.config.display(),
        config.data.source,
        config.data.instrument_id,
        config.data.quotes,
        config.strategy.name,
    );
    fs::write(&summary_path, summary)
        .with_context(|| format!("failed to write summary '{}'", summary_path.display()))?;

    println!(
        "backtest.run status=ok mode={} run_id={} config={} input={} instrument_id={} quotes={} strategy={} output={} summary={} engine_started=false runtime_status=deferred",
        config.run.mode,
        run_id,
        opt.config.display(),
        config.data.source,
        config.data.instrument_id,
        config.data.quotes,
        config.strategy.name,
        output_dir.display(),
        summary_path.display(),
    );

    Ok(())
}

fn run_backtest_engine_smoke(
    opt: &BacktestRunOpt,
    config: &MinimalBacktestConfig,
) -> anyhow::Result<()> {
    validate_exact("run.mode", &config.run.mode, ENGINE_SMOKE_MODE)?;
    validate_exact("strategy.name", &config.strategy.name, EMA_CROSS_STRATEGY)?;
    validate_exact(
        "data.instrument_id",
        &config.data.instrument_id,
        AUDUSD_SIM_INSTRUMENT_ID,
    )?;

    let run_id = opt.run_id.as_deref().unwrap_or(config.run.id.as_str());
    validate_non_empty("run_id", run_id)?;
    let output_dir = resolve_output_dir(run_id, opt.output.as_ref(), config.output.as_ref());
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create output dir '{}'", output_dir.display()))?;

    let strategy = resolve_ema_cross_strategy(&config.strategy)?;
    let quotes_loaded = run_ema_cross_engine(config, &strategy)?;

    let summary_path = output_dir.join("summary.txt");
    let summary = format!(
        "command=backtest.run\nstatus=ok\nmode={}\nrun_id={run_id}\nconfig={}\ninput={}\ninstrument_id={}\nquotes_loaded={quotes_loaded}\nstrategy={}\nfast_period={}\nslow_period={}\nengine_started=true\nruntime_status=completed\n",
        config.run.mode,
        opt.config.display(),
        config.data.source,
        config.data.instrument_id,
        config.strategy.name,
        strategy.fast_period,
        strategy.slow_period,
    );
    fs::write(&summary_path, summary)
        .with_context(|| format!("failed to write summary '{}'", summary_path.display()))?;

    println!(
        "backtest.run status=ok mode={} run_id={} config={} input={} instrument_id={} quotes_loaded={} strategy={} output={} summary={} engine_started=true runtime_status=completed",
        config.run.mode,
        run_id,
        opt.config.display(),
        config.data.source,
        config.data.instrument_id,
        quotes_loaded,
        config.strategy.name,
        output_dir.display(),
        summary_path.display(),
    );

    Ok(())
}

#[derive(Debug)]
struct EmaCrossStrategySettings {
    trade_size: Quantity,
    fast_period: usize,
    slow_period: usize,
}

fn resolve_ema_cross_strategy(
    config: &MinimalStrategyConfig,
) -> anyhow::Result<EmaCrossStrategySettings> {
    let fast_period = config.fast_period.unwrap_or(10);
    let slow_period = config.slow_period.unwrap_or(20);
    if fast_period == 0 {
        anyhow::bail!("strategy.fast_period must be greater than zero");
    }
    if slow_period == 0 {
        anyhow::bail!("strategy.slow_period must be greater than zero");
    }
    if fast_period >= slow_period {
        anyhow::bail!("strategy.fast_period must be less than strategy.slow_period");
    }

    let trade_size = Quantity::from(config.trade_size.as_deref().unwrap_or("100000"));
    Ok(EmaCrossStrategySettings {
        trade_size,
        fast_period,
        slow_period,
    })
}

fn run_ema_cross_engine(
    config: &MinimalBacktestConfig,
    strategy: &EmaCrossStrategySettings,
) -> anyhow::Result<usize> {
    let mut engine = BacktestEngine::new(BacktestEngineConfig::default())?;

    engine.add_venue(
        SimulatedVenueConfig::builder()
            .venue(Venue::from("SIM"))
            .oms_type(OmsType::Hedging)
            .account_type(AccountType::Margin)
            .book_type(BookType::L1_MBP)
            .starting_balances(vec![Money::from("1_000_000 USD")])
            .build(),
    )?;

    let instrument = InstrumentAny::CurrencyPair(audusd_sim());
    let instrument_id = instrument.id();
    engine.add_instrument(&instrument)?;

    engine.add_strategy(EmaCross::new(
        instrument_id,
        strategy.trade_size,
        strategy.fast_period,
        strategy.slow_period,
    ))?;

    let quotes = generate_quotes(instrument_id, config.data.quotes);
    let quotes_loaded = quotes.len();
    engine.add_data(quotes, None, true, true)?;
    engine.run(None, None, None, false)?;

    Ok(quotes_loaded)
}

fn quote(instrument_id: InstrumentId, bid: &str, ask: &str, ts: u64) -> Data {
    Data::Quote(QuoteTick::new(
        instrument_id,
        Price::from(bid),
        Price::from(ask),
        Quantity::from("100000"),
        Quantity::from("100000"),
        ts.into(),
        ts.into(),
    ))
}

fn generate_quotes(instrument_id: InstrumentId, requested: usize) -> Vec<Data> {
    let spread = 0.00020;
    let base_ts: u64 = 1_735_689_600_000_000_000;
    let interval: u64 = 1_000_000_000;
    let mut quotes = Vec::with_capacity(requested);

    for tick in 0..requested {
        let cycle = tick as f64 / 12.0;
        let mid = 0.65000 + (cycle.sin() * 0.00400) + ((tick % 40) as f64 * 0.00008);
        let bid = format!("{mid:.5}");
        let ask = format!("{:.5}", mid + spread);
        quotes.push(quote(
            instrument_id,
            &bid,
            &ask,
            base_ts + tick as u64 * interval,
        ));
    }

    quotes
}

fn load_minimal_backtest_config(path: &Path) -> anyhow::Result<MinimalBacktestConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read backtest config '{}'", path.display()))?;
    let config: MinimalBacktestConfig = toml::from_str(&raw)
        .with_context(|| format!("failed to parse backtest config '{}'", path.display()))?;
    validate_minimal_backtest_config(&config)?;
    Ok(config)
}

fn validate_minimal_backtest_config(config: &MinimalBacktestConfig) -> anyhow::Result<()> {
    validate_non_empty("run.id", &config.run.id)?;
    validate_one_of(
        "run.mode",
        &config.run.mode,
        &[DRY_RUN_MODE, ENGINE_SMOKE_MODE],
    )?;
    validate_exact("data.source", &config.data.source, SYNTHETIC_QUOTES_SOURCE)?;
    validate_non_empty("data.instrument_id", &config.data.instrument_id)?;
    if config.data.quotes == 0 {
        anyhow::bail!("data.quotes must be greater than zero");
    }
    if config.run.mode == ENGINE_SMOKE_MODE && config.data.quotes < 30 {
        anyhow::bail!("data.quotes must be at least 30 for engine-smoke mode");
    }
    validate_one_of(
        "strategy.name",
        &config.strategy.name,
        &[NO_OP_STRATEGY, EMA_CROSS_STRATEGY],
    )?;
    if config.run.mode == DRY_RUN_MODE {
        validate_exact("strategy.name", &config.strategy.name, NO_OP_STRATEGY)?;
    } else {
        validate_exact("strategy.name", &config.strategy.name, EMA_CROSS_STRATEGY)?;
        let strategy = resolve_ema_cross_strategy(&config.strategy)?;
        if config.data.quotes <= strategy.slow_period {
            anyhow::bail!("data.quotes must be greater than strategy.slow_period");
        }
    }
    if let Some(output) = &config.output
        && let Some(dir) = &output.dir
    {
        validate_non_empty("output.dir", dir.to_string_lossy().as_ref())?;
    }
    Ok(())
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

fn validate_one_of(field: &str, value: &str, expected: &[&str]) -> anyhow::Result<()> {
    if !expected.contains(&value) {
        anyhow::bail!(
            "{field} must be one of {}, got '{value}'",
            expected.join(", ")
        );
    }
    Ok(())
}

fn resolve_output_dir(
    run_id: &str,
    cli_output: Option<&PathBuf>,
    config_output: Option<&MinimalOutputConfig>,
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
    use crate::opt::BacktestRunOpt;

    fn write_config(name: &str, content: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ntpro-rhard-006-{}-{}", name, std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        fs::write(&path, content).unwrap();
        path
    }

    fn minimal_config(output_dir: &Path) -> String {
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

    fn engine_smoke_config(output_dir: &Path) -> String {
        format!(
            r#"[run]
id = "minimal-backtest-engine-smoke"
mode = "engine-smoke"

[data]
source = "synthetic-quotes"
instrument_id = "AUD/USD.SIM"
quotes = 120

[strategy]
name = "ema-cross"
trade_size = "100000"
fast_period = 10
slow_period = 20

[output]
dir = "{}"
"#,
            output_dir.display()
        )
    }

    #[test]
    fn validates_minimal_backtest_config() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-rhard-006-validate-{}", std::process::id()));
        let path = write_config("validate", &minimal_config(&output_dir));

        let config = load_minimal_backtest_config(&path).unwrap();

        assert_eq!(config.run.id, "minimal-backtest-dry-run");
        assert_eq!(config.run.mode, DRY_RUN_MODE);
        assert_eq!(config.data.source, SYNTHETIC_QUOTES_SOURCE);
        assert_eq!(config.data.instrument_id, "AUD/USD.SIM");
        assert_eq!(config.data.quotes, 3);
        assert_eq!(config.strategy.name, NO_OP_STRATEGY);
    }

    #[test]
    fn run_dry_run_writes_summary() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-rhard-006-run-{}", std::process::id()));
        let path = write_config("run", &minimal_config(&output_dir));

        run_backtest_run(&BacktestRunOpt {
            config: path,
            run_id: None,
            output: None,
            dry_run: true,
        })
        .unwrap();

        let summary = fs::read_to_string(output_dir.join("summary.txt")).unwrap();
        assert!(summary.contains("command=backtest.run"));
        assert!(summary.contains("status=ok"));
        assert!(summary.contains("engine_started=false"));
        assert!(summary.contains("runtime_status=deferred"));
    }

    #[test]
    fn run_without_dry_run_rejects_dry_run_mode() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-rhard-006-blocker-{}", std::process::id()));
        let path = write_config("blocker", &minimal_config(&output_dir));

        let error = run_backtest_run(&BacktestRunOpt {
            config: path,
            run_id: None,
            output: None,
            dry_run: false,
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("run.mode must be 'engine-smoke'"));
        assert!(!output_dir.join("summary.txt").exists());
    }

    #[test]
    fn run_without_dry_run_executes_engine_smoke() {
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-drg-005-engine-{}", std::process::id()));
        let path = write_config("engine", &engine_smoke_config(&output_dir));

        run_backtest_run(&BacktestRunOpt {
            config: path,
            run_id: None,
            output: None,
            dry_run: false,
        })
        .unwrap();

        let summary = fs::read_to_string(output_dir.join("summary.txt")).unwrap();
        assert!(summary.contains("command=backtest.run"));
        assert!(summary.contains("mode=engine-smoke"));
        assert!(summary.contains("engine_started=true"));
        assert!(summary.contains("runtime_status=completed"));
    }
}
