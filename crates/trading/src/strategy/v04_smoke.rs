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

//! Deterministic v0.4 Binance sandbox strategy smoke support.

use anyhow::Context;
use serde::{Deserialize, Serialize};

use super::config::{V04EmaSignalMode, V04SandboxStrategyConfig, V04SandboxStrategyName};

/// Stable smoke id for the v0.4 Binance sandbox EMA path.
pub const V04_BINANCE_EMA_SMOKE_ID: &str = "v04-binance-ema-smoke";
/// Stable smoke id for the v0.4 Binance sandbox RSI path.
pub const V04_BINANCE_RSI_SMOKE_ID: &str = "v04-binance-rsi-smoke";
/// Fixture id produced by the v0.4 Binance bar replay task.
pub const V04_BINANCE_EMA_FIXTURE_ID: &str = "v04-binance-spot-bars";
/// Fixture id produced by the v0.4 Binance bar replay task for RSI.
pub const V04_BINANCE_RSI_FIXTURE_ID: &str = V04_BINANCE_EMA_FIXTURE_ID;
/// Fixture path produced by the v0.4 Binance bar replay task.
pub const V04_BINANCE_EMA_FIXTURE_PATH: &str =
    "crates/adapters/binance/test_data/v04/binance_spot_bars.csv";
/// Fixture path produced by the v0.4 Binance bar replay task for RSI.
pub const V04_BINANCE_RSI_FIXTURE_PATH: &str = V04_BINANCE_EMA_FIXTURE_PATH;
/// Fixture checksum produced by the v0.4 Binance bar replay task.
pub const V04_BINANCE_EMA_FIXTURE_CHECKSUM: &str = "be481da0f80f7ca2";
/// Fixture checksum produced by the v0.4 Binance bar replay task for RSI.
pub const V04_BINANCE_RSI_FIXTURE_CHECKSUM: &str = V04_BINANCE_EMA_FIXTURE_CHECKSUM;
/// Mock lifecycle id produced by the v0.4 Binance order lifecycle task.
pub const V04_BINANCE_EMA_MOCK_LIFECYCLE_ID: &str = "v04-binance-mock-order-lifecycle";
/// Mock lifecycle id produced by the v0.4 Binance order lifecycle task for RSI.
pub const V04_BINANCE_RSI_MOCK_LIFECYCLE_ID: &str = V04_BINANCE_EMA_MOCK_LIFECYCLE_ID;
/// Risk rejection smoke id produced by the v0.4 Binance risk task.
pub const V04_BINANCE_EMA_RISK_SMOKE_ID: &str = "v04-binance-risk-rejection-smoke";
/// Risk rejection smoke id produced by the v0.4 Binance risk task for RSI.
pub const V04_BINANCE_RSI_RISK_SMOKE_ID: &str = V04_BINANCE_EMA_RISK_SMOKE_ID;
/// Instrument covered by the v0.4 Binance sandbox EMA smoke.
pub const V04_BINANCE_EMA_INSTRUMENT_ID: &str = "BTCUSDT.BINANCE";
/// Instrument covered by the v0.4 Binance sandbox RSI smoke.
pub const V04_BINANCE_RSI_INSTRUMENT_ID: &str = V04_BINANCE_EMA_INSTRUMENT_ID;
/// Bar type covered by the v0.4 Binance sandbox EMA smoke.
pub const V04_BINANCE_EMA_BAR_TYPE: &str = "BTCUSDT.BINANCE-1-MINUTE-LAST-EXTERNAL";
/// Bar type covered by the v0.4 Binance sandbox RSI smoke.
pub const V04_BINANCE_RSI_BAR_TYPE: &str = V04_BINANCE_EMA_BAR_TYPE;

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001b3;

/// One close-price input consumed by the deterministic v0.4 EMA smoke.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct V04EmaSmokeBar {
    pub ts_event: u64,
    pub close: f64,
}

/// One deterministic EMA cross signal emitted by the v0.4 EMA smoke.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct V04EmaSmokeSignal {
    pub ts_event: u64,
    pub signal: String,
    pub fast_ema: String,
    pub slow_ema: String,
}

/// Close-price input consumed by the deterministic v0.4 RSI smoke.
pub type V04RsiSmokeBar = V04EmaSmokeBar;

/// One deterministic RSI threshold signal emitted by the v0.4 RSI smoke.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct V04RsiSmokeSignal {
    pub ts_event: u64,
    pub signal: String,
    pub rsi: String,
}

/// Dashboard and evidence friendly summary for the v0.4 Binance sandbox EMA smoke.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct V04EmaSmokeSummary {
    pub smoke_id: String,
    pub fixture_id: String,
    pub fixture_path: String,
    pub fixture_checksum: String,
    pub mock_lifecycle_id: String,
    pub risk_smoke_id: String,
    pub instrument_id: String,
    pub bar_type: String,
    pub strategy_name: String,
    pub signal_mode: String,
    pub fast_period: usize,
    pub slow_period: usize,
    pub warmup_bars: usize,
    pub bars_processed: usize,
    pub signals_emitted: usize,
    pub mock_orders_requested: usize,
    pub mock_orders_allowed: bool,
    pub final_signal: String,
    pub first_ts_event: u64,
    pub last_ts_event: u64,
    pub first_close: String,
    pub last_close: String,
    pub final_fast_ema: String,
    pub final_slow_ema: String,
    pub external_adapter: bool,
    pub real_exchange_connection: bool,
    pub real_orders_submitted: bool,
    pub checksum: String,
    pub signals: Vec<V04EmaSmokeSignal>,
}

/// Dashboard and evidence friendly summary for the v0.4 Binance sandbox RSI smoke.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct V04RsiSmokeSummary {
    pub smoke_id: String,
    pub fixture_id: String,
    pub fixture_path: String,
    pub fixture_checksum: String,
    pub mock_lifecycle_id: String,
    pub risk_smoke_id: String,
    pub instrument_id: String,
    pub bar_type: String,
    pub strategy_name: String,
    pub period: usize,
    pub warmup_bars: usize,
    pub oversold_threshold: String,
    pub overbought_threshold: String,
    pub bars_processed: usize,
    pub signals_emitted: usize,
    pub mock_orders_requested: usize,
    pub mock_orders_allowed: bool,
    pub final_signal: String,
    pub first_ts_event: u64,
    pub last_ts_event: u64,
    pub first_close: String,
    pub last_close: String,
    pub final_rsi: String,
    pub external_adapter: bool,
    pub real_exchange_connection: bool,
    pub real_orders_submitted: bool,
    pub checksum: String,
    pub signals: Vec<V04RsiSmokeSignal>,
}

impl V04EmaSmokeSummary {
    /// Returns a line-oriented artifact for evidence and dashboard diagnostics.
    #[must_use]
    pub fn summary_artifact(&self) -> String {
        [
            "command=binance.ema_smoke".to_string(),
            "status=ok".to_string(),
            format!("smoke_id={}", self.smoke_id),
            format!("fixture_id={}", self.fixture_id),
            format!("fixture_path={}", self.fixture_path),
            format!("fixture_checksum={}", self.fixture_checksum),
            format!("mock_lifecycle_id={}", self.mock_lifecycle_id),
            format!("risk_smoke_id={}", self.risk_smoke_id),
            format!("instrument_id={}", self.instrument_id),
            format!("bar_type={}", self.bar_type),
            format!("strategy_name={}", self.strategy_name),
            format!("signal_mode={}", self.signal_mode),
            format!("fast_period={}", self.fast_period),
            format!("slow_period={}", self.slow_period),
            format!("warmup_bars={}", self.warmup_bars),
            format!("bars_processed={}", self.bars_processed),
            format!("signals_emitted={}", self.signals_emitted),
            format!("mock_orders_requested={}", self.mock_orders_requested),
            format!("mock_orders_allowed={}", self.mock_orders_allowed),
            format!("final_signal={}", self.final_signal),
            format!("first_ts_event={}", self.first_ts_event),
            format!("last_ts_event={}", self.last_ts_event),
            format!("first_close={}", self.first_close),
            format!("last_close={}", self.last_close),
            format!("final_fast_ema={}", self.final_fast_ema),
            format!("final_slow_ema={}", self.final_slow_ema),
            format!("external_adapter={}", self.external_adapter),
            format!("real_exchange_connection={}", self.real_exchange_connection),
            format!("real_orders_submitted={}", self.real_orders_submitted),
            format!("checksum={}", self.checksum),
            "runtime_status=ema_smoke_ready".to_string(),
            String::new(),
        ]
        .join("\n")
    }
}

impl V04RsiSmokeSummary {
    /// Returns a line-oriented artifact for evidence and dashboard diagnostics.
    #[must_use]
    pub fn summary_artifact(&self) -> String {
        [
            "command=binance.rsi_smoke".to_string(),
            "status=ok".to_string(),
            format!("smoke_id={}", self.smoke_id),
            format!("fixture_id={}", self.fixture_id),
            format!("fixture_path={}", self.fixture_path),
            format!("fixture_checksum={}", self.fixture_checksum),
            format!("mock_lifecycle_id={}", self.mock_lifecycle_id),
            format!("risk_smoke_id={}", self.risk_smoke_id),
            format!("instrument_id={}", self.instrument_id),
            format!("bar_type={}", self.bar_type),
            format!("strategy_name={}", self.strategy_name),
            format!("period={}", self.period),
            format!("warmup_bars={}", self.warmup_bars),
            format!("oversold_threshold={}", self.oversold_threshold),
            format!("overbought_threshold={}", self.overbought_threshold),
            format!("bars_processed={}", self.bars_processed),
            format!("signals_emitted={}", self.signals_emitted),
            format!("mock_orders_requested={}", self.mock_orders_requested),
            format!("mock_orders_allowed={}", self.mock_orders_allowed),
            format!("final_signal={}", self.final_signal),
            format!("first_ts_event={}", self.first_ts_event),
            format!("last_ts_event={}", self.last_ts_event),
            format!("first_close={}", self.first_close),
            format!("last_close={}", self.last_close),
            format!("final_rsi={}", self.final_rsi),
            format!("external_adapter={}", self.external_adapter),
            format!("real_exchange_connection={}", self.real_exchange_connection),
            format!("real_orders_submitted={}", self.real_orders_submitted),
            format!("checksum={}", self.checksum),
            "runtime_status=rsi_smoke_ready".to_string(),
            String::new(),
        ]
        .join("\n")
    }
}

/// Runs the deterministic v0.4 EMA smoke from the checked-in Binance fixture CSV content.
///
/// # Errors
///
/// Returns an error when the CSV is malformed, does not match the v0.4 Binance
/// fixture checksum, or does not contain enough bars for the configured EMA
/// periods.
pub fn v04_ema_smoke_from_csv(csv: &str) -> anyhow::Result<V04EmaSmokeSummary> {
    let (bars, fixture_checksum) = parse_v04_fixture_csv(csv)?;
    if fixture_checksum != V04_BINANCE_EMA_FIXTURE_CHECKSUM {
        anyhow::bail!(
            "expected fixture checksum {V04_BINANCE_EMA_FIXTURE_CHECKSUM}, got {fixture_checksum}"
        );
    }
    v04_ema_smoke_from_bars(&bars, &fixture_checksum)
}

/// Runs the deterministic v0.4 RSI smoke from the checked-in Binance fixture CSV content.
///
/// # Errors
///
/// Returns an error when the CSV is malformed, does not match the v0.4 Binance
/// fixture checksum, or does not contain enough bars for the configured RSI
/// period.
pub fn v04_rsi_smoke_from_csv(csv: &str) -> anyhow::Result<V04RsiSmokeSummary> {
    let (bars, fixture_checksum) = parse_v04_fixture_csv(csv)?;
    if fixture_checksum != V04_BINANCE_RSI_FIXTURE_CHECKSUM {
        anyhow::bail!(
            "expected fixture checksum {V04_BINANCE_RSI_FIXTURE_CHECKSUM}, got {fixture_checksum}"
        );
    }
    v04_rsi_smoke_from_bars(&bars, &fixture_checksum)
}

/// Runs the deterministic v0.4 EMA smoke from already parsed close-price bars.
///
/// # Errors
///
/// Returns an error when the fixed v0.4 EMA configuration is invalid or the bar
/// stream is too short for the configured warmup period.
pub fn v04_ema_smoke_from_bars(
    bars: &[V04EmaSmokeBar],
    fixture_checksum: &str,
) -> anyhow::Result<V04EmaSmokeSummary> {
    let config = default_v04_ema_smoke_config();
    config.validate()?;

    let fast_period = config
        .fast_period
        .context("v0.4 EMA smoke fast period missing")?;
    let slow_period = config
        .slow_period
        .context("v0.4 EMA smoke slow period missing")?;
    let warmup_bars = config.resolved_warmup_bars()?;
    if bars.len() < warmup_bars {
        anyhow::bail!(
            "v0.4 EMA smoke requires at least {warmup_bars} bars, got {}",
            bars.len()
        );
    }

    let first = bars
        .first()
        .context("v0.4 EMA smoke requires at least one bar")?;
    let last = bars
        .last()
        .context("v0.4 EMA smoke requires at least one bar")?;
    let mut fast_ema = None;
    let mut slow_ema = None;
    let mut previous_direction = None;
    let mut final_signal = "flat".to_string();
    let mut signals = Vec::new();

    for (index, bar) in bars.iter().enumerate() {
        fast_ema = Some(next_ema(fast_ema, bar.close, fast_period));
        slow_ema = Some(next_ema(slow_ema, bar.close, slow_period));

        let fast = fast_ema.context("fast EMA must be initialized")?;
        let slow = slow_ema.context("slow EMA must be initialized")?;
        if index + 1 < warmup_bars {
            continue;
        }

        let direction = ema_direction(fast, slow);
        if let Some(previous) = previous_direction
            && previous != direction
        {
            let signal = if direction > 0 { "long" } else { "flat" };
            final_signal = signal.to_string();
            signals.push(V04EmaSmokeSignal {
                ts_event: bar.ts_event,
                signal: signal.to_string(),
                fast_ema: format_decimal(fast),
                slow_ema: format_decimal(slow),
            });
        }
        previous_direction = Some(direction);
    }

    let final_fast_ema = fast_ema.context("fast EMA must be initialized")?;
    let final_slow_ema = slow_ema.context("slow EMA must be initialized")?;
    if signals.is_empty() {
        final_signal = "flat".to_string();
    }

    let signals_emitted = signals.len();
    let mock_orders_requested = signals_emitted;
    let mock_orders_allowed = mock_orders_requested <= config.max_orders;
    let strategy_name = config.strategy_label().to_string();
    let checksum = checksum_fields(&[
        V04_BINANCE_EMA_SMOKE_ID,
        V04_BINANCE_EMA_FIXTURE_ID,
        V04_BINANCE_EMA_FIXTURE_PATH,
        fixture_checksum,
        V04_BINANCE_EMA_MOCK_LIFECYCLE_ID,
        V04_BINANCE_EMA_RISK_SMOKE_ID,
        V04_BINANCE_EMA_INSTRUMENT_ID,
        V04_BINANCE_EMA_BAR_TYPE,
        &strategy_name,
        &format!("{fast_period}"),
        &format!("{slow_period}"),
        &format!("{warmup_bars}"),
        &format!("{}", bars.len()),
        &format!("{signals_emitted}"),
        &format!("{mock_orders_requested}"),
        &mock_orders_allowed.to_string(),
        &final_signal,
        &format_decimal(final_fast_ema),
        &format_decimal(final_slow_ema),
    ]);

    Ok(V04EmaSmokeSummary {
        smoke_id: V04_BINANCE_EMA_SMOKE_ID.to_string(),
        fixture_id: V04_BINANCE_EMA_FIXTURE_ID.to_string(),
        fixture_path: V04_BINANCE_EMA_FIXTURE_PATH.to_string(),
        fixture_checksum: fixture_checksum.to_string(),
        mock_lifecycle_id: V04_BINANCE_EMA_MOCK_LIFECYCLE_ID.to_string(),
        risk_smoke_id: V04_BINANCE_EMA_RISK_SMOKE_ID.to_string(),
        instrument_id: config.instrument_id,
        bar_type: config.bar_type,
        strategy_name,
        signal_mode: "cross".to_string(),
        fast_period,
        slow_period,
        warmup_bars,
        bars_processed: bars.len(),
        signals_emitted,
        mock_orders_requested,
        mock_orders_allowed,
        final_signal,
        first_ts_event: first.ts_event,
        last_ts_event: last.ts_event,
        first_close: format_decimal(first.close),
        last_close: format_decimal(last.close),
        final_fast_ema: format_decimal(final_fast_ema),
        final_slow_ema: format_decimal(final_slow_ema),
        external_adapter: false,
        real_exchange_connection: false,
        real_orders_submitted: false,
        checksum,
        signals,
    })
}

/// Runs the deterministic v0.4 RSI smoke from already parsed close-price bars.
///
/// # Errors
///
/// Returns an error when the fixed v0.4 RSI configuration is invalid, threshold
/// strings are malformed, or the bar stream is too short for the configured
/// period.
pub fn v04_rsi_smoke_from_bars(
    bars: &[V04RsiSmokeBar],
    fixture_checksum: &str,
) -> anyhow::Result<V04RsiSmokeSummary> {
    let config = default_v04_rsi_smoke_config();
    config.validate()?;

    let period = config.period.context("v0.4 RSI smoke period missing")?;
    let warmup_bars = config.resolved_warmup_bars()?;
    if bars.len() <= period {
        anyhow::bail!(
            "v0.4 RSI smoke requires more than {period} bars, got {}",
            bars.len()
        );
    }

    let oversold_threshold = parse_threshold(
        "strategy.oversold_threshold",
        config.oversold_threshold.as_deref(),
    )?;
    let overbought_threshold = parse_threshold(
        "strategy.overbought_threshold",
        config.overbought_threshold.as_deref(),
    )?;
    let first = bars
        .first()
        .context("v0.4 RSI smoke requires at least one bar")?;
    let last = bars
        .last()
        .context("v0.4 RSI smoke requires at least one bar")?;

    let mut previous_close = first.close;
    let mut initial_gain = 0.0;
    let mut initial_loss = 0.0;
    let mut average_gain = None;
    let mut average_loss = None;
    let mut final_signal = "flat".to_string();
    let mut final_rsi = None;
    let mut signals = Vec::new();

    for (index, bar) in bars.iter().enumerate().skip(1) {
        let change = bar.close - previous_close;
        previous_close = bar.close;
        let gain = change.max(0.0);
        let loss = (-change).max(0.0);

        if index <= period {
            initial_gain += gain;
            initial_loss += loss;
            if index == period {
                average_gain = Some(initial_gain / period as f64);
                average_loss = Some(initial_loss / period as f64);
            } else {
                continue;
            }
        } else {
            let gain_average = average_gain.context("RSI gain average must be initialized")?;
            let loss_average = average_loss.context("RSI loss average must be initialized")?;
            average_gain = Some(((gain_average * (period - 1) as f64) + gain) / period as f64);
            average_loss = Some(((loss_average * (period - 1) as f64) + loss) / period as f64);
        }

        let gain_average = average_gain.context("RSI gain average must be initialized")?;
        let loss_average = average_loss.context("RSI loss average must be initialized")?;
        let rsi = normalized_rsi(gain_average, loss_average);
        final_rsi = Some(rsi);
        if index + 1 < warmup_bars {
            continue;
        }

        if rsi <= oversold_threshold && final_signal != "long" {
            final_signal = "long".to_string();
            signals.push(V04RsiSmokeSignal {
                ts_event: bar.ts_event,
                signal: final_signal.clone(),
                rsi: format_decimal(rsi),
            });
        } else if rsi >= overbought_threshold && final_signal != "flat" {
            final_signal = "flat".to_string();
            signals.push(V04RsiSmokeSignal {
                ts_event: bar.ts_event,
                signal: final_signal.clone(),
                rsi: format_decimal(rsi),
            });
        }
    }

    let final_rsi = final_rsi.context("RSI value must be initialized")?;
    let signals_emitted = signals.len();
    let mock_orders_requested = signals_emitted;
    let mock_orders_allowed = mock_orders_requested <= config.max_orders;
    let strategy_name = config.strategy_label().to_string();
    let oversold_threshold = config
        .oversold_threshold
        .context("v0.4 RSI smoke oversold threshold missing")?;
    let overbought_threshold = config
        .overbought_threshold
        .context("v0.4 RSI smoke overbought threshold missing")?;
    let checksum = checksum_fields(&[
        V04_BINANCE_RSI_SMOKE_ID,
        V04_BINANCE_RSI_FIXTURE_ID,
        V04_BINANCE_RSI_FIXTURE_PATH,
        fixture_checksum,
        V04_BINANCE_RSI_MOCK_LIFECYCLE_ID,
        V04_BINANCE_RSI_RISK_SMOKE_ID,
        V04_BINANCE_RSI_INSTRUMENT_ID,
        V04_BINANCE_RSI_BAR_TYPE,
        &strategy_name,
        &format!("{period}"),
        &format!("{warmup_bars}"),
        &oversold_threshold,
        &overbought_threshold,
        &format!("{}", bars.len()),
        &format!("{signals_emitted}"),
        &format!("{mock_orders_requested}"),
        &mock_orders_allowed.to_string(),
        &final_signal,
        &format_decimal(final_rsi),
    ]);

    Ok(V04RsiSmokeSummary {
        smoke_id: V04_BINANCE_RSI_SMOKE_ID.to_string(),
        fixture_id: V04_BINANCE_RSI_FIXTURE_ID.to_string(),
        fixture_path: V04_BINANCE_RSI_FIXTURE_PATH.to_string(),
        fixture_checksum: fixture_checksum.to_string(),
        mock_lifecycle_id: V04_BINANCE_RSI_MOCK_LIFECYCLE_ID.to_string(),
        risk_smoke_id: V04_BINANCE_RSI_RISK_SMOKE_ID.to_string(),
        instrument_id: config.instrument_id,
        bar_type: config.bar_type,
        strategy_name,
        period,
        warmup_bars,
        oversold_threshold,
        overbought_threshold,
        bars_processed: bars.len(),
        signals_emitted,
        mock_orders_requested,
        mock_orders_allowed,
        final_signal,
        first_ts_event: first.ts_event,
        last_ts_event: last.ts_event,
        first_close: format_decimal(first.close),
        last_close: format_decimal(last.close),
        final_rsi: format_decimal(final_rsi),
        external_adapter: false,
        real_exchange_connection: false,
        real_orders_submitted: false,
        checksum,
        signals,
    })
}

/// Returns the fixed v0.4 EMA smoke configuration.
#[must_use]
pub fn default_v04_ema_smoke_config() -> V04SandboxStrategyConfig {
    V04SandboxStrategyConfig {
        strategy_name: V04SandboxStrategyName::Ema,
        instrument_id: V04_BINANCE_EMA_INSTRUMENT_ID.to_string(),
        bar_type: V04_BINANCE_EMA_BAR_TYPE.to_string(),
        trade_size: "0.01".to_string(),
        max_orders: 4,
        risk_profile: "sandbox".to_string(),
        fast_period: Some(3),
        slow_period: Some(8),
        signal_mode: Some(V04EmaSignalMode::Cross),
        period: None,
        oversold_threshold: None,
        overbought_threshold: None,
        warmup_bars: Some(8),
    }
}

/// Returns the fixed v0.4 RSI smoke configuration.
#[must_use]
pub fn default_v04_rsi_smoke_config() -> V04SandboxStrategyConfig {
    V04SandboxStrategyConfig {
        strategy_name: V04SandboxStrategyName::Rsi,
        instrument_id: V04_BINANCE_RSI_INSTRUMENT_ID.to_string(),
        bar_type: V04_BINANCE_RSI_BAR_TYPE.to_string(),
        trade_size: "0.01".to_string(),
        max_orders: 4,
        risk_profile: "sandbox".to_string(),
        fast_period: None,
        slow_period: None,
        signal_mode: None,
        period: Some(7),
        oversold_threshold: Some("0.45".to_string()),
        overbought_threshold: Some("0.55".to_string()),
        warmup_bars: Some(7),
    }
}

fn parse_v04_fixture_csv(csv: &str) -> anyhow::Result<(Vec<V04EmaSmokeBar>, String)> {
    let mut lines = csv.lines().enumerate().filter_map(|(index, line)| {
        let trimmed = line.trim();
        (!trimmed.is_empty()).then_some((index + 1, trimmed))
    });

    let (_, header) = lines
        .next()
        .context("v0.4 EMA smoke fixture must contain a header row")?;
    if header != "ts_event,open,high,low,close,volume" {
        anyhow::bail!("unexpected v0.4 EMA smoke fixture header: {header}");
    }

    let mut previous_ts = None;
    let mut checksum_fields = Vec::new();
    let mut bars = Vec::new();
    for (line_number, line) in lines {
        let fields = line.split(',').map(str::trim).collect::<Vec<_>>();
        if fields.len() != 6 {
            anyhow::bail!(
                "v0.4 EMA smoke fixture row {line_number} must contain 6 CSV fields, got {}",
                fields.len()
            );
        }

        let ts_event = fields[0]
            .parse::<u64>()
            .with_context(|| format!("invalid ts_event at row {line_number}"))?;
        if let Some(previous) = previous_ts
            && ts_event <= previous
        {
            anyhow::bail!(
                "v0.4 EMA smoke fixture ts_event must be strictly increasing at row {line_number}"
            );
        }
        previous_ts = Some(ts_event);

        for (field_name, raw) in [
            ("open", fields[1]),
            ("high", fields[2]),
            ("low", fields[3]),
            ("close", fields[4]),
            ("volume", fields[5]),
        ] {
            validate_positive_decimal(field_name, raw, line_number)?;
        }

        let close = fields[4]
            .parse::<f64>()
            .with_context(|| format!("invalid close decimal at row {line_number}"))?;
        bars.push(V04EmaSmokeBar { ts_event, close });
        checksum_fields.extend(fields.into_iter().map(str::to_string));
    }

    if bars.is_empty() {
        anyhow::bail!("v0.4 EMA smoke fixture must contain at least one bar");
    }

    Ok((bars, checksum_fixture_fields(&checksum_fields)))
}

fn validate_positive_decimal(
    field_name: &str,
    raw: &str,
    line_number: usize,
) -> anyhow::Result<()> {
    let value = raw
        .parse::<f64>()
        .with_context(|| format!("invalid {field_name} decimal at row {line_number}"))?;
    if !value.is_finite() || value <= 0.0 {
        anyhow::bail!("{field_name} must be a positive decimal at row {line_number}");
    }
    Ok(())
}

fn parse_threshold(field_name: &str, raw: Option<&str>) -> anyhow::Result<f64> {
    let raw = raw.with_context(|| format!("{field_name} is required"))?;
    let value = raw
        .parse::<f64>()
        .with_context(|| format!("{field_name} must be a decimal string"))?;
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        anyhow::bail!("{field_name} must be between 0.0 and 1.0");
    }
    Ok(value)
}

fn next_ema(previous: Option<f64>, price: f64, period: usize) -> f64 {
    let alpha = 2.0 / (period as f64 + 1.0);
    previous.map_or(price, |previous| {
        alpha.mul_add(price, (1.0 - alpha) * previous)
    })
}

fn ema_direction(fast: f64, slow: f64) -> i8 {
    if fast > slow { 1 } else { -1 }
}

fn normalized_rsi(average_gain: f64, average_loss: f64) -> f64 {
    if average_loss == 0.0 {
        return 1.0;
    }
    let relative_strength = average_gain / average_loss;
    1.0 - (1.0 / (1.0 + relative_strength))
}

fn format_decimal(value: f64) -> String {
    format!("{value:.6}")
}

fn checksum_fixture_fields(fields: &[String]) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for field in fields {
        for byte in field.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash ^= u64::from(b'|');
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{hash:016x}")
}

fn checksum_fields(fields: &[&str]) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for field in fields {
        for byte in field.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash ^= u64::from(b'\n');
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{hash:016x}")
}
