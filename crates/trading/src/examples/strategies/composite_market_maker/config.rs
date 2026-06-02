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

//! Configuration for the composite market making strategy.

use nautilus_model::{
    identifiers::{InstrumentId, StrategyId},
    types::Quantity,
};

use crate::strategy::StrategyConfig;

/// Configuration for the composite market making strategy.
#[derive(Debug, Clone)]
pub struct CompositeMarketMakerConfig {
    /// Base strategy configuration.
    pub base: StrategyConfig,
    /// Target instrument the strategy quotes on.
    pub instrument_id: InstrumentId,
    /// Signal instrument that drives the signal skew. Typically a
    /// `SyntheticInstrument`, but any instrument that publishes quotes works.
    pub signal_instrument_id: InstrumentId,
    /// Trade size per quote. When `None`, resolved from the instrument's
    /// `min_quantity` during `on_start`.
    pub trade_size: Option<Quantity>,
    /// Half of the desired quoted spread, in basis points of the anchor.
    /// E.g. `5` = 5 bps, so the full quoted spread is 10 bps before skew.
    pub half_spread_bps: u32,
    /// Inventory skew gain in price units per unit of net position. Both sides
    /// shift down by `factor * net_position` so a long position widens the bid
    /// and tightens the ask.
    pub inventory_skew_factor: f64,
    /// Signal skew gain in price units per unit of normalized signal residual.
    /// Both sides shift up by `factor * residual` where
    /// `residual = (signal_mid - baseline) / baseline`.
    pub signal_skew_factor: f64,
    /// Optional baseline price for the signal residual. When `None`, the first
    /// observed signal mid is captured as the baseline. When `Some(_)`, the
    /// configured value is used so backtests are deterministic.
    pub signal_baseline: Option<f64>,
    /// Hard cap on net exposure (long or short).
    pub max_position: Quantity,
    /// Minimum movement in basis points of the anchor before re-quoting.
    /// Applied to both anchor movement and the signal residual's price impact
    /// (`signal_skew_factor * residual`); whichever clears the threshold first
    /// triggers a requote.
    pub requote_threshold_bps: u32,
    /// Optional order expiry in seconds. When set, orders use GTD time-in-force
    /// with `expire_time = now + expire_time_secs`.
    pub expire_time_secs: Option<u64>,
    /// When `true`, resubmit on the next quote tick after an external cancel.
    /// Useful for venues that proactively cancel short-term orders.
    pub on_cancel_resubmit: bool,
}

impl CompositeMarketMakerConfig {
    /// Creates a new [`CompositeMarketMakerConfig`] with required fields and sensible defaults.
    #[must_use]
    pub fn new(
        instrument_id: InstrumentId,
        signal_instrument_id: InstrumentId,
        max_position: Quantity,
    ) -> Self {
        Self {
            base: StrategyConfig {
                strategy_id: Some(StrategyId::from("COMPOSITE_MM-001")),
                order_id_tag: Some("001".to_string()),
                ..Default::default()
            },
            instrument_id,
            signal_instrument_id,
            trade_size: None,
            half_spread_bps: 5,
            inventory_skew_factor: 0.0,
            signal_skew_factor: 0.0,
            signal_baseline: None,
            max_position,
            requote_threshold_bps: 5,
            expire_time_secs: None,
            on_cancel_resubmit: false,
        }
    }

    #[must_use]
    pub fn with_trade_size(mut self, trade_size: Quantity) -> Self {
        self.trade_size = Some(trade_size);
        self
    }

    #[must_use]
    pub fn with_half_spread_bps(mut self, bps: u32) -> Self {
        self.half_spread_bps = bps;
        self
    }

    #[must_use]
    pub fn with_inventory_skew_factor(mut self, factor: f64) -> Self {
        self.inventory_skew_factor = factor;
        self
    }

    #[must_use]
    pub fn with_signal_skew_factor(mut self, factor: f64) -> Self {
        self.signal_skew_factor = factor;
        self
    }

    #[must_use]
    pub fn with_signal_baseline(mut self, baseline: f64) -> Self {
        self.signal_baseline = Some(baseline);
        self
    }

    #[must_use]
    pub fn with_requote_threshold_bps(mut self, bps: u32) -> Self {
        self.requote_threshold_bps = bps;
        self
    }

    #[must_use]
    pub fn with_expire_time_secs(mut self, secs: u64) -> Self {
        self.expire_time_secs = Some(secs);
        self
    }

    #[must_use]
    pub fn with_on_cancel_resubmit(mut self, enabled: bool) -> Self {
        self.on_cancel_resubmit = enabled;
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
}
