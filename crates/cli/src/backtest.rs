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
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{ErrorKind, Read, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::Context;
use aws_lc_rs::digest::{SHA256, digest};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use nautilus_backtest::result::BacktestResult;
use nautilus_backtest::{
    config::{BacktestEngineConfig, SimulatedVenueConfig},
    engine::BacktestEngine,
};
use nautilus_model::{
    data::{Data, QuoteTick},
    enums::{AccountType, BookType, OmsType},
    identifiers::{InstrumentId, Venue},
    instruments::{
        Instrument, InstrumentAny,
        stubs::{audusd_sim, currency_pair_btcusdt},
    },
    types::{Money, Price, Quantity},
};
use nautilus_trading::examples::strategies::EmaCross;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::artifacts::atomic_write_json;
use crate::{
    artifacts::atomic_write_text,
    opt::{BacktestCommand, BacktestOpt, BacktestRunOpt, BacktestValidateOpt},
};

const DRY_RUN_MODE: &str = "dry-run";
const ENGINE_SMOKE_MODE: &str = "engine-smoke";
const SYNTHETIC_QUOTES_SOURCE: &str = "synthetic-quotes";
const NO_OP_STRATEGY: &str = "no-op";
const EMA_CROSS_STRATEGY: &str = "ema-cross";
const AUDUSD_SIM_INSTRUMENT_ID: &str = "AUD/USD.SIM";
const BTCUSDT_BINANCE_INSTRUMENT_ID: &str = "BTCUSDT.BINANCE";
const BACKTEST_RESULT_SCHEMA_VERSION: &str = "ntpro.backtest_result.v1";
static IMMUTABLE_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MinimalBacktestConfig {
    run: MinimalRunConfig,
    data: MinimalDataConfig,
    strategy: MinimalStrategyConfig,
    product: Option<MinimalProductConfig>,
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MinimalProductConfig {
    strategy_id: String,
    strategy_version_id: String,
    strategy_version_content_hash: String,
    data_ref: String,
    config_ref: String,
    result_ref: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestResultArtifact {
    schema_version: String,
    run_id: String,
    strategy_id: String,
    strategy_version_id: String,
    strategy_version_content_hash: String,
    data_ref: String,
    data_sha256: String,
    config_ref: String,
    config_sha256: String,
    result_ref: String,
    instrument_id: String,
    strategy: String,
    parameters: BacktestParameters,
    backtest_start: String,
    backtest_end: String,
    metrics: BacktestMetrics,
    boundaries: BacktestResultBoundaries,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestParameters {
    trade_size: String,
    fast_period: usize,
    slow_period: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestMetrics {
    quotes: usize,
    iterations: usize,
    total_events: usize,
    total_orders: usize,
    total_positions: usize,
    pnl_stats: BTreeMap<String, BTreeMap<String, String>>,
    return_stats: BTreeMap<String, String>,
    general_stats: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestResultBoundaries {
    read_only: bool,
    external_venue_connection: bool,
    order_submission_allowed: bool,
    order_mutation_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    real_orders_submitted: bool,
    trading_controls_enabled: bool,
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
    if config.product.is_some() && run_id != config.run.id {
        anyhow::bail!("product-bound backtest run_id cannot override run.id");
    }
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
    atomic_write_text(&summary_path, &summary)
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
    validate_one_of(
        "data.instrument_id",
        &config.data.instrument_id,
        &[AUDUSD_SIM_INSTRUMENT_ID, BTCUSDT_BINANCE_INSTRUMENT_ID],
    )?;

    let run_id = opt.run_id.as_deref().unwrap_or(config.run.id.as_str());
    validate_non_empty("run_id", run_id)?;
    if config.product.is_some() && run_id != config.run.id {
        anyhow::bail!("product-bound backtest run_id cannot override run.id");
    }
    let output_dir = resolve_output_dir(run_id, opt.output.as_ref(), config.output.as_ref());
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create output dir '{}'", output_dir.display()))?;

    let strategy = resolve_ema_cross_strategy(&config.strategy)?;
    let engine_run = run_ema_cross_engine(config, &strategy)?;
    let quotes_loaded = engine_run.quotes_loaded;

    if let Some(product) = &config.product {
        let config_bytes = fs::read(&opt.config).with_context(|| {
            format!("failed to read backtest config '{}'", opt.config.display())
        })?;
        let artifact = build_backtest_result_artifact(
            run_id,
            config,
            product,
            &strategy,
            &engine_run,
            &sha256_ref(&config_bytes),
        )?;
        write_immutable_result(&output_dir.join("summary.json"), &artifact)?;
    }

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
    atomic_write_text(&summary_path, &summary)
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

struct EmaCrossEngineRun {
    quotes_loaded: usize,
    data_sha256: String,
    result: BacktestResult,
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
) -> anyhow::Result<EmaCrossEngineRun> {
    let mut engine = BacktestEngine::new(BacktestEngineConfig::default())?;

    let (venue, starting_balance, instrument) = match config.data.instrument_id.as_str() {
        AUDUSD_SIM_INSTRUMENT_ID => (
            Venue::from("SIM"),
            Money::from("1_000_000 USD"),
            InstrumentAny::CurrencyPair(audusd_sim()),
        ),
        BTCUSDT_BINANCE_INSTRUMENT_ID => (
            Venue::from("BINANCE"),
            Money::from("1_000_000 USDT"),
            InstrumentAny::CurrencyPair(currency_pair_btcusdt()),
        ),
        value => anyhow::bail!("unsupported data.instrument_id '{value}'"),
    };

    engine.add_venue(
        SimulatedVenueConfig::builder()
            .venue(venue)
            .oms_type(OmsType::Hedging)
            .account_type(AccountType::Margin)
            .book_type(BookType::L1_MBP)
            .starting_balances(vec![starting_balance])
            .build(),
    )?;

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
    let data_sha256 = sha256_ref(
        &serde_json::to_vec(&quotes).context("failed to serialize deterministic backtest input")?,
    );
    engine.add_data(quotes, None, true, true)?;
    engine.run(None, None, None, false)?;

    let result = engine.get_result();

    Ok(EmaCrossEngineRun {
        quotes_loaded,
        data_sha256,
        result,
    })
}

fn quote(instrument_id: InstrumentId, bid: &str, ask: &str, ts: u64) -> Data {
    let size = if instrument_id.to_string() == BTCUSDT_BINANCE_INSTRUMENT_ID {
        Quantity::from("1.000000")
    } else {
        Quantity::from("100000")
    };
    Data::Quote(QuoteTick::new(
        instrument_id,
        Price::from(bid),
        Price::from(ask),
        size,
        size,
        ts.into(),
        ts.into(),
    ))
}

fn generate_quotes(instrument_id: InstrumentId, requested: usize) -> Vec<Data> {
    let is_btcusdt = instrument_id.to_string() == BTCUSDT_BINANCE_INSTRUMENT_ID;
    let (base, spread, amplitude, drift) = if is_btcusdt {
        (50_000.0, 1.0, 400.0, 8.0)
    } else {
        (0.65000, 0.00020, 0.00400, 0.00008)
    };
    let base_ts: u64 = 1_735_689_600_000_000_000;
    let interval: u64 = 1_000_000_000;
    let mut quotes = Vec::with_capacity(requested);

    for tick in 0..requested {
        let cycle = tick as f64 / 12.0;
        let mid = base + (cycle.sin() * amplitude) + ((tick % 40) as f64 * drift);
        let bid = if is_btcusdt {
            format!("{mid:.2}")
        } else {
            format!("{mid:.5}")
        };
        let ask = if is_btcusdt {
            format!("{:.2}", mid + spread)
        } else {
            format!("{:.5}", mid + spread)
        };
        quotes.push(quote(
            instrument_id,
            &bid,
            &ask,
            base_ts + tick as u64 * interval,
        ));
    }

    quotes
}

fn build_backtest_result_artifact(
    run_id: &str,
    config: &MinimalBacktestConfig,
    product: &MinimalProductConfig,
    strategy: &EmaCrossStrategySettings,
    engine_run: &EmaCrossEngineRun,
    config_sha256: &str,
) -> anyhow::Result<BacktestResultArtifact> {
    let result = &engine_run.result;
    let backtest_start = result
        .backtest_start
        .map(|value| value.as_u64().to_string())
        .context("backtest result is missing backtest_start")?;
    let backtest_end = result
        .backtest_end
        .map(|value| value.as_u64().to_string())
        .context("backtest result is missing backtest_end")?;

    Ok(BacktestResultArtifact {
        schema_version: BACKTEST_RESULT_SCHEMA_VERSION.to_string(),
        run_id: run_id.to_string(),
        strategy_id: product.strategy_id.clone(),
        strategy_version_id: product.strategy_version_id.clone(),
        strategy_version_content_hash: product.strategy_version_content_hash.clone(),
        data_ref: product.data_ref.clone(),
        data_sha256: engine_run.data_sha256.clone(),
        config_ref: product.config_ref.clone(),
        config_sha256: config_sha256.to_string(),
        result_ref: product.result_ref.clone(),
        instrument_id: config.data.instrument_id.clone(),
        strategy: config.strategy.name.clone(),
        parameters: BacktestParameters {
            trade_size: strategy.trade_size.to_string(),
            fast_period: strategy.fast_period,
            slow_period: strategy.slow_period,
        },
        backtest_start,
        backtest_end,
        metrics: BacktestMetrics {
            quotes: engine_run.quotes_loaded,
            iterations: result.iterations,
            total_events: result.total_events,
            total_orders: result.total_orders,
            total_positions: result.total_positions,
            pnl_stats: result
                .stats_pnls
                .iter()
                .map(|(currency, stats)| (currency.clone(), canonical_stats(stats.iter())))
                .collect(),
            return_stats: canonical_stats(result.stats_returns.iter()),
            general_stats: canonical_stats(result.stats_general.iter()),
        },
        boundaries: BacktestResultBoundaries {
            read_only: true,
            external_venue_connection: false,
            order_submission_allowed: false,
            order_mutation_allowed: false,
            automatic_retry_allowed: false,
            automatic_remediation_allowed: false,
            real_orders_submitted: false,
            trading_controls_enabled: false,
        },
    })
}

fn canonical_stats<'a>(
    values: impl Iterator<Item = (&'a String, &'a f64)>,
) -> BTreeMap<String, String> {
    values
        .map(|(name, value)| (name.clone(), canonical_float(*value)))
        .collect()
}

fn canonical_float(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.12}")
    } else {
        value.to_string()
    }
}

fn sha256_ref(bytes: &[u8]) -> String {
    let hash = digest(&SHA256, bytes);
    let mut value = String::with_capacity(71);
    value.push_str("sha256:");
    for byte in hash.as_ref() {
        value.push_str(&format!("{byte:02x}"));
    }
    value
}

fn write_immutable_result(path: &Path, artifact: &BacktestResultArtifact) -> anyhow::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .context("immutable result path must have a parent")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create result directory '{}'", parent.display()))?;
    let raw = format!("{}\n", serde_json::to_string_pretty(artifact)?);
    let sequence = IMMUTABLE_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let file_name = path
        .file_name()
        .context("immutable result path must have a file name")?
        .to_string_lossy();
    let temp_path = parent.join(format!(
        ".{file_name}.immutable.{}.{}",
        std::process::id(),
        sequence
    ));
    let write_result = (|| -> anyhow::Result<()> {
        let mut temp = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .with_context(|| {
                format!(
                    "failed to create immutable result temp '{}'",
                    temp_path.display()
                )
            })?;
        temp.write_all(raw.as_bytes()).with_context(|| {
            format!(
                "failed to write immutable result temp '{}'",
                temp_path.display()
            )
        })?;
        temp.sync_all().with_context(|| {
            format!(
                "failed to sync immutable result temp '{}'",
                temp_path.display()
            )
        })?;
        match fs::hard_link(&temp_path, path) {
            Ok(()) => {
                sync_parent_dir_best_effort(path);
                Ok(())
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                let existing = read_verified_result(path)?;
                if existing == *artifact {
                    Ok(())
                } else {
                    anyhow::bail!(
                        "existing backtest result '{}' is immutable and differs",
                        path.display()
                    )
                }
            }
            Err(error) => Err(error).with_context(|| {
                format!(
                    "failed to claim immutable backtest result '{}' from '{}'",
                    path.display(),
                    temp_path.display()
                )
            }),
        }
    })();
    let _ = fs::remove_file(&temp_path);
    write_result
}

fn read_verified_result(path: &Path) -> anyhow::Result<BacktestResultArtifact> {
    let raw = read_result_bytes_nofollow(path)?;
    serde_json::from_slice(&raw)
        .with_context(|| format!("existing result '{}' is invalid", path.display()))
}

fn read_result_bytes_nofollow(path: &Path) -> anyhow::Result<Vec<u8>> {
    use cap_std::fs::{Dir, OpenOptions as CapOpenOptions};

    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .context("immutable result path must have a parent")?;
    let canonical_parent = fs::canonicalize(parent).with_context(|| {
        format!(
            "failed to resolve existing result parent '{}'",
            parent.display()
        )
    })?;
    let (root, components) = absolute_root_and_components(&canonical_parent)?;
    let mut directory = Dir::open_ambient_dir(root, cap_std::ambient_authority())
        .context("failed to open filesystem root")?;
    for name in components {
        directory = directory.open_dir_nofollow(&name).with_context(|| {
            format!(
                "existing result parent '{}' is invalid",
                canonical_parent.display()
            )
        })?;
    }
    let file_name = path
        .file_name()
        .context("immutable result path must have a file name")?;
    let mut options = CapOpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    let file = directory
        .open_with(file_name, &options)
        .with_context(|| format!("failed to open existing result '{}'", path.display()))?;
    if !file.metadata()?.is_file() {
        anyhow::bail!("existing result '{}' is not a regular file", path.display());
    }
    let mut raw = Vec::new();
    file.into_std()
        .read_to_end(&mut raw)
        .with_context(|| format!("failed to read existing result '{}'", path.display()))?;
    Ok(raw)
}

fn absolute_root_and_components(path: &Path) -> anyhow::Result<(PathBuf, Vec<PathBuf>)> {
    let mut source = path.components();
    let mut root = PathBuf::new();
    match source.next() {
        Some(Component::Prefix(prefix)) => {
            root.push(prefix.as_os_str());
            if !matches!(source.next(), Some(Component::RootDir)) {
                anyhow::bail!("path '{}' is not absolute", path.display());
            }
            root.push(Path::new(std::path::MAIN_SEPARATOR_STR));
        }
        Some(Component::RootDir) => root.push(Path::new(std::path::MAIN_SEPARATOR_STR)),
        _ => anyhow::bail!("path '{}' is not absolute", path.display()),
    }
    let mut components = Vec::new();
    for component in source {
        match component {
            Component::Normal(name) => components.push(PathBuf::from(name)),
            _ => anyhow::bail!("path '{}' contains an invalid component", path.display()),
        }
    }
    Ok((root, components))
}

fn sync_parent_dir_best_effort(path: &Path) {
    if let Some(parent) = path.parent()
        && let Ok(directory) = OpenOptions::new().read(true).open(parent)
    {
        let _ = directory.sync_all();
    }
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
    if let Some(product) = &config.product {
        for (field, value) in [
            ("product.strategy_id", product.strategy_id.as_str()),
            (
                "product.strategy_version_id",
                product.strategy_version_id.as_str(),
            ),
            (
                "product.strategy_version_content_hash",
                product.strategy_version_content_hash.as_str(),
            ),
            ("product.data_ref", product.data_ref.as_str()),
            ("product.config_ref", product.config_ref.as_str()),
            ("product.result_ref", product.result_ref.as_str()),
        ] {
            validate_non_empty(field, value)?;
        }
        let expected_result_ref = format!("artifact://backtests/{}/summary.json", config.run.id);
        validate_exact(
            "product.result_ref",
            &product.result_ref,
            &expected_result_ref,
        )?;
        if !product.strategy_version_content_hash.starts_with("sha256:")
            || product.strategy_version_content_hash.len() != 71
        {
            anyhow::bail!("product.strategy_version_content_hash must be a sha256 reference");
        }
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
    use std::collections::BTreeMap;

    use super::*;
    use crate::opt::BacktestRunOpt;
    use serde_json::{Value, json};

    const MVP_EMA_CASE_ID: &str = "mvp.ema_cross_deterministic.001";

    #[cfg(unix)]
    fn create_file_symlink(original: &Path, link: &Path) -> std::io::Result<()> {
        std::os::unix::fs::symlink(original, link)
    }

    #[cfg(windows)]
    fn create_file_symlink(original: &Path, link: &Path) -> std::io::Result<()> {
        std::os::windows::fs::symlink_file(original, link)
    }

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

    #[test]
    fn product_backtest_writes_deterministic_immutable_result() {
        let config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("configs/backtests/ema-cross-btcusdt-product.toml");
        let output_dir =
            std::env::temp_dir().join(format!("ntpro-s1-bt-001-product-{}", std::process::id()));
        let _ = fs::remove_dir_all(&output_dir);
        let opt = BacktestRunOpt {
            config: config_path,
            run_id: None,
            output: Some(output_dir.clone()),
            dry_run: false,
        };

        run_backtest_run(&opt).expect("first product backtest should complete");
        let result_path = output_dir.join("summary.json");
        let first = fs::read(&result_path).expect("product result should exist");
        let artifact: BacktestResultArtifact =
            serde_json::from_slice(&first).expect("product result should parse");
        assert_eq!(artifact.schema_version, BACKTEST_RESULT_SCHEMA_VERSION);
        assert_eq!(artifact.run_id, "ema-cross-btcusdt-baseline-v1");
        assert_eq!(artifact.instrument_id, BTCUSDT_BINANCE_INSTRUMENT_ID);
        assert_eq!(
            artifact.config_sha256,
            "sha256:fb6cbc40cf8e82dc295620243d5cfdc2cf82c89b45fb9097cae2961bbc6d2838"
        );
        assert_eq!(
            artifact.data_sha256,
            "sha256:18ed30b352b17a11c33294df39387976f15a587b859f729ffbe5e59bc9c75d1e"
        );
        assert_eq!(
            sha256_ref(&first),
            "sha256:4b9bc548f226e55b136eb4c08f2ef5e0274bed104b8626d5431b39fb0a3b8760"
        );
        assert_eq!(artifact.metrics.quotes, 120);
        assert_eq!(artifact.metrics.iterations, 120);
        assert!(artifact.metrics.total_orders > 0);
        assert!(artifact.boundaries.read_only);
        assert!(!artifact.boundaries.order_submission_allowed);

        run_backtest_run(&opt).expect("identical replay should be idempotent");
        assert_eq!(
            fs::read(&result_path).expect("replayed result should exist"),
            first
        );

        let mut changed = artifact;
        changed.metrics.quotes += 1;
        atomic_write_json(&result_path, &changed).expect("changed fixture should be written");
        let error = run_backtest_run(&opt)
            .expect_err("different existing result must not be overwritten")
            .to_string();
        assert!(error.contains("is immutable and differs"));
        let _ = fs::remove_dir_all(output_dir);
    }

    #[test]
    fn immutable_result_concurrent_writers_never_overwrite_the_winner() {
        use std::sync::{Arc, Barrier};

        let config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("configs/backtests/ema-cross-btcusdt-product.toml");
        let source_dir = std::env::temp_dir().join(format!(
            "ntpro-s1-bt-001-race-source-{}",
            std::process::id()
        ));
        let race_dir = std::env::temp_dir().join(format!(
            "ntpro-s1-bt-001-race-target-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&source_dir);
        let _ = fs::remove_dir_all(&race_dir);
        run_backtest_run(&BacktestRunOpt {
            config: config_path,
            run_id: None,
            output: Some(source_dir.clone()),
            dry_run: false,
        })
        .expect("source product backtest should complete");
        let first: BacktestResultArtifact = serde_json::from_slice(
            &fs::read(source_dir.join("summary.json")).expect("source result should be readable"),
        )
        .expect("source result should parse");
        let mut second = first.clone();
        second.metrics.total_events += 1;
        fs::create_dir_all(&race_dir).expect("race directory should be created");
        let target = race_dir.join("summary.json");
        let barrier = Arc::new(Barrier::new(3));
        let handles = [first.clone(), second.clone()].map(|artifact| {
            let target = target.clone();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                write_immutable_result(&target, &artifact)
            })
        });
        barrier.wait();
        let results = handles.map(|handle| handle.join().expect("writer thread should finish"));

        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(results.iter().filter(|result| result.is_err()).count(), 1);
        let winner = read_verified_result(&target).expect("winning result should be complete");
        assert!(winner == first || winner == second);
        assert_eq!(
            fs::read_dir(&race_dir)
                .expect("race directory should be readable")
                .filter_map(Result::ok)
                .count(),
            1,
            "temporary files must be removed"
        );
        let _ = fs::remove_dir_all(source_dir);
        let _ = fs::remove_dir_all(race_dir);
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn immutable_result_rejects_existing_symlink_target() {
        let config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("configs/backtests/ema-cross-btcusdt-product.toml");
        let source_dir = std::env::temp_dir().join(format!(
            "ntpro-s1-bt-001-symlink-source-{}",
            std::process::id()
        ));
        let target_dir = std::env::temp_dir().join(format!(
            "ntpro-s1-bt-001-symlink-target-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&source_dir);
        let _ = fs::remove_dir_all(&target_dir);
        run_backtest_run(&BacktestRunOpt {
            config: config_path,
            run_id: None,
            output: Some(source_dir.clone()),
            dry_run: false,
        })
        .expect("source product backtest should complete");
        let artifact: BacktestResultArtifact = serde_json::from_slice(
            &fs::read(source_dir.join("summary.json")).expect("source result should be readable"),
        )
        .expect("source result should parse");
        fs::create_dir_all(&target_dir).expect("target directory should be created");
        let outside = target_dir.join("outside.json");
        fs::write(&outside, b"outside-must-remain-unchanged")
            .expect("outside target should be written");
        let target = target_dir.join("summary.json");
        create_file_symlink(&outside, &target).expect("target symlink should be created");

        let error = write_immutable_result(&target, &artifact)
            .expect_err("an existing symlink must fail closed")
            .to_string();
        assert!(error.contains("failed to open existing result"));
        assert_eq!(
            fs::read(&outside).expect("outside target should remain readable"),
            b"outside-must-remain-unchanged"
        );
        let _ = fs::remove_dir_all(source_dir);
        let _ = fs::remove_dir_all(target_dir);
    }

    #[test]
    fn mvp_ema_strategy_product_path_has_canonical_result() {
        let config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("examples/rust/backtest/minimal_engine_smoke.toml");
        let config = load_minimal_backtest_config(&config_path).unwrap();
        let strategy = resolve_ema_cross_strategy(&config.strategy).unwrap();
        let engine_run = run_ema_cross_engine(&config, &strategy).unwrap();
        let actual = canonical_mvp_ema_result(&config, &engine_run);

        println!(
            "mvp_ema_canonical_result={}",
            serde_json::to_string(&actual).expect("canonical result should serialize")
        );

        let expected = json!({
            "backtest_end": "1735689719000000000",
            "backtest_start": "1735689600000000000",
            "case_id": "mvp.ema_cross_deterministic.001",
            "general_stats": {
                "Long Ratio": "0.330000000000",
            },
            "instrument_id": "AUD/USD.SIM",
            "iterations": 120,
            "pnl_stats": {
                "USD": {
                    "Avg Loser": "-1.306666666667",
                    "Avg Winner": "NaN",
                    "Expectancy": "-1.306666666667",
                    "Max Loser": "-1.310000000000",
                    "Max Winner": "NaN",
                    "Min Loser": "-1.300000000000",
                    "Min Winner": "NaN",
                    "PnL (total)": "-3.920000000042",
                    "PnL% (total)": "-0.000392000000",
                    "Win Rate": "0.000000000000",
                },
            },
            "quotes": 120,
            "return_stats": {
                "Average (Return)": "NaN",
                "Average Loss (Return)": "NaN",
                "Average Win (Return)": "NaN",
                "Profit Factor": "NaN",
                "Returns Volatility (252 days)": "NaN",
                "Risk Return Ratio": "NaN",
                "Sharpe Ratio (252 days)": "NaN",
                "Sortino Ratio (252 days)": "NaN",
            },
            "strategy": "ema-cross",
            "total_events": 9,
            "total_orders": 3,
            "total_positions": 3,
        });
        assert_eq!(
            actual, expected,
            "MVP EMA product path result must remain deterministic"
        );
    }

    fn canonical_mvp_ema_result(
        config: &MinimalBacktestConfig,
        engine_run: &EmaCrossEngineRun,
    ) -> Value {
        let result = &engine_run.result;
        let pnl_stats = result
            .stats_pnls
            .iter()
            .map(|(currency, stats)| {
                let values = stats
                    .iter()
                    .map(|(name, value)| (name.clone(), canonical_float(*value)))
                    .collect::<BTreeMap<_, _>>();
                (currency.clone(), values)
            })
            .collect::<BTreeMap<_, _>>();
        let return_stats = result
            .stats_returns
            .iter()
            .map(|(name, value)| (name.clone(), canonical_float(*value)))
            .collect::<BTreeMap<_, _>>();
        let general_stats = result
            .stats_general
            .iter()
            .map(|(name, value)| (name.clone(), canonical_float(*value)))
            .collect::<BTreeMap<_, _>>();

        json!({
            "case_id": MVP_EMA_CASE_ID,
            "instrument_id": config.data.instrument_id,
            "strategy": config.strategy.name,
            "quotes": engine_run.quotes_loaded,
            "iterations": result.iterations,
            "total_events": result.total_events,
            "total_orders": result.total_orders,
            "total_positions": result.total_positions,
            "backtest_start": result.backtest_start.map(|value| value.as_u64().to_string()),
            "backtest_end": result.backtest_end.map(|value| value.as_u64().to_string()),
            "pnl_stats": pnl_stats,
            "return_stats": return_stats,
            "general_stats": general_stats,
        })
    }

    fn canonical_float(value: f64) -> String {
        if value.is_finite() {
            format!("{value:.12}")
        } else {
            value.to_string()
        }
    }
}
