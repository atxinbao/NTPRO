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

use nautilus_trading::strategy::v04_smoke::{
    V04_BINANCE_RSI_FIXTURE_CHECKSUM, V04_BINANCE_RSI_FIXTURE_ID,
    V04_BINANCE_RSI_MOCK_LIFECYCLE_ID, V04_BINANCE_RSI_RISK_SMOKE_ID, V04_BINANCE_RSI_SMOKE_ID,
    default_v04_rsi_smoke_config, v04_rsi_smoke_from_csv,
};

const BINANCE_SPOT_BARS_CSV: &str =
    include_str!("../../adapters/binance/test_data/v04/binance_spot_bars.csv");

#[test]
fn test_v04_rsi_smoke_uses_binance_fixture_and_sandbox_boundaries() {
    let summary = v04_rsi_smoke_from_csv(BINANCE_SPOT_BARS_CSV).unwrap();

    assert_eq!(summary.smoke_id, V04_BINANCE_RSI_SMOKE_ID);
    assert_eq!(summary.fixture_id, V04_BINANCE_RSI_FIXTURE_ID);
    assert_eq!(summary.fixture_checksum, V04_BINANCE_RSI_FIXTURE_CHECKSUM);
    assert_eq!(summary.mock_lifecycle_id, V04_BINANCE_RSI_MOCK_LIFECYCLE_ID);
    assert_eq!(summary.risk_smoke_id, V04_BINANCE_RSI_RISK_SMOKE_ID);
    assert_eq!(summary.instrument_id, "BTCUSDT.BINANCE");
    assert_eq!(summary.bar_type, "BTCUSDT.BINANCE-1-MINUTE-LAST-EXTERNAL");
    assert_eq!(summary.strategy_name, "rsi");
    assert_eq!(summary.period, 7);
    assert_eq!(summary.warmup_bars, 7);
    assert_eq!(summary.oversold_threshold, "0.45");
    assert_eq!(summary.overbought_threshold, "0.55");
    assert_eq!(summary.bars_processed, 40);
    assert_eq!(summary.first_ts_event, 1_735_689_600_000_000_000);
    assert_eq!(summary.last_ts_event, 1_735_691_940_000_000_000);
    assert_eq!(summary.first_close, "100.000000");
    assert_eq!(summary.last_close, "101.100000");
    assert!(!summary.external_adapter);
    assert!(!summary.real_exchange_connection);
    assert!(!summary.real_orders_submitted);
}

#[test]
fn test_v04_rsi_smoke_is_deterministic_and_emits_threshold_signals() {
    let first = v04_rsi_smoke_from_csv(BINANCE_SPOT_BARS_CSV).unwrap();
    let second = v04_rsi_smoke_from_csv(BINANCE_SPOT_BARS_CSV).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.signals_emitted, 4);
    assert_eq!(first.mock_orders_requested, 4);
    assert!(first.mock_orders_allowed);
    assert_eq!(first.final_signal, "flat");
    assert_eq!(first.final_rsi, "0.636954");
    assert_eq!(first.checksum, "85bab1cca7fcc872");

    let signals = first
        .signals
        .iter()
        .map(|signal| (signal.ts_event, signal.signal.as_str(), signal.rsi.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        signals,
        vec![
            (1_735_690_020_000_000_000, "long", "0.000000"),
            (1_735_690_500_000_000_000, "flat", "0.640479"),
            (1_735_691_280_000_000_000, "long", "0.448993"),
            (1_735_691_940_000_000_000, "flat", "0.636954"),
        ]
    );
}

#[test]
fn test_v04_rsi_smoke_artifact_exposes_dashboard_ready_fields() {
    let summary = v04_rsi_smoke_from_csv(BINANCE_SPOT_BARS_CSV).unwrap();
    let artifact = summary.summary_artifact();

    assert!(artifact.contains("command=binance.rsi_smoke"));
    assert!(artifact.contains("status=ok"));
    assert!(artifact.contains("fixture_id=v04-binance-spot-bars"));
    assert!(artifact.contains("mock_lifecycle_id=v04-binance-mock-order-lifecycle"));
    assert!(artifact.contains("risk_smoke_id=v04-binance-risk-rejection-smoke"));
    assert!(artifact.contains("external_adapter=false"));
    assert!(artifact.contains("real_exchange_connection=false"));
    assert!(artifact.contains("real_orders_submitted=false"));
    assert!(artifact.contains("runtime_status=rsi_smoke_ready"));
}

#[test]
fn test_default_v04_rsi_smoke_config_validates() {
    let config = default_v04_rsi_smoke_config();

    config.validate().unwrap();
    assert_eq!(config.strategy_label(), "rsi");
    assert_eq!(config.resolved_warmup_bars().unwrap(), 7);
}
