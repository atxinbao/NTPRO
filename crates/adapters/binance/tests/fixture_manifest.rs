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

//! Fixture manifest checks for the Binance Rust adapter parity gate.

use std::{collections::BTreeSet, fs, path::PathBuf};

use serde_json::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data/rust_fixture_manifest.json")
}

fn test_data_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test_data")
        .join(relative)
}

#[test]
fn rust_fixture_manifest_is_complete_and_resolvable() {
    let manifest_text =
        fs::read_to_string(manifest_path()).expect("rust fixture manifest must be readable");
    let manifest: Value =
        serde_json::from_str(&manifest_text).expect("rust fixture manifest must be valid JSON");

    assert_eq!(manifest["schema_version"], 1);
    assert_eq!(manifest["task_id"], "RADP-002");
    assert_eq!(manifest["closed_by_task"], "RADP-003");

    let groups = manifest["fixture_groups"]
        .as_array()
        .expect("fixture_groups must be an array");
    assert!(
        groups.len() >= 7,
        "expected spot and futures parser/lifecycle fixture groups"
    );

    let required_surfaces = [
        "spot_http_parser",
        "spot_execution_lifecycle",
        "spot_user_data_lifecycle",
        "spot_sbe_user_data_lifecycle",
        "futures_execution_lifecycle",
        "futures_market_data_parser",
        "futures_user_data_lifecycle",
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

        let product_scope = group["product_scope"]
            .as_str()
            .expect("fixture group product_scope must be set");
        assert!(
            !product_scope.is_empty(),
            "fixture group {id} must have product scope"
        );

        let classification = group["classification"]
            .as_str()
            .expect("fixture group classification must be set");
        assert!(
            !classification.is_empty(),
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

            let full_path = test_data_path(relative);
            assert!(
                full_path.exists(),
                "fixture path listed in manifest does not exist: {relative}"
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
    assert!(
        !blockers.is_empty(),
        "manifest must record scoped blockers for deferred Binance surfaces"
    );

    for blocker in blockers {
        let id = blocker["id"].as_str().expect("blocker id must be set");
        assert!(
            id.starts_with("BIN-ADP-"),
            "blocker id must link back to RADP-001 gap ids: {id}"
        );

        let owner_task = blocker["owner_task"]
            .as_str()
            .expect("blocker owner_task must be set");
        assert!(
            owner_task == "RADP-003",
            "Binance fixture blockers should be owned by RADP-003, got {owner_task}"
        );

        let summary = blocker["summary"]
            .as_str()
            .expect("blocker summary must be set");
        assert!(!summary.is_empty(), "blocker {id} must have a summary");

        let status = blocker["status"]
            .as_str()
            .expect("blocker status must be set");
        assert_ne!(status, "open", "RADP-003 should scope blocker {id}");

        let resolution = blocker["resolution"]
            .as_str()
            .expect("blocker resolution must be set");
        assert!(
            !resolution.is_empty(),
            "blocker {id} must have a RADP-003 resolution"
        );
    }

    let closure = manifest["gap_closure"]
        .as_array()
        .expect("gap_closure must be an array");
    let mut closure_ids = BTreeSet::new();

    for entry in closure {
        let id = entry["id"].as_str().expect("closure id must be set");
        assert!(
            id.starts_with("BIN-ADP-"),
            "closure id must link back to RADP-001 gap ids: {id}"
        );
        assert!(
            closure_ids.insert(id.to_string()),
            "duplicate closure id {id}"
        );

        let status = entry["status"]
            .as_str()
            .expect("closure status must be set");
        assert_ne!(status, "open", "RADP-003 should not leave {id} open");

        let review_task = entry["review_task"]
            .as_str()
            .expect("closure review_task must be set");
        assert_eq!(review_task, "RADP-003");

        let decision = entry["decision"]
            .as_str()
            .expect("closure decision must be set");
        assert!(!decision.is_empty(), "closure {id} must have a decision");

        let evidence_refs = entry["evidence_refs"]
            .as_array()
            .expect("closure evidence_refs must be an array");
        assert!(
            !evidence_refs.is_empty(),
            "closure {id} must list evidence references"
        );
    }

    for expected in [
        "BIN-ADP-001",
        "BIN-ADP-002",
        "BIN-ADP-003",
        "BIN-ADP-004",
        "BIN-ADP-005",
        "BIN-ADP-006",
        "BIN-ADP-007",
        "BIN-ADP-008",
        "BIN-ADP-009",
        "BIN-ADP-010",
    ] {
        assert!(
            closure_ids.contains(expected),
            "missing RADP-003 closure decision for {expected}"
        );
    }
}
