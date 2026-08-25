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
use nautilus_backtest::{
    config::{BacktestEngineConfig, BacktestRunConfig, BacktestVenueConfig, SimulatedVenueConfig},
    engine::BacktestEngine,
    node::BacktestNode,
    result::BacktestResult,
};
use nautilus_model::{
    data::{Data, QuoteTick},
    enums::{AccountType, BookType, OmsType},
    events::OrderEventAny,
    identifiers::{InstrumentId, Venue},
    instruments::{
        Instrument, InstrumentAny,
        stubs::{audusd_sim, currency_pair_btcusdt},
    },
    orders::Order,
    types::{Money, Price, Quantity},
};
use nautilus_trading::examples::strategies::EmaCross;
use rust_decimal::Decimal;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use ustr::Ustr;

#[cfg(test)]
use crate::artifacts::atomic_write_json;
use crate::{
    artifacts::atomic_write_text,
    catalog_dataset::{LocalQuoteDatasetInspection, inspect_local_quote_dataset},
    opt::{BacktestCommand, BacktestOpt, BacktestRunOpt, BacktestValidateOpt},
};

const DRY_RUN_MODE: &str = "dry-run";
const ENGINE_SMOKE_MODE: &str = "engine-smoke";
const SYNTHETIC_QUOTES_SOURCE: &str = "synthetic-quotes";
const LOCAL_PARQUET_CATALOG_SOURCE: &str = "local-parquet-catalog";
const NO_OP_STRATEGY: &str = "no-op";
const EMA_CROSS_STRATEGY: &str = "ema-cross";
const AUDUSD_SIM_INSTRUMENT_ID: &str = "AUD/USD.SIM";
const BTCUSDT_BINANCE_INSTRUMENT_ID: &str = "BTCUSDT.BINANCE";
const BACKTEST_RESULT_SCHEMA_VERSION: &str = "ntpro.backtest_result.v1";
const BACKTEST_DETAILS_SCHEMA_VERSION: &str = "ntpro.backtest_details.v1";
const BACKTEST_ANALYSIS_SCHEMA_VERSION: &str = "ntpro.backtest_analysis.v1";
static IMMUTABLE_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MinimalBacktestConfig {
    run: MinimalRunConfig,
    data: MinimalDataConfig,
    strategy: MinimalStrategyConfig,
    venue: Option<MinimalVenueConfig>,
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
    catalog_path: Option<PathBuf>,
    data_sha256: Option<String>,
    start_time_ns: Option<String>,
    end_time_ns: Option<String>,
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
struct MinimalVenueConfig {
    name: String,
    starting_balance: String,
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

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BacktestDetailsArtifact {
    schema_version: String,
    run_id: String,
    strategy_id: String,
    strategy_version_id: String,
    strategy_version_content_hash: String,
    data_ref: String,
    data_sha256: String,
    config_ref: String,
    config_sha256: String,
    details_ref: String,
    instrument_id: String,
    equity_basis: String,
    trades: Vec<BacktestTrade>,
    positions: Vec<BacktestPosition>,
    equity_curve: Vec<BacktestEquityPoint>,
    boundaries: BacktestResultBoundaries,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BacktestAnalysisArtifact {
    schema_version: String,
    run_id: String,
    strategy_id: String,
    strategy_version_id: String,
    strategy_version_content_hash: String,
    analysis_ref: String,
    instrument_id: String,
    risk: BacktestRiskSummary,
    drawdown_curve: Vec<BacktestDrawdownPoint>,
    timeline: Vec<BacktestTimelineEvent>,
    provenance: BacktestAnalysisProvenance,
    boundaries: BacktestResultBoundaries,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestRiskSummary {
    currency: String,
    starting_equity: String,
    ending_equity: String,
    peak_equity: String,
    max_drawdown_amount: String,
    max_drawdown_rate: String,
    max_drawdown_started_at: String,
    max_drawdown_trough_at: String,
    current_drawdown_amount: String,
    current_drawdown_rate: String,
    open_positions: usize,
    closed_positions: usize,
    profitable_positions: usize,
    losing_positions: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestDrawdownPoint {
    ts_event: String,
    equity: String,
    peak_equity: String,
    drawdown_amount: String,
    drawdown_rate: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestTimelineEvent {
    event_id: String,
    event_type: String,
    ts_event: String,
    entity_ref: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestAnalysisProvenance {
    generator: String,
    engine_mode: String,
    data_ref: String,
    data_sha256: String,
    config_ref: String,
    config_sha256: String,
    summary_ref: String,
    summary_sha256: String,
    details_ref: String,
    details_sha256: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestTrade {
    trade_id: String,
    client_order_id: String,
    venue_order_id: String,
    position_id: Option<String>,
    side: String,
    order_type: String,
    quantity: String,
    price: String,
    currency: String,
    liquidity_side: String,
    commission: Option<String>,
    ts_event: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestPosition {
    position_id: String,
    account_id: String,
    side: String,
    entry_side: String,
    peak_quantity: String,
    buy_quantity: String,
    sell_quantity: String,
    avg_price_open: String,
    avg_price_close: Option<String>,
    realized_return: String,
    realized_pnl: Option<String>,
    trade_count: usize,
    ts_opened: String,
    ts_closed: Option<String>,
    duration_ns: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct BacktestEquityPoint {
    account_id: String,
    currency: String,
    total: String,
    free: String,
    locked: String,
    ts_event: String,
}

#[derive(Debug)]
pub(crate) struct ProductBacktestArtifacts {
    pub(crate) summary: Vec<u8>,
    pub(crate) details: Vec<u8>,
    pub(crate) analysis: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct DemoSimulationInput {
    pub(crate) price: f64,
    pub(crate) observed_at_unix_ms: u64,
}

#[derive(Debug)]
pub(crate) struct ProductDemoSimulationArtifacts {
    pub(crate) summary: Vec<u8>,
    pub(crate) fills: Vec<u8>,
    pub(crate) positions: Vec<u8>,
    pub(crate) equity_curve: Vec<u8>,
    pub(crate) fill_count: usize,
    pub(crate) position_count: usize,
    pub(crate) equity_point_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DemoSimulationSummaryArtifact {
    schema_version: String,
    session_id: String,
    strategy_id: String,
    instrument_id: String,
    engine: String,
    execution_mode: String,
    data_sha256: String,
    parameters: BacktestParameters,
    fill_count: usize,
    position_count: usize,
    equity_point_count: usize,
    boundaries: DemoSimulationBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DemoSimulationBoundaries {
    simulation_only: bool,
    external_venue_connection: bool,
    order_submission_allowed: bool,
    order_mutation_allowed: bool,
    automatic_retry_allowed: bool,
    automatic_remediation_allowed: bool,
    real_orders_submitted: bool,
    trading_controls_enabled: bool,
}

impl DemoSimulationBoundaries {
    const fn enforced() -> Self {
        Self {
            simulation_only: true,
            external_venue_connection: false,
            order_submission_allowed: false,
            order_mutation_allowed: false,
            automatic_retry_allowed: false,
            automatic_remediation_allowed: false,
            real_orders_submitted: false,
            trading_controls_enabled: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DemoSimulatedFillArtifact {
    schema_version: String,
    session_id: String,
    strategy_id: String,
    simulation_only: bool,
    #[serde(flatten)]
    fill: BacktestTrade,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DemoSimulatedPositionArtifact {
    schema_version: String,
    session_id: String,
    strategy_id: String,
    simulation_only: bool,
    #[serde(flatten)]
    position: BacktestPosition,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct DemoEquityPointArtifact {
    schema_version: String,
    session_id: String,
    strategy_id: String,
    simulation_only: bool,
    #[serde(flatten)]
    equity: BacktestEquityPoint,
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
        let details = build_backtest_details_artifact(
            run_id,
            config,
            product,
            &engine_run,
            &sha256_ref(&config_bytes),
        );
        let summary_raw = serialized_artifact(&artifact)?;
        let details_raw = serialized_artifact(&details)?;
        let analysis = build_backtest_analysis_artifact(
            run_id,
            product,
            &artifact,
            &details,
            &sha256_ref(&summary_raw),
            &sha256_ref(&details_raw),
        )?;
        write_immutable_result(&output_dir.join("summary.json"), &artifact)?;
        write_immutable_details(&output_dir.join("details.json"), &details)?;
        write_immutable_analysis(&output_dir.join("analysis.json"), &analysis)?;
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

/// Executes a product-bound Backtest from trusted in-memory configuration.
///
/// The caller owns immutable persistence. Keeping execution in memory avoids exposing host
/// filesystem paths through the Product API while reusing the same engine path as the CLI.
pub(crate) fn execute_product_backtest(
    config_raw: &[u8],
) -> anyhow::Result<ProductBacktestArtifacts> {
    let raw = std::str::from_utf8(config_raw).context("backtest config must be UTF-8")?;
    let config = parse_minimal_backtest_config(raw, "product request")?;
    validate_exact("run.mode", &config.run.mode, ENGINE_SMOKE_MODE)?;
    validate_exact("strategy.name", &config.strategy.name, EMA_CROSS_STRATEGY)?;
    let product = config
        .product
        .as_ref()
        .context("product-bound backtest requires product configuration")?;
    let strategy = resolve_ema_cross_strategy(&config.strategy)?;
    let engine_run = run_ema_cross_engine(&config, &strategy)?;
    let artifact = build_backtest_result_artifact(
        &config.run.id,
        &config,
        product,
        &strategy,
        &engine_run,
        &sha256_ref(config_raw),
    )?;
    let details = build_backtest_details_artifact(
        &config.run.id,
        &config,
        product,
        &engine_run,
        &sha256_ref(config_raw),
    );
    let summary = serialized_artifact(&artifact)?;
    let details_raw = serialized_artifact(&details)?;
    let analysis = build_backtest_analysis_artifact(
        &config.run.id,
        product,
        &artifact,
        &details,
        &sha256_ref(&summary),
        &sha256_ref(&details_raw),
    )?;
    Ok(ProductBacktestArtifacts {
        summary,
        details: details_raw,
        analysis: serialized_artifact(&analysis)?,
    })
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
    details: BacktestEngineDetails,
}

struct BacktestEngineDetails {
    trades: Vec<BacktestTrade>,
    positions: Vec<BacktestPosition>,
    equity_curve: Vec<BacktestEquityPoint>,
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
    match config.data.source.as_str() {
        SYNTHETIC_QUOTES_SOURCE => run_synthetic_ema_cross_engine(config, strategy),
        LOCAL_PARQUET_CATALOG_SOURCE => run_catalog_ema_cross_engine(config, strategy),
        source => anyhow::bail!("unsupported data.source '{source}'"),
    }
}

fn run_synthetic_ema_cross_engine(
    config: &MinimalBacktestConfig,
    strategy: &EmaCrossStrategySettings,
) -> anyhow::Result<EmaCrossEngineRun> {
    let mut engine = BacktestEngine::new(BacktestEngineConfig::default())?;

    let (venue, default_starting_balance, instrument) = resolve_backtest_instrument(config)?;
    let starting_balance = resolve_starting_balance(config, venue, default_starting_balance)?;

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

    let details = collect_engine_details(&engine, venue)?;
    let result = engine.get_result();

    Ok(EmaCrossEngineRun {
        quotes_loaded,
        data_sha256,
        result,
        details,
    })
}

fn run_catalog_ema_cross_engine(
    config: &MinimalBacktestConfig,
    strategy: &EmaCrossStrategySettings,
) -> anyhow::Result<EmaCrossEngineRun> {
    let catalog_path = config
        .data
        .catalog_path
        .as_deref()
        .context("data.catalog_path is required for local parquet data")?;
    let expected_sha256 = config
        .data
        .data_sha256
        .as_deref()
        .context("data.data_sha256 is required for local parquet data")?;
    let expected_start =
        parse_unix_nanos("data.start_time_ns", config.data.start_time_ns.as_deref())?;
    let expected_end = parse_unix_nanos("data.end_time_ns", config.data.end_time_ns.as_deref())?;
    let mut before = inspect_local_quote_dataset(catalog_path, &config.data.instrument_id)
        .context("local parquet catalog validation failed before backtest")?;
    validate_catalog_inspection(
        &before,
        config.data.quotes,
        expected_sha256,
        expected_start,
        expected_end,
    )?;

    let (venue, default_starting_balance, _) = resolve_backtest_instrument(config)?;
    anyhow::ensure!(
        before.venue == venue.as_str(),
        "local dataset venue '{}' does not match configured venue '{}'",
        before.venue,
        venue
    );
    let starting_balance = resolve_starting_balance(config, venue, default_starting_balance)?;
    let instrument_id = InstrumentId::from(config.data.instrument_id.as_str());
    let venue_config = BacktestVenueConfig::builder()
        .name(Ustr::from(venue.as_str()))
        .oms_type(OmsType::Hedging)
        .account_type(AccountType::Margin)
        .book_type(BookType::L1_MBP)
        .starting_balances(vec![starting_balance.to_string()])
        .build();
    let run_config = BacktestRunConfig::builder()
        .id(config.run.id.clone())
        .venues(vec![venue_config])
        .data(Vec::new())
        .dispose_on_completion(false)
        .build();
    let mut node = BacktestNode::new(vec![run_config])?;
    node.build()?;
    {
        let engine = node
            .get_engine_mut(&config.run.id)
            .context("catalog backtest engine was not built")?;
        engine.add_instrument(&before.instrument)?;
        engine.add_strategy(EmaCross::new(
            instrument_id,
            strategy.trade_size,
            strategy.fast_period,
            strategy.slow_period,
        ))?;
        let quotes = std::mem::take(&mut before.quotes)
            .into_iter()
            .map(Data::Quote)
            .collect();
        engine.add_data(quotes, None, true, true)?;
        engine.run(None, None, None, false)?;
    }
    let engine = node
        .get_engine(&config.run.id)
        .context("catalog backtest engine is unavailable after execution")?;
    let result = engine.get_result();
    let details = collect_engine_details(engine, venue)?;

    let after = inspect_local_quote_dataset(catalog_path, &config.data.instrument_id)
        .context("local parquet catalog validation failed after backtest")?;
    anyhow::ensure!(
        before.same_content_as(&after),
        "local parquet dataset changed while the backtest was running"
    );

    Ok(EmaCrossEngineRun {
        quotes_loaded: before.record_count,
        data_sha256: before.data_sha256,
        result,
        details,
    })
}

fn resolve_backtest_instrument(
    config: &MinimalBacktestConfig,
) -> anyhow::Result<(Venue, Money, InstrumentAny)> {
    Ok(match config.data.instrument_id.as_str() {
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
    })
}

fn resolve_starting_balance(
    config: &MinimalBacktestConfig,
    venue: Venue,
    default_starting_balance: Money,
) -> anyhow::Result<Money> {
    if let Some(configured) = &config.venue {
        validate_exact("venue.name", &configured.name, venue.as_str())?;
        let balance = configured
            .starting_balance
            .parse::<Money>()
            .map_err(|error| anyhow::anyhow!("venue.starting_balance is invalid: {error}"))?;
        if balance.raw <= 0 {
            anyhow::bail!("venue.starting_balance must be greater than zero");
        }
        if balance.currency != default_starting_balance.currency {
            anyhow::bail!(
                "venue.starting_balance currency must be {}",
                default_starting_balance.currency
            );
        }
        Ok(balance)
    } else {
        Ok(default_starting_balance)
    }
}

fn parse_unix_nanos(field: &str, value: Option<&str>) -> anyhow::Result<u64> {
    let value = value.with_context(|| format!("{field} is required for local parquet data"))?;
    value
        .parse::<u64>()
        .with_context(|| format!("{field} must be an unsigned nanosecond timestamp"))
}

fn validate_catalog_inspection(
    inspection: &LocalQuoteDatasetInspection,
    expected_quotes: usize,
    expected_sha256: &str,
    expected_start: u64,
    expected_end: u64,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        inspection.record_count == expected_quotes,
        "local parquet quote count changed: expected {expected_quotes}, found {}",
        inspection.record_count
    );
    anyhow::ensure!(
        inspection.data_sha256 == expected_sha256,
        "local parquet data fingerprint changed"
    );
    anyhow::ensure!(
        inspection.start_time_ns == expected_start && inspection.end_time_ns == expected_end,
        "local parquet data time range changed"
    );
    Ok(())
}

pub(crate) fn execute_product_demo_simulation(
    session_id: &str,
    strategy_id: &str,
    input: &[DemoSimulationInput],
) -> anyhow::Result<ProductDemoSimulationArtifacts> {
    validate_non_empty("session_id", session_id)?;
    validate_non_empty("strategy_id", strategy_id)?;
    if input.len() < 5 {
        anyhow::bail!("demo simulation requires at least five market points");
    }
    if input.iter().any(|point| {
        !point.price.is_finite() || point.price <= 0.0 || point.observed_at_unix_ms == 0
    }) || input
        .windows(2)
        .any(|pair| pair[0].observed_at_unix_ms >= pair[1].observed_at_unix_ms)
    {
        anyhow::bail!(
            "demo simulation input must contain positive prices and increasing timestamps"
        );
    }

    let venue = Venue::from("BINANCE");
    let instrument = InstrumentAny::CurrencyPair(currency_pair_btcusdt());
    let instrument_id = instrument.id();
    let mut engine = BacktestEngine::new(BacktestEngineConfig::default())?;
    engine.add_venue(
        SimulatedVenueConfig::builder()
            .venue(venue)
            .oms_type(OmsType::Hedging)
            .account_type(AccountType::Margin)
            .book_type(BookType::L1_MBP)
            .starting_balances(vec![Money::from("1_000_000 USDT")])
            .build(),
    )?;
    engine.add_instrument(&instrument)?;
    engine.add_strategy(EmaCross::new(
        instrument_id,
        Quantity::from("1.000000"),
        3,
        5,
    ))?;

    let quotes = input
        .iter()
        .map(|point| {
            let bid = format!("{:.2}", point.price);
            let ask = format!("{:.2}", point.price + 0.01);
            let timestamp_ns = point
                .observed_at_unix_ms
                .checked_mul(1_000_000)
                .context("demo simulation timestamp overflow")?;
            Ok(quote(instrument_id, &bid, &ask, timestamp_ns))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let data_sha256 = sha256_ref(
        &serde_json::to_vec(input).context("failed to serialize demo simulation input")?,
    );
    engine.add_data(quotes, None, true, true)?;
    engine.run(None, None, None, false)?;
    let details = collect_engine_details(&engine, venue)?;

    let fills = details
        .trades
        .into_iter()
        .map(|fill| DemoSimulatedFillArtifact {
            schema_version: "ntpro.demo_simulated_fill.v1".to_string(),
            session_id: session_id.to_string(),
            strategy_id: strategy_id.to_string(),
            simulation_only: true,
            fill,
        })
        .collect::<Vec<_>>();
    let positions = details
        .positions
        .into_iter()
        .map(|position| DemoSimulatedPositionArtifact {
            schema_version: "ntpro.demo_simulated_position.v1".to_string(),
            session_id: session_id.to_string(),
            strategy_id: strategy_id.to_string(),
            simulation_only: true,
            position,
        })
        .collect::<Vec<_>>();
    let equity_curve = details
        .equity_curve
        .into_iter()
        .map(|equity| DemoEquityPointArtifact {
            schema_version: "ntpro.demo_equity_point.v1".to_string(),
            session_id: session_id.to_string(),
            strategy_id: strategy_id.to_string(),
            simulation_only: true,
            equity,
        })
        .collect::<Vec<_>>();
    if fills.is_empty() || positions.is_empty() || equity_curve.is_empty() {
        anyhow::bail!("demo simulation must produce fills, positions, and account equity");
    }
    let summary = DemoSimulationSummaryArtifact {
        schema_version: "ntpro.demo_simulation_summary.v1".to_string(),
        session_id: session_id.to_string(),
        strategy_id: strategy_id.to_string(),
        instrument_id: BTCUSDT_BINANCE_INSTRUMENT_ID.to_string(),
        engine: "nautilus_backtest::engine::BacktestEngine".to_string(),
        execution_mode: "simulated".to_string(),
        data_sha256,
        parameters: BacktestParameters {
            trade_size: "1.000000".to_string(),
            fast_period: 3,
            slow_period: 5,
        },
        fill_count: fills.len(),
        position_count: positions.len(),
        equity_point_count: equity_curve.len(),
        boundaries: DemoSimulationBoundaries::enforced(),
    };

    Ok(ProductDemoSimulationArtifacts {
        summary: serialized_artifact(&summary)?,
        fills: serialized_jsonl(&fills)?,
        positions: serialized_jsonl(&positions)?,
        equity_curve: serialized_jsonl(&equity_curve)?,
        fill_count: fills.len(),
        position_count: positions.len(),
        equity_point_count: equity_curve.len(),
    })
}

fn collect_engine_details(
    engine: &BacktestEngine,
    venue: Venue,
) -> anyhow::Result<BacktestEngineDetails> {
    let cache = engine.kernel().cache.borrow();
    let mut trades = Vec::new();
    for order in cache.orders(None, None, None, None, None) {
        for event in order.events() {
            if let OrderEventAny::Filled(fill) = event {
                trades.push(BacktestTrade {
                    trade_id: fill.trade_id.to_string(),
                    client_order_id: fill.client_order_id.to_string(),
                    venue_order_id: fill.venue_order_id.to_string(),
                    position_id: fill.position_id.map(|value| value.to_string()),
                    side: fill.order_side.to_string(),
                    order_type: fill.order_type.to_string(),
                    quantity: fill.last_qty.to_string(),
                    price: fill.last_px.to_string(),
                    currency: fill.currency.to_string(),
                    liquidity_side: fill.liquidity_side.to_string(),
                    commission: fill.commission.map(|value| value.to_string()),
                    ts_event: fill.ts_event.as_u64().to_string(),
                });
            }
        }
    }
    trades.sort_by(|left, right| {
        left.ts_event
            .cmp(&right.ts_event)
            .then_with(|| left.trade_id.cmp(&right.trade_id))
    });

    let mut positions = cache
        .positions(None, None, None, None, None)
        .into_iter()
        .map(|position| BacktestPosition {
            position_id: position.id.to_string(),
            account_id: position.account_id.to_string(),
            side: position.side.to_string(),
            entry_side: position.entry.to_string(),
            peak_quantity: position.peak_qty.to_string(),
            buy_quantity: position.buy_qty.to_string(),
            sell_quantity: position.sell_qty.to_string(),
            avg_price_open: canonical_float(position.avg_px_open),
            avg_price_close: position.avg_px_close.map(canonical_float),
            realized_return: canonical_float(position.realized_return),
            realized_pnl: position.realized_pnl.map(|value| value.to_string()),
            trade_count: position.trade_ids.len(),
            ts_opened: position.ts_opened.as_u64().to_string(),
            ts_closed: position.ts_closed.map(|value| value.as_u64().to_string()),
            duration_ns: position.duration_ns.to_string(),
        })
        .collect::<Vec<_>>();
    positions.sort_by(|left, right| {
        left.ts_opened
            .cmp(&right.ts_opened)
            .then_with(|| left.position_id.cmp(&right.position_id))
    });

    let account = cache
        .account_for_venue(&venue)
        .context("backtest result is missing simulated venue account")?;
    let mut equity_curve = account
        .events()
        .into_iter()
        .flat_map(|event| {
            event
                .balances
                .into_iter()
                .map(move |balance| BacktestEquityPoint {
                    account_id: event.account_id.to_string(),
                    currency: balance.currency.to_string(),
                    total: balance.total.to_string(),
                    free: balance.free.to_string(),
                    locked: balance.locked.to_string(),
                    ts_event: event.ts_event.as_u64().to_string(),
                })
        })
        .collect::<Vec<_>>();
    equity_curve.sort_by(|left, right| {
        left.ts_event
            .cmp(&right.ts_event)
            .then_with(|| left.currency.cmp(&right.currency))
            .then_with(|| left.total.cmp(&right.total))
    });
    equity_curve.dedup();

    Ok(BacktestEngineDetails {
        trades,
        positions,
        equity_curve,
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

fn build_backtest_details_artifact(
    run_id: &str,
    config: &MinimalBacktestConfig,
    product: &MinimalProductConfig,
    engine_run: &EmaCrossEngineRun,
    config_sha256: &str,
) -> BacktestDetailsArtifact {
    BacktestDetailsArtifact {
        schema_version: BACKTEST_DETAILS_SCHEMA_VERSION.to_string(),
        run_id: run_id.to_string(),
        strategy_id: product.strategy_id.clone(),
        strategy_version_id: product.strategy_version_id.clone(),
        strategy_version_content_hash: product.strategy_version_content_hash.clone(),
        data_ref: product.data_ref.clone(),
        data_sha256: engine_run.data_sha256.clone(),
        config_ref: product.config_ref.clone(),
        config_sha256: config_sha256.to_string(),
        details_ref: format!("artifact://backtests/{run_id}/details.json"),
        instrument_id: config.data.instrument_id.clone(),
        equity_basis: "account_balance_total".to_string(),
        trades: engine_run.details.trades.clone(),
        positions: engine_run.details.positions.clone(),
        equity_curve: engine_run.details.equity_curve.clone(),
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
    }
}

fn build_backtest_analysis_artifact(
    run_id: &str,
    product: &MinimalProductConfig,
    summary: &BacktestResultArtifact,
    details: &BacktestDetailsArtifact,
    summary_sha256: &str,
    details_sha256: &str,
) -> anyhow::Result<BacktestAnalysisArtifact> {
    let first = details
        .equity_curve
        .first()
        .context("backtest analysis requires at least one equity point")?;
    let currency = first.currency.as_str();
    let starting = first.total.parse::<Money>().map_err(anyhow::Error::msg)?;
    let mut peak = starting;
    let mut peak_at = first.ts_event.clone();
    let mut max_amount = Decimal::ZERO;
    let mut max_rate = Decimal::ZERO;
    let mut max_started_at = peak_at.clone();
    let mut max_trough_at = peak_at.clone();
    let mut drawdown_curve = Vec::with_capacity(details.equity_curve.len());

    for point in &details.equity_curve {
        let equity = point.total.parse::<Money>().map_err(anyhow::Error::msg)?;
        if point.currency != currency || equity.currency != starting.currency {
            anyhow::bail!("backtest analysis equity currency changed");
        }
        if equity > peak {
            peak = equity;
            peak_at.clone_from(&point.ts_event);
        }
        let amount = peak.as_decimal() - equity.as_decimal();
        let rate = if peak.as_decimal() == Decimal::ZERO {
            Decimal::ZERO
        } else {
            amount / peak.as_decimal()
        };
        if rate > max_rate {
            max_amount = amount;
            max_rate = rate;
            max_started_at.clone_from(&peak_at);
            max_trough_at.clone_from(&point.ts_event);
        }
        drawdown_curve.push(BacktestDrawdownPoint {
            ts_event: point.ts_event.clone(),
            equity: point.total.clone(),
            peak_equity: peak.to_string(),
            drawdown_amount: Money::from_decimal(amount, starting.currency)?.to_string(),
            drawdown_rate: canonical_decimal(rate),
        });
    }

    let ending = details
        .equity_curve
        .last()
        .context("backtest analysis requires ending equity")?
        .total
        .parse::<Money>()
        .map_err(anyhow::Error::msg)?;
    let current_amount = peak.as_decimal() - ending.as_decimal();
    let current_rate = if peak.as_decimal() == Decimal::ZERO {
        Decimal::ZERO
    } else {
        current_amount / peak.as_decimal()
    };
    let open_positions = details
        .positions
        .iter()
        .filter(|position| position.ts_closed.is_none())
        .count();
    let profitable_positions = details
        .positions
        .iter()
        .filter(|position| {
            position.ts_closed.is_some()
                && position
                    .realized_pnl
                    .as_deref()
                    .and_then(|value| value.parse::<Money>().ok())
                    .is_some_and(|value| value.raw > 0)
        })
        .count();
    let losing_positions = details
        .positions
        .iter()
        .filter(|position| {
            position.ts_closed.is_some()
                && position
                    .realized_pnl
                    .as_deref()
                    .and_then(|value| value.parse::<Money>().ok())
                    .is_some_and(|value| value.raw < 0)
        })
        .count();

    Ok(BacktestAnalysisArtifact {
        schema_version: BACKTEST_ANALYSIS_SCHEMA_VERSION.to_string(),
        run_id: run_id.to_string(),
        strategy_id: product.strategy_id.clone(),
        strategy_version_id: product.strategy_version_id.clone(),
        strategy_version_content_hash: product.strategy_version_content_hash.clone(),
        analysis_ref: format!("artifact://backtests/{run_id}/analysis.json"),
        instrument_id: summary.instrument_id.clone(),
        risk: BacktestRiskSummary {
            currency: currency.to_string(),
            starting_equity: starting.to_string(),
            ending_equity: ending.to_string(),
            peak_equity: peak.to_string(),
            max_drawdown_amount: Money::from_decimal(max_amount, starting.currency)?.to_string(),
            max_drawdown_rate: canonical_decimal(max_rate),
            max_drawdown_started_at: max_started_at,
            max_drawdown_trough_at: max_trough_at,
            current_drawdown_amount: Money::from_decimal(current_amount, starting.currency)?
                .to_string(),
            current_drawdown_rate: canonical_decimal(current_rate),
            open_positions,
            closed_positions: details.positions.len().saturating_sub(open_positions),
            profitable_positions,
            losing_positions,
        },
        drawdown_curve,
        timeline: build_backtest_timeline(run_id, summary, details),
        provenance: BacktestAnalysisProvenance {
            generator: "nautilus_backtest::engine::BacktestEngine".to_string(),
            engine_mode: ENGINE_SMOKE_MODE.to_string(),
            data_ref: product.data_ref.clone(),
            data_sha256: summary.data_sha256.clone(),
            config_ref: product.config_ref.clone(),
            config_sha256: summary.config_sha256.clone(),
            summary_ref: summary.result_ref.clone(),
            summary_sha256: summary_sha256.to_string(),
            details_ref: details.details_ref.clone(),
            details_sha256: details_sha256.to_string(),
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

fn build_backtest_timeline(
    run_id: &str,
    summary: &BacktestResultArtifact,
    details: &BacktestDetailsArtifact,
) -> Vec<BacktestTimelineEvent> {
    let mut events = vec![
        (
            summary.backtest_start.clone(),
            0_u8,
            "run_started".to_string(),
            format!("run://{run_id}"),
        ),
        (
            summary.backtest_end.clone(),
            5_u8,
            "run_completed".to_string(),
            format!("run://{run_id}"),
        ),
    ];
    events.extend(details.equity_curve.iter().map(|point| {
        (
            point.ts_event.clone(),
            1_u8,
            "equity_updated".to_string(),
            format!("account://{}", point.account_id),
        )
    }));
    events.extend(details.trades.iter().map(|trade| {
        (
            trade.ts_event.clone(),
            2_u8,
            "trade_filled".to_string(),
            format!("trade://{}", trade.trade_id),
        )
    }));
    events.extend(details.positions.iter().flat_map(|position| {
        let opened = (
            position.ts_opened.clone(),
            3_u8,
            "position_opened".to_string(),
            format!("position://{}", position.position_id),
        );
        let closed = position.ts_closed.as_ref().map(|timestamp| {
            (
                timestamp.clone(),
                4_u8,
                "position_closed".to_string(),
                format!("position://{}", position.position_id),
            )
        });
        std::iter::once(opened).chain(closed)
    }));
    events.sort_by(|left, right| {
        numeric_timestamp(&left.0)
            .cmp(&numeric_timestamp(&right.0))
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.3.cmp(&right.3))
    });
    events
        .into_iter()
        .enumerate()
        .map(
            |(index, (ts_event, _, event_type, entity_ref))| BacktestTimelineEvent {
                event_id: format!("event-{index:06}"),
                event_type,
                ts_event,
                entity_ref,
            },
        )
        .collect()
}

fn numeric_timestamp(value: &str) -> Option<u64> {
    value.parse::<u64>().ok()
}

fn canonical_decimal(value: Decimal) -> String {
    format!("{value:.12}")
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

pub(crate) fn sha256_ref(bytes: &[u8]) -> String {
    let hash = digest(&SHA256, bytes);
    let mut value = String::with_capacity(71);
    value.push_str("sha256:");
    for byte in hash.as_ref() {
        value.push_str(&format!("{byte:02x}"));
    }
    value
}

fn write_immutable_result(path: &Path, artifact: &BacktestResultArtifact) -> anyhow::Result<()> {
    write_immutable_artifact(path, artifact)
}

fn write_immutable_details(path: &Path, artifact: &BacktestDetailsArtifact) -> anyhow::Result<()> {
    write_immutable_artifact(path, artifact)
}

fn write_immutable_analysis(
    path: &Path,
    artifact: &BacktestAnalysisArtifact,
) -> anyhow::Result<()> {
    write_immutable_artifact(path, artifact)
}

fn serialized_artifact<T>(artifact: &T) -> anyhow::Result<Vec<u8>>
where
    T: Serialize,
{
    Ok(format!("{}\n", serde_json::to_string_pretty(artifact)?).into_bytes())
}

fn serialized_jsonl<T>(records: &[T]) -> anyhow::Result<Vec<u8>>
where
    T: Serialize,
{
    let mut raw = Vec::new();
    for record in records {
        serde_json::to_writer(&mut raw, record)?;
        raw.push(b'\n');
    }
    Ok(raw)
}

fn write_immutable_artifact<T>(path: &Path, artifact: &T) -> anyhow::Result<()>
where
    T: DeserializeOwned + PartialEq + Serialize,
{
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
                let existing: T = read_verified_artifact(path)?;
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

#[cfg(test)]
fn read_verified_result(path: &Path) -> anyhow::Result<BacktestResultArtifact> {
    read_verified_artifact(path)
}

fn read_verified_artifact<T>(path: &Path) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
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
    parse_minimal_backtest_config(&raw, &path.display().to_string())
}

fn parse_minimal_backtest_config(raw: &str, source: &str) -> anyhow::Result<MinimalBacktestConfig> {
    let config: MinimalBacktestConfig = toml::from_str(raw)
        .with_context(|| format!("failed to parse backtest config '{source}'"))?;
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
    validate_one_of(
        "data.source",
        &config.data.source,
        &[SYNTHETIC_QUOTES_SOURCE, LOCAL_PARQUET_CATALOG_SOURCE],
    )?;
    validate_non_empty("data.instrument_id", &config.data.instrument_id)?;
    if config.data.quotes == 0 {
        anyhow::bail!("data.quotes must be greater than zero");
    }
    if config.run.mode == ENGINE_SMOKE_MODE && config.data.quotes < 30 {
        anyhow::bail!("data.quotes must be at least 30 for engine-smoke mode");
    }
    match config.data.source.as_str() {
        SYNTHETIC_QUOTES_SOURCE => {
            if config.data.catalog_path.is_some()
                || config.data.data_sha256.is_some()
                || config.data.start_time_ns.is_some()
                || config.data.end_time_ns.is_some()
            {
                anyhow::bail!("synthetic data must not declare local catalog fields");
            }
        }
        LOCAL_PARQUET_CATALOG_SOURCE => {
            if config.run.mode != ENGINE_SMOKE_MODE {
                anyhow::bail!("local parquet data requires engine-smoke mode");
            }
            let catalog_path = config
                .data
                .catalog_path
                .as_deref()
                .context("data.catalog_path is required for local parquet data")?;
            if !catalog_path.is_absolute() {
                anyhow::bail!("data.catalog_path must be absolute");
            }
            let data_sha256 = config
                .data
                .data_sha256
                .as_deref()
                .context("data.data_sha256 is required for local parquet data")?;
            if !data_sha256.starts_with("sha256:") || data_sha256.len() != 71 {
                anyhow::bail!("data.data_sha256 must be a sha256 reference");
            }
            let start_time_ns =
                parse_unix_nanos("data.start_time_ns", config.data.start_time_ns.as_deref())?;
            let end_time_ns =
                parse_unix_nanos("data.end_time_ns", config.data.end_time_ns.as_deref())?;
            if start_time_ns > end_time_ns {
                anyhow::bail!("data.start_time_ns must not exceed data.end_time_ns");
            }
        }
        value => anyhow::bail!("unsupported data.source '{value}'"),
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
    if let Some(venue) = &config.venue {
        validate_non_empty("venue.name", &venue.name)?;
        validate_non_empty("venue.starting_balance", &venue.starting_balance)?;
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

    #[test]
    fn golden_trace_demo_simulation_replays_engine_outputs() {
        let trace_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("tests/golden/demo_simulation_schema.jsonl");
        let raw = fs::read_to_string(&trace_path).expect("Demo simulation golden trace must exist");
        let case: Value =
            serde_json::from_str(raw.trim()).expect("golden trace must be valid JSON");
        assert_eq!(case["case_id"], "s2.demo_simulation.001");
        let input = &case["input"];
        let prices = input["prices"]
            .as_array()
            .expect("golden prices must be an array");
        let base = input["base_timestamp_unix_ms"]
            .as_u64()
            .expect("golden base timestamp must be a u64");
        let interval = input["interval_ms"]
            .as_u64()
            .expect("golden interval must be a u64");
        let points = prices
            .iter()
            .enumerate()
            .map(|(index, value)| DemoSimulationInput {
                price: value.as_f64().expect("golden price must be numeric"),
                observed_at_unix_ms: base + u64::try_from(index).unwrap() * interval,
            })
            .collect::<Vec<_>>();
        let artifacts = execute_product_demo_simulation(
            input["session_id"].as_str().unwrap(),
            input["strategy_id"].as_str().unwrap(),
            &points,
        )
        .expect("golden Demo simulation must execute");
        let summary: Value = serde_json::from_slice(&artifacts.summary).unwrap();
        let expected = &case["expected"];
        assert_eq!(summary["engine"], expected["engine"]);
        assert_eq!(summary["execution_mode"], expected["execution_mode"]);
        assert_eq!(summary["fill_count"], expected["fill_count"]);
        assert_eq!(summary["position_count"], expected["position_count"]);
        assert_eq!(
            summary["equity_point_count"],
            expected["equity_point_count"]
        );
        assert_eq!(
            summary["boundaries"]["simulation_only"],
            expected["simulation_only"]
        );
        assert_eq!(
            summary["boundaries"]["external_venue_connection"],
            expected["external_venue_connection"]
        );
        assert_eq!(
            summary["boundaries"]["order_submission_allowed"],
            expected["order_submission_allowed"]
        );
        assert_eq!(
            summary["boundaries"]["real_orders_submitted"],
            expected["real_orders_submitted"]
        );
        assert_eq!(artifacts.fill_count, 2);
        assert_eq!(artifacts.position_count, 2);
        assert_eq!(artifacts.equity_point_count, 3);
    }

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
        let details_path = output_dir.join("details.json");
        let analysis_path = output_dir.join("analysis.json");
        let first = fs::read(&result_path).expect("product result should exist");
        let first_details = fs::read(&details_path).expect("product details should exist");
        let first_analysis = fs::read(&analysis_path).expect("product analysis should exist");
        let artifact: BacktestResultArtifact =
            serde_json::from_slice(&first).expect("product result should parse");
        assert_eq!(artifact.schema_version, BACKTEST_RESULT_SCHEMA_VERSION);
        assert_eq!(artifact.run_id, "ema-cross-btcusdt-baseline-v1");
        assert_eq!(artifact.instrument_id, BTCUSDT_BINANCE_INSTRUMENT_ID);
        assert_eq!(
            artifact.config_sha256,
            "sha256:5c066d811a86248899d9d3c896b37925db03bea5243ca7ae3a58a8ca889356cb"
        );
        assert_eq!(
            artifact.data_sha256,
            "sha256:18ed30b352b17a11c33294df39387976f15a587b859f729ffbe5e59bc9c75d1e"
        );
        assert_eq!(
            sha256_ref(&first),
            "sha256:51ca83710448e0433153415411a30b1480a8d1518ce7f4af1d47ed5b17317f29"
        );
        assert_eq!(artifact.metrics.quotes, 120);
        assert_eq!(artifact.metrics.iterations, 120);
        assert!(artifact.metrics.total_orders > 0);
        assert!(artifact.boundaries.read_only);
        assert!(!artifact.boundaries.order_submission_allowed);
        let details: BacktestDetailsArtifact =
            serde_json::from_slice(&first_details).expect("product details should parse");
        assert_eq!(details.schema_version, BACKTEST_DETAILS_SCHEMA_VERSION);
        assert_eq!(details.run_id, artifact.run_id);
        assert_eq!(details.data_sha256, artifact.data_sha256);
        assert_eq!(details.positions.len(), artifact.metrics.total_positions);
        assert!(!details.trades.is_empty());
        assert!(!details.equity_curve.is_empty());
        assert_eq!(details.equity_basis, "account_balance_total");
        assert!(details.boundaries.read_only);
        assert!(!details.boundaries.trading_controls_enabled);
        let analysis: BacktestAnalysisArtifact =
            serde_json::from_slice(&first_analysis).expect("product analysis should parse");
        assert_eq!(analysis.schema_version, BACKTEST_ANALYSIS_SCHEMA_VERSION);
        assert_eq!(analysis.run_id, artifact.run_id);
        assert_eq!(analysis.drawdown_curve.len(), details.equity_curve.len());
        assert_eq!(analysis.provenance.summary_sha256, sha256_ref(&first));
        assert_eq!(
            analysis.provenance.details_sha256,
            sha256_ref(&first_details)
        );
        assert_eq!(analysis.timeline.first().unwrap().event_type, "run_started");
        assert_eq!(
            analysis.timeline.last().unwrap().event_type,
            "run_completed"
        );
        assert!(analysis.boundaries.read_only);
        assert!(!analysis.boundaries.trading_controls_enabled);

        run_backtest_run(&opt).expect("identical replay should be idempotent");
        assert_eq!(
            fs::read(&result_path).expect("replayed result should exist"),
            first
        );
        assert_eq!(
            fs::read(&details_path).expect("replayed details should exist"),
            first_details
        );
        assert_eq!(
            fs::read(&analysis_path).expect("replayed analysis should exist"),
            first_analysis
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
    fn product_backtest_executes_from_memory_with_validated_starting_balance() {
        let config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("configs/backtests/ema-cross-btcusdt-product.toml");
        let raw = fs::read_to_string(config_path)
            .expect("tracked product Backtest config should be readable")
            .replace(
                "[product]",
                "[venue]\nname = \"BINANCE\"\nstarting_balance = \"250000 USDT\"\n\n[product]",
            );
        let result = execute_product_backtest(raw.as_bytes())
            .expect("in-memory product Backtest should complete");
        let artifact: BacktestResultArtifact =
            serde_json::from_slice(&result.summary).expect("result should parse");
        let details: BacktestDetailsArtifact =
            serde_json::from_slice(&result.details).expect("details should parse");
        let analysis: BacktestAnalysisArtifact =
            serde_json::from_slice(&result.analysis).expect("analysis should parse");
        assert_eq!(artifact.metrics.quotes, 120);
        assert_eq!(artifact.config_sha256, sha256_ref(raw.as_bytes()));
        assert_eq!(details.config_sha256, artifact.config_sha256);
        assert_eq!(details.positions.len(), artifact.metrics.total_positions);
        assert_eq!(
            analysis.provenance.summary_sha256,
            sha256_ref(&result.summary)
        );
        assert_eq!(
            analysis.provenance.details_sha256,
            sha256_ref(&result.details)
        );

        let invalid = raw.replace("250000 USDT", "250000 USD");
        let error = execute_product_backtest(invalid.as_bytes())
            .expect_err("wrong starting balance currency must fail")
            .to_string();
        assert!(error.contains("currency must be USDT"));
    }

    #[test]
    fn analysis_excludes_partially_closed_open_position_from_closed_outcomes() {
        let config_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("configs/backtests/ema-cross-btcusdt-product.toml");
        let raw = fs::read_to_string(config_path)
            .expect("tracked product Backtest config should be readable");
        let artifacts = execute_product_backtest(raw.as_bytes())
            .expect("in-memory product Backtest should complete");
        let summary: BacktestResultArtifact =
            serde_json::from_slice(&artifacts.summary).expect("summary should parse");
        let mut details: BacktestDetailsArtifact =
            serde_json::from_slice(&artifacts.details).expect("details should parse");
        for position in &mut details.positions {
            position.realized_pnl = Some("0.00000000 USDT".to_string());
        }
        let partially_closed = details
            .positions
            .iter_mut()
            .find(|position| position.ts_closed.is_none())
            .expect("engine fixture should contain an open position");
        partially_closed.side = "LONG".to_string();
        partially_closed.realized_pnl = Some("1.00000000 USDT".to_string());
        partially_closed.ts_closed = None;
        partially_closed.duration_ns = "0".to_string();

        let config = parse_minimal_backtest_config(&raw, "partial-close-fixture")
            .expect("product config should parse");
        let product = config
            .product
            .as_ref()
            .expect("product section should exist");
        let analysis = build_backtest_analysis_artifact(
            &summary.run_id,
            product,
            &summary,
            &details,
            &sha256_ref(&artifacts.summary),
            &sha256_ref(
                &serde_json::to_vec_pretty(&details).expect("details fixture should serialize"),
            ),
        )
        .expect("partial-close analysis should build");

        let expected_open = details
            .positions
            .iter()
            .filter(|position| position.ts_closed.is_none())
            .count();
        assert!(expected_open > 0);
        assert_eq!(analysis.risk.open_positions, expected_open);
        assert_eq!(
            analysis.risk.closed_positions,
            details.positions.len().saturating_sub(expected_open)
        );
        assert_eq!(analysis.risk.profitable_positions, 0);
        assert_eq!(analysis.risk.losing_positions, 0);
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
