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

//! Fixture manifest checks for the Sandbox Rust adapter parity gate.

use std::{collections::BTreeSet, fs, path::PathBuf};

use serde_json::Value;

fn crate_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn manifest_path() -> PathBuf {
    crate_path("test_data/rust_fixture_manifest.json")
}

fn closure_manifest_path() -> PathBuf {
    crate_path("test_data/rust_adapter_parity_closure.json")
}

fn lifecycle_fixture_path() -> PathBuf {
    crate_path("test_data/sandbox_lifecycle_fixture.json")
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
    assert_eq!(manifest["adapter"], "sandbox");
    assert_eq!(
        manifest["fixture_strategy"],
        "executable_rust_lifecycle_fixtures_no_external_payloads"
    );
    assert_eq!(manifest["external_payloads_required"], false);

    let groups = manifest["fixture_groups"]
        .as_array()
        .expect("fixture_groups must be an array");
    assert!(
        groups.len() >= 6,
        "expected Sandbox config, lifecycle, matching, order, settlement, and report fixture groups"
    );

    let required_surfaces = [
        "sandbox_config_lifecycle",
        "sandbox_client_connect_lifecycle",
        "sandbox_market_data_matching_inputs",
        "sandbox_order_lifecycle",
        "sandbox_binary_option_settlement",
        "sandbox_internal_report_boundaries",
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
            "fixture group {id} must reference lifecycle fixture descriptors"
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
            "fixture group {id} must list primary tests or implementation files"
        );
        for primary in primary_tests {
            let relative = primary
                .as_str()
                .expect("primary_tests entries must be strings");
            assert!(
                crate_path(relative).exists(),
                "primary test/source path listed in manifest does not exist: {relative}"
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
        assert!(id.starts_with("SBX-ADP-"), "unexpected blocker id: {id}");
        assert!(blocker_ids.insert(id), "duplicate blocker id {id}");
        assert_eq!(blocker["inventory_task"], "RADP-022");
        assert_eq!(blocker["owner_task"], "RADP-024");
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
        "SBX-ADP-001",
        "SBX-ADP-002",
        "SBX-ADP-003",
        "SBX-ADP-004",
        "SBX-ADP-005",
    ] {
        assert!(
            blocker_ids.contains(expected),
            "missing RADP-022 blocker entry for {expected}"
        );
    }
}

#[test]
fn rust_adapter_gap_closure_is_complete_and_scoped() {
    let fixture_manifest_text =
        fs::read_to_string(manifest_path()).expect("rust fixture manifest must be readable");
    let fixture_manifest: Value = serde_json::from_str(&fixture_manifest_text)
        .expect("rust fixture manifest must be valid JSON");

    let closure_manifest_text = fs::read_to_string(closure_manifest_path())
        .expect("rust adapter parity closure manifest must be readable");
    let closure_manifest: Value = serde_json::from_str(&closure_manifest_text)
        .expect("rust adapter parity closure manifest must be valid JSON");

    assert_eq!(closure_manifest["schema_version"], 1);
    assert_eq!(closure_manifest["task_id"], "RADP-024");
    assert_eq!(closure_manifest["inventory_task"], "RADP-022");
    assert_eq!(closure_manifest["fixture_task"], "RADP-023");
    assert_eq!(closure_manifest["adapter"], "sandbox");
    assert_eq!(
        closure_manifest["release_gate_decision"],
        "parity_resolved_with_scoped_constraints"
    );

    let blockers = fixture_manifest["scoped_blockers"]
        .as_array()
        .expect("scoped_blockers must be an array");
    let expected_gap_ids = blockers
        .iter()
        .map(|blocker| blocker["id"].as_str().expect("blocker id must be a string"))
        .collect::<BTreeSet<_>>();

    let decisions = closure_manifest["decisions"]
        .as_array()
        .expect("decisions must be an array");
    assert_eq!(
        decisions.len(),
        expected_gap_ids.len(),
        "every Sandbox scoped blocker must have one closure entry"
    );

    let mut decision_gap_ids = BTreeSet::new();
    for decision in decisions {
        let gap_id = decision["gap_id"]
            .as_str()
            .expect("gap_id must be a string");
        assert!(gap_id.starts_with("SBX-ADP-"), "unexpected gap id {gap_id}");
        assert!(decision_gap_ids.insert(gap_id), "duplicate gap id {gap_id}");

        let status = decision["status"].as_str().expect("status must be set");
        assert!(
            matches!(status, "closed" | "scoped" | "deferred"),
            "unexpected closure status for {gap_id}: {status}"
        );
        assert_ne!(status, "open", "RADP-024 must not leave open gap {gap_id}");

        assert!(
            !decision["decision"].as_str().unwrap_or_default().is_empty(),
            "decision text must be set for {gap_id}"
        );
        assert!(
            !decision["release_gate_note"]
                .as_str()
                .unwrap_or_default()
                .is_empty(),
            "release gate note must be set for {gap_id}"
        );

        let evidence_paths = decision["evidence_paths"]
            .as_array()
            .expect("evidence_paths must be an array");
        assert!(
            !evidence_paths.is_empty(),
            "decision {gap_id} must include evidence paths"
        );
        for evidence in evidence_paths {
            let relative = evidence
                .as_str()
                .expect("evidence_paths entries must be strings");
            assert!(
                crate_path(relative).exists(),
                "closure evidence path listed in manifest does not exist: {relative}"
            );
        }

        if status == "deferred" {
            assert_eq!(
                decision["requires_removal_gate"], true,
                "deferred gap {gap_id} must require the removal gate"
            );
            assert!(
                decision["release_gate_note"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("removal gate"),
                "deferred gap {gap_id} must mention the removal gate"
            );
        }
    }

    assert_eq!(
        decision_gap_ids, expected_gap_ids,
        "closure decisions must match the RADP-023 Sandbox scoped blockers"
    );
}

#[test]
fn rust_fixture_manifest_pins_sandbox_boundary_decisions() {
    let manifest_text =
        fs::read_to_string(manifest_path()).expect("rust fixture manifest must be readable");
    let manifest: Value =
        serde_json::from_str(&manifest_text).expect("rust fixture manifest must be valid JSON");

    let blockers = manifest["scoped_blockers"]
        .as_array()
        .expect("scoped_blockers must be an array");

    for expected in [
        ("SBX-ADP-002", "simulated execution"),
        ("SBX-ADP-002", "external venue"),
        ("SBX-ADP-003", "precision"),
        ("SBX-ADP-004", "internal"),
        ("SBX-ADP-005", "removal gate"),
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

#[test]
fn sandbox_lifecycle_fixture_declares_no_external_payloads() {
    let fixture_text =
        fs::read_to_string(lifecycle_fixture_path()).expect("lifecycle fixture must be readable");
    let fixture: Value =
        serde_json::from_str(&fixture_text).expect("lifecycle fixture must be valid JSON");

    assert_eq!(fixture["schema_version"], 1);
    assert_eq!(fixture["task_id"], "RADP-023");
    assert_eq!(fixture["adapter"], "sandbox");
    assert_eq!(fixture["fixture_kind"], "internal_lifecycle_descriptor");
    assert_eq!(fixture["external_payloads_required"], false);

    let surfaces = fixture["surfaces"]
        .as_array()
        .expect("surfaces must be an array");
    assert!(
        surfaces.len() >= 5,
        "Sandbox lifecycle descriptor must cover config, lifecycle, matching, settlement, and report boundaries"
    );

    let text = fixture.to_string();
    for expected in [
        "no external venue REST payloads",
        "no external venue WebSocket payloads",
        "precision mismatch drops",
        "InstrumentClose settlement",
        "mass status",
    ] {
        assert!(
            text.contains(expected),
            "Sandbox lifecycle fixture must document {expected}"
        );
    }
}
