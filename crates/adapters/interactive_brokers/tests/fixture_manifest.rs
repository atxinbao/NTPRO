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

//! Fixture manifest checks for the Interactive Brokers Rust adapter parity gate.

use std::{collections::BTreeSet, fs, path::PathBuf};

use serde_json::Value;

fn crate_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn manifest_path() -> PathBuf {
    crate_path("test_data/rust_fixture_manifest.json")
}

fn scope_fixture_path() -> PathBuf {
    crate_path("test_data/rust_adapter_scope_fixture.json")
}

fn closure_manifest_path() -> PathBuf {
    crate_path("test_data/rust_adapter_parity_closure.json")
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
    assert_eq!(manifest["task_id"], "RADP-017");
    assert_eq!(manifest["inventory_task"], "RADP-016");

    let groups = manifest["fixture_groups"]
        .as_array()
        .expect("fixture_groups must be an array");
    assert!(
        groups.len() >= 6,
        "expected IB parser and lifecycle fixture groups"
    );

    let required_surfaces = [
        "interactive_brokers_instrument_provider_parser",
        "interactive_brokers_market_data_parser_lifecycle",
        "interactive_brokers_historical_request_lifecycle",
        "interactive_brokers_execution_order_lifecycle",
        "interactive_brokers_account_position_lifecycle",
        "interactive_brokers_connection_gateway_lifecycle",
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
        assert!(id.starts_with("IB-ADP-"), "unexpected blocker id: {id}");
        assert!(blocker_ids.insert(id), "duplicate blocker id {id}");
        assert_eq!(blocker["inventory_task"], "RADP-016");
        assert_eq!(blocker["owner_task"], "RADP-018");
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
        assert_ne!(status, "open", "RADP-018 should scope blocker {id}");
        assert!(
            !blocker["resolution"]
                .as_str()
                .unwrap_or_default()
                .is_empty(),
            "blocker {id} must have a RADP-018 resolution"
        );
    }

    for expected in [
        "IB-ADP-001",
        "IB-ADP-002",
        "IB-ADP-003",
        "IB-ADP-004",
        "IB-ADP-005",
        "IB-ADP-006",
        "IB-ADP-007",
        "IB-ADP-008",
    ] {
        assert!(
            blocker_ids.contains(expected),
            "missing RADP-016 blocker entry for {expected}"
        );
    }
}

#[test]
fn rust_adapter_scope_fixture_pins_ib_boundaries() {
    let fixture_text =
        fs::read_to_string(scope_fixture_path()).expect("IB scope fixture must be readable");
    let fixture: Value =
        serde_json::from_str(&fixture_text).expect("IB scope fixture must be valid JSON");

    assert_eq!(fixture["schema_version"], 1);
    assert_eq!(fixture["task_id"], "RADP-017");
    assert_eq!(fixture["adapter"], "interactive_brokers");

    let constraints = fixture["gateway_constraints"]
        .as_array()
        .expect("gateway_constraints must be an array");
    for required in [
        "external_tws_or_ib_gateway_required",
        "utc_timestamps_required",
        "live_smoke_env_gated",
        "docker_gateway_optional_feature",
    ] {
        assert!(
            constraints.iter().any(|value| value == required),
            "missing gateway constraint {required}"
        );
    }

    let security_types = fixture["supported_security_types"]
        .as_array()
        .expect("supported_security_types must be an array");
    for required in [
        "Stock",
        "ForexPair",
        "Crypto",
        "Future",
        "Option",
        "FuturesOption",
        "Index",
        "CFD",
        "Commodity",
        "Bond",
        "Spread",
    ] {
        assert!(
            security_types.iter().any(|value| value == required),
            "missing supported security type {required}"
        );
    }

    let scoped_rejections = fixture["scoped_rejections"]
        .as_array()
        .expect("scoped_rejections must be an array");
    for required in [
        "post_only",
        "non_inverse_quote_quantity",
        "non_price_trailing_offset",
        "l3_mbo_book_deltas",
        "currency_pair_trade_ticks",
        "non_utc_execution_timestamp",
    ] {
        assert!(
            scoped_rejections.iter().any(|value| value == required),
            "missing scoped rejection {required}"
        );
    }
}

#[test]
fn rust_adapter_gap_closure_is_complete_and_scoped() {
    let manifest_text = fs::read_to_string(closure_manifest_path())
        .expect("rust adapter parity closure manifest must be readable");
    let manifest: Value = serde_json::from_str(&manifest_text)
        .expect("rust adapter parity closure manifest must be valid JSON");

    assert_eq!(manifest["schema_version"], 1);
    assert_eq!(manifest["task_id"], "RADP-018");
    assert_eq!(manifest["inventory_task"], "RADP-016");
    assert_eq!(manifest["fixture_task"], "RADP-017");
    assert_eq!(manifest["adapter"], "interactive_brokers");
    assert_eq!(
        manifest["release_gate_decision"],
        "parity_resolved_with_scoped_constraints"
    );

    let decisions = manifest["decisions"]
        .as_array()
        .expect("decisions must be an array");
    assert_eq!(
        decisions.len(),
        8,
        "all Interactive Brokers gaps must be resolved"
    );

    let mut gap_ids = BTreeSet::new();
    for decision in decisions {
        let gap_id = decision["gap_id"]
            .as_str()
            .expect("gap_id must be a string");
        assert!(gap_id.starts_with("IB-ADP-"), "unexpected gap id {gap_id}");
        assert!(gap_ids.insert(gap_id), "duplicate gap id {gap_id}");

        let status = decision["status"].as_str().expect("status must be set");
        assert!(
            matches!(status, "closed" | "scoped" | "deferred"),
            "unexpected closure status for {gap_id}: {status}"
        );
        assert_ne!(status, "open", "RADP-018 must not leave open gap {gap_id}");

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
        }
    }

    for expected in [
        "IB-ADP-001",
        "IB-ADP-002",
        "IB-ADP-003",
        "IB-ADP-004",
        "IB-ADP-005",
        "IB-ADP-006",
        "IB-ADP-007",
        "IB-ADP-008",
    ] {
        assert!(
            gap_ids.contains(expected),
            "missing RADP-018 closure decision for {expected}"
        );
    }
}
