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

use nautilus_binance::mock_lifecycle::{
    V04_BINANCE_MOCK_ORDER_LIFECYCLE_ID, load_v04_binance_mock_order_lifecycle,
};

#[test]
fn v04_binance_mock_order_lifecycle_is_deterministic() {
    let first = load_v04_binance_mock_order_lifecycle().unwrap();
    let second = load_v04_binance_mock_order_lifecycle().unwrap();

    assert_eq!(first, second);

    let summary = first.summary();
    assert_eq!(summary.lifecycle_id, V04_BINANCE_MOCK_ORDER_LIFECYCLE_ID);
    assert_eq!(summary.instrument_id, "BTCUSDT.BINANCE");
    assert_eq!(summary.event_count, 7);
    assert_eq!(summary.submitted_count, 2);
    assert_eq!(summary.accepted_count, 2);
    assert_eq!(summary.filled_count, 1);
    assert_eq!(summary.canceled_count, 1);
    assert_eq!(summary.rejected_count, 1);
    assert_eq!(summary.checksum, "e8ae306f45b53368");
    assert_eq!(summary.checksum, second.summary().checksum);
}

#[test]
fn v04_binance_mock_order_lifecycle_covers_required_states() {
    let lifecycle = load_v04_binance_mock_order_lifecycle().unwrap();
    let summary = lifecycle.summary();

    assert_eq!(
        summary.event_types,
        vec![
            "order.accepted",
            "order.canceled",
            "order.filled",
            "order.rejected",
            "order.submitted",
        ]
    );
}

#[test]
fn v04_binance_mock_order_lifecycle_summary_marks_local_boundary() {
    let lifecycle = load_v04_binance_mock_order_lifecycle().unwrap();
    let artifact = lifecycle.summary_artifact();

    assert!(artifact.contains("command=binance.mock_order_lifecycle"));
    assert!(artifact.contains("status=ok"));
    assert!(artifact.contains("event_count=7"));
    assert!(artifact.contains("filled_count=1"));
    assert!(artifact.contains("canceled_count=1"));
    assert!(artifact.contains("rejected_count=1"));
    assert!(artifact.contains("external_adapter=false"));
    assert!(artifact.contains("real_exchange_connection=false"));
    assert!(artifact.contains("real_orders_submitted=false"));
    assert!(artifact.contains("runtime_status=mock_order_lifecycle_ready"));
}
