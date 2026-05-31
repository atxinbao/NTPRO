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

//! Fixture manifest checks for the Hyperliquid Rust adapter parity gate.

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
    assert_eq!(manifest["task_id"], "RADP-011");
    assert_eq!(manifest["inventory_task"], "RADP-010");

    let groups = manifest["fixture_groups"]
        .as_array()
        .expect("fixture_groups must be an array");
    assert!(
        groups.len() >= 5,
        "expected Hyperliquid parser and lifecycle fixture groups"
    );

    let required_surfaces = [
        "hyperliquid_http_perp_instrument_parser",
        "hyperliquid_http_book_parser",
        "hyperliquid_http_account_and_funding",
        "hyperliquid_ws_market_data_parser",
        "hyperliquid_outcome_operational_scope",
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
        assert!(id.starts_with("HYP-ADP-"), "unexpected blocker id: {id}");
        assert!(blocker_ids.insert(id), "duplicate blocker id {id}");
        assert_eq!(blocker["inventory_task"], "RADP-010");
        assert_eq!(blocker["owner_task"], "RADP-012");
        assert!(
            !blocker["summary"].as_str().unwrap_or_default().is_empty(),
            "blocker {id} must have a summary"
        );
        let status = blocker["status"]
            .as_str()
            .expect("blocker status must be set");
        assert_ne!(status, "open", "RADP-012 should scope blocker {id}");
        assert!(
            !blocker["resolution"]
                .as_str()
                .unwrap_or_default()
                .is_empty(),
            "blocker {id} must have a RADP-012 resolution"
        );
    }

    for expected in [
        "HYP-ADP-001",
        "HYP-ADP-002",
        "HYP-ADP-003",
        "HYP-ADP-004",
        "HYP-ADP-005",
        "HYP-ADP-006",
    ] {
        assert!(
            blocker_ids.contains(expected),
            "missing RADP-010 blocker entry for {expected}"
        );
    }

    let closure = manifest["gap_closure"]
        .as_array()
        .expect("gap_closure must be an array");
    let mut closure_ids = BTreeSet::new();

    for entry in closure {
        let id = entry["id"].as_str().expect("closure id must be set");
        assert!(id.starts_with("HYP-ADP-"), "unexpected closure id: {id}");
        assert!(closure_ids.insert(id), "duplicate gap closure id {id}");

        let status = entry["status"]
            .as_str()
            .expect("closure status must be set");
        assert_ne!(status, "open", "RADP-012 should not leave {id} open");
        assert_eq!(entry["review_task"], "RADP-012");
        assert!(
            !entry["decision"].as_str().unwrap_or_default().is_empty(),
            "closure {id} must have a decision"
        );
        assert!(
            !entry["evidence_refs"]
                .as_array()
                .expect("closure evidence_refs must be an array")
                .is_empty(),
            "closure {id} must list evidence references"
        );
    }

    for expected in [
        "HYP-ADP-001",
        "HYP-ADP-002",
        "HYP-ADP-003",
        "HYP-ADP-004",
        "HYP-ADP-005",
        "HYP-ADP-006",
    ] {
        assert!(
            closure_ids.contains(expected),
            "missing RADP-012 gap closure entry for {expected}"
        );
    }
}
