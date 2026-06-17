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
}

impl StrategySessionArtifacts {
    pub fn new(root: &Path) -> Self {
        let strategy_root = root.join("strategy");
        Self {
            session_status: strategy_root.join("session_status.json"),
            events: strategy_root.join("events.jsonl"),
        }
    }

    fn as_status_paths(&self) -> StrategySessionArtifactPaths {
        StrategySessionArtifactPaths {
            session_status: self.session_status.display().to_string(),
            events: self.events.display().to_string(),
        }
    }
}

#[derive(Debug)]
pub struct StrategySession {
    status: StrategySessionStatus,
    events: Vec<StrategySessionEvent>,
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

    fn persist(&self) -> anyhow::Result<()> {
        atomic_write_json(&self.artifacts.session_status, &self.status)?;
        let mut body = String::new();
        for event in &self.events {
            body.push_str(&serde_json::to_string(event)?);
            body.push('\n');
        }
        atomic_write_text(&self.artifacts.events, &body)?;
        Ok(())
    }
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

        let events = fs::read_to_string(root.join("strategy/events.jsonl")).unwrap();
        let event_lines = events.lines().collect::<Vec<_>>();
        assert_eq!(event_lines.len(), 6);
        assert!(event_lines[0].contains(r#""state":"created""#));
        assert!(event_lines[5].contains(r#""state":"stopped""#));
        assert!(event_lines[5].contains(r#""previous_state":"stopping""#));
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
