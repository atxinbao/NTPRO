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
    io::ErrorKind,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{
    artifacts::{atomic_write_json, atomic_write_text},
    backtest::{
        DemoSimulationInput, ProductDemoSimulationArtifacts, execute_product_demo_simulation,
    },
};

const STRATEGY_SESSION_STATUS_SCHEMA_VERSION: &str = "ntpro.v09_strategy_session_status.v1";
const STRATEGY_SESSION_EVENT_SCHEMA_VERSION: &str = "ntpro.v09_strategy_session_event.v1";
const STRATEGY_MARKET_STATUS_SCHEMA_VERSION: &str = "ntpro.v09_market_stream_status.v1";
const STRATEGY_MARKET_EVENT_SCHEMA_VERSION: &str = "ntpro.v09_market_stream_event.v1";
const STRATEGY_SIGNAL_SCHEMA_VERSION: &str = "ntpro.v09_strategy_signal.v1";
const STRATEGY_ORDER_INTENT_SCHEMA_VERSION: &str = "ntpro.v09_order_intent.v1";
const STRATEGY_RISK_DECISION_SCHEMA_VERSION: &str = "ntpro.v09_risk_decision.v1";
const STRATEGY_SESSION_SUMMARY_SCHEMA_VERSION: &str = "ntpro.v09_strategy_session_summary.v1";
const STRATEGY_SESSION_MANIFEST_SCHEMA_VERSION: &str = "ntpro.v091_strategy_session_manifest.v1";
pub const STRATEGY_ORDER_PREFLIGHT_SCHEMA_VERSION: &str = "ntpro.v100_order_preflight_input.v1";
const MARKET_STATE_EXHAUSTED: &str = "exhausted";
const MARKET_STATE_STOPPED: &str = "stopped";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategySessionState {
    Created,
    Validated,
    Starting,
    Running,
    Paused,
    RiskHalted,
    Stopping,
    Stopped,
    Failed,
}

impl StrategySessionState {
    const fn label(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::RiskHalted => "risk_halted",
            Self::Stopping => "stopping",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategySessionStatus {
    pub schema_version: String,
    pub session_id: String,
    pub strategy_id: String,
    pub state: StrategySessionState,
    pub reason: String,
    pub updated_at_unix_ms: u64,
    pub artifacts: StrategySessionArtifactPaths,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategySessionArtifactPaths {
    pub session_status: String,
    pub events: String,
    pub market_status: String,
    pub market_events: String,
    pub signal: String,
    pub order_intent: String,
    pub risk_decision: String,
    pub summary: String,
    pub simulation_summary: String,
    pub simulated_fills: String,
    pub simulated_positions: String,
    pub equity_curve: String,
    pub manifest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategySessionEvent {
    pub schema_version: String,
    pub event_type: String,
    pub session_id: String,
    pub strategy_id: String,
    pub previous_state: Option<StrategySessionState>,
    pub state: StrategySessionState,
    pub reason: String,
    pub occurred_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategySessionArtifacts {
    pub session_status: PathBuf,
    pub events: PathBuf,
    pub market_status: PathBuf,
    pub market_events: PathBuf,
    pub signal: PathBuf,
    pub order_intent: PathBuf,
    pub risk_decision: PathBuf,
    pub summary: PathBuf,
    pub simulation_summary: PathBuf,
    pub simulated_fills: PathBuf,
    pub simulated_positions: PathBuf,
    pub equity_curve: PathBuf,
    pub manifest: PathBuf,
}

impl StrategySessionArtifacts {
    pub fn new(root: &Path) -> Self {
        let strategy_root = root.join("strategy");
        Self {
            session_status: strategy_root.join("session_status.json"),
            events: strategy_root.join("events.jsonl"),
            market_status: strategy_root.join("market_status.json"),
            market_events: strategy_root.join("market_events.jsonl"),
            signal: strategy_root.join("signal.jsonl"),
            order_intent: strategy_root.join("order_intent.jsonl"),
            risk_decision: strategy_root.join("risk_decision.jsonl"),
            summary: strategy_root.join("summary.json"),
            simulation_summary: strategy_root.join("simulation_summary.json"),
            simulated_fills: strategy_root.join("simulated_fills.jsonl"),
            simulated_positions: strategy_root.join("simulated_positions.jsonl"),
            equity_curve: strategy_root.join("equity_curve.jsonl"),
            manifest: strategy_root.join("manifest.json"),
        }
    }

    fn as_status_paths(&self) -> StrategySessionArtifactPaths {
        StrategySessionArtifactPaths {
            session_status: self.session_status.display().to_string(),
            events: self.events.display().to_string(),
            market_status: self.market_status.display().to_string(),
            market_events: self.market_events.display().to_string(),
            signal: self.signal.display().to_string(),
            order_intent: self.order_intent.display().to_string(),
            risk_decision: self.risk_decision.display().to_string(),
            summary: self.summary.display().to_string(),
            simulation_summary: self.simulation_summary.display().to_string(),
            simulated_fills: self.simulated_fills.display().to_string(),
            simulated_positions: self.simulated_positions.display().to_string(),
            equity_curve: self.equity_curve.display().to_string(),
            manifest: self.manifest.display().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategySessionManifest {
    pub schema_version: String,
    pub session_id: String,
    pub strategy_id: String,
    pub state: StrategySessionState,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
    pub artifacts: Vec<StrategySessionManifestArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategySessionManifestArtifact {
    pub name: String,
    pub path: String,
    pub format: String,
    pub present: bool,
    pub record_count: Option<u64>,
    pub byte_len: Option<u64>,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategySessionArtifactAuditHealth {
    Healthy,
    Degraded,
}

impl StrategySessionArtifactAuditHealth {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrategySessionArtifactAudit {
    pub health: StrategySessionArtifactAuditHealth,
    pub manifest_path: PathBuf,
    pub diagnostics: Vec<String>,
}

impl StrategySessionArtifactAudit {
    #[must_use]
    pub fn diagnostic_label(&self) -> String {
        if self.diagnostics.is_empty() {
            "strategy_session_artifacts_ok".to_string()
        } else {
            self.diagnostics.join("; ")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyMarketBar {
    pub seq: u64,
    pub symbol: String,
    pub close: f64,
    pub closed_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyMarketTick {
    pub seq: u64,
    pub symbol: String,
    pub price: f64,
    pub observed_at_unix_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategyMarketEventKind {
    FixtureBar,
    MockBar,
    MockTick,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyMarketStreamStatus {
    pub schema_version: String,
    pub session_id: String,
    pub strategy_id: String,
    pub connection: String,
    pub state: String,
    pub source: String,
    pub event_count: u64,
    pub last_event_at_unix_ms: Option<u64>,
    pub updated_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyMarketEvent {
    pub schema_version: String,
    pub session_id: String,
    pub strategy_id: String,
    pub event_type: StrategyMarketEventKind,
    pub source: String,
    pub seq: u64,
    pub symbol: String,
    pub price: f64,
    pub event_at_unix_ms: u64,
    pub recorded_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategySignal {
    pub schema_version: String,
    pub session_id: String,
    pub strategy_id: String,
    pub symbol: String,
    pub signal: String,
    pub confidence: f64,
    pub market_event_seq: u64,
    pub generated_at: String,
    pub generated_at_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyOrderIntent {
    pub schema_version: String,
    pub session_id: String,
    pub strategy_id: String,
    pub intent_id: String,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: f64,
    pub source_signal: String,
    pub confidence: f64,
    pub market_event_seq: u64,
    pub signal_generated_at: String,
    pub created_at: String,
    pub created_at_unix_ms: u64,
    pub submission_allowed: bool,
    pub submission_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyRiskDecision {
    pub schema_version: String,
    pub session_id: String,
    pub strategy_id: String,
    pub decision_id: String,
    pub intent_id: String,
    pub symbol: String,
    pub decision: String,
    pub reasons: Vec<String>,
    pub mode: String,
    pub order_submission: String,
    pub kill_switch_enabled: bool,
    pub kill_switch_active: bool,
    pub account_state: String,
    pub market_state: String,
    pub actual_submission: bool,
    pub evaluated_at: String,
    pub evaluated_at_unix_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrategyRiskControls {
    pub kill_switch_enabled: bool,
    pub kill_switch_active: bool,
}

impl Default for StrategyRiskControls {
    fn default() -> Self {
        Self {
            kill_switch_enabled: true,
            kill_switch_active: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategySessionSummary {
    pub schema_version: String,
    pub session_id: String,
    pub strategy_id: String,
    pub state: StrategySessionState,
    pub event_count: u64,
    pub market_event_count: u64,
    pub signal_count: u64,
    pub intent_count: u64,
    pub risk_decision_count: u64,
    pub rejection_count: u64,
    pub actual_submission_count: u64,
    pub updated_at_unix_ms: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyRuntimeCounters {
    pub market_event_count: u64,
    pub signal_count: u64,
    pub intent_count: u64,
    pub risk_decision_count: u64,
    pub rejection_count: u64,
    pub actual_submission_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyOrderPreflightInput {
    pub schema_version: String,
    pub session: StrategyOrderPreflightSession,
    pub market: StrategyOrderPreflightMarket,
    pub account: StrategyOrderPreflightAccount,
    pub risk: StrategyOrderPreflightRisk,
    pub limits: StrategyOrderPreflightLimits,
    pub endpoint: StrategyOrderPreflightEndpoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyOrderPreflightSession {
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyOrderPreflightMarket {
    pub symbol: String,
    pub last_event_at_unix_ms: u64,
    pub now_unix_ms: u64,
    pub max_age_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyOrderPreflightAccount {
    pub readable: bool,
    pub account_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyOrderPreflightRisk {
    pub kill_switch_active: bool,
    pub allowed_symbols: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyOrderPreflightLimits {
    pub max_order_notional: String,
    pub max_open_orders: u64,
    pub open_order_count: u64,
    pub max_clock_skew_ms: u64,
    pub observed_clock_skew_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyOrderPreflightEndpoint {
    pub http_base_url: String,
    pub production_endpoint_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DemoStrategyRuntimeSummary {
    pub schema_version: String,
    pub session_id: String,
    pub strategy_id: String,
    pub strategy: String,
    pub processed_events: u64,
    pub signal_count: u64,
    pub signal_artifact: String,
    pub order_intent_count: u64,
    pub order_intent_artifact: String,
    pub risk_decision_count: u64,
    pub risk_decision_artifact: String,
    pub summary_artifact: String,
    pub simulated_fill_count: u64,
    pub simulated_position_count: u64,
    pub equity_point_count: u64,
    pub simulation_summary_artifact: String,
    pub order_submission_allowed: bool,
    pub counters: StrategyRuntimeCounters,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyMarketStreamSummary {
    pub schema_version: String,
    pub session_id: String,
    pub strategy_id: String,
    pub connection: String,
    pub state: String,
    pub source: String,
    pub event_count: u64,
    pub last_event_at_unix_ms: Option<u64>,
    pub market_status_artifact: String,
    pub market_events_artifact: String,
}

#[derive(Debug)]
pub struct StrategySession {
    status: StrategySessionStatus,
    events: Vec<StrategySessionEvent>,
    market_status: Option<StrategyMarketStreamStatus>,
    market_events: Vec<StrategyMarketEvent>,
    signals: Vec<StrategySignal>,
    order_intents: Vec<StrategyOrderIntent>,
    risk_decisions: Vec<StrategyRiskDecision>,
    summary: Option<StrategySessionSummary>,
    demo_simulation: Option<ProductDemoSimulationArtifacts>,
    risk_controls: StrategyRiskControls,
    artifacts: StrategySessionArtifacts,
}

impl StrategySession {
    /// Creates a new strategy session in the `created` state and writes the
    /// initial status/event artifacts.
    ///
    /// # Errors
    ///
    /// Returns an error when `session_id` or `strategy_id` is empty, or when
    /// status/event artifacts cannot be serialized or written.
    pub fn new(
        session_id: impl Into<String>,
        strategy_id: impl Into<String>,
        artifact_root: &Path,
    ) -> anyhow::Result<Self> {
        let session_id = non_empty("session_id", session_id.into())?;
        let strategy_id = non_empty("strategy_id", strategy_id.into())?;
        let artifacts = StrategySessionArtifacts::new(artifact_root);
        let now = unix_timestamp_millis();
        let status = StrategySessionStatus {
            schema_version: STRATEGY_SESSION_STATUS_SCHEMA_VERSION.to_string(),
            session_id: session_id.clone(),
            strategy_id: strategy_id.clone(),
            state: StrategySessionState::Created,
            reason: "session created".to_string(),
            updated_at_unix_ms: now,
            artifacts: artifacts.as_status_paths(),
        };
        let events = vec![StrategySessionEvent {
            schema_version: STRATEGY_SESSION_EVENT_SCHEMA_VERSION.to_string(),
            event_type: "strategy_session_state_changed".to_string(),
            session_id,
            strategy_id,
            previous_state: None,
            state: StrategySessionState::Created,
            reason: "session created".to_string(),
            occurred_at_unix_ms: now,
        }];
        let session = Self {
            status,
            events,
            market_status: None,
            market_events: Vec::new(),
            signals: Vec::new(),
            order_intents: Vec::new(),
            risk_decisions: Vec::new(),
            summary: None,
            demo_simulation: None,
            risk_controls: StrategyRiskControls::default(),
            artifacts,
        };
        session.persist()?;
        Ok(session)
    }

    pub const fn status(&self) -> &StrategySessionStatus {
        &self.status
    }

    pub const fn set_risk_controls(&mut self, risk_controls: StrategyRiskControls) {
        self.risk_controls = risk_controls;
    }

    #[must_use]
    pub fn counters(&self) -> StrategyRuntimeCounters {
        StrategyRuntimeCounters {
            market_event_count: u64::try_from(self.market_events.len()).unwrap_or(u64::MAX),
            signal_count: u64::try_from(self.signals.len()).unwrap_or(u64::MAX),
            intent_count: u64::try_from(self.order_intents.len()).unwrap_or(u64::MAX),
            risk_decision_count: u64::try_from(self.risk_decisions.len()).unwrap_or(u64::MAX),
            rejection_count: u64::try_from(
                self.risk_decisions
                    .iter()
                    .filter(|decision| decision.decision == "rejected")
                    .count(),
            )
            .unwrap_or(u64::MAX),
            actual_submission_count: u64::try_from(
                self.risk_decisions
                    .iter()
                    .filter(|decision| decision.actual_submission)
                    .count(),
            )
            .unwrap_or(u64::MAX),
        }
    }

    /// Transitions the session to the next lifecycle state and persists updated
    /// status/event artifacts.
    ///
    /// # Errors
    ///
    /// Returns an error when the reason is empty, the transition is illegal, or
    /// status/event artifacts cannot be serialized or written.
    pub fn transition(
        &mut self,
        next: StrategySessionState,
        reason: impl Into<String>,
    ) -> anyhow::Result<()> {
        let reason = non_empty("reason", reason.into())?;
        let previous = self.status.state;
        if !is_legal_transition(previous, next) {
            anyhow::bail!(
                "illegal StrategySession transition from {} to {}",
                previous.label(),
                next.label()
            );
        }

        let now = unix_timestamp_millis();
        self.status.state = next;
        self.status.reason = reason.clone();
        self.status.updated_at_unix_ms = now;
        self.events.push(StrategySessionEvent {
            schema_version: STRATEGY_SESSION_EVENT_SCHEMA_VERSION.to_string(),
            event_type: "strategy_session_state_changed".to_string(),
            session_id: self.status.session_id.clone(),
            strategy_id: self.status.strategy_id.clone(),
            previous_state: Some(previous),
            state: next,
            reason,
            occurred_at_unix_ms: now,
        });
        self.persist()
    }

    /// Records a deterministic local fixture bar stream for the strategy
    /// session.
    ///
    /// # Errors
    ///
    /// Returns an error when bars are empty or invalid, or when market status
    /// and event artifacts cannot be serialized or written.
    pub fn record_fixture_bar_stream(
        &mut self,
        bars: &[StrategyMarketBar],
    ) -> anyhow::Result<StrategyMarketStreamSummary> {
        validate_fixture_bars(bars)?;
        self.record_market_bar_stream(
            bars,
            StrategyMarketEventKind::FixtureBar,
            "fixture_bar_stream",
            MARKET_STATE_EXHAUSTED,
        )
    }

    /// Records a deterministic local mock bar stream for the strategy session.
    ///
    /// # Errors
    ///
    /// Returns an error when bars are empty or invalid, or when market status
    /// and event artifacts cannot be serialized or written.
    pub fn record_mock_bar_stream(
        &mut self,
        bars: &[StrategyMarketBar],
    ) -> anyhow::Result<StrategyMarketStreamSummary> {
        validate_fixture_bars(bars)?;
        self.record_market_bar_stream(
            bars,
            StrategyMarketEventKind::MockBar,
            "mock_bar_stream",
            MARKET_STATE_EXHAUSTED,
        )
    }

    /// Records a deterministic local mock tick stream for the strategy session.
    ///
    /// # Errors
    ///
    /// Returns an error when ticks are empty or invalid, or when market status
    /// and event artifacts cannot be serialized or written.
    pub fn record_mock_tick_stream(
        &mut self,
        ticks: &[StrategyMarketTick],
    ) -> anyhow::Result<StrategyMarketStreamSummary> {
        validate_mock_ticks(ticks)?;
        let now = unix_timestamp_millis();
        let events = ticks
            .iter()
            .map(|tick| StrategyMarketEvent {
                schema_version: STRATEGY_MARKET_EVENT_SCHEMA_VERSION.to_string(),
                session_id: self.status.session_id.clone(),
                strategy_id: self.status.strategy_id.clone(),
                event_type: StrategyMarketEventKind::MockTick,
                source: "mock_tick_stream".to_string(),
                seq: tick.seq,
                symbol: tick.symbol.clone(),
                price: tick.price,
                event_at_unix_ms: tick.observed_at_unix_ms,
                recorded_at_unix_ms: now,
            })
            .collect::<Vec<_>>();
        self.record_market_events(events, "mock_tick_stream", MARKET_STATE_EXHAUSTED)
    }

    /// Runs the built-in deterministic EMA-cross demo strategy over local
    /// fixture bars, writes strategy artifacts, and leaves the session in the
    /// `running` state until an explicit shutdown path stops it.
    ///
    /// # Errors
    ///
    /// Returns an error when the session cannot transition through the startup
    /// lifecycle, when fixture bars are invalid, or when signal/status
    /// artifacts cannot be serialized or written.
    pub fn run_ema_cross_demo(
        &mut self,
        bars: &[StrategyMarketBar],
    ) -> anyhow::Result<DemoStrategyRuntimeSummary> {
        validate_fixture_bars(bars)?;
        self.transition(StrategySessionState::Validated, "strategy config validated")?;
        self.transition(StrategySessionState::Starting, "demo strategy starting")?;
        self.transition(StrategySessionState::Running, "demo strategy running")?;
        self.record_fixture_bar_stream(bars)?;

        let mut fast: Option<f64> = None;
        let mut slow: Option<f64> = None;

        for bar in bars {
            let previous = fast.zip(slow);
            let fast_now = ema_next(fast, bar.close, 3);
            let slow_now = ema_next(slow, bar.close, 5);
            fast = Some(fast_now);
            slow = Some(slow_now);

            let signal = previous.and_then(|(prev_fast, prev_slow)| {
                if prev_fast <= prev_slow && fast_now > slow_now {
                    Some("long")
                } else if prev_fast >= prev_slow && fast_now < slow_now {
                    Some("flat")
                } else {
                    None
                }
            });

            if let Some(signal) = signal {
                let generated_at_unix_ms = unix_timestamp_millis();
                self.signals.push(StrategySignal {
                    schema_version: STRATEGY_SIGNAL_SCHEMA_VERSION.to_string(),
                    session_id: self.status.session_id.clone(),
                    strategy_id: self.status.strategy_id.clone(),
                    symbol: bar.symbol.clone(),
                    signal: signal.to_string(),
                    confidence: confidence_from_ema_gap(fast_now, slow_now),
                    market_event_seq: bar.seq,
                    generated_at: format_unix_millis(generated_at_unix_ms),
                    generated_at_unix_ms,
                });
            }
        }

        if self.signals.is_empty() {
            anyhow::bail!("ema_cross_demo must generate at least one signal for fixture input");
        }
        self.generate_order_intents_from_signals();
        self.evaluate_shadow_risk_decisions();
        let demo_simulation = execute_product_demo_simulation(
            &self.status.session_id,
            &self.status.strategy_id,
            &bars
                .iter()
                .map(|bar| DemoSimulationInput {
                    price: bar.close,
                    observed_at_unix_ms: bar.closed_at_unix_ms,
                })
                .collect::<Vec<_>>(),
        )?;
        let simulated_fill_count = u64::try_from(demo_simulation.fill_count).unwrap_or(u64::MAX);
        let simulated_position_count =
            u64::try_from(demo_simulation.position_count).unwrap_or(u64::MAX);
        let equity_point_count =
            u64::try_from(demo_simulation.equity_point_count).unwrap_or(u64::MAX);
        self.demo_simulation = Some(demo_simulation);

        self.record_summary();
        self.persist()?;

        let counters = self.counters();
        Ok(DemoStrategyRuntimeSummary {
            schema_version: "ntpro.v09_demo_strategy_runtime_summary.v1".to_string(),
            session_id: self.status.session_id.clone(),
            strategy_id: self.status.strategy_id.clone(),
            strategy: "ema_cross_demo".to_string(),
            processed_events: counters.market_event_count,
            signal_count: counters.signal_count,
            signal_artifact: self.artifacts.signal.display().to_string(),
            order_intent_count: counters.intent_count,
            order_intent_artifact: self.artifacts.order_intent.display().to_string(),
            risk_decision_count: counters.risk_decision_count,
            risk_decision_artifact: self.artifacts.risk_decision.display().to_string(),
            summary_artifact: self.artifacts.summary.display().to_string(),
            simulated_fill_count,
            simulated_position_count,
            equity_point_count,
            simulation_summary_artifact: self.artifacts.simulation_summary.display().to_string(),
            order_submission_allowed: false,
            counters,
        })
    }

    /// Stops a running strategy session after a node shutdown trigger.
    ///
    /// # Errors
    ///
    /// Returns an error when the reason is empty, the lifecycle transition is
    /// illegal, or status/event artifacts cannot be serialized or written.
    pub fn stop_after_shutdown(&mut self, reason: impl Into<String>) -> anyhow::Result<()> {
        let reason = non_empty("reason", reason.into())?;
        self.transition(
            StrategySessionState::Stopping,
            format!("shutdown requested: {reason}"),
        )?;
        self.transition(
            StrategySessionState::Stopped,
            format!("shutdown complete: {reason}"),
        )?;
        self.mark_market_stopped();
        self.record_summary();
        self.persist()
    }

    fn mark_market_stopped(&mut self) {
        if let Some(market_status) = &mut self.market_status {
            market_status.connection = MARKET_STATE_STOPPED.to_string();
            market_status.state = MARKET_STATE_STOPPED.to_string();
            market_status.updated_at_unix_ms = unix_timestamp_millis();
        }
    }

    fn generate_order_intents_from_signals(&mut self) {
        self.order_intents = self
            .signals
            .iter()
            .map(|signal| {
                let created_at_unix_ms = unix_timestamp_millis();
                StrategyOrderIntent {
                    schema_version: STRATEGY_ORDER_INTENT_SCHEMA_VERSION.to_string(),
                    session_id: signal.session_id.clone(),
                    strategy_id: signal.strategy_id.clone(),
                    intent_id: format!(
                        "{}:{}:{}",
                        signal.session_id, signal.strategy_id, signal.market_event_seq
                    ),
                    symbol: signal.symbol.clone(),
                    side: order_intent_side(&signal.signal).to_string(),
                    order_type: "market".to_string(),
                    quantity: 1.0,
                    source_signal: signal.signal.clone(),
                    confidence: signal.confidence,
                    market_event_seq: signal.market_event_seq,
                    signal_generated_at: signal.generated_at.clone(),
                    created_at: format_unix_millis(created_at_unix_ms),
                    created_at_unix_ms,
                    submission_allowed: false,
                    submission_status: "blocked_by_v09_strategy_runtime_boundary".to_string(),
                }
            })
            .collect();
    }

    fn evaluate_shadow_risk_decisions(&mut self) {
        let market_state = if self
            .market_status
            .as_ref()
            .is_some_and(|status| status.event_count > 0)
        {
            "available"
        } else {
            "missing"
        };

        let mut decisions = Vec::new();
        let mut events = Vec::new();
        for order_intent in &self.order_intents {
            let evaluated_at_unix_ms = unix_timestamp_millis();
            let reasons = shadow_risk_rejection_reasons(
                "disabled",
                self.risk_controls.kill_switch_active,
                "shadow",
                "missing",
                market_state,
            );
            let decision = StrategyRiskDecision {
                schema_version: STRATEGY_RISK_DECISION_SCHEMA_VERSION.to_string(),
                session_id: order_intent.session_id.clone(),
                strategy_id: order_intent.strategy_id.clone(),
                decision_id: format!("risk:{}", order_intent.intent_id),
                intent_id: order_intent.intent_id.clone(),
                symbol: order_intent.symbol.clone(),
                decision: "rejected".to_string(),
                reasons,
                mode: "shadow".to_string(),
                order_submission: "disabled".to_string(),
                kill_switch_enabled: self.risk_controls.kill_switch_enabled,
                kill_switch_active: self.risk_controls.kill_switch_active,
                account_state: "missing".to_string(),
                market_state: market_state.to_string(),
                actual_submission: false,
                evaluated_at: format_unix_millis(evaluated_at_unix_ms),
                evaluated_at_unix_ms,
            };
            events.push(StrategySessionEvent {
                schema_version: STRATEGY_SESSION_EVENT_SCHEMA_VERSION.to_string(),
                event_type: "strategy_risk_decision_rejected".to_string(),
                session_id: self.status.session_id.clone(),
                strategy_id: self.status.strategy_id.clone(),
                previous_state: None,
                state: self.status.state,
                reason: format!("risk decision rejected intent {}", order_intent.intent_id),
                occurred_at_unix_ms: evaluated_at_unix_ms,
            });
            decisions.push(decision);
        }
        self.risk_decisions = decisions;
        self.events.extend(events);
    }

    fn record_summary(&mut self) {
        let counters = self.counters();
        self.summary = Some(StrategySessionSummary {
            schema_version: STRATEGY_SESSION_SUMMARY_SCHEMA_VERSION.to_string(),
            session_id: self.status.session_id.clone(),
            strategy_id: self.status.strategy_id.clone(),
            state: self.status.state,
            event_count: u64::try_from(self.events.len()).unwrap_or(u64::MAX),
            market_event_count: counters.market_event_count,
            signal_count: counters.signal_count,
            intent_count: counters.intent_count,
            risk_decision_count: counters.risk_decision_count,
            rejection_count: counters.rejection_count,
            actual_submission_count: counters.actual_submission_count,
            updated_at_unix_ms: unix_timestamp_millis(),
        });
    }

    fn record_market_bar_stream(
        &mut self,
        bars: &[StrategyMarketBar],
        event_type: StrategyMarketEventKind,
        source: &str,
        connection: &str,
    ) -> anyhow::Result<StrategyMarketStreamSummary> {
        let now = unix_timestamp_millis();
        let events = bars
            .iter()
            .map(|bar| StrategyMarketEvent {
                schema_version: STRATEGY_MARKET_EVENT_SCHEMA_VERSION.to_string(),
                session_id: self.status.session_id.clone(),
                strategy_id: self.status.strategy_id.clone(),
                event_type,
                source: source.to_string(),
                seq: bar.seq,
                symbol: bar.symbol.clone(),
                price: bar.close,
                event_at_unix_ms: bar.closed_at_unix_ms,
                recorded_at_unix_ms: now,
            })
            .collect::<Vec<_>>();
        self.record_market_events(events, source, connection)
    }

    fn record_market_events(
        &mut self,
        events: Vec<StrategyMarketEvent>,
        source: &str,
        connection: &str,
    ) -> anyhow::Result<StrategyMarketStreamSummary> {
        let last_event_at_unix_ms = events.last().map(|event| event.event_at_unix_ms);
        self.market_events.extend(events);
        let event_count = u64::try_from(self.market_events.len()).unwrap_or(u64::MAX);
        let status = StrategyMarketStreamStatus {
            schema_version: STRATEGY_MARKET_STATUS_SCHEMA_VERSION.to_string(),
            session_id: self.status.session_id.clone(),
            strategy_id: self.status.strategy_id.clone(),
            connection: connection.to_string(),
            state: connection.to_string(),
            source: source.to_string(),
            event_count,
            last_event_at_unix_ms,
            updated_at_unix_ms: unix_timestamp_millis(),
        };
        self.market_status = Some(status);
        self.persist()?;

        Ok(StrategyMarketStreamSummary {
            schema_version: "ntpro.v09_market_stream_summary.v1".to_string(),
            session_id: self.status.session_id.clone(),
            strategy_id: self.status.strategy_id.clone(),
            connection: connection.to_string(),
            state: connection.to_string(),
            source: source.to_string(),
            event_count,
            last_event_at_unix_ms,
            market_status_artifact: self.artifacts.market_status.display().to_string(),
            market_events_artifact: self.artifacts.market_events.display().to_string(),
        })
    }

    fn persist(&self) -> anyhow::Result<()> {
        atomic_write_json(&self.artifacts.session_status, &self.status)?;
        let mut body = String::new();
        for event in &self.events {
            body.push_str(&serde_json::to_string(event)?);
            body.push('\n');
        }
        atomic_write_text(&self.artifacts.events, &body)?;

        if let Some(market_status) = &self.market_status {
            atomic_write_json(&self.artifacts.market_status, market_status)?;
        }
        let mut market_body = String::new();
        for market_event in &self.market_events {
            market_body.push_str(&serde_json::to_string(market_event)?);
            market_body.push('\n');
        }
        atomic_write_text(&self.artifacts.market_events, &market_body)?;

        let mut signal_body = String::new();
        for signal in &self.signals {
            signal_body.push_str(&serde_json::to_string(signal)?);
            signal_body.push('\n');
        }
        atomic_write_text(&self.artifacts.signal, &signal_body)?;

        let mut order_intent_body = String::new();
        for order_intent in &self.order_intents {
            order_intent_body.push_str(&serde_json::to_string(order_intent)?);
            order_intent_body.push('\n');
        }
        atomic_write_text(&self.artifacts.order_intent, &order_intent_body)?;

        let mut risk_decision_body = String::new();
        for risk_decision in &self.risk_decisions {
            risk_decision_body.push_str(&serde_json::to_string(risk_decision)?);
            risk_decision_body.push('\n');
        }
        atomic_write_text(&self.artifacts.risk_decision, &risk_decision_body)?;

        if let Some(summary) = &self.summary {
            atomic_write_json(&self.artifacts.summary, summary)?;
        }
        if let Some(simulation) = &self.demo_simulation {
            atomic_write_text(
                &self.artifacts.simulation_summary,
                std::str::from_utf8(&simulation.summary)
                    .map_err(|error| anyhow::anyhow!("simulation summary is not UTF-8: {error}"))?,
            )?;
            atomic_write_text(
                &self.artifacts.simulated_fills,
                std::str::from_utf8(&simulation.fills)
                    .map_err(|error| anyhow::anyhow!("simulated fills are not UTF-8: {error}"))?,
            )?;
            atomic_write_text(
                &self.artifacts.simulated_positions,
                std::str::from_utf8(&simulation.positions).map_err(|error| {
                    anyhow::anyhow!("simulated positions are not UTF-8: {error}")
                })?,
            )?;
            atomic_write_text(
                &self.artifacts.equity_curve,
                std::str::from_utf8(&simulation.equity_curve)
                    .map_err(|error| anyhow::anyhow!("equity curve is not UTF-8: {error}"))?,
            )?;
        }
        let manifest = self.manifest()?;
        atomic_write_json(&self.artifacts.manifest, &manifest)?;
        Ok(())
    }

    fn manifest(&self) -> anyhow::Result<StrategySessionManifest> {
        let created_at_unix_ms = self
            .events
            .first()
            .map_or(self.status.updated_at_unix_ms, |event| {
                event.occurred_at_unix_ms
            });
        let artifacts = vec![
            manifest_artifact(
                "session_status",
                "json",
                &self.artifacts.session_status,
                Some(1),
            )?,
            manifest_artifact(
                "events",
                "jsonl",
                &self.artifacts.events,
                Some(u64::try_from(self.events.len()).unwrap_or(u64::MAX)),
            )?,
            manifest_artifact(
                "market_status",
                "json",
                &self.artifacts.market_status,
                self.market_status.as_ref().map(|_| 1),
            )?,
            manifest_artifact(
                "market_events",
                "jsonl",
                &self.artifacts.market_events,
                Some(u64::try_from(self.market_events.len()).unwrap_or(u64::MAX)),
            )?,
            manifest_artifact(
                "signal",
                "jsonl",
                &self.artifacts.signal,
                Some(u64::try_from(self.signals.len()).unwrap_or(u64::MAX)),
            )?,
            manifest_artifact(
                "order_intent",
                "jsonl",
                &self.artifacts.order_intent,
                Some(u64::try_from(self.order_intents.len()).unwrap_or(u64::MAX)),
            )?,
            manifest_artifact(
                "risk_decision",
                "jsonl",
                &self.artifacts.risk_decision,
                Some(u64::try_from(self.risk_decisions.len()).unwrap_or(u64::MAX)),
            )?,
            manifest_artifact(
                "summary",
                "json",
                &self.artifacts.summary,
                self.summary.as_ref().map(|_| 1),
            )?,
            manifest_artifact(
                "simulation_summary",
                "json",
                &self.artifacts.simulation_summary,
                self.demo_simulation.as_ref().map(|_| 1),
            )?,
            manifest_artifact(
                "simulated_fills",
                "jsonl",
                &self.artifacts.simulated_fills,
                self.demo_simulation
                    .as_ref()
                    .map(|simulation| u64::try_from(simulation.fill_count).unwrap_or(u64::MAX)),
            )?,
            manifest_artifact(
                "simulated_positions",
                "jsonl",
                &self.artifacts.simulated_positions,
                self.demo_simulation
                    .as_ref()
                    .map(|simulation| u64::try_from(simulation.position_count).unwrap_or(u64::MAX)),
            )?,
            manifest_artifact(
                "equity_curve",
                "jsonl",
                &self.artifacts.equity_curve,
                self.demo_simulation.as_ref().map(|simulation| {
                    u64::try_from(simulation.equity_point_count).unwrap_or(u64::MAX)
                }),
            )?,
        ];

        Ok(StrategySessionManifest {
            schema_version: STRATEGY_SESSION_MANIFEST_SCHEMA_VERSION.to_string(),
            session_id: self.status.session_id.clone(),
            strategy_id: self.status.strategy_id.clone(),
            state: self.status.state,
            created_at_unix_ms,
            updated_at_unix_ms: self.status.updated_at_unix_ms,
            artifacts,
        })
    }
}

fn manifest_artifact(
    name: &str,
    format: &str,
    path: &Path,
    record_count: Option<u64>,
) -> anyhow::Result<StrategySessionManifestArtifact> {
    match fs::read(path) {
        Ok(bytes) => Ok(StrategySessionManifestArtifact {
            name: name.to_string(),
            path: path.display().to_string(),
            format: format.to_string(),
            present: true,
            record_count,
            byte_len: Some(u64::try_from(bytes.len()).unwrap_or(u64::MAX)),
            checksum: Some(checksum_bytes(&bytes)),
        }),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(StrategySessionManifestArtifact {
            name: name.to_string(),
            path: path.display().to_string(),
            format: format.to_string(),
            present: false,
            record_count: None,
            byte_len: None,
            checksum: None,
        }),
        Err(err) => Err(err).map_err(|err| {
            anyhow::anyhow!(
                "failed to read Strategy Session artifact '{}' for manifest: {err}",
                path.display()
            )
        }),
    }
}

#[cfg(test)]
fn checksum_file(path: &Path) -> anyhow::Result<String> {
    let bytes = fs::read(path).map_err(|err| {
        anyhow::anyhow!(
            "failed to read Strategy Session artifact '{}' for checksum: {err}",
            path.display()
        )
    })?;
    Ok(checksum_bytes(&bytes))
}

fn checksum_bytes(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

#[must_use]
pub fn audit_strategy_session_artifacts(
    strategy_root: &Path,
    node_lifecycle_state: Option<&str>,
) -> StrategySessionArtifactAudit {
    let manifest_path = strategy_root.join("manifest.json");
    let mut diagnostics = Vec::new();
    let manifest = match fs::read_to_string(&manifest_path) {
        Ok(raw) => match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(value) => value,
            Err(error) => {
                diagnostics.push(format!("strategy manifest JSON invalid: {error}"));
                return StrategySessionArtifactAudit {
                    health: StrategySessionArtifactAuditHealth::Degraded,
                    manifest_path,
                    diagnostics,
                };
            }
        },
        Err(error) if error.kind() == ErrorKind::NotFound => {
            diagnostics.push("strategy manifest missing".to_string());
            return StrategySessionArtifactAudit {
                health: StrategySessionArtifactAuditHealth::Degraded,
                manifest_path,
                diagnostics,
            };
        }
        Err(error) => {
            diagnostics.push(format!("strategy manifest unreadable: {error}"));
            return StrategySessionArtifactAudit {
                health: StrategySessionArtifactAuditHealth::Degraded,
                manifest_path,
                diagnostics,
            };
        }
    };

    if manifest
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        != Some(STRATEGY_SESSION_MANIFEST_SCHEMA_VERSION)
    {
        diagnostics.push("strategy manifest schema mismatch".to_string());
    }

    let session_state = manifest
        .get("state")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    if let Some(node_state) = node_lifecycle_state {
        if node_state == "stopped" && session_state != "stopped" {
            diagnostics.push(format!(
                "node lifecycle stopped but strategy session state is {session_state}"
            ));
        }
        if node_state == "running" && session_state == "stopped" {
            diagnostics
                .push("node lifecycle running but strategy session state is stopped".to_string());
        }
    }

    if let Some(market_state) = read_strategy_market_state(strategy_root) {
        if session_state == "running" && market_state == MARKET_STATE_STOPPED {
            diagnostics.push("strategy session running but market stream is stopped".to_string());
        }
        if session_state == "stopped"
            && !matches!(
                market_state.as_str(),
                MARKET_STATE_STOPPED | MARKET_STATE_EXHAUSTED
            )
        {
            diagnostics.push(format!(
                "strategy session stopped but market stream state is {market_state}"
            ));
        }
    }

    let Some(artifacts) = manifest
        .get("artifacts")
        .and_then(serde_json::Value::as_array)
    else {
        diagnostics.push("strategy manifest artifacts array missing".to_string());
        return StrategySessionArtifactAudit {
            health: StrategySessionArtifactAuditHealth::Degraded,
            manifest_path,
            diagnostics,
        };
    };

    for artifact in artifacts {
        audit_manifest_child_artifact(strategy_root, artifact, &mut diagnostics);
    }

    StrategySessionArtifactAudit {
        health: if diagnostics.is_empty() {
            StrategySessionArtifactAuditHealth::Healthy
        } else {
            StrategySessionArtifactAuditHealth::Degraded
        },
        manifest_path,
        diagnostics,
    }
}

fn read_strategy_market_state(strategy_root: &Path) -> Option<String> {
    let raw = fs::read_to_string(strategy_root.join("market_status.json")).ok()?;
    let value = serde_json::from_str::<serde_json::Value>(&raw).ok()?;
    value
        .get("state")
        .or_else(|| value.get("connection"))
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
}

fn audit_manifest_child_artifact(
    strategy_root: &Path,
    artifact: &serde_json::Value,
    diagnostics: &mut Vec<String>,
) {
    let name = artifact
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let present = artifact
        .get("present")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let Some(path_value) = artifact.get("path").and_then(serde_json::Value::as_str) else {
        diagnostics.push(format!("strategy artifact {name} path missing"));
        return;
    };
    let path = strategy_artifact_path(strategy_root, path_value);
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            if present {
                diagnostics.push(format!("strategy artifact {name} missing"));
            }
            return;
        }
        Err(error) => {
            diagnostics.push(format!("strategy artifact {name} unreadable: {error}"));
            return;
        }
    };

    if !present {
        diagnostics.push(format!(
            "strategy artifact {name} exists but manifest marks it missing"
        ));
    }

    if artifact
        .get("byte_len")
        .and_then(serde_json::Value::as_u64)
        .is_some_and(|expected| expected != u64::try_from(bytes.len()).unwrap_or(u64::MAX))
    {
        diagnostics.push(format!("strategy artifact {name} byte_len mismatch"));
    }

    if artifact
        .get("checksum")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|expected| expected != checksum_bytes(&bytes))
    {
        diagnostics.push(format!("strategy artifact {name} checksum mismatch"));
    }

    let format = artifact
        .get("format")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    if format == "jsonl"
        && artifact
            .get("record_count")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|expected| expected != jsonl_record_count(&bytes))
    {
        diagnostics.push(format!("strategy artifact {name} record_count mismatch"));
    }
}

fn strategy_artifact_path(strategy_root: &Path, path_value: &str) -> PathBuf {
    let path = PathBuf::from(path_value);
    if path.is_absolute() {
        path
    } else {
        strategy_root.join(path)
    }
}

fn jsonl_record_count(bytes: &[u8]) -> u64 {
    let text = String::from_utf8_lossy(bytes);
    u64::try_from(text.lines().filter(|line| !line.trim().is_empty()).count()).unwrap_or(u64::MAX)
}

pub fn ema_cross_demo_fixture_bars(symbol: &str) -> Vec<StrategyMarketBar> {
    let base_ts = 1_725_000_000_000;
    [
        100.0, 99.0, 98.0, 99.5, 101.0, 103.5, 102.0, 100.5, 98.0, 96.0, 99.0, 102.0,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, close)| StrategyMarketBar {
        seq: u64::try_from(index + 1).unwrap_or(u64::MAX),
        symbol: symbol.to_string(),
        close,
        closed_at_unix_ms: base_ts + u64::try_from(index).unwrap_or(0) * 60_000,
    })
    .collect()
}

fn validate_fixture_bars(bars: &[StrategyMarketBar]) -> anyhow::Result<()> {
    if bars.is_empty() {
        anyhow::bail!("fixture bars must not be empty");
    }
    let mut previous_seq = 0;
    for bar in bars {
        if bar.seq == 0 {
            anyhow::bail!("fixture bar seq must be greater than zero");
        }
        if bar.seq <= previous_seq {
            anyhow::bail!("fixture bar seq must be strictly increasing");
        }
        previous_seq = bar.seq;
        if bar.symbol.trim().is_empty() {
            anyhow::bail!("fixture bar symbol must not be empty");
        }
        if !bar.close.is_finite() || bar.close <= 0.0 {
            anyhow::bail!("fixture bar close must be a positive finite number");
        }
    }
    Ok(())
}

fn validate_mock_ticks(ticks: &[StrategyMarketTick]) -> anyhow::Result<()> {
    if ticks.is_empty() {
        anyhow::bail!("mock ticks must not be empty");
    }
    let mut previous_seq = 0;
    for tick in ticks {
        if tick.seq == 0 {
            anyhow::bail!("mock tick seq must be greater than zero");
        }
        if tick.seq <= previous_seq {
            anyhow::bail!("mock tick seq must be strictly increasing");
        }
        previous_seq = tick.seq;
        if tick.symbol.trim().is_empty() {
            anyhow::bail!("mock tick symbol must not be empty");
        }
        if !tick.price.is_finite() || tick.price <= 0.0 {
            anyhow::bail!("mock tick price must be a positive finite number");
        }
    }
    Ok(())
}

fn ema_next(previous: Option<f64>, price: f64, period: u32) -> f64 {
    let multiplier = 2.0 / (f64::from(period) + 1.0);
    previous.map_or(price, |previous| {
        price.mul_add(multiplier, previous * (1.0 - multiplier))
    })
}

fn confidence_from_ema_gap(fast: f64, slow: f64) -> f64 {
    let gap = ((fast - slow).abs() / slow.abs()).min(1.0);
    (0.5 + gap).min(0.99)
}

fn order_intent_side(signal: &str) -> &'static str {
    match signal {
        "long" => "buy",
        "flat" => "flatten",
        _ => "observe",
    }
}

fn shadow_risk_rejection_reasons(
    order_submission: &str,
    kill_switch_active: bool,
    mode: &str,
    account_state: &str,
    market_state: &str,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if order_submission == "disabled" {
        reasons.push("order_submission_disabled".to_string());
    }
    if kill_switch_active {
        reasons.push("kill_switch_active".to_string());
    }
    if mode == "shadow" {
        reasons.push("shadow_mode_actual_submission_disabled".to_string());
    }
    if account_state == "missing" {
        reasons.push("account_state_missing".to_string());
    }
    if market_state == "missing" {
        reasons.push("market_state_missing".to_string());
    }
    reasons
}

fn is_legal_transition(previous: StrategySessionState, next: StrategySessionState) -> bool {
    use StrategySessionState::{
        Created, Failed, Paused, RiskHalted, Running, Starting, Stopped, Stopping, Validated,
    };

    matches!(
        (previous, next),
        (Created, Validated | Failed)
            | (Validated, Starting | Failed)
            | (Starting | Paused, Running)
            | (Starting | Running | Paused | RiskHalted | Stopping, Failed)
            | (Running, Paused | RiskHalted | Stopping)
            | (Paused | RiskHalted, Stopping)
            | (Stopping, Stopped)
    )
}

fn non_empty(field: &str, value: String) -> anyhow::Result<String> {
    if value.trim().is_empty() {
        anyhow::bail!("{field} must not be empty");
    }
    Ok(value)
}

fn unix_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
        })
}

fn format_unix_millis(timestamp_ms: u64) -> String {
    format!("unix:{timestamp_ms}")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::Value;

    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "ntpro-v090-strategy-session-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn manifest_artifact<'a>(manifest: &'a Value, name: &str) -> &'a Value {
        manifest["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|artifact| artifact["name"] == name)
            .unwrap_or_else(|| panic!("manifest artifact '{name}' not found"))
    }

    #[test]
    fn strategy_session_lifecycle_writes_status_and_events() {
        let root = temp_root("lifecycle");
        let mut session = StrategySession::new("session-001", "ema-cross-demo", &root).unwrap();

        session
            .transition(StrategySessionState::Validated, "config validated")
            .unwrap();
        session
            .transition(StrategySessionState::Starting, "runtime starting")
            .unwrap();
        session
            .transition(StrategySessionState::Running, "runtime running")
            .unwrap();
        session
            .transition(StrategySessionState::Paused, "operator pause")
            .unwrap();
        session
            .transition(StrategySessionState::Running, "operator resume")
            .unwrap();
        session
            .transition(StrategySessionState::Stopping, "shutdown requested")
            .unwrap();
        session
            .transition(StrategySessionState::Stopped, "shutdown complete")
            .unwrap();

        assert_eq!(session.status().state, StrategySessionState::Stopped);

        let status_path = root.join("strategy/session_status.json");
        let status: Value =
            serde_json::from_str(&fs::read_to_string(status_path).unwrap()).unwrap();
        assert_eq!(
            status["schema_version"],
            STRATEGY_SESSION_STATUS_SCHEMA_VERSION
        );
        assert_eq!(status["session_id"], "session-001");
        assert_eq!(status["strategy_id"], "ema-cross-demo");
        assert_eq!(status["state"], "stopped");
        assert_eq!(status["reason"], "shutdown complete");
        assert!(
            status["artifacts"]["events"]
                .as_str()
                .unwrap()
                .ends_with("strategy/events.jsonl")
        );
        assert!(
            status["artifacts"]["market_status"]
                .as_str()
                .unwrap()
                .ends_with("strategy/market_status.json")
        );
        assert!(
            status["artifacts"]["market_events"]
                .as_str()
                .unwrap()
                .ends_with("strategy/market_events.jsonl")
        );
        assert!(
            status["artifacts"]["signal"]
                .as_str()
                .unwrap()
                .ends_with("strategy/signal.jsonl")
        );
        assert!(
            status["artifacts"]["order_intent"]
                .as_str()
                .unwrap()
                .ends_with("strategy/order_intent.jsonl")
        );
        assert!(
            status["artifacts"]["risk_decision"]
                .as_str()
                .unwrap()
                .ends_with("strategy/risk_decision.jsonl")
        );
        assert!(
            status["artifacts"]["summary"]
                .as_str()
                .unwrap()
                .ends_with("strategy/summary.json")
        );
        assert!(
            status["artifacts"]["manifest"]
                .as_str()
                .unwrap()
                .ends_with("strategy/manifest.json")
        );

        let events = fs::read_to_string(root.join("strategy/events.jsonl")).unwrap();
        let event_lines = events.lines().collect::<Vec<_>>();
        assert_eq!(event_lines.len(), 8);
        assert!(event_lines[0].contains(r#""state":"created""#));
        assert!(
            event_lines
                .iter()
                .any(|event| event.contains(r#""state":"paused""#))
        );
        assert!(event_lines[7].contains(r#""state":"stopped""#));
        assert!(event_lines[7].contains(r#""previous_state":"stopping""#));
    }

    #[test]
    fn fixture_market_stream_writes_status_and_events() {
        let root = temp_root("fixture-market-stream");
        let mut session = StrategySession::new("session-006", "ema-cross-demo", &root).unwrap();
        let bars = ema_cross_demo_fixture_bars("BTCUSDT.BINANCE");

        let summary = session.record_fixture_bar_stream(&bars).unwrap();

        assert_eq!(summary.connection, MARKET_STATE_EXHAUSTED);
        assert_eq!(summary.state, MARKET_STATE_EXHAUSTED);
        assert_eq!(summary.source, "fixture_bar_stream");
        assert_eq!(summary.event_count, u64::try_from(bars.len()).unwrap());
        assert_eq!(
            summary.last_event_at_unix_ms,
            bars.last().map(|bar| bar.closed_at_unix_ms)
        );

        let status: Value = serde_json::from_str(
            &fs::read_to_string(root.join("strategy/market_status.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            status["schema_version"],
            STRATEGY_MARKET_STATUS_SCHEMA_VERSION
        );
        assert_eq!(status["connection"], MARKET_STATE_EXHAUSTED);
        assert_eq!(status["state"], MARKET_STATE_EXHAUSTED);
        assert_eq!(status["source"], "fixture_bar_stream");
        assert_eq!(status["event_count"], bars.len());

        let events = fs::read_to_string(root.join("strategy/market_events.jsonl")).unwrap();
        let event_lines = events.lines().collect::<Vec<_>>();
        assert_eq!(event_lines.len(), bars.len());
        let first_event: Value = serde_json::from_str(event_lines[0]).unwrap();
        assert_eq!(
            first_event["schema_version"],
            STRATEGY_MARKET_EVENT_SCHEMA_VERSION
        );
        assert_eq!(first_event["event_type"], "fixture_bar");
        assert_eq!(first_event["source"], "fixture_bar_stream");
        assert_eq!(first_event["symbol"], "BTCUSDT.BINANCE");
    }

    #[test]
    fn mock_tick_stream_writes_market_events() {
        let root = temp_root("mock-tick-stream");
        let mut session = StrategySession::new("session-007", "ema-cross-demo", &root).unwrap();
        let ticks = vec![
            StrategyMarketTick {
                seq: 1,
                symbol: "BTCUSDT.BINANCE".to_string(),
                price: 101.0,
                observed_at_unix_ms: 1_725_000_000_000,
            },
            StrategyMarketTick {
                seq: 2,
                symbol: "BTCUSDT.BINANCE".to_string(),
                price: 102.0,
                observed_at_unix_ms: 1_725_000_001_000,
            },
        ];

        let summary = session.record_mock_tick_stream(&ticks).unwrap();

        assert_eq!(summary.connection, MARKET_STATE_EXHAUSTED);
        assert_eq!(summary.state, MARKET_STATE_EXHAUSTED);
        assert_eq!(summary.source, "mock_tick_stream");
        assert_eq!(summary.event_count, 2);

        let events = fs::read_to_string(root.join("strategy/market_events.jsonl")).unwrap();
        let event_lines = events.lines().collect::<Vec<_>>();
        assert_eq!(event_lines.len(), 2);
        let second_event: Value = serde_json::from_str(event_lines[1]).unwrap();
        assert_eq!(second_event["event_type"], "mock_tick");
        assert_eq!(second_event["price"], 102.0);
    }

    #[test]
    fn ema_cross_demo_generates_signal_artifact() {
        let root = temp_root("ema-cross-demo");
        let mut session = StrategySession::new("session-004", "ema-cross-demo", &root).unwrap();

        let summary = session
            .run_ema_cross_demo(&ema_cross_demo_fixture_bars("BTCUSDT.BINANCE"))
            .unwrap();

        assert_eq!(summary.strategy, "ema_cross_demo");
        assert_eq!(summary.processed_events, 12);
        assert!(summary.signal_count >= 1);
        assert_eq!(summary.order_intent_count, summary.signal_count);
        assert!(
            summary
                .order_intent_artifact
                .ends_with("strategy/order_intent.jsonl")
        );
        assert_eq!(summary.risk_decision_count, summary.order_intent_count);
        assert!(
            summary
                .risk_decision_artifact
                .ends_with("strategy/risk_decision.jsonl")
        );
        assert!(summary.summary_artifact.ends_with("strategy/summary.json"));
        assert!(!summary.order_submission_allowed);
        assert_eq!(session.status().state, StrategySessionState::Running);
        assert_eq!(
            summary.counters.market_event_count,
            summary.processed_events
        );
        assert_eq!(summary.counters.signal_count, summary.signal_count);
        assert_eq!(summary.counters.intent_count, summary.order_intent_count);
        assert_eq!(
            summary.counters.risk_decision_count,
            summary.risk_decision_count
        );
        assert_eq!(
            summary.counters.rejection_count,
            summary.risk_decision_count
        );
        assert_eq!(summary.counters.actual_submission_count, 0);

        let market_status: Value = serde_json::from_str(
            &fs::read_to_string(root.join("strategy/market_status.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            market_status["connection"], MARKET_STATE_EXHAUSTED,
            "demo strategy exhausts the finite fixture market stream"
        );
        assert_eq!(market_status["state"], MARKET_STATE_EXHAUSTED);
        assert_eq!(market_status["event_count"], summary.processed_events);
        let market_events = fs::read_to_string(root.join("strategy/market_events.jsonl")).unwrap();
        assert_eq!(
            market_events.lines().count(),
            usize::try_from(summary.processed_events).unwrap()
        );

        let signal_path = root.join("strategy/signal.jsonl");
        let signal_lines = fs::read_to_string(signal_path).unwrap();
        let signals = signal_lines.lines().collect::<Vec<_>>();
        assert_eq!(
            signals.len(),
            usize::try_from(summary.signal_count).unwrap()
        );
        let parsed_signals = signals
            .iter()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert!(
            parsed_signals
                .iter()
                .any(|signal| signal["signal"] == "long")
        );
        for signal in parsed_signals {
            assert_eq!(signal["schema_version"], STRATEGY_SIGNAL_SCHEMA_VERSION);
            assert_eq!(signal["session_id"], "session-004");
            assert_eq!(signal["strategy_id"], "ema-cross-demo");
            assert_eq!(signal["symbol"], "BTCUSDT.BINANCE");
            assert!(
                signal["generated_at"]
                    .as_str()
                    .unwrap()
                    .starts_with("unix:")
            );
            assert!(signal["market_event_seq"].as_u64().unwrap() > 0);
            assert!(signal["confidence"].as_f64().unwrap() > 0.5);
        }

        let order_intent_lines =
            fs::read_to_string(root.join("strategy/order_intent.jsonl")).unwrap();
        assert_eq!(
            order_intent_lines.lines().count(),
            usize::try_from(summary.order_intent_count).unwrap()
        );

        let risk_decision_lines =
            fs::read_to_string(root.join("strategy/risk_decision.jsonl")).unwrap();
        assert_eq!(
            risk_decision_lines.lines().count(),
            usize::try_from(summary.risk_decision_count).unwrap()
        );

        let summary_json: Value =
            serde_json::from_str(&fs::read_to_string(root.join("strategy/summary.json")).unwrap())
                .unwrap();
        assert_eq!(
            summary_json["schema_version"],
            STRATEGY_SESSION_SUMMARY_SCHEMA_VERSION
        );
        assert_eq!(summary_json["state"], "running");
        assert_eq!(summary_json["signal_count"], summary.signal_count);
        assert_eq!(summary_json["intent_count"], summary.order_intent_count);
        assert_eq!(
            summary_json["risk_decision_count"],
            summary.risk_decision_count
        );
        assert_eq!(
            summary_json["market_event_count"],
            summary.counters.market_event_count
        );
        assert_eq!(summary_json["rejection_count"], summary.risk_decision_count);
        assert_eq!(summary_json["actual_submission_count"], 0);

        let audit_events = fs::read_to_string(root.join("strategy/events.jsonl")).unwrap();
        assert_eq!(
            summary_json["event_count"],
            u64::try_from(audit_events.lines().count()).unwrap()
        );
        assert!(audit_events.contains("demo strategy starting"));
        assert!(!audit_events.contains("demo strategy stopped"));
        assert_eq!(
            audit_events
                .lines()
                .filter(|event| event.contains("strategy_risk_decision_rejected"))
                .count(),
            usize::try_from(summary.risk_decision_count).unwrap()
        );

        let manifest_path = root.join("strategy/manifest.json");
        let manifest: Value =
            serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
        assert_eq!(
            manifest["schema_version"],
            STRATEGY_SESSION_MANIFEST_SCHEMA_VERSION
        );
        assert_eq!(manifest["session_id"], "session-004");
        assert_eq!(manifest["strategy_id"], "ema-cross-demo");
        assert_eq!(manifest["state"], "running");
        assert_eq!(manifest["artifacts"].as_array().unwrap().len(), 12);
        assert!(!manifest.to_string().contains("exchange_order_id"));
        assert!(!manifest.to_string().contains("venue_order_id"));

        for name in [
            "session_status",
            "events",
            "market_status",
            "market_events",
            "signal",
            "order_intent",
            "risk_decision",
            "summary",
        ] {
            let artifact = manifest_artifact(&manifest, name);
            assert_eq!(artifact["present"], true);
            assert!(
                artifact["checksum"]
                    .as_str()
                    .is_some_and(|checksum| checksum.starts_with("blake3:"))
            );
            assert!(artifact["byte_len"].as_u64().is_some_and(|value| value > 0));
        }
        assert_eq!(
            manifest_artifact(&manifest, "events")["record_count"],
            u64::try_from(audit_events.lines().count()).unwrap()
        );
        assert_eq!(
            manifest_artifact(&manifest, "market_events")["record_count"],
            summary.processed_events
        );
        assert_eq!(
            manifest_artifact(&manifest, "signal")["record_count"],
            summary.signal_count
        );
        assert_eq!(
            manifest_artifact(&manifest, "order_intent")["record_count"],
            summary.order_intent_count
        );
        assert_eq!(
            manifest_artifact(&manifest, "risk_decision")["record_count"],
            summary.risk_decision_count
        );

        let signal_manifest_checksum = manifest_artifact(&manifest, "signal")["checksum"]
            .as_str()
            .unwrap()
            .to_string();
        fs::write(root.join("strategy/signal.jsonl"), "corrupted\n").unwrap();
        let corrupted_signal_checksum = checksum_file(&root.join("strategy/signal.jsonl")).unwrap();
        assert_ne!(signal_manifest_checksum, corrupted_signal_checksum);
    }

    #[test]
    fn signal_jsonl_contains_required_contract_fields() {
        let root = temp_root("signal-contract");
        let mut session = StrategySession::new("session-008", "ema-cross-demo", &root).unwrap();

        session
            .run_ema_cross_demo(&ema_cross_demo_fixture_bars("BTCUSDT.BINANCE"))
            .unwrap();

        let signal_lines = fs::read_to_string(root.join("strategy/signal.jsonl")).unwrap();
        assert!(!signal_lines.trim().is_empty());
        for line in signal_lines.lines() {
            let signal: Value = serde_json::from_str(line).unwrap();
            assert_eq!(signal["schema_version"], STRATEGY_SIGNAL_SCHEMA_VERSION);
            assert!(
                signal["session_id"]
                    .as_str()
                    .is_some_and(|value| !value.is_empty())
            );
            assert!(
                signal["strategy_id"]
                    .as_str()
                    .is_some_and(|value| !value.is_empty())
            );
            assert!(
                signal["symbol"]
                    .as_str()
                    .is_some_and(|value| !value.is_empty())
            );
            assert!(
                signal["signal"]
                    .as_str()
                    .is_some_and(|value| !value.is_empty())
            );
            assert!(signal["confidence"].as_f64().is_some());
            assert!(signal["market_event_seq"].as_u64().is_some());
            assert!(
                signal["generated_at"]
                    .as_str()
                    .is_some_and(|value| value.starts_with("unix:"))
            );
        }
    }

    #[test]
    fn order_intent_jsonl_blocks_exchange_submission() {
        let root = temp_root("order-intent-contract");
        let mut session = StrategySession::new("session-009", "ema-cross-demo", &root).unwrap();

        session
            .run_ema_cross_demo(&ema_cross_demo_fixture_bars("BTCUSDT.BINANCE"))
            .unwrap();

        let order_intent_lines =
            fs::read_to_string(root.join("strategy/order_intent.jsonl")).unwrap();
        assert!(!order_intent_lines.trim().is_empty());
        for line in order_intent_lines.lines() {
            let order_intent: Value = serde_json::from_str(line).unwrap();
            assert_eq!(
                order_intent["schema_version"],
                STRATEGY_ORDER_INTENT_SCHEMA_VERSION
            );
            assert_eq!(order_intent["session_id"], "session-009");
            assert_eq!(order_intent["strategy_id"], "ema-cross-demo");
            assert_eq!(order_intent["symbol"], "BTCUSDT.BINANCE");
            assert!(
                order_intent["intent_id"]
                    .as_str()
                    .is_some_and(|value| value.starts_with("session-009:ema-cross-demo:"))
            );
            assert!(
                order_intent["side"]
                    .as_str()
                    .is_some_and(|value| matches!(value, "buy" | "flatten" | "observe"))
            );
            assert_eq!(order_intent["order_type"], "market");
            assert_eq!(order_intent["quantity"], 1.0);
            assert!(
                order_intent["source_signal"]
                    .as_str()
                    .is_some_and(|value| !value.is_empty())
            );
            assert!(order_intent["confidence"].as_f64().is_some());
            assert!(order_intent["market_event_seq"].as_u64().is_some());
            assert!(
                order_intent["signal_generated_at"]
                    .as_str()
                    .is_some_and(|value| value.starts_with("unix:"))
            );
            assert!(
                order_intent["created_at"]
                    .as_str()
                    .is_some_and(|value| value.starts_with("unix:"))
            );
            assert_eq!(order_intent["submission_allowed"], false);
            assert_eq!(
                order_intent["submission_status"],
                "blocked_by_v09_strategy_runtime_boundary"
            );
            assert!(
                order_intent.get("exchange_order_id").is_none(),
                "v0.9 order intents must not claim exchange order identity"
            );
            assert!(
                order_intent.get("venue_order_id").is_none(),
                "v0.9 order intents must not claim venue order identity"
            );
        }
    }

    #[test]
    fn risk_decision_jsonl_rejects_shadow_order_intents() {
        let root = temp_root("risk-decision-contract");
        let mut session = StrategySession::new("session-010", "ema-cross-demo", &root).unwrap();

        session
            .run_ema_cross_demo(&ema_cross_demo_fixture_bars("BTCUSDT.BINANCE"))
            .unwrap();

        let order_intent_lines =
            fs::read_to_string(root.join("strategy/order_intent.jsonl")).unwrap();
        let order_intents = order_intent_lines
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        let risk_decision_lines =
            fs::read_to_string(root.join("strategy/risk_decision.jsonl")).unwrap();
        let risk_decisions = risk_decision_lines
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(risk_decisions.len(), order_intents.len());
        assert!(!risk_decisions.is_empty());

        for (risk_decision, order_intent) in risk_decisions.iter().zip(order_intents.iter()) {
            assert_eq!(
                risk_decision["schema_version"],
                STRATEGY_RISK_DECISION_SCHEMA_VERSION
            );
            assert_eq!(risk_decision["session_id"], "session-010");
            assert_eq!(risk_decision["strategy_id"], "ema-cross-demo");
            assert_eq!(risk_decision["intent_id"], order_intent["intent_id"]);
            assert_eq!(risk_decision["symbol"], "BTCUSDT.BINANCE");
            assert_eq!(risk_decision["decision"], "rejected");
            assert_eq!(risk_decision["mode"], "shadow");
            assert_eq!(risk_decision["order_submission"], "disabled");
            assert_eq!(risk_decision["kill_switch_enabled"], true);
            assert_eq!(risk_decision["kill_switch_active"], false);
            assert!(risk_decision.get("kill_switch").is_none());
            assert_eq!(risk_decision["account_state"], "missing");
            assert_eq!(risk_decision["market_state"], "available");
            assert_eq!(risk_decision["actual_submission"], false);
            assert!(
                risk_decision["decision_id"]
                    .as_str()
                    .is_some_and(|value| value.starts_with("risk:"))
            );
            assert!(
                risk_decision["evaluated_at"]
                    .as_str()
                    .is_some_and(|value| value.starts_with("unix:"))
            );
            let reasons = risk_decision["reasons"].as_array().unwrap();
            assert!(
                reasons
                    .iter()
                    .any(|reason| reason == "order_submission_disabled")
            );
            assert!(
                reasons
                    .iter()
                    .any(|reason| reason == "shadow_mode_actual_submission_disabled")
            );
            assert!(
                reasons
                    .iter()
                    .any(|reason| reason == "account_state_missing")
            );
            assert!(
                !reasons
                    .iter()
                    .any(|reason| reason == "market_state_missing"),
                "fixture market state is available for the demo run"
            );
            assert!(
                risk_decision.get("exchange_order_id").is_none(),
                "v0.9 risk decisions must not claim exchange order identity"
            );
            assert!(
                risk_decision.get("venue_order_id").is_none(),
                "v0.9 risk decisions must not claim venue order identity"
            );
        }
    }

    #[test]
    fn shadow_risk_rejection_reasons_cover_default_reject_rules() {
        let reasons =
            shadow_risk_rejection_reasons("disabled", true, "shadow", "missing", "missing");

        assert_eq!(
            reasons,
            vec![
                "order_submission_disabled",
                "kill_switch_active",
                "shadow_mode_actual_submission_disabled",
                "account_state_missing",
                "market_state_missing",
            ]
        );
    }

    #[test]
    fn active_kill_switch_adds_explicit_rejection_reason() {
        let root = temp_root("risk-decision-active-kill-switch");
        let mut session = StrategySession::new("session-012", "ema-cross-demo", &root).unwrap();
        session.set_risk_controls(StrategyRiskControls {
            kill_switch_enabled: true,
            kill_switch_active: true,
        });

        session
            .run_ema_cross_demo(&ema_cross_demo_fixture_bars("BTCUSDT.BINANCE"))
            .unwrap();

        let risk_decision_lines =
            fs::read_to_string(root.join("strategy/risk_decision.jsonl")).unwrap();
        for line in risk_decision_lines.lines() {
            let risk_decision: Value = serde_json::from_str(line).unwrap();
            assert_eq!(risk_decision["kill_switch_enabled"], true);
            assert_eq!(risk_decision["kill_switch_active"], true);
            assert!(
                risk_decision["reasons"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|reason| reason == "kill_switch_active")
            );
        }
    }

    #[test]
    fn strategy_session_stops_only_after_shutdown_input() {
        let root = temp_root("shutdown-stop");
        let mut session = StrategySession::new("session-011", "ema-cross-demo", &root).unwrap();

        session
            .run_ema_cross_demo(&ema_cross_demo_fixture_bars("BTCUSDT.BINANCE"))
            .unwrap();
        assert_eq!(session.status().state, StrategySessionState::Running);

        session.stop_after_shutdown("stop-file").unwrap();
        assert_eq!(session.status().state, StrategySessionState::Stopped);

        let status: Value = serde_json::from_str(
            &fs::read_to_string(root.join("strategy/session_status.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(status["state"], "stopped");
        assert_eq!(status["reason"], "shutdown complete: stop-file");

        let summary_json: Value =
            serde_json::from_str(&fs::read_to_string(root.join("strategy/summary.json")).unwrap())
                .unwrap();
        assert_eq!(summary_json["state"], "stopped");

        let market_status: Value = serde_json::from_str(
            &fs::read_to_string(root.join("strategy/market_status.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(market_status["connection"], MARKET_STATE_STOPPED);
        assert_eq!(market_status["state"], MARKET_STATE_STOPPED);

        let audit_events = fs::read_to_string(root.join("strategy/events.jsonl")).unwrap();
        assert!(audit_events.contains(r#""state":"running""#));
        assert!(audit_events.contains("shutdown requested: stop-file"));
        assert!(audit_events.contains("shutdown complete: stop-file"));
    }

    #[test]
    fn ema_cross_demo_rejects_empty_fixture() {
        let root = temp_root("ema-cross-empty");
        let mut session = StrategySession::new("session-005", "ema-cross-demo", &root).unwrap();

        let error = session.run_ema_cross_demo(&[]).unwrap_err().to_string();

        assert!(error.contains("fixture bars must not be empty"));
        assert_eq!(session.status().state, StrategySessionState::Created);
    }

    #[test]
    fn strategy_session_rejects_illegal_transition() {
        let root = temp_root("illegal-transition");
        let mut session = StrategySession::new("session-002", "ema-cross-demo", &root).unwrap();

        let error = session
            .transition(StrategySessionState::Running, "skip validation")
            .unwrap_err()
            .to_string();

        assert!(error.contains("illegal StrategySession transition from created to running"));
        assert_eq!(session.status().state, StrategySessionState::Created);
    }

    #[test]
    fn strategy_session_rejects_transition_from_terminal_state() {
        let root = temp_root("terminal-transition");
        let mut session = StrategySession::new("session-003", "ema-cross-demo", &root).unwrap();

        session
            .transition(StrategySessionState::Failed, "startup failed")
            .unwrap();
        let error = session
            .transition(StrategySessionState::Starting, "retry in same session")
            .unwrap_err()
            .to_string();

        assert!(error.contains("illegal StrategySession transition from failed to starting"));
        assert_eq!(session.status().state, StrategySessionState::Failed);
    }
}
