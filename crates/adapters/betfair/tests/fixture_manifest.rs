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

//! Fixture manifest checks for the Betfair Rust adapter parity gate.

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
    assert_eq!(manifest["task_id"], "RADP-020");
    assert_eq!(manifest["inventory_task"], "RADP-019");

    let groups = manifest["fixture_groups"]
        .as_array()
        .expect("fixture_groups must be an array");
    assert!(
        groups.len() >= 7,
        "expected Betfair REST and stream fixture groups"
    );

    let required_surfaces = [
        "betfair_rest_auth_account_lifecycle",
        "betfair_rest_market_catalogue_parser",
        "betfair_rest_order_command_lifecycle",
        "betfair_rest_current_cleared_order_parser",
        "betfair_stream_market_data_lifecycle",
        "betfair_stream_bsp_race_custom_data",
        "betfair_stream_order_reconnect_lifecycle",
    ];

    let mut ids = BTreeSet::new();
    let mut surfaces = BTreeSet::new();
    let mut all_paths = BTreeSet::new();

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
                all_paths.insert(relative),
                "duplicate fixture path in manifest: {relative}"
            );
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
        assert!(id.starts_with("BF-ADP-"), "unexpected blocker id: {id}");
        assert!(blocker_ids.insert(id), "duplicate blocker id {id}");
        assert_eq!(blocker["inventory_task"], "RADP-019");
        assert_eq!(blocker["owner_task"], "RADP-021");
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
        assert_ne!(status, "open", "RADP-021 should scope blocker {id}");
        assert!(
            !blocker["resolution"]
                .as_str()
                .unwrap_or_default()
                .is_empty(),
            "blocker {id} must have a RADP-021 resolution"
        );
    }

    for expected in [
        "BF-ADP-001",
        "BF-ADP-002",
        "BF-ADP-003",
        "BF-ADP-004",
        "BF-ADP-005",
        "BF-ADP-006",
        "BF-ADP-007",
    ] {
        assert!(
            blocker_ids.contains(expected),
            "missing RADP-019 blocker entry for {expected}"
        );
    }
}

#[test]
fn rust_fixture_manifest_pins_betfair_boundary_decisions() {
    let manifest_text =
        fs::read_to_string(manifest_path()).expect("rust fixture manifest must be readable");
    let manifest: Value =
        serde_json::from_str(&manifest_text).expect("rust fixture manifest must be valid JSON");

    let blockers = manifest["scoped_blockers"]
        .as_array()
        .expect("scoped_blockers must be an array");

    for expected in [
        ("BF-ADP-002", "betting exchange"),
        ("BF-ADP-003", "BSP"),
        ("BF-ADP-004", "reconnect"),
        ("BF-ADP-005", "PyO3"),
        ("BF-ADP-006", "credentials"),
        ("BF-ADP-007", "removal gate"),
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
