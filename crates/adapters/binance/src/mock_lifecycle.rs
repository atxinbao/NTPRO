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

//! Deterministic Binance mock order lifecycle support for NTPRO v0.4 sandbox flows.

use std::collections::BTreeSet;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::replay::V04_BINANCE_SPOT_INSTRUMENT_ID;

/// Stable lifecycle id used by v0.4 Binance sandbox order and risk smokes.
pub const V04_BINANCE_MOCK_ORDER_LIFECYCLE_ID: &str = "v04-binance-mock-order-lifecycle";
/// Checked-in lifecycle path, relative to the repository root.
pub const V04_BINANCE_MOCK_ORDER_LIFECYCLE_PATH: &str =
    "crates/adapters/binance/test_data/v04/mock_order_lifecycle.jsonl";

const V04_BINANCE_MOCK_ORDER_LIFECYCLE_JSONL: &str =
    include_str!("../test_data/v04/mock_order_lifecycle.jsonl");
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001b3;

/// One deterministic mock order lifecycle event for the v0.4 Binance sandbox path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinanceMockOrderLifecycleEvent {
    pub sequence: u64,
    pub ts_event: u64,
    pub event_type: String,
    pub client_order_id: String,
    pub venue_order_id: Option<String>,
    pub trade_id: Option<String>,
    pub order_status: String,
    pub reason: Option<String>,
    pub quantity: String,
    pub filled_qty: String,
    pub leaves_qty: String,
}

/// Summary fields that later strategy, risk, and dashboard tasks can consume.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinanceMockOrderLifecycleSummary {
    pub lifecycle_id: String,
    pub source_path: String,
    pub instrument_id: String,
    pub event_count: usize,
    pub submitted_count: usize,
    pub accepted_count: usize,
    pub filled_count: usize,
    pub canceled_count: usize,
    pub rejected_count: usize,
    pub event_types: Vec<String>,
    pub checksum: String,
}

/// Deterministic local mock order lifecycle payload for v0.4 Binance sandbox flows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinanceMockOrderLifecycle {
    pub lifecycle_id: String,
    pub source_path: String,
    pub instrument_id: String,
    pub events: Vec<BinanceMockOrderLifecycleEvent>,
}

impl BinanceMockOrderLifecycle {
    /// Returns a stable summary for evidence files and later product smokes.
    #[must_use]
    pub fn summary(&self) -> BinanceMockOrderLifecycleSummary {
        let mut event_types = BTreeSet::new();
        let mut submitted_count = 0;
        let mut accepted_count = 0;
        let mut filled_count = 0;
        let mut canceled_count = 0;
        let mut rejected_count = 0;

        for event in &self.events {
            event_types.insert(event.event_type.clone());
            match event.event_type.as_str() {
                "order.submitted" => submitted_count += 1,
                "order.accepted" => accepted_count += 1,
                "order.filled" => filled_count += 1,
                "order.canceled" => canceled_count += 1,
                "order.rejected" => rejected_count += 1,
                _ => {}
            }
        }

        BinanceMockOrderLifecycleSummary {
            lifecycle_id: self.lifecycle_id.clone(),
            source_path: self.source_path.clone(),
            instrument_id: self.instrument_id.clone(),
            event_count: self.events.len(),
            submitted_count,
            accepted_count,
            filled_count,
            canceled_count,
            rejected_count,
            event_types: event_types.into_iter().collect(),
            checksum: checksum_events(&self.events),
        }
    }

    /// Returns a line-oriented artifact body for CLI logs and evidence.
    #[must_use]
    pub fn summary_artifact(&self) -> String {
        let summary = self.summary();
        [
            "command=binance.mock_order_lifecycle".to_string(),
            "status=ok".to_string(),
            format!("lifecycle_id={}", summary.lifecycle_id),
            format!("source_path={}", summary.source_path),
            format!("instrument_id={}", summary.instrument_id),
            format!("event_count={}", summary.event_count),
            format!("submitted_count={}", summary.submitted_count),
            format!("accepted_count={}", summary.accepted_count),
            format!("filled_count={}", summary.filled_count),
            format!("canceled_count={}", summary.canceled_count),
            format!("rejected_count={}", summary.rejected_count),
            format!("event_types={}", summary.event_types.join(",")),
            format!("checksum={}", summary.checksum),
            "external_adapter=false".to_string(),
            "real_exchange_connection=false".to_string(),
            "real_orders_submitted=false".to_string(),
            "runtime_status=mock_order_lifecycle_ready".to_string(),
            String::new(),
        ]
        .join("\n")
    }
}

/// Loads the checked-in v0.4 Binance mock order lifecycle fixture.
///
/// # Errors
///
/// Returns an error when the checked-in JSONL fixture is malformed, empty,
/// out of order, or missing required lifecycle states.
pub fn load_v04_binance_mock_order_lifecycle() -> anyhow::Result<BinanceMockOrderLifecycle> {
    mock_order_lifecycle_from_jsonl(
        V04_BINANCE_MOCK_ORDER_LIFECYCLE_ID,
        V04_BINANCE_MOCK_ORDER_LIFECYCLE_PATH,
        V04_BINANCE_SPOT_INSTRUMENT_ID,
        V04_BINANCE_MOCK_ORDER_LIFECYCLE_JSONL,
    )
}

/// Builds a mock order lifecycle payload from JSONL content.
///
/// # Errors
///
/// Returns an error when any JSONL row is invalid or the lifecycle coverage is incomplete.
pub fn mock_order_lifecycle_from_jsonl(
    lifecycle_id: &str,
    source_path: &str,
    instrument_id: &str,
    jsonl: &str,
) -> anyhow::Result<BinanceMockOrderLifecycle> {
    let mut previous_sequence = None;
    let mut previous_ts_event = None;
    let mut event_types = BTreeSet::new();
    let mut events = Vec::new();

    for (line_number, line) in jsonl.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let event =
            serde_json::from_str::<BinanceMockOrderLifecycleEvent>(trimmed).with_context(|| {
                format!(
                    "invalid Binance mock order lifecycle JSONL row {}",
                    line_number + 1
                )
            })?;
        validate_event_order(
            &event,
            line_number + 1,
            &mut previous_sequence,
            &mut previous_ts_event,
        )?;
        validate_event_shape(&event, line_number + 1)?;
        event_types.insert(event.event_type.clone());
        events.push(event);
    }

    if events.is_empty() {
        anyhow::bail!("Binance mock order lifecycle fixture must contain at least one event");
    }

    for required in [
        "order.submitted",
        "order.accepted",
        "order.filled",
        "order.canceled",
        "order.rejected",
    ] {
        if !event_types.contains(required) {
            anyhow::bail!(
                "Binance mock order lifecycle fixture missing required event type {required}"
            );
        }
    }

    Ok(BinanceMockOrderLifecycle {
        lifecycle_id: lifecycle_id.to_string(),
        source_path: source_path.to_string(),
        instrument_id: instrument_id.to_string(),
        events,
    })
}

fn validate_event_order(
    event: &BinanceMockOrderLifecycleEvent,
    line_number: usize,
    previous_sequence: &mut Option<u64>,
    previous_ts_event: &mut Option<u64>,
) -> anyhow::Result<()> {
    if let Some(previous) = *previous_sequence
        && event.sequence <= previous
    {
        anyhow::bail!(
            "Binance mock order lifecycle sequence must be strictly increasing at row {line_number}"
        );
    }
    if let Some(previous) = *previous_ts_event
        && event.ts_event <= previous
    {
        anyhow::bail!(
            "Binance mock order lifecycle ts_event must be strictly increasing at row {line_number}"
        );
    }
    *previous_sequence = Some(event.sequence);
    *previous_ts_event = Some(event.ts_event);
    Ok(())
}

fn validate_event_shape(
    event: &BinanceMockOrderLifecycleEvent,
    line_number: usize,
) -> anyhow::Result<()> {
    validate_non_empty("client_order_id", &event.client_order_id, line_number)?;
    validate_non_empty("order_status", &event.order_status, line_number)?;
    validate_positive_decimal("quantity", &event.quantity, line_number)?;
    validate_non_negative_decimal("filled_qty", &event.filled_qty, line_number)?;
    validate_non_negative_decimal("leaves_qty", &event.leaves_qty, line_number)?;

    match event.event_type.as_str() {
        "order.submitted" => validate_status(event, "submitted", line_number),
        "order.accepted" => {
            require_some("venue_order_id", &event.venue_order_id, line_number)?;
            validate_status(event, "accepted", line_number)
        }
        "order.filled" => {
            require_some("venue_order_id", &event.venue_order_id, line_number)?;
            require_some("trade_id", &event.trade_id, line_number)?;
            validate_status(event, "filled", line_number)?;
            validate_exact("leaves_qty", &event.leaves_qty, "0.000", line_number)
        }
        "order.canceled" => {
            require_some("venue_order_id", &event.venue_order_id, line_number)?;
            validate_status(event, "canceled", line_number)?;
            validate_exact("leaves_qty", &event.leaves_qty, "0.000", line_number)
        }
        "order.rejected" => {
            require_some("reason", &event.reason, line_number)?;
            validate_status(event, "rejected", line_number)
        }
        other => anyhow::bail!(
            "unsupported Binance mock order lifecycle event_type '{other}' at row {line_number}"
        ),
    }
}

fn validate_status(
    event: &BinanceMockOrderLifecycleEvent,
    expected: &str,
    line_number: usize,
) -> anyhow::Result<()> {
    validate_exact("order_status", &event.order_status, expected, line_number)
}

fn validate_exact(
    field_name: &str,
    actual: &str,
    expected: &str,
    line_number: usize,
) -> anyhow::Result<()> {
    if actual != expected {
        anyhow::bail!("{field_name} must be '{expected}' at row {line_number}, got '{actual}'");
    }
    Ok(())
}

fn validate_non_empty(field_name: &str, raw: &str, line_number: usize) -> anyhow::Result<()> {
    if raw.trim().is_empty() {
        anyhow::bail!("{field_name} must not be empty at row {line_number}");
    }
    Ok(())
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

fn validate_non_negative_decimal(
    field_name: &str,
    raw: &str,
    line_number: usize,
) -> anyhow::Result<()> {
    let value = raw
        .parse::<f64>()
        .with_context(|| format!("invalid {field_name} decimal at row {line_number}"))?;
    if !value.is_finite() || value < 0.0 {
        anyhow::bail!("{field_name} must be a non-negative decimal at row {line_number}");
    }
    Ok(())
}

fn require_some(
    field_name: &str,
    value: &Option<String>,
    line_number: usize,
) -> anyhow::Result<()> {
    if value.as_deref().is_none_or(str::is_empty) {
        anyhow::bail!("{field_name} is required at row {line_number}");
    }
    Ok(())
}

fn checksum_events(events: &[BinanceMockOrderLifecycleEvent]) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for event in events {
        for part in [
            event.sequence.to_string(),
            event.ts_event.to_string(),
            event.event_type.clone(),
            event.client_order_id.clone(),
            event.venue_order_id.clone().unwrap_or_default(),
            event.trade_id.clone().unwrap_or_default(),
            event.order_status.clone(),
            event.reason.clone().unwrap_or_default(),
            event.quantity.clone(),
            event.filled_qty.clone(),
            event.leaves_qty.clone(),
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
