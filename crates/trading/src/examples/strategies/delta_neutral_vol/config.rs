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

//! Configuration for the delta-neutral volatility hedger.

use nautilus_model::{
    enums::TimeInForce,
    identifiers::{ClientId, InstrumentId, StrategyId},
};

use crate::strategy::StrategyConfig;

/// Configuration for the delta-neutral short volatility hedger.
///
/// Tracks a short OTM call and put (strangle) and delta-hedges with the
/// underlying perpetual swap. Rehedges when portfolio delta exceeds a
/// configurable threshold or on a periodic timer.
#[derive(Debug, Clone)]
pub struct DeltaNeutralVolConfig {
    /// Base strategy configuration.
    pub base: StrategyConfig,
    /// Option instrument family (e.g. "BTC-USD").
    pub option_family: String,
    /// Hedge instrument ID (e.g. BTC-USD-SWAP.OKX).
    pub hedge_instrument_id: InstrumentId,
    /// Data and execution client ID (e.g. "OKX").
    pub client_id: ClientId,
    /// Target call delta used by the startup strike heuristic.
    pub target_call_delta: f64,
    /// Target put delta used by the startup strike heuristic.
    pub target_put_delta: f64,
    /// Number of option contracts per leg.
    pub contracts: u64,
    /// Portfolio delta threshold that triggers a rehedge.
    pub rehedge_delta_threshold: f64,
    /// Periodic rehedge check interval in seconds.
    pub rehedge_interval_secs: u64,
    /// Optional expiry date filter (e.g. "260327").
    pub expiry_filter: Option<String>,
    /// Place strangle entry orders when Greeks are first initialized.
    /// When false the strategy only hedges externally-entered positions.
    pub enter_strangle: bool,
    /// Implied volatility offset subtracted from mark IV for entry limit
    /// price. A value of 0.02 sells 2 vol points below mark (more aggressive).
    pub entry_iv_offset: f64,
    /// Time-in-force for strangle entry orders.
    pub entry_time_in_force: TimeInForce,
    /// Param key for implied volatility passed to `submit_order`.
    /// Adapter-specific: Bybit uses `"order_iv"`, OKX uses `"px_vol"`.
    pub iv_param_key: String,
}

impl DeltaNeutralVolConfig {
    /// Creates a new [`DeltaNeutralVolConfig`] with required fields and defaults.
    #[must_use]
    pub fn new(
        option_family: String,
        hedge_instrument_id: InstrumentId,
        client_id: ClientId,
    ) -> Self {
        Self {
            base: StrategyConfig {
                strategy_id: Some(StrategyId::from("DELTA_NEUTRAL_VOL-001")),
                order_id_tag: Some("001".to_string()),
                ..Default::default()
            },
            option_family,
            hedge_instrument_id,
            client_id,
            target_call_delta: 0.20,
            target_put_delta: -0.20,
            contracts: 1,
            rehedge_delta_threshold: 0.5,
            rehedge_interval_secs: 30,
            expiry_filter: None,
            enter_strangle: true,
            entry_iv_offset: 0.0,
            entry_time_in_force: TimeInForce::Gtc,
            iv_param_key: "px_vol".to_string(),
        }
    }

    #[must_use]
    pub fn with_target_call_delta(mut self, delta: f64) -> Self {
        self.target_call_delta = delta;
        self
    }

    #[must_use]
    pub fn with_target_put_delta(mut self, delta: f64) -> Self {
        self.target_put_delta = delta;
        self
    }

    #[must_use]
    pub fn with_contracts(mut self, contracts: u64) -> Self {
        self.contracts = contracts;
        self
    }

    #[must_use]
    pub fn with_rehedge_delta_threshold(mut self, threshold: f64) -> Self {
        self.rehedge_delta_threshold = threshold;
        self
    }

    #[must_use]
    pub fn with_rehedge_interval_secs(mut self, secs: u64) -> Self {
        self.rehedge_interval_secs = secs;
        self
    }

    #[must_use]
    pub fn with_expiry_filter(mut self, expiry: String) -> Self {
        self.expiry_filter = Some(expiry);
        self
    }

    #[must_use]
    pub fn with_enter_strangle(mut self, enter: bool) -> Self {
        self.enter_strangle = enter;
        self
    }

    #[must_use]
    pub fn with_entry_iv_offset(mut self, offset: f64) -> Self {
        self.entry_iv_offset = offset;
        self
    }

    #[must_use]
    pub fn with_entry_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.entry_time_in_force = tif;
        self
    }

    #[must_use]
    pub fn with_strategy_id(mut self, strategy_id: StrategyId) -> Self {
        self.base.strategy_id = Some(strategy_id);
        self
    }

    #[must_use]
    pub fn with_order_id_tag(mut self, tag: String) -> Self {
        self.base.order_id_tag = Some(tag);
        self
    }

    #[must_use]
    pub fn with_iv_param_key(mut self, key: String) -> Self {
        self.iv_param_key = key;
        self
    }
}
