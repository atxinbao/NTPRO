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

//! Deterministic Binance fixture replay support for NTPRO v0.4 sandbox flows.

use anyhow::Context;
use serde::{Deserialize, Serialize};

/// Stable fixture id used by v0.4 Binance sandbox strategy smokes.
pub const V04_BINANCE_SPOT_BAR_FIXTURE_ID: &str = "v04-binance-spot-bars";
/// Checked-in fixture path, relative to the repository root.
pub const V04_BINANCE_SPOT_BAR_FIXTURE_PATH: &str =
    "crates/adapters/binance/test_data/v04/binance_spot_bars.csv";
/// Instrument covered by the v0.4 Binance sandbox replay fixture.
pub const V04_BINANCE_SPOT_INSTRUMENT_ID: &str = "BTCUSDT.BINANCE";
/// Bar type covered by the v0.4 Binance sandbox replay fixture.
pub const V04_BINANCE_SPOT_BAR_TYPE: &str = "BTCUSDT.BINANCE-1-MINUTE-LAST-EXTERNAL";

const V04_BINANCE_SPOT_BARS_CSV: &str = include_str!("../test_data/v04/binance_spot_bars.csv");
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001b3;

/// One deterministic bar from the v0.4 Binance sandbox fixture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinanceReplayBar {
    pub ts_event: u64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
}

/// Summary fields that later strategy, order, risk, and dashboard tasks can consume.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinanceReplaySummary {
    pub fixture_id: String,
    pub source_path: String,
    pub instrument_id: String,
    pub bar_type: String,
    pub bar_count: usize,
    pub first_ts_event: u64,
    pub last_ts_event: u64,
    pub first_close: String,
    pub last_close: String,
    pub checksum: String,
}

/// Deterministic local replay payload for the v0.4 Binance sandbox product path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinanceFixtureReplay {
    pub fixture_id: String,
    pub source_path: String,
    pub instrument_id: String,
    pub bar_type: String,
    pub bars: Vec<BinanceReplayBar>,
}

impl BinanceFixtureReplay {
    /// Returns a stable summary for evidence files and later product smokes.
    ///
    /// # Panics
    ///
    /// Panics only if the replay was constructed with an empty bar list. Public
    /// constructors in this module reject empty fixtures before creating the
    /// replay value.
    #[must_use]
    pub fn summary(&self) -> BinanceReplaySummary {
        let first = self
            .bars
            .first()
            .expect("BinanceFixtureReplay is constructed with at least one bar");
        let last = self
            .bars
            .last()
            .expect("BinanceFixtureReplay is constructed with at least one bar");

        BinanceReplaySummary {
            fixture_id: self.fixture_id.clone(),
            source_path: self.source_path.clone(),
            instrument_id: self.instrument_id.clone(),
            bar_type: self.bar_type.clone(),
            bar_count: self.bars.len(),
            first_ts_event: first.ts_event,
            last_ts_event: last.ts_event,
            first_close: first.close.clone(),
            last_close: last.close.clone(),
            checksum: checksum_bars(&self.bars),
        }
    }

    /// Returns a line-oriented artifact body for CLI logs and evidence.
    ///
    /// # Panics
    ///
    /// Panics only if the replay was constructed with an empty bar list. Public
    /// constructors in this module reject empty fixtures before creating the
    /// replay value.
    #[must_use]
    pub fn summary_artifact(&self) -> String {
        let summary = self.summary();
        [
            "command=binance.fixture.replay".to_string(),
            "status=ok".to_string(),
            format!("fixture_id={}", summary.fixture_id),
            format!("source_path={}", summary.source_path),
            format!("instrument_id={}", summary.instrument_id),
            format!("bar_type={}", summary.bar_type),
            format!("bar_count={}", summary.bar_count),
            format!("first_ts_event={}", summary.first_ts_event),
            format!("last_ts_event={}", summary.last_ts_event),
            format!("first_close={}", summary.first_close),
            format!("last_close={}", summary.last_close),
            format!("checksum={}", summary.checksum),
            "external_adapter=false".to_string(),
            "real_exchange_connection=false".to_string(),
            "runtime_status=fixture_replay_ready".to_string(),
            String::new(),
        ]
        .join("\n")
    }
}

/// Loads the checked-in v0.4 Binance Spot bar replay fixture.
///
/// # Errors
///
/// Returns an error when the checked-in CSV fixture is malformed, empty, not
/// strictly timestamp ordered, or contains non-positive numeric fields.
pub fn load_v04_binance_spot_bar_replay() -> anyhow::Result<BinanceFixtureReplay> {
    replay_from_csv(
        V04_BINANCE_SPOT_BAR_FIXTURE_ID,
        V04_BINANCE_SPOT_BAR_FIXTURE_PATH,
        V04_BINANCE_SPOT_INSTRUMENT_ID,
        V04_BINANCE_SPOT_BAR_TYPE,
        V04_BINANCE_SPOT_BARS_CSV,
    )
}

/// Builds a replay payload from CSV content.
///
/// # Errors
///
/// Returns an error when the CSV header or any data row is invalid.
pub fn replay_from_csv(
    fixture_id: &str,
    source_path: &str,
    instrument_id: &str,
    bar_type: &str,
    csv: &str,
) -> anyhow::Result<BinanceFixtureReplay> {
    let mut lines = csv.lines().enumerate().filter_map(|(index, line)| {
        let trimmed = line.trim();
        (!trimmed.is_empty()).then_some((index + 1, trimmed))
    });

    let (_, header) = lines
        .next()
        .context("Binance replay fixture must contain a header row")?;
    if header != "ts_event,open,high,low,close,volume" {
        anyhow::bail!("unexpected Binance replay fixture header: {header}");
    }

    let mut previous_ts = None;
    let mut bars = Vec::new();
    for (line_number, line) in lines {
        let fields = line.split(',').map(str::trim).collect::<Vec<_>>();
        if fields.len() != 6 {
            anyhow::bail!(
                "Binance replay fixture row {line_number} must contain 6 CSV fields, got {}",
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
                "Binance replay fixture ts_event must be strictly increasing at row {line_number}"
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

        bars.push(BinanceReplayBar {
            ts_event,
            open: fields[1].to_string(),
            high: fields[2].to_string(),
            low: fields[3].to_string(),
            close: fields[4].to_string(),
            volume: fields[5].to_string(),
        });
    }

    if bars.is_empty() {
        anyhow::bail!("Binance replay fixture must contain at least one bar");
    }

    Ok(BinanceFixtureReplay {
        fixture_id: fixture_id.to_string(),
        source_path: source_path.to_string(),
        instrument_id: instrument_id.to_string(),
        bar_type: bar_type.to_string(),
        bars,
    })
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

fn checksum_bars(bars: &[BinanceReplayBar]) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for bar in bars {
        for part in [
            bar.ts_event.to_string(),
            bar.open.clone(),
            bar.high.clone(),
            bar.low.clone(),
            bar.close.clone(),
            bar.volume.clone(),
        ] {
            for byte in part.as_bytes() {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(FNV_PRIME);
            }
            hash ^= u64::from(b'|');
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    format!("{hash:016x}")
}
