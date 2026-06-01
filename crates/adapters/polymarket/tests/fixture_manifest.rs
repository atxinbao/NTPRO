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

//! Fixture manifest checks for the Polymarket Rust adapter parity gate.

use std::{collections::BTreeSet, fs, path::PathBuf};

use serde_json::Value;

fn crate_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn manifest_path() -> PathBuf {
    crate_path("test_data/rust_fixture_manifest.json")
}

fn test_data_path(relative: &str) -> PathBuf {
    crate_path("test_data").join(relative)
}

#[test]
fn rust_fixture_manifest_is_complete_and_resolvable() {
    let manifest_text =
        fs::read_to_string(manifest_path()).expect("rust fixture manifest must be readable");
    let manifest: Value =
        serde_json::from_str(&manifest_text).expect("rust fixture manifest must be valid JSON");

    assert_eq!(manifest["schema_version"], 1);
    assert_eq!(manifest["task_id"], "RADP-023");
    assert_eq!(manifest["inventory_task"], "RADP-022");
    assert_eq!(manifest["adapter"], "polymarket");

    let groups = manifest["fixture_groups"]
        .as_array()
        .expect("fixture_groups must be an array");
    assert!(
        groups.len() >= 7,
        "expected Polymarket Gamma, CLOB, Data API, WebSocket, execution, auth, fee, and precision fixture groups"
    );

    let required_surfaces = [
        "polymarket_gamma_instrument_discovery",
        "polymarket_clob_public_market_data",
        "polymarket_data_api_history_positions",
        "polymarket_ws_market_data_lifecycle",
        "polymarket_ws_user_order_lifecycle",
        "polymarket_execution_order_lifecycle",
        "polymarket_auth_signing_fee_precision",
    ];

    let mut ids = BTreeSet::new();
    let mut surfaces = BTreeSet::new();

    for group in groups {
        let id = group["id"].as_str().expect("fixture group id must be set");
        assert!(ids.insert(id), "duplicate fixture group id {id}");

        let surface = group["surface"]
            .as_str()
            .expect("fixture group surface must be set");
        surfaces.insert(surface);

        assert!(
            !group["product_scope"]
                .as_str()
                .unwrap_or_default()
                .is_empty(),
            "fixture group {id} must have product scope"
        );
        assert!(
            !group["classification"]
                .as_str()
                .unwrap_or_default()
                .is_empty(),
            "fixture group {id} must have classification"
        );

        let covers = group["covers"]
            .as_array()
            .expect("fixture group covers must be an array");
        assert!(
            !covers.is_empty(),
            "fixture group {id} must list covered behavior"
        );

        let fixture_paths = group["fixture_paths"]
            .as_array()
            .expect("fixture group fixture_paths must be an array");
        assert!(
            !fixture_paths.is_empty(),
            "fixture group {id} must reference at least one fixture"
        );

        for fixture in fixture_paths {
            let relative = fixture
                .as_str()
                .expect("fixture path entries must be strings");
            assert!(
                test_data_path(relative).exists(),
                "fixture path listed in manifest does not exist: {relative}"
            );
        }

        let primary_tests = group["primary_tests"]
            .as_array()
            .expect("fixture group primary_tests must be an array");
        assert!(
            !primary_tests.is_empty(),
            "fixture group {id} must list primary tests or parser files"
        );
        for primary in primary_tests {
            let relative = primary
                .as_str()
                .expect("primary_tests entries must be strings");
            assert!(
                crate_path(relative).exists(),
                "primary test/parser path listed in manifest does not exist: {relative}"
            );
        }
    }

    for required in required_surfaces {
        assert!(
            surfaces.contains(required),
            "missing fixture surface classification: {required}"
        );
    }

    let blockers = manifest["scoped_blockers"]
        .as_array()
        .expect("scoped_blockers must be an array");
    let mut blocker_ids = BTreeSet::new();

    for blocker in blockers {
        let id = blocker["id"].as_str().expect("blocker id must be set");
        assert!(id.starts_with("PM-ADP-"), "unexpected blocker id: {id}");
        assert!(blocker_ids.insert(id), "duplicate blocker id {id}");
        assert_eq!(blocker["inventory_task"], "RADP-022");
        assert_eq!(blocker["owner_task"], "RADP-024");
        assert!(
            !blocker["summary"].as_str().unwrap_or_default().is_empty(),
            "blocker {id} must have a summary"
        );
        let status = blocker["status"]
            .as_str()
            .expect("blocker status must be set");
        assert!(
            matches!(status, "closed" | "scoped" | "deferred"),
            "unexpected blocker status for {id}: {status}"
        );
        assert_ne!(status, "open", "RADP-023 should scope blocker {id}");
        assert!(
            !blocker["resolution"]
                .as_str()
                .unwrap_or_default()
                .is_empty(),
            "blocker {id} must have a RADP-023 resolution"
        );
    }

    for expected in [
        "PM-ADP-001",
        "PM-ADP-002",
        "PM-ADP-003",
        "PM-ADP-004",
        "PM-ADP-005",
        "PM-ADP-006",
        "PM-ADP-007",
        "PM-ADP-008",
    ] {
        assert!(
            blocker_ids.contains(expected),
            "missing RADP-022 blocker entry for {expected}"
        );
    }
}

#[test]
fn rust_fixture_manifest_pins_polymarket_boundary_decisions() {
    let manifest_text =
        fs::read_to_string(manifest_path()).expect("rust fixture manifest must be readable");
    let manifest: Value =
        serde_json::from_str(&manifest_text).expect("rust fixture manifest must be valid JSON");

    let blockers = manifest["scoped_blockers"]
        .as_array()
        .expect("scoped_blockers must be an array");

    for expected in [
        ("PM-ADP-002", "BinaryOption"),
        ("PM-ADP-003", "L2_MBP"),
        ("PM-ADP-004", "reduce-only"),
        ("PM-ADP-005", "EIP-712"),
        ("PM-ADP-006", "Data API"),
        ("PM-ADP-007", "fee"),
        ("PM-ADP-008", "removal gate"),
    ] {
        let entry = blockers
            .iter()
            .find(|blocker| blocker["id"] == expected.0)
            .unwrap_or_else(|| panic!("missing blocker {}", expected.0));
        let text = format!(
            "{} {}",
            entry["summary"].as_str().unwrap_or_default(),
            entry["resolution"].as_str().unwrap_or_default()
        );
        assert!(
            text.contains(expected.1),
            "blocker {} must document boundary keyword {}",
            expected.0,
            expected.1
        );
    }
}
