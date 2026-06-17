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
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::artifacts::{atomic_write_json, atomic_write_text};

const STRATEGY_SESSION_STATUS_SCHEMA_VERSION: &str = "ntpro.v09_strategy_session_status.v1";
const STRATEGY_SESSION_EVENT_SCHEMA_VERSION: &str = "ntpro.v09_strategy_session_event.v1";
const STRATEGY_MARKET_STATUS_SCHEMA_VERSION: &str = "ntpro.v09_market_stream_status.v1";
const STRATEGY_MARKET_EVENT_SCHEMA_VERSION: &str = "ntpro.v09_market_stream_event.v1";
const STRATEGY_SIGNAL_SCHEMA_VERSION: &str = "ntpro.v09_strategy_signal.v1";

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
        }
    }

    fn as_status_paths(&self) -> StrategySessionArtifactPaths {
        StrategySessionArtifactPaths {
            session_status: self.session_status.display().to_string(),
            events: self.events.display().to_string(),
            market_status: self.market_status.display().to_string(),
            market_events: self.market_events.display().to_string(),
            signal: self.signal.display().to_string(),
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
    pub generated_at_unix_ms: u64,
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
    pub order_submission_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyMarketStreamSummary {
    pub schema_version: String,
    pub session_id: String,
    pub strategy_id: String,
    pub connection: String,
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
            artifacts,
        };
        session.persist()?;
        Ok(session)
    }

    pub const fn status(&self) -> &StrategySessionStatus {
        &self.status
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
            "fixture_stream_running",
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
            "mock_stream_running",
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
        self.record_market_events(events, "mock_tick_stream", "mock_stream_running")
    }

    /// Runs the built-in deterministic EMA-cross demo strategy over local
    /// fixture bars and writes signal artifacts.
    ///
    /// # Errors
    ///
    /// Returns an error when the session cannot transition through the demo
    /// runtime lifecycle, when fixture bars are invalid, or when signal/status
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
                self.signals.push(StrategySignal {
                    schema_version: STRATEGY_SIGNAL_SCHEMA_VERSION.to_string(),
                    session_id: self.status.session_id.clone(),
                    strategy_id: self.status.strategy_id.clone(),
                    symbol: bar.symbol.clone(),
                    signal: signal.to_string(),
                    confidence: confidence_from_ema_gap(fast_now, slow_now),
                    market_event_seq: bar.seq,
                    generated_at_unix_ms: unix_timestamp_millis(),
                });
            }
        }

        if self.signals.is_empty() {
            anyhow::bail!("ema_cross_demo must generate at least one signal for fixture input");
        }

        self.transition(StrategySessionState::Stopping, "demo strategy completed")?;
        self.transition(StrategySessionState::Stopped, "demo strategy stopped")?;
        self.persist()?;

        Ok(DemoStrategyRuntimeSummary {
            schema_version: "ntpro.v09_demo_strategy_runtime_summary.v1".to_string(),
            session_id: self.status.session_id.clone(),
            strategy_id: self.status.strategy_id.clone(),
            strategy: "ema_cross_demo".to_string(),
            processed_events: u64::try_from(bars.len()).unwrap_or(u64::MAX),
            signal_count: u64::try_from(self.signals.len()).unwrap_or(u64::MAX),
            signal_artifact: self.artifacts.signal.display().to_string(),
            order_submission_allowed: false,
        })
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
        Ok(())
    }
}

pub fn ema_cross_demo_fixture_bars(symbol: &str) -> Vec<StrategyMarketBar> {
    let base_ts = 1_725_000_000_000;
    [100.0, 99.0, 98.0, 99.5, 101.0, 103.5, 102.0, 100.5]
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

        let events = fs::read_to_string(root.join("strategy/events.jsonl")).unwrap();
        let event_lines = events.lines().collect::<Vec<_>>();
        assert_eq!(event_lines.len(), 6);
        assert!(event_lines[0].contains(r#""state":"created""#));
        assert!(event_lines[5].contains(r#""state":"stopped""#));
        assert!(event_lines[5].contains(r#""previous_state":"stopping""#));
    }

    #[test]
    fn fixture_market_stream_writes_status_and_events() {
        let root = temp_root("fixture-market-stream");
        let mut session = StrategySession::new("session-006", "ema-cross-demo", &root).unwrap();
        let bars = ema_cross_demo_fixture_bars("BTCUSDT.BINANCE");

        let summary = session.record_fixture_bar_stream(&bars).unwrap();

        assert_eq!(summary.connection, "fixture_stream_running");
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
        assert_eq!(status["connection"], "fixture_stream_running");
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

        assert_eq!(summary.connection, "mock_stream_running");
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
        assert_eq!(summary.processed_events, 8);
        assert!(summary.signal_count >= 1);
        assert!(!summary.order_submission_allowed);
        assert_eq!(session.status().state, StrategySessionState::Stopped);

        let market_status: Value = serde_json::from_str(
            &fs::read_to_string(root.join("strategy/market_status.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            market_status["connection"], "fixture_stream_running",
            "demo strategy consumes the fixture market stream"
        );
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
            assert!(signal["market_event_seq"].as_u64().unwrap() > 0);
            assert!(signal["confidence"].as_f64().unwrap() > 0.5);
        }
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
