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

use nautilus_binance::replay::{
    V04_BINANCE_SPOT_BAR_FIXTURE_ID, V04_BINANCE_SPOT_BAR_TYPE, V04_BINANCE_SPOT_INSTRUMENT_ID,
    load_v04_binance_spot_bar_replay,
};

#[test]
fn v04_binance_fixture_replay_is_deterministic() {
    let first = load_v04_binance_spot_bar_replay().unwrap();
    let second = load_v04_binance_spot_bar_replay().unwrap();

    assert_eq!(first, second);

    let summary = first.summary();
    assert_eq!(summary.fixture_id, V04_BINANCE_SPOT_BAR_FIXTURE_ID);
    assert_eq!(summary.instrument_id, V04_BINANCE_SPOT_INSTRUMENT_ID);
    assert_eq!(summary.bar_type, V04_BINANCE_SPOT_BAR_TYPE);
    assert_eq!(summary.bar_count, 40);
    assert_eq!(summary.first_ts_event, 1_735_689_600_000_000_000);
    assert_eq!(summary.last_ts_event, 1_735_691_940_000_000_000);
    assert_eq!(summary.first_close, "100.00");
    assert_eq!(summary.last_close, "101.10");
    assert_eq!(summary.checksum, "be481da0f80f7ca2");
    assert_eq!(summary.checksum, second.summary().checksum);
}

#[test]
fn v04_binance_fixture_summary_marks_local_replay_boundary() {
    let replay = load_v04_binance_spot_bar_replay().unwrap();
    let artifact = replay.summary_artifact();

    assert!(artifact.contains("command=binance.fixture.replay"));
    assert!(artifact.contains("status=ok"));
    assert!(artifact.contains("bar_count=40"));
    assert!(artifact.contains("external_adapter=false"));
    assert!(artifact.contains("real_exchange_connection=false"));
    assert!(artifact.contains("runtime_status=fixture_replay_ready"));
}
