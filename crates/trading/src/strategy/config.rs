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

use std::collections::HashMap;

use anyhow::{Context, bail};
use nautilus_core::serialization::{default_false, default_true};
use nautilus_model::{
    enums::{OmsType, TimeInForce},
    identifiers::{InstrumentId, StrategyId},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// The base model for all trading strategy configurations.
#[derive(Clone, Debug, Deserialize, Serialize, bon::Builder)]
#[serde(deny_unknown_fields)]
pub struct StrategyConfig {
    /// The unique ID for the strategy. Will become the strategy ID if not None.
    pub strategy_id: Option<StrategyId>,
    /// The unique order ID tag for the strategy. Must be unique
    /// amongst all running strategies for a particular trader ID.
    pub order_id_tag: Option<String>,
    /// If UUID4's should be used for client order ID values.
    #[serde(default = "default_false")]
    #[builder(default)]
    pub use_uuid_client_order_ids: bool,
    /// If hyphens should be used in generated client order ID values.
    #[serde(default = "default_true")]
    #[builder(default = true)]
    pub use_hyphens_in_client_order_ids: bool,
    /// The order management system type for the strategy. This will determine
    /// how the `ExecutionEngine` handles position IDs.
    pub oms_type: Option<OmsType>,
    /// The external order claim instrument IDs.
    /// External orders, fills, and materialized reconciliation activity for matching instrument IDs
    /// will be associated with the strategy.
    pub external_order_claims: Option<Vec<InstrumentId>>,
    /// If OTO, OCO, and OUO **open** contingent orders should be managed automatically by the strategy.
    /// Any emulated orders which are active local will be managed by the `OrderEmulator` instead.
    #[serde(default = "default_false")]
    #[builder(default)]
    pub manage_contingent_orders: bool,
    /// If all order GTD time in force expirations should be managed by the strategy.
    /// If True, then will ensure open orders have their GTD timers re-activated on start.
    #[serde(default = "default_false")]
    #[builder(default)]
    pub manage_gtd_expiry: bool,
    /// If the strategy should automatically perform a market exit when stopped.
    /// If true, calling stop() will first cancel all orders and close all positions
    /// before the strategy transitions to the STOPPED state.
    #[serde(default = "default_false")]
    #[builder(default)]
    pub manage_stop: bool,
    /// The interval in milliseconds to check for in-flight orders and open positions
    /// during a market exit.
    #[serde(default = "default_market_exit_interval_ms")]
    #[builder(default = 100)]
    pub market_exit_interval_ms: u64,
    /// The maximum number of attempts to wait for orders and positions to close
    /// during a market exit before completing. Defaults to 100 attempts
    /// (10 seconds at 100ms intervals).
    #[serde(default = "default_market_exit_max_attempts")]
    #[builder(default = 100)]
    pub market_exit_max_attempts: u64,
    /// The time in force for closing market orders during a market exit.
    #[serde(default = "default_market_exit_time_in_force")]
    #[builder(default = TimeInForce::Gtc)]
    pub market_exit_time_in_force: TimeInForce,
    /// If closing market orders during a market exit should be reduce only.
    #[serde(default = "default_true")]
    #[builder(default = true)]
    pub market_exit_reduce_only: bool,
    /// If events should be logged by the strategy.
    /// If False, then only warning events and above are logged.
    #[serde(default = "default_true")]
    #[builder(default = true)]
    pub log_events: bool,
    /// If commands should be logged by the strategy.
    #[serde(default = "default_true")]
    #[builder(default = true)]
    pub log_commands: bool,
    /// If order rejected events where `due_post_only` is True should be logged as warnings.
    #[serde(default = "default_true")]
    #[builder(default = true)]
    pub log_rejected_due_post_only_as_warning: bool,
}

const fn default_market_exit_interval_ms() -> u64 {
    100
}

const fn default_market_exit_max_attempts() -> u64 {
    100
}

const fn default_market_exit_time_in_force() -> TimeInForce {
    TimeInForce::Gtc
}

/// Configuration for creating strategies from importable paths.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImportableStrategyConfig {
    /// The fully qualified name of the Strategy class.
    pub strategy_path: String,
    /// The fully qualified name of the Strategy config class.
    pub config_path: String,
    /// The strategy configuration as a dictionary.
    pub config: HashMap<String, serde_json::Value>,
}

/// Built-in strategy names supported by the v0.4 Binance sandbox product path.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum V04SandboxStrategyName {
    /// Exponential moving average cross strategy.
    Ema,
    /// Relative strength index threshold strategy.
    Rsi,
}

impl V04SandboxStrategyName {
    /// Returns the stable product label used in CLI output and evidence.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ema => "ema",
            Self::Rsi => "rsi",
        }
    }
}

/// EMA signal modes supported by the v0.4 sandbox product path.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum V04EmaSignalMode {
    /// Emit signals only on fast/slow EMA cross events.
    Cross,
}

/// Stable strategy configuration DTO for the v0.4 Binance sandbox path.
///
/// The DTO is intentionally flat so TOML configs stay easy to read. Validation
/// decides which parameter subset belongs to `ema` or `rsi`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct V04SandboxStrategyConfig {
    /// Built-in strategy name. Must be `ema` or `rsi`.
    pub strategy_name: V04SandboxStrategyName,
    /// Sandbox instrument identifier used by fixture replay.
    pub instrument_id: String,
    /// Bar stream identifier used by fixture replay.
    pub bar_type: String,
    /// Positive decimal quantity represented as a string.
    pub trade_size: String,
    /// Maximum number of mock orders allowed during a deterministic smoke.
    pub max_orders: usize,
    /// Risk profile name. v0.4 only accepts `sandbox`.
    pub risk_profile: String,
    /// EMA fast period. Required for `ema`.
    pub fast_period: Option<usize>,
    /// EMA slow period. Required for `ema`.
    pub slow_period: Option<usize>,
    /// EMA signal mode. Required for `ema`.
    pub signal_mode: Option<V04EmaSignalMode>,
    /// RSI period. Required for `rsi`.
    pub period: Option<usize>,
    /// RSI oversold threshold in the normalized range `0.0..=1.0`.
    pub oversold_threshold: Option<String>,
    /// RSI overbought threshold in the normalized range `0.0..=1.0`.
    pub overbought_threshold: Option<String>,
    /// Optional warmup bars. Defaults to the required strategy period.
    pub warmup_bars: Option<usize>,
}

impl V04SandboxStrategyConfig {
    /// Validates that the DTO matches the v0.4 sandbox strategy contract.
    ///
    /// # Errors
    ///
    /// Returns an error when required shared fields are empty, numeric fields
    /// are out of bounds, a production risk profile is selected, or EMA/RSI
    /// strategy-specific fields are missing or mixed.
    pub fn validate(&self) -> anyhow::Result<()> {
        validate_v04_non_empty("strategy.instrument_id", &self.instrument_id)?;
        validate_v04_non_empty("strategy.bar_type", &self.bar_type)?;
        validate_v04_non_empty("strategy.trade_size", &self.trade_size)?;
        validate_v04_positive_decimal("strategy.trade_size", &self.trade_size)?;
        if self.max_orders == 0 {
            bail!("strategy.max_orders must be greater than zero");
        }
        validate_v04_non_empty("strategy.risk_profile", &self.risk_profile)?;
        if self.risk_profile != "sandbox" {
            bail!(
                "strategy.risk_profile must be 'sandbox', got '{}'",
                self.risk_profile
            );
        }

        match self.strategy_name {
            V04SandboxStrategyName::Ema => self.validate_ema(),
            V04SandboxStrategyName::Rsi => self.validate_rsi(),
        }
    }

    /// Returns the stable product strategy label.
    #[must_use]
    pub const fn strategy_label(&self) -> &'static str {
        self.strategy_name.as_str()
    }

    /// Returns the configured warmup bars after applying the strategy default.
    ///
    /// # Errors
    ///
    /// Returns an error if the strategy-specific required period fields are not
    /// present. Call [`Self::validate`] first when handling user input.
    pub fn resolved_warmup_bars(&self) -> anyhow::Result<usize> {
        match self.strategy_name {
            V04SandboxStrategyName::Ema => {
                let slow_period = self
                    .slow_period
                    .context("strategy.slow_period is required for strategy_name='ema'")?;
                Ok(self.warmup_bars.unwrap_or(slow_period))
            }
            V04SandboxStrategyName::Rsi => {
                let period = self
                    .period
                    .context("strategy.period is required for strategy_name='rsi'")?;
                Ok(self.warmup_bars.unwrap_or(period))
            }
        }
    }

    fn validate_ema(&self) -> anyhow::Result<()> {
        reject_v04_field("strategy.period", self.period.is_some(), "ema")?;
        reject_v04_field(
            "strategy.oversold_threshold",
            self.oversold_threshold.is_some(),
            "ema",
        )?;
        reject_v04_field(
            "strategy.overbought_threshold",
            self.overbought_threshold.is_some(),
            "ema",
        )?;

        let fast_period = self
            .fast_period
            .context("strategy.fast_period is required for strategy_name='ema'")?;
        let slow_period = self
            .slow_period
            .context("strategy.slow_period is required for strategy_name='ema'")?;
        self.signal_mode
            .context("strategy.signal_mode is required for strategy_name='ema'")?;

        if fast_period <= 1 {
            bail!("strategy.fast_period must be greater than 1");
        }
        if slow_period <= fast_period {
            bail!("strategy.slow_period must be greater than strategy.fast_period");
        }
        let warmup_bars = self.resolved_warmup_bars()?;
        if warmup_bars < slow_period {
            bail!("strategy.warmup_bars must be at least strategy.slow_period");
        }
        Ok(())
    }

    fn validate_rsi(&self) -> anyhow::Result<()> {
        reject_v04_field("strategy.fast_period", self.fast_period.is_some(), "rsi")?;
        reject_v04_field("strategy.slow_period", self.slow_period.is_some(), "rsi")?;
        reject_v04_field("strategy.signal_mode", self.signal_mode.is_some(), "rsi")?;

        let period = self
            .period
            .context("strategy.period is required for strategy_name='rsi'")?;
        if period <= 1 {
            bail!("strategy.period must be greater than 1");
        }

        let oversold = validate_v04_threshold(
            "strategy.oversold_threshold",
            self.oversold_threshold.as_deref(),
        )?;
        let overbought = validate_v04_threshold(
            "strategy.overbought_threshold",
            self.overbought_threshold.as_deref(),
        )?;
        if oversold >= overbought {
            bail!("strategy.oversold_threshold must be less than strategy.overbought_threshold");
        }

        let warmup_bars = self.resolved_warmup_bars()?;
        if warmup_bars < period {
            bail!("strategy.warmup_bars must be at least strategy.period");
        }
        Ok(())
    }
}

fn validate_v04_non_empty(field: &str, value: &str) -> anyhow::Result<()> {
    if value.trim().is_empty() {
        bail!("{field} must not be empty");
    }
    Ok(())
}

fn validate_v04_positive_decimal(field: &str, value: &str) -> anyhow::Result<()> {
    let decimal = value
        .parse::<Decimal>()
        .with_context(|| format!("{field} must be a decimal string"))?;
    if decimal <= Decimal::ZERO {
        bail!("{field} must be greater than zero");
    }
    Ok(())
}

fn validate_v04_threshold(field: &str, value: Option<&str>) -> anyhow::Result<Decimal> {
    let value = value.with_context(|| format!("{field} is required for strategy_name='rsi'"))?;
    validate_v04_non_empty(field, value)?;
    let decimal = value
        .parse::<Decimal>()
        .with_context(|| format!("{field} must be a decimal string"))?;
    if decimal < Decimal::ZERO || decimal > Decimal::ONE {
        bail!("{field} must be between 0.0 and 1.0");
    }
    Ok(decimal)
}

fn reject_v04_field(field: &str, present: bool, strategy_name: &str) -> anyhow::Result<()> {
    if present {
        bail!("{field} is not valid for strategy_name='{strategy_name}'");
    }
    Ok(())
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_strategy_config_default() {
        let config = StrategyConfig::default();

        assert!(config.strategy_id.is_none());
        assert!(config.order_id_tag.is_none());
        assert!(!config.use_uuid_client_order_ids);
        assert!(config.use_hyphens_in_client_order_ids);
        assert!(config.oms_type.is_none());
        assert!(config.external_order_claims.is_none());
        assert!(!config.manage_contingent_orders);
        assert!(!config.manage_gtd_expiry);
        assert!(!config.manage_stop);
        assert_eq!(config.market_exit_interval_ms, 100);
        assert_eq!(config.market_exit_max_attempts, 100);
        assert_eq!(config.market_exit_time_in_force, TimeInForce::Gtc);
        assert!(config.market_exit_reduce_only);
        assert!(config.log_events);
        assert!(config.log_commands);
        assert!(config.log_rejected_due_post_only_as_warning);
    }

    #[rstest]
    fn test_strategy_config_with_strategy_id() {
        let strategy_id = StrategyId::from("TEST-001");
        let config = StrategyConfig {
            strategy_id: Some(strategy_id),
            ..Default::default()
        };

        assert_eq!(config.strategy_id, Some(strategy_id));
    }

    #[rstest]
    fn test_strategy_config_serialization() {
        let config = StrategyConfig {
            strategy_id: Some(StrategyId::from("TEST-001")),
            order_id_tag: Some("TAG1".to_string()),
            use_uuid_client_order_ids: true,
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: StrategyConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.strategy_id, deserialized.strategy_id);
        assert_eq!(config.order_id_tag, deserialized.order_id_tag);
        assert_eq!(
            config.use_uuid_client_order_ids,
            deserialized.use_uuid_client_order_ids
        );
    }

    fn base_v04_config(strategy_name: V04SandboxStrategyName) -> V04SandboxStrategyConfig {
        V04SandboxStrategyConfig {
            strategy_name,
            instrument_id: "BTCUSDT.BINANCE".to_string(),
            bar_type: "BTCUSDT.BINANCE-1-MINUTE-LAST-EXTERNAL".to_string(),
            trade_size: "0.01".to_string(),
            max_orders: 4,
            risk_profile: "sandbox".to_string(),
            fast_period: None,
            slow_period: None,
            signal_mode: None,
            period: None,
            oversold_threshold: None,
            overbought_threshold: None,
            warmup_bars: None,
        }
    }

    #[rstest]
    fn test_v04_ema_config_validates_and_defaults_warmup() {
        let config = V04SandboxStrategyConfig {
            fast_period: Some(10),
            slow_period: Some(20),
            signal_mode: Some(V04EmaSignalMode::Cross),
            ..base_v04_config(V04SandboxStrategyName::Ema)
        };

        config.validate().unwrap();

        assert_eq!(config.strategy_label(), "ema");
        assert_eq!(config.resolved_warmup_bars().unwrap(), 20);
    }

    #[rstest]
    fn test_v04_ema_rejects_invalid_period_order() {
        let config = V04SandboxStrategyConfig {
            fast_period: Some(20),
            slow_period: Some(10),
            signal_mode: Some(V04EmaSignalMode::Cross),
            ..base_v04_config(V04SandboxStrategyName::Ema)
        };

        let error = config.validate().unwrap_err().to_string();

        assert!(error.contains("strategy.slow_period must be greater than strategy.fast_period"));
    }

    #[rstest]
    fn test_v04_rsi_config_validates_and_defaults_warmup() {
        let config = V04SandboxStrategyConfig {
            period: Some(14),
            oversold_threshold: Some("0.30".to_string()),
            overbought_threshold: Some("0.70".to_string()),
            ..base_v04_config(V04SandboxStrategyName::Rsi)
        };

        config.validate().unwrap();

        assert_eq!(config.strategy_label(), "rsi");
        assert_eq!(config.resolved_warmup_bars().unwrap(), 14);
    }

    #[rstest]
    fn test_v04_rsi_rejects_inverted_thresholds() {
        let config = V04SandboxStrategyConfig {
            period: Some(14),
            oversold_threshold: Some("0.80".to_string()),
            overbought_threshold: Some("0.70".to_string()),
            ..base_v04_config(V04SandboxStrategyName::Rsi)
        };

        let error = config.validate().unwrap_err().to_string();

        assert!(error.contains(
            "strategy.oversold_threshold must be less than strategy.overbought_threshold"
        ));
    }
}
