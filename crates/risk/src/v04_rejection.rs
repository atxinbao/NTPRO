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

//! Deterministic v0.4 Binance sandbox risk rejection evidence helpers.

use nautilus_model::events::OrderEventAny;
use serde::{Deserialize, Serialize};

/// Stable smoke id for the v0.4 Binance sandbox risk rejection path.
pub const V04_BINANCE_RISK_REJECTION_SMOKE_ID: &str = "v04-binance-risk-rejection-smoke";
/// The v0.4 lifecycle fixture that feeds the rejected sandbox order.
pub const V04_BINANCE_RISK_REJECTION_LIFECYCLE_ID: &str = "v04-binance-mock-order-lifecycle";
/// The sandbox instrument covered by v0.4 Binance fixture work.
pub const V04_BINANCE_RISK_REJECTION_INSTRUMENT_ID: &str = "BTCUSDT.BINANCE";
/// The deterministic rejected client order id shared with the v0.4 lifecycle fixture.
pub const V04_BINANCE_RISK_REJECTION_CLIENT_ORDER_ID: &str = "O-V04-003";
/// Local fixture reason before the order reaches the risk engine.
pub const V04_BINANCE_RISK_REJECTION_FIXTURE_REASON: &str = "mock_reject_requested";
/// Risk-engine reason used by the deterministic halted-state rejection smoke.
pub const V04_BINANCE_RISK_REJECTION_REASON: &str = "TradingState::HALTED";

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001b3;

/// Dashboard and evidence friendly summary for one deterministic risk rejection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct V04BinanceRiskRejectionSummary {
    pub smoke_id: String,
    pub lifecycle_id: String,
    pub instrument_id: String,
    pub client_order_id: String,
    pub fixture_reason: String,
    pub risk_reason: String,
    pub order_status: String,
    pub forwarded_to_execution: bool,
    pub external_adapter: bool,
    pub real_exchange_connection: bool,
    pub real_orders_submitted: bool,
    pub checksum: String,
}

impl V04BinanceRiskRejectionSummary {
    /// Returns a line-oriented artifact for evidence and later dashboard panels.
    #[must_use]
    pub fn summary_artifact(&self) -> String {
        [
            "command=binance.risk_rejection_smoke".to_string(),
            "status=ok".to_string(),
            format!("smoke_id={}", self.smoke_id),
            format!("lifecycle_id={}", self.lifecycle_id),
            format!("instrument_id={}", self.instrument_id),
            format!("client_order_id={}", self.client_order_id),
            format!("fixture_reason={}", self.fixture_reason),
            format!("risk_reason={}", self.risk_reason),
            format!("order_status={}", self.order_status),
            format!("forwarded_to_execution={}", self.forwarded_to_execution),
            format!("external_adapter={}", self.external_adapter),
            format!("real_exchange_connection={}", self.real_exchange_connection),
            format!("real_orders_submitted={}", self.real_orders_submitted),
            format!("checksum={}", self.checksum),
            "runtime_status=risk_rejection_smoke_ready".to_string(),
            String::new(),
        ]
        .join("\n")
    }
}

/// Converts a real [`OrderEventAny::Denied`] into the deterministic v0.4 Binance
/// sandbox risk rejection summary.
///
/// # Errors
///
/// Returns an error when the event is not a denied event, does not match the
/// v0.4 Binance sandbox order, or has the wrong deterministic risk reason.
pub fn v04_binance_risk_rejection_summary(
    event: &OrderEventAny,
    forwarded_to_execution: bool,
) -> Result<V04BinanceRiskRejectionSummary, String> {
    let OrderEventAny::Denied(denied) = event else {
        return Err(format!("expected OrderDenied event, got {event:?}"));
    };

    let instrument_id = denied.instrument_id.to_string();
    let client_order_id = denied.client_order_id.to_string();
    let risk_reason = denied.reason.to_string();

    if instrument_id != V04_BINANCE_RISK_REJECTION_INSTRUMENT_ID {
        return Err(format!(
            "expected instrument {V04_BINANCE_RISK_REJECTION_INSTRUMENT_ID}, got {instrument_id}"
        ));
    }
    if client_order_id != V04_BINANCE_RISK_REJECTION_CLIENT_ORDER_ID {
        return Err(format!(
            "expected client order {V04_BINANCE_RISK_REJECTION_CLIENT_ORDER_ID}, got {client_order_id}"
        ));
    }
    if risk_reason != V04_BINANCE_RISK_REJECTION_REASON {
        return Err(format!(
            "expected risk reason {V04_BINANCE_RISK_REJECTION_REASON}, got {risk_reason}"
        ));
    }

    let checksum = checksum_fields(&[
        V04_BINANCE_RISK_REJECTION_SMOKE_ID,
        V04_BINANCE_RISK_REJECTION_LIFECYCLE_ID,
        &instrument_id,
        &client_order_id,
        V04_BINANCE_RISK_REJECTION_FIXTURE_REASON,
        &risk_reason,
        "denied",
        &forwarded_to_execution.to_string(),
    ]);

    Ok(V04BinanceRiskRejectionSummary {
        smoke_id: V04_BINANCE_RISK_REJECTION_SMOKE_ID.to_string(),
        lifecycle_id: V04_BINANCE_RISK_REJECTION_LIFECYCLE_ID.to_string(),
        instrument_id,
        client_order_id,
        fixture_reason: V04_BINANCE_RISK_REJECTION_FIXTURE_REASON.to_string(),
        risk_reason,
        order_status: "denied".to_string(),
        forwarded_to_execution,
        external_adapter: false,
        real_exchange_connection: false,
        real_orders_submitted: false,
        checksum,
    })
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
