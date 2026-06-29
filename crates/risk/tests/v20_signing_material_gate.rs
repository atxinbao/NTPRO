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

use std::collections::BTreeMap;

use nautilus_risk::v20_signing_material_gate::{
    SigningMaterialCode, SigningMaterialDecision, SigningMaterialEnvSnapshot,
    SigningMaterialPolicy, SigningMaterialRequirement, SigningMaterialSource,
    V20_SIGNING_MATERIAL_GATE_SCHEMA_VERSION, evaluate_signing_material_env_gate,
    signing_material_fingerprint,
};

const API_KEY: &str = "prod-key-should-never-appear";
const API_SECRET: &str = "prod-secret-should-never-appear";

#[test]
fn allows_env_only_material_and_redacts_raw_values() {
    let evidence = evaluate_signing_material_env_gate(&policy(), &snapshot());
    let json = serde_json::to_string(&evidence).expect("evidence serializes");
    let artifact = evidence.redacted_artifact();

    assert_eq!(
        evidence.schema_version,
        V20_SIGNING_MATERIAL_GATE_SCHEMA_VERSION
    );
    assert_eq!(evidence.decision, SigningMaterialDecision::Ready);
    assert_eq!(evidence.code, SigningMaterialCode::Ready);
    assert!(evidence.submit_builder_credential_ready);
    assert!(evidence.env_only_gate_required);
    assert!(!evidence.raw_key_persisted);
    assert!(!evidence.raw_secret_persisted);
    assert!(!evidence.raw_token_persisted);
    assert!(!evidence.raw_signature_material_persisted);
    assert!(!evidence.stdout_stderr_contains_secret);
    assert!(!evidence.diagnostics_contains_secret);
    assert!(!evidence.dashboard_credential_output_enabled);
    assert!(!evidence.dashboard_credential_input_enabled);
    assert!(!evidence.remote_secret_manager_used);
    assert!(json.contains("ntpro-fnv64:"));
    assert!(artifact.contains("ntpro-fnv64:"));
    assert!(!json.contains(API_KEY));
    assert!(!json.contains(API_SECRET));
    assert!(!artifact.contains(API_KEY));
    assert!(!artifact.contains(API_SECRET));
}

#[test]
fn blocks_missing_env_material_with_stable_evidence() {
    let mut snapshot = snapshot();
    snapshot.values.remove("NTPRO_BINANCE_API_SECRET");

    let evidence = evaluate_signing_material_env_gate(&policy(), &snapshot);

    assert_eq!(evidence.decision, SigningMaterialDecision::Blocked);
    assert_eq!(evidence.code, SigningMaterialCode::Missing);
    assert_eq!(evidence.code.as_str(), "v200_signing_material_missing");
    assert!(!evidence.submit_builder_credential_ready);
}

#[test]
fn blocks_environment_mismatch() {
    let mut snapshot = snapshot();
    snapshot.environment = "sandbox".to_string();

    let evidence = evaluate_signing_material_env_gate(&policy(), &snapshot);

    assert_eq!(evidence.decision, SigningMaterialDecision::Blocked);
    assert_eq!(evidence.code, SigningMaterialCode::EnvironmentMismatch);
    assert_eq!(
        evidence.code.as_str(),
        "v200_signing_material_environment_mismatch"
    );
}

#[test]
fn blocks_non_env_material_source() {
    let mut snapshot = snapshot();
    snapshot.sources.insert(
        "NTPRO_BINANCE_API_SECRET".to_string(),
        SigningMaterialSource::Config,
    );

    let evidence = evaluate_signing_material_env_gate(&policy(), &snapshot);

    assert_eq!(evidence.decision, SigningMaterialDecision::Blocked);
    assert_eq!(evidence.code, SigningMaterialCode::SourceNotEnv);
    assert_eq!(
        evidence.code.as_str(),
        "v200_signing_material_source_not_env"
    );
}

#[test]
fn blocks_empty_material() {
    let mut snapshot = snapshot();
    snapshot
        .values
        .insert("NTPRO_BINANCE_API_KEY".to_string(), " ".to_string());

    let evidence = evaluate_signing_material_env_gate(&policy(), &snapshot);

    assert_eq!(evidence.decision, SigningMaterialDecision::Blocked);
    assert_eq!(evidence.code, SigningMaterialCode::Empty);
    assert_eq!(evidence.code.as_str(), "v200_signing_material_empty");
}

#[test]
fn fingerprint_is_stable_and_non_raw() {
    let first = signing_material_fingerprint(API_SECRET);
    let second = signing_material_fingerprint(API_SECRET);

    assert_eq!(first, second);
    assert!(first.starts_with("ntpro-fnv64:"));
    assert_ne!(first, API_SECRET);
}

fn policy() -> SigningMaterialPolicy {
    SigningMaterialPolicy {
        gate_id: "signing-gate-v200-004".to_string(),
        lifecycle_id: "lc-v200-004".to_string(),
        expected_environment: "production".to_string(),
        requirements: vec![
            SigningMaterialRequirement {
                env_var: "NTPRO_BINANCE_API_KEY".to_string(),
                material_kind: "api_key".to_string(),
            },
            SigningMaterialRequirement {
                env_var: "NTPRO_BINANCE_API_SECRET".to_string(),
                material_kind: "api_secret".to_string(),
            },
        ],
    }
}

fn snapshot() -> SigningMaterialEnvSnapshot {
    SigningMaterialEnvSnapshot {
        environment: "production".to_string(),
        values: BTreeMap::from([
            ("NTPRO_BINANCE_API_KEY".to_string(), API_KEY.to_string()),
            (
                "NTPRO_BINANCE_API_SECRET".to_string(),
                API_SECRET.to_string(),
            ),
        ]),
        sources: BTreeMap::from([
            (
                "NTPRO_BINANCE_API_KEY".to_string(),
                SigningMaterialSource::Env,
            ),
            (
                "NTPRO_BINANCE_API_SECRET".to_string(),
                SigningMaterialSource::Env,
            ),
        ]),
    }
}
