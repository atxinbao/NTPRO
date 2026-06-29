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

//! V200 env-only signing material gate evidence for production submit.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::v20_pre_submit_gate::V20_ORDER_LIFECYCLE_CONTRACT_ID;

/// Stable schema for V200 signing material readiness evidence.
pub const V20_SIGNING_MATERIAL_GATE_SCHEMA_VERSION: &str =
    "ntpro.v200_signing_material_env_gate.v1";

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001b3;

/// Gate-level outcome for signing material readiness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SigningMaterialDecision {
    Ready,
    Blocked,
}

/// Stable code for signing material env gate outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SigningMaterialCode {
    #[serde(rename = "v200_signing_material_ready")]
    Ready,
    #[serde(rename = "v200_signing_material_environment_mismatch")]
    EnvironmentMismatch,
    #[serde(rename = "v200_signing_material_missing")]
    Missing,
    #[serde(rename = "v200_signing_material_empty")]
    Empty,
    #[serde(rename = "v200_signing_material_source_not_env")]
    SourceNotEnv,
}

impl SigningMaterialCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "v200_signing_material_ready",
            Self::EnvironmentMismatch => "v200_signing_material_environment_mismatch",
            Self::Missing => "v200_signing_material_missing",
            Self::Empty => "v200_signing_material_empty",
            Self::SourceNotEnv => "v200_signing_material_source_not_env",
        }
    }
}

/// Declared source for one signing material field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SigningMaterialSource {
    Env,
    Config,
    File,
    Dashboard,
    Unknown,
}

/// One required env material for production submit signing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SigningMaterialRequirement {
    pub env_var: String,
    pub material_kind: String,
}

/// Policy for a V200 env-only signing material gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SigningMaterialPolicy {
    pub gate_id: String,
    pub lifecycle_id: String,
    pub expected_environment: String,
    pub requirements: Vec<SigningMaterialRequirement>,
}

/// Runtime env snapshot supplied to the gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SigningMaterialEnvSnapshot {
    pub environment: String,
    pub values: BTreeMap<String, String>,
    pub sources: BTreeMap<String, SigningMaterialSource>,
}

/// Redacted evidence for one material requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SigningMaterialRefEvidence {
    pub env_var: String,
    pub material_kind: String,
    pub source: SigningMaterialSource,
    pub present: bool,
    pub fingerprint: Option<String>,
    pub raw_value_recorded: bool,
}

/// Auditable readiness evidence for signing material.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SigningMaterialGateEvidence {
    pub schema_version: String,
    pub contract_id: String,
    pub gate_id: String,
    pub lifecycle_id: String,
    pub decision: SigningMaterialDecision,
    pub code: SigningMaterialCode,
    pub reason: String,
    pub expected_environment: String,
    pub observed_environment: String,
    pub material_refs: Vec<SigningMaterialRefEvidence>,
    pub submit_builder_credential_ready: bool,
    pub env_only_gate_required: bool,
    pub raw_key_persisted: bool,
    pub raw_secret_persisted: bool,
    pub raw_token_persisted: bool,
    pub raw_signature_material_persisted: bool,
    pub stdout_stderr_contains_secret: bool,
    pub diagnostics_contains_secret: bool,
    pub dashboard_credential_output_enabled: bool,
    pub dashboard_credential_input_enabled: bool,
    pub remote_secret_manager_used: bool,
}

impl SigningMaterialGateEvidence {
    fn new(policy: &SigningMaterialPolicy, snapshot: &SigningMaterialEnvSnapshot) -> Self {
        let material_refs = policy
            .requirements
            .iter()
            .map(|requirement| material_ref(requirement, snapshot))
            .collect();

        Self {
            schema_version: V20_SIGNING_MATERIAL_GATE_SCHEMA_VERSION.to_string(),
            contract_id: V20_ORDER_LIFECYCLE_CONTRACT_ID.to_string(),
            gate_id: policy.gate_id.clone(),
            lifecycle_id: policy.lifecycle_id.clone(),
            decision: SigningMaterialDecision::Blocked,
            code: SigningMaterialCode::Missing,
            reason: String::new(),
            expected_environment: policy.expected_environment.clone(),
            observed_environment: snapshot.environment.clone(),
            material_refs,
            submit_builder_credential_ready: false,
            env_only_gate_required: true,
            raw_key_persisted: false,
            raw_secret_persisted: false,
            raw_token_persisted: false,
            raw_signature_material_persisted: false,
            stdout_stderr_contains_secret: false,
            diagnostics_contains_secret: false,
            dashboard_credential_output_enabled: false,
            dashboard_credential_input_enabled: false,
            remote_secret_manager_used: false,
        }
    }

    fn finish(
        mut self,
        decision: SigningMaterialDecision,
        code: SigningMaterialCode,
        reason: impl Into<String>,
    ) -> Self {
        self.decision = decision;
        self.code = code;
        self.reason = reason.into();
        self.submit_builder_credential_ready = decision == SigningMaterialDecision::Ready;
        self
    }

    /// Returns a line-oriented redacted artifact for PR evidence and later
    /// dashboard read-only views.
    #[must_use]
    pub fn redacted_artifact(&self) -> String {
        let mut lines = vec![
            format!("schema_version={}", self.schema_version),
            format!("contract_id={}", self.contract_id),
            format!("gate_id={}", self.gate_id),
            format!("lifecycle_id={}", self.lifecycle_id),
            format!("decision={:?}", self.decision),
            format!("code={}", self.code.as_str()),
            format!("expected_environment={}", self.expected_environment),
            format!("observed_environment={}", self.observed_environment),
            format!(
                "submit_builder_credential_ready={}",
                self.submit_builder_credential_ready
            ),
            "raw_key_persisted=false".to_string(),
            "raw_secret_persisted=false".to_string(),
            "raw_token_persisted=false".to_string(),
            "raw_signature_material_persisted=false".to_string(),
            "dashboard_credential_output_enabled=false".to_string(),
        ];

        for material in &self.material_refs {
            lines.push(format!(
                "material env_var={} kind={} source={:?} present={} fingerprint={}",
                material.env_var,
                material.material_kind,
                material.source,
                material.present,
                material.fingerprint.as_deref().unwrap_or("none")
            ));
        }
        lines.push(String::new());
        lines.join("\n")
    }
}

/// Evaluates signing material readiness without exposing raw credential values.
#[must_use]
pub fn evaluate_signing_material_env_gate(
    policy: &SigningMaterialPolicy,
    snapshot: &SigningMaterialEnvSnapshot,
) -> SigningMaterialGateEvidence {
    let evidence = SigningMaterialGateEvidence::new(policy, snapshot);

    if snapshot.environment != policy.expected_environment {
        return evidence.finish(
            SigningMaterialDecision::Blocked,
            SigningMaterialCode::EnvironmentMismatch,
            "runtime environment does not match signing material policy",
        );
    }

    for requirement in &policy.requirements {
        let source = snapshot
            .sources
            .get(&requirement.env_var)
            .copied()
            .unwrap_or(SigningMaterialSource::Unknown);
        if source != SigningMaterialSource::Env {
            return evidence.finish(
                SigningMaterialDecision::Blocked,
                SigningMaterialCode::SourceNotEnv,
                format!("{} must be sourced from env", requirement.env_var),
            );
        }

        let Some(value) = snapshot.values.get(&requirement.env_var) else {
            return evidence.finish(
                SigningMaterialDecision::Blocked,
                SigningMaterialCode::Missing,
                format!("{} is missing", requirement.env_var),
            );
        };
        if value.trim().is_empty() {
            return evidence.finish(
                SigningMaterialDecision::Blocked,
                SigningMaterialCode::Empty,
                format!("{} is empty", requirement.env_var),
            );
        }
    }

    evidence.finish(
        SigningMaterialDecision::Ready,
        SigningMaterialCode::Ready,
        "all required signing material is present through env-only sources",
    )
}

/// Computes a stable non-secret fingerprint for evidence.
#[must_use]
pub fn signing_material_fingerprint(value: &str) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("ntpro-fnv64:{hash:016x}")
}

fn material_ref(
    requirement: &SigningMaterialRequirement,
    snapshot: &SigningMaterialEnvSnapshot,
) -> SigningMaterialRefEvidence {
    let source = snapshot
        .sources
        .get(&requirement.env_var)
        .copied()
        .unwrap_or(SigningMaterialSource::Unknown);
    let value = snapshot.values.get(&requirement.env_var);

    SigningMaterialRefEvidence {
        env_var: requirement.env_var.clone(),
        material_kind: requirement.material_kind.clone(),
        source,
        present: value.is_some_and(|value| !value.trim().is_empty()),
        fingerprint: value
            .filter(|value| !value.trim().is_empty())
            .map(|value| signing_material_fingerprint(value)),
        raw_value_recorded: false,
    }
}
