//! Live Run 外部单调审计锚点客户端与签名回执合同。

#[cfg(test)]
use std::sync::Mutex;
use std::{
    fmt,
    fs::{self, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, VerifyingKey};
#[cfg(test)]
use ed25519_dalek::{Signer, SigningKey};
use reqwest::{
    Url,
    blocking::{Client, Response},
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderValue},
};
use serde::{Deserialize, Serialize};

use super::{ProductError, ProductErrorKind, product_error};

pub(super) const LIVE_RUN_ANCHOR_ENDPOINT_ENV: &str = "NTPRO_S3_AUDIT_ANCHOR_ENDPOINT";
pub(super) const LIVE_RUN_ANCHOR_NAMESPACE_ENV: &str = "NTPRO_S3_AUDIT_ANCHOR_NAMESPACE";
pub(super) const LIVE_RUN_ANCHOR_KEY_ID_ENV: &str = "NTPRO_S3_AUDIT_ANCHOR_KEY_ID";
pub(super) const LIVE_RUN_ANCHOR_PUBLIC_KEY_ENV: &str = "NTPRO_S3_AUDIT_ANCHOR_PUBLIC_KEY_BASE64";
pub(super) const LIVE_RUN_ANCHOR_TOKEN_ENV: &str = "NTPRO_S3_AUDIT_ANCHOR_TOKEN";

pub(super) const LIVE_RUN_ANCHOR_RECEIPT_SCHEMA_VERSION: &str =
    "ntpro.live_run.audit_anchor_receipt.v1";
const LIVE_RUN_ANCHOR_APPEND_SCHEMA_VERSION: &str = "ntpro.live_run.audit_anchor_append.v1";
const LIVE_RUN_ANCHOR_MAX_RESPONSE_BYTES: u64 = 64 * 1024;
const LIVE_RUN_ANCHOR_TIMEOUT: Duration = Duration::from_secs(3);
const LIVE_RUN_ANCHOR_CLOCK_SKEW_MS: u64 = 5_000;
pub(super) const LIVE_EXECUTION_RUNTIME_CLAIM_FILE: &str = "execution-runtime-claim.json";
pub(super) const LIVE_EXECUTION_RUNTIME_CLAIM_RECEIPT_FILE: &str =
    "execution-runtime-claim-receipt.json";
const LIVE_EXECUTION_RUNTIME_CLAIM_SCHEMA_VERSION: &str = "ntpro.live_execution.runtime_claim.v1";
const LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE: &str = "live-run-audit-anchor-head.json";

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LiveRunAnchorAppendRequest {
    schema_version: String,
    namespace: String,
    run_id: String,
    revision: u64,
    workspace_revision: u64,
    state_sha256: String,
    commit_sha256: String,
    previous_receipt_sha256: Option<String>,
    observed_at_unix_ms: u64,
    idempotency_key: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct LiveRunAnchorRevision {
    run: u64,
    workspace: u64,
}

impl LiveRunAnchorRevision {
    pub(super) const fn new(run: u64, workspace: u64) -> Self {
        Self { run, workspace }
    }
}

impl LiveRunAnchorAppendRequest {
    pub(super) fn new(
        namespace: &str,
        run_id: &str,
        revision: LiveRunAnchorRevision,
        state_sha256: String,
        commit_sha256: String,
        previous_receipt_sha256: Option<String>,
        observed_at_unix_ms: u64,
    ) -> Self {
        let idempotency_key = format!(
            "{namespace}:{}:{run_id}:{}:{state_sha256}:{commit_sha256}",
            revision.workspace, revision.run
        );
        Self {
            schema_version: LIVE_RUN_ANCHOR_APPEND_SCHEMA_VERSION.to_string(),
            namespace: namespace.to_string(),
            run_id: run_id.to_string(),
            revision: revision.run,
            workspace_revision: revision.workspace,
            state_sha256,
            commit_sha256,
            previous_receipt_sha256,
            observed_at_unix_ms,
            idempotency_key,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LiveRunAnchorReceipt {
    pub(super) schema_version: String,
    pub(super) namespace: String,
    pub(super) run_id: String,
    pub(super) revision: u64,
    pub(super) workspace_revision: u64,
    pub(super) state_sha256: String,
    pub(super) commit_sha256: String,
    pub(super) previous_receipt_sha256: Option<String>,
    pub(super) anchored_at_unix_ms: u64,
    pub(super) key_id: String,
    pub(super) receipt_id: String,
    pub(super) signature_base64: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LiveExecutionRuntimeClaimArtifact {
    pub(super) schema_version: String,
    pub(super) claim_id: String,
    pub(super) run_id: String,
    pub(super) control_state_revision: u64,
    pub(super) starting_receipt_sha256: String,
    pub(super) source_manifest_sha256: String,
    pub(super) execution_admission_sha256: String,
    pub(super) runtime_config_sha256: String,
    pub(super) runtime_artifact_root: String,
    pub(super) claimed_at_unix_ms: u64,
}

pub(crate) struct LiveExecutionRuntimeClaim<'a> {
    pub(crate) candidate_root: &'a Path,
    pub(crate) run_id: &'a str,
    pub(crate) control_state_revision: u64,
    pub(crate) starting_receipt_raw: &'a [u8],
    pub(crate) expected_starting_receipt_sha256: &'a str,
    pub(crate) source_manifest_sha256: &'a str,
    pub(crate) execution_admission_sha256: &'a str,
    pub(crate) runtime_config_sha256: &'a str,
    pub(crate) runtime_artifact_root: &'a Path,
    pub(crate) claimed_at_unix_ms: u64,
}

impl LiveRunAnchorReceipt {
    pub(super) fn sha256(&self) -> String {
        use aws_lc_rs::digest::{SHA256, digest};

        let raw = canonical_receipt(self, true);
        let value = digest(&SHA256, raw.as_bytes());
        let mut encoded = String::with_capacity(value.as_ref().len() * 2 + 7);
        encoded.push_str("sha256:");
        for byte in value.as_ref() {
            encoded.push_str(&format!("{byte:02x}"));
        }
        encoded
    }
}

#[derive(Clone)]
pub(in crate::dashboard) struct ExternalAnchorConfig {
    endpoint: Url,
    namespace: String,
    key_id: String,
    verifying_key: VerifyingKey,
    bearer_token: Arc<str>,
    client: Client,
}

impl fmt::Debug for ExternalAnchorConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ExternalAnchorConfig")
            .field("endpoint", &self.endpoint)
            .field("namespace", &self.namespace)
            .field("key_id", &self.key_id)
            .field("bearer_token", &"<redacted>")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[derive(Debug)]
pub(in crate::dashboard) struct MemoryAnchor {
    namespace: String,
    key_id: String,
    signing_key: SigningKey,
    receipts: Mutex<Vec<LiveRunAnchorReceipt>>,
}

#[derive(Clone, Debug)]
pub(in crate::dashboard) enum LiveRunAuditAnchorClient {
    Unconfigured,
    Invalid,
    External(Box<ExternalAnchorConfig>),
    #[cfg(test)]
    Memory(Arc<MemoryAnchor>),
}

impl LiveRunAuditAnchorClient {
    pub(in crate::dashboard) fn from_environment() -> Self {
        let endpoint = std::env::var(LIVE_RUN_ANCHOR_ENDPOINT_ENV).ok();
        let namespace = std::env::var(LIVE_RUN_ANCHOR_NAMESPACE_ENV).ok();
        let key_id = std::env::var(LIVE_RUN_ANCHOR_KEY_ID_ENV).ok();
        let public_key = std::env::var(LIVE_RUN_ANCHOR_PUBLIC_KEY_ENV).ok();
        let token = std::env::var(LIVE_RUN_ANCHOR_TOKEN_ENV).ok();
        if endpoint.is_none()
            && namespace.is_none()
            && key_id.is_none()
            && public_key.is_none()
            && token.is_none()
        {
            return Self::Unconfigured;
        }
        let Some(config) = endpoint
            .zip(namespace)
            .zip(key_id)
            .zip(public_key)
            .zip(token)
            .and_then(|((((endpoint, namespace), key_id), public_key), token)| {
                ExternalAnchorConfig::new(&endpoint, &namespace, &key_id, &public_key, token).ok()
            })
        else {
            return Self::Invalid;
        };
        Self::External(Box::new(config))
    }

    pub(super) fn namespace(&self) -> Result<&str, ProductError> {
        match self {
            Self::External(config) => Ok(&config.namespace),
            #[cfg(test)]
            Self::Memory(anchor) => Ok(&anchor.namespace),
            Self::Unconfigured | Self::Invalid => Err(anchor_error("live_run_audit_anchor_config")),
        }
    }

    pub(super) fn append(
        &self,
        request: &LiveRunAnchorAppendRequest,
    ) -> Result<LiveRunAnchorReceipt, ProductError> {
        match self {
            Self::External(config) => config.append(request),
            #[cfg(test)]
            Self::Memory(anchor) => anchor.append(request),
            Self::Unconfigured | Self::Invalid => Err(anchor_error("live_run_audit_anchor_config")),
        }
    }

    pub(super) fn latest(&self) -> Result<Option<LiveRunAnchorReceipt>, ProductError> {
        match self {
            Self::External(config) => config.latest(),
            #[cfg(test)]
            Self::Memory(anchor) => anchor.latest(),
            Self::Unconfigured | Self::Invalid => Err(anchor_error("live_run_audit_anchor_config")),
        }
    }

    pub(super) fn validate_receipt(
        &self,
        receipt: &LiveRunAnchorReceipt,
        request: &LiveRunAnchorAppendRequest,
    ) -> Result<(), ProductError> {
        let (namespace, key_id, verifying_key) = match self {
            Self::External(config) => (
                config.namespace.as_str(),
                config.key_id.as_str(),
                &config.verifying_key,
            ),
            #[cfg(test)]
            Self::Memory(anchor) => (
                anchor.namespace.as_str(),
                anchor.key_id.as_str(),
                &anchor.signing_key.verifying_key(),
            ),
            Self::Unconfigured | Self::Invalid => {
                return Err(anchor_error("live_run_audit_anchor_config"));
            }
        };
        validate_receipt_contract(receipt, request, namespace, key_id, verifying_key)
    }

    #[cfg(test)]
    pub(in crate::dashboard) fn memory_for_test() -> Self {
        Self::Memory(Arc::new(MemoryAnchor {
            namespace: "ntpro-live-test".to_string(),
            key_id: "test-ed25519-1".to_string(),
            signing_key: SigningKey::from_bytes(&[7_u8; 32]),
            receipts: Mutex::new(Vec::new()),
        }))
    }
}

/// Verifies that a Runtime is starting from the exact externally anchored control-plane state.
pub(crate) fn validate_runtime_authority(
    run_id: &str,
    revision: u64,
    state_raw: &[u8],
    commit_sha256: &str,
    receipt_raw: &[u8],
    expected_receipt_sha256: &str,
    observed_at_unix_ms: u64,
) -> anyhow::Result<()> {
    let client = LiveRunAuditAnchorClient::from_environment();
    let authority = RuntimeAuthorityBinding {
        run_id,
        revision,
        state_raw,
        commit_sha256,
        receipt_raw,
        expected_receipt_sha256,
        observed_at_unix_ms,
    };
    validate_runtime_authority_with_client(&client, &authority)
}

struct RuntimeAuthorityBinding<'a> {
    run_id: &'a str,
    revision: u64,
    state_raw: &'a [u8],
    commit_sha256: &'a str,
    receipt_raw: &'a [u8],
    expected_receipt_sha256: &'a str,
    observed_at_unix_ms: u64,
}

fn validate_runtime_authority_with_client(
    client: &LiveRunAuditAnchorClient,
    authority: &RuntimeAuthorityBinding<'_>,
) -> anyhow::Result<()> {
    let receipt: LiveRunAnchorReceipt = serde_json::from_slice(authority.receipt_raw)
        .map_err(|_| anyhow::anyhow!("live execution anchor receipt is invalid"))?;
    if receipt.sha256() != authority.expected_receipt_sha256 {
        anyhow::bail!("live execution anchor receipt hash does not match the control state head");
    }
    let request = LiveRunAnchorAppendRequest::new(
        client
            .namespace()
            .map_err(|_| anyhow::anyhow!("live execution anchor is not configured"))?,
        authority.run_id,
        LiveRunAnchorRevision::new(authority.revision, receipt.workspace_revision),
        prefixed_sha256(authority.state_raw),
        authority.commit_sha256.to_string(),
        receipt.previous_receipt_sha256.clone(),
        authority.observed_at_unix_ms,
    );
    client
        .validate_receipt(&receipt, &request)
        .map_err(|_| anyhow::anyhow!("live execution anchor receipt validation failed"))?;
    let latest = client
        .latest()
        .map_err(|_| anyhow::anyhow!("live execution latest anchor is unavailable"))?
        .ok_or_else(|| anyhow::anyhow!("live execution latest anchor is missing"))?;
    if latest != receipt {
        anyhow::bail!("live execution control-plane state is not the latest external anchor");
    }
    Ok(())
}

/// Atomically consumes the externally anchored starting authority before an execution client is
/// registered. A second Runtime observes a compare-and-append conflict and must fail closed.
pub(crate) fn claim_runtime_authority(claim: &LiveExecutionRuntimeClaim<'_>) -> anyhow::Result<()> {
    let client = LiveRunAuditAnchorClient::from_environment();
    claim_runtime_authority_with_client(&client, claim)
}

pub(crate) struct LiveExecutionControlResultAnchor<'a> {
    pub(crate) candidate_root: &'a Path,
    pub(crate) run_id: &'a str,
    pub(crate) action: &'a str,
    pub(crate) result_raw: &'a [u8],
    pub(crate) request_sha256: &'a str,
    pub(crate) completed_at_unix_ms: u64,
}

/// Publishes a Runtime control result only after it is appended to the external monotonic anchor.
pub(crate) fn anchor_runtime_control_result(
    result: &LiveExecutionControlResultAnchor<'_>,
) -> anyhow::Result<()> {
    let client = LiveRunAuditAnchorClient::from_environment();
    anchor_runtime_control_result_with_client(&client, result)
}

fn anchor_runtime_control_result_with_client(
    client: &LiveRunAuditAnchorClient,
    result: &LiveExecutionControlResultAnchor<'_>,
) -> anyhow::Result<()> {
    let (result_file, receipt_file) = match result.action {
        "reconcile" => (
            "execution-reconcile-result.json",
            "execution-reconcile-result-receipt.json",
        ),
        "cancel" => (
            "execution-cancel-result.json",
            "execution-cancel-result-receipt.json",
        ),
        _ => anyhow::bail!("live execution control result action is invalid"),
    };
    let candidate_root = fs::canonicalize(result.candidate_root)
        .map_err(|_| anyhow::anyhow!("live execution candidate root is unavailable"))?;
    if !candidate_root.is_dir() || result.result_raw.len() > 64 * 1024 {
        anyhow::bail!("live execution control result is not bounded");
    }
    let result_path = candidate_root.join(result_file);
    let receipt_path = candidate_root.join(receipt_file);
    let existing_result = if result_path.exists() {
        let raw = read_bounded_anchor_file(&result_path)?;
        if raw != result.result_raw {
            anyhow::bail!("live execution control result bytes do not match");
        }
        Some(raw)
    } else {
        None
    };
    let existing_receipt = if receipt_path.exists() {
        Some(
            serde_json::from_slice::<LiveRunAnchorReceipt>(&read_bounded_anchor_file(
                &receipt_path,
            )?)
            .map_err(|_| anyhow::anyhow!("live execution control result receipt is invalid"))?,
        )
    } else {
        None
    };
    if existing_receipt.is_some() && existing_result.is_none() {
        anyhow::bail!("live execution control result receipt exists without result bytes");
    }
    let state_head_raw = read_bounded_anchor_file(&candidate_root.join("state-head.json"))?;
    let state_head: serde_json::Value = serde_json::from_slice(&state_head_raw)
        .map_err(|_| anyhow::anyhow!("live execution control state head is invalid"))?;
    let run_revision = state_head
        .get("revision")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow::anyhow!("live execution control state revision is invalid"))?;
    if state_head.get("run_id").and_then(serde_json::Value::as_str) != Some(result.run_id) {
        anyhow::bail!("live execution control state head identity is invalid");
    }
    let artifacts_root = candidate_root
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| anyhow::anyhow!("live execution artifact root is invalid"))?;
    let workspace_head_path = artifacts_root.join(LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE);
    let workspace_head_raw = read_bounded_anchor_file(&workspace_head_path)?;
    let workspace_head: LiveRunAnchorReceipt = serde_json::from_slice(&workspace_head_raw)
        .map_err(|_| anyhow::anyhow!("live execution local workspace anchor head is invalid"))?;
    let latest = client
        .latest()
        .map_err(|_| anyhow::anyhow!("live execution latest anchor is unavailable"))?
        .ok_or_else(|| anyhow::anyhow!("live execution latest anchor is missing"))?;
    let result_sha256 = prefixed_sha256(result.result_raw);
    let receipt_matches_result = |receipt: &LiveRunAnchorReceipt| {
        receipt.state_sha256 == result_sha256
            && receipt.commit_sha256 == result.request_sha256
            && receipt.run_id == result.run_id
    };
    let workspace_head_sha256 = workspace_head.sha256();
    let existing_or_published_receipt = existing_receipt
        .or_else(|| receipt_matches_result(&workspace_head).then(|| workspace_head.clone()))
        .or_else(|| {
            (existing_result.is_some()
                && receipt_matches_result(&latest)
                && latest.previous_receipt_sha256.as_deref()
                    == Some(workspace_head_sha256.as_str()))
            .then(|| latest.clone())
        });
    let receipt = if let Some(receipt) = existing_or_published_receipt {
        let request = LiveRunAnchorAppendRequest::new(
            client
                .namespace()
                .map_err(|_| anyhow::anyhow!("live execution anchor is not configured"))?,
            result.run_id,
            LiveRunAnchorRevision::new(receipt.revision, receipt.workspace_revision),
            result_sha256,
            result.request_sha256.to_string(),
            receipt.previous_receipt_sha256.clone(),
            result.completed_at_unix_ms,
        );
        client
            .validate_receipt(&receipt, &request)
            .map_err(|_| anyhow::anyhow!("live execution control result receipt is invalid"))?;
        let pending_local_publication = workspace_head != receipt
            && receipt.previous_receipt_sha256.as_deref() == Some(workspace_head_sha256.as_str());
        let already_current = workspace_head == receipt && latest == receipt;
        let valid_historical_result = workspace_head.workspace_revision
            > receipt.workspace_revision
            && latest == workspace_head;
        if pending_local_publication && latest != receipt {
            anyhow::bail!("live execution control result recovery anchor is no longer current");
        }
        if !pending_local_publication && !already_current && !valid_historical_result {
            anyhow::bail!("live execution control result recovery chain is invalid");
        }
        receipt
    } else {
        if existing_result.is_some() {
            anyhow::bail!("unreceipted local control result is not externally anchored");
        }
        let completed_at_unix_ms = result
            .completed_at_unix_ms
            .max(workspace_head.anchored_at_unix_ms);
        let request = LiveRunAnchorAppendRequest::new(
            client
                .namespace()
                .map_err(|_| anyhow::anyhow!("live execution anchor is not configured"))?,
            result.run_id,
            LiveRunAnchorRevision::new(run_revision, workspace_head.workspace_revision + 1),
            result_sha256,
            result.request_sha256.to_string(),
            Some(workspace_head.sha256()),
            completed_at_unix_ms,
        );
        let receipt = if latest == workspace_head {
            client
                .append(&request)
                .map_err(|_| anyhow::anyhow!("live execution control result anchor was rejected"))?
        } else {
            latest
        };
        client
            .validate_receipt(&receipt, &request)
            .map_err(|_| anyhow::anyhow!("live execution control result receipt is invalid"))?;
        if client
            .latest()
            .map_err(|_| anyhow::anyhow!("live execution latest result anchor is unavailable"))?
            .as_ref()
            != Some(&receipt)
        {
            anyhow::bail!("live execution control result is not the latest external anchor");
        }
        receipt
    };
    let receipt_raw = serde_json::to_vec_pretty(&receipt)
        .map_err(|_| anyhow::anyhow!("live execution control result receipt is invalid"))?;
    if existing_result.is_none() {
        write_new_anchor_file(&result_path, result.result_raw)?;
    }
    if !receipt_path.exists() {
        write_new_anchor_file(&receipt_path, &receipt_raw)?;
    }
    if workspace_head.workspace_revision < receipt.workspace_revision {
        publish_runtime_claim_workspace_head(artifacts_root, &receipt_raw)?;
    }
    Ok(())
}

fn claim_runtime_authority_with_client(
    client: &LiveRunAuditAnchorClient,
    claim: &LiveExecutionRuntimeClaim<'_>,
) -> anyhow::Result<()> {
    let candidate_root = fs::canonicalize(claim.candidate_root)
        .map_err(|_| anyhow::anyhow!("live execution candidate root is unavailable"))?;
    if !candidate_root.is_dir() {
        anyhow::bail!("live execution candidate root is not canonical");
    }
    let claim_path = candidate_root.join(LIVE_EXECUTION_RUNTIME_CLAIM_FILE);
    let claim_receipt_path = candidate_root.join(LIVE_EXECUTION_RUNTIME_CLAIM_RECEIPT_FILE);
    if claim_path.exists() || claim_receipt_path.exists() {
        anyhow::bail!("live execution Runtime authority has already been claimed");
    }
    let starting_receipt: LiveRunAnchorReceipt = serde_json::from_slice(claim.starting_receipt_raw)
        .map_err(|_| anyhow::anyhow!("live execution starting receipt is invalid"))?;
    if starting_receipt.sha256() != claim.expected_starting_receipt_sha256
        || starting_receipt.run_id != claim.run_id
        || starting_receipt.revision != claim.control_state_revision
        || client
            .latest()
            .map_err(|_| anyhow::anyhow!("live execution latest anchor is unavailable"))?
            .as_ref()
            != Some(&starting_receipt)
    {
        anyhow::bail!("live execution starting authority is no longer current");
    }
    let artifacts_root = candidate_root
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| anyhow::anyhow!("live execution artifact root is invalid"))?;
    let workspace_head_path = artifacts_root.join(LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE);
    let workspace_head_raw = read_bounded_anchor_file(&workspace_head_path)?;
    let workspace_head: LiveRunAnchorReceipt = serde_json::from_slice(&workspace_head_raw)
        .map_err(|_| anyhow::anyhow!("live execution local workspace anchor head is invalid"))?;
    if workspace_head != starting_receipt {
        anyhow::bail!("live execution local workspace anchor head is stale");
    }
    let canonical_runtime_root = fs::canonicalize(claim.runtime_artifact_root)
        .map_err(|_| anyhow::anyhow!("live execution Runtime artifact root is unavailable"))?;
    let claim_artifact = LiveExecutionRuntimeClaimArtifact {
        schema_version: LIVE_EXECUTION_RUNTIME_CLAIM_SCHEMA_VERSION.to_string(),
        claim_id: uuid::Uuid::new_v4().to_string(),
        run_id: claim.run_id.to_string(),
        control_state_revision: claim.control_state_revision,
        starting_receipt_sha256: starting_receipt.sha256(),
        source_manifest_sha256: claim.source_manifest_sha256.to_string(),
        execution_admission_sha256: claim.execution_admission_sha256.to_string(),
        runtime_config_sha256: claim.runtime_config_sha256.to_string(),
        runtime_artifact_root: canonical_runtime_root.display().to_string(),
        claimed_at_unix_ms: claim
            .claimed_at_unix_ms
            .max(starting_receipt.anchored_at_unix_ms),
    };
    let claim_raw = serde_json::to_vec_pretty(&claim_artifact)
        .map_err(|_| anyhow::anyhow!("live execution Runtime claim is invalid"))?;
    let request = LiveRunAnchorAppendRequest::new(
        client
            .namespace()
            .map_err(|_| anyhow::anyhow!("live execution anchor is not configured"))?,
        claim.run_id,
        LiveRunAnchorRevision::new(
            claim.control_state_revision,
            starting_receipt.workspace_revision + 1,
        ),
        prefixed_sha256(&claim_raw),
        claim.runtime_config_sha256.to_string(),
        Some(starting_receipt.sha256()),
        claim_artifact.claimed_at_unix_ms,
    );
    let claim_receipt = client
        .append(&request)
        .map_err(|_| anyhow::anyhow!("live execution Runtime authority claim was rejected"))?;
    client
        .validate_receipt(&claim_receipt, &request)
        .map_err(|_| anyhow::anyhow!("live execution Runtime claim receipt is invalid"))?;
    if client
        .latest()
        .map_err(|_| anyhow::anyhow!("live execution latest claim anchor is unavailable"))?
        .as_ref()
        != Some(&claim_receipt)
    {
        anyhow::bail!("live execution Runtime claim is not the latest external anchor");
    }
    let claim_receipt_raw = serde_json::to_vec_pretty(&claim_receipt)
        .map_err(|_| anyhow::anyhow!("live execution Runtime claim receipt is invalid"))?;
    write_new_anchor_file(&claim_path, &claim_raw)?;
    write_new_anchor_file(&claim_receipt_path, &claim_receipt_raw)?;
    publish_runtime_claim_workspace_head(artifacts_root, &claim_receipt_raw)?;
    Ok(())
}

fn read_bounded_anchor_file(path: &Path) -> anyhow::Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 64 * 1024 {
        anyhow::bail!("live execution anchor artifact must be a bounded regular file");
    }
    fs::read(path).map_err(Into::into)
}

fn write_new_anchor_file(path: &Path, raw: &[u8]) -> anyhow::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    std::io::Write::write_all(&mut file, raw)?;
    file.sync_all()?;
    Ok(())
}

fn publish_runtime_claim_workspace_head(
    artifacts_root: &Path,
    receipt_raw: &[u8],
) -> anyhow::Result<()> {
    let next_path: PathBuf = artifacts_root.join(format!(
        ".live-run-audit-anchor-head.{}.next",
        uuid::Uuid::new_v4()
    ));
    write_new_anchor_file(&next_path, receipt_raw)?;
    fs::rename(
        &next_path,
        artifacts_root.join(LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE),
    )?;
    Ok(())
}

fn prefixed_sha256(raw: &[u8]) -> String {
    use aws_lc_rs::digest::{SHA256, digest};

    let value = digest(&SHA256, raw);
    let mut encoded = String::with_capacity(value.as_ref().len() * 2 + 7);
    encoded.push_str("sha256:");
    for byte in value.as_ref() {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

impl ExternalAnchorConfig {
    fn new(
        endpoint: &str,
        namespace: &str,
        key_id: &str,
        public_key_base64: &str,
        bearer_token: String,
    ) -> Result<Self, ()> {
        Self::new_with_policy(
            endpoint,
            namespace,
            key_id,
            public_key_base64,
            bearer_token,
            false,
        )
    }

    fn new_with_policy(
        endpoint: &str,
        namespace: &str,
        key_id: &str,
        public_key_base64: &str,
        bearer_token: String,
        allow_loopback_http: bool,
    ) -> Result<Self, ()> {
        Self::new_with_policy_and_timeout(
            endpoint,
            namespace,
            key_id,
            public_key_base64,
            bearer_token,
            allow_loopback_http,
            LIVE_RUN_ANCHOR_TIMEOUT,
        )
    }

    fn new_with_policy_and_timeout(
        endpoint: &str,
        namespace: &str,
        key_id: &str,
        public_key_base64: &str,
        bearer_token: String,
        allow_loopback_http: bool,
        timeout: Duration,
    ) -> Result<Self, ()> {
        let endpoint = Url::parse(endpoint).map_err(|_| ())?;
        let secure_transport = endpoint.scheme() == "https";
        let test_loopback = allow_loopback_http
            && endpoint.scheme() == "http"
            && endpoint
                .host_str()
                .is_some_and(|host| matches!(host, "127.0.0.1" | "::1" | "localhost"));
        if (!secure_transport && !test_loopback)
            || endpoint.host_str().is_none()
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
            || !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || !valid_identifier(namespace)
            || !valid_identifier(key_id)
            || bearer_token.trim().is_empty()
            || bearer_token.len() > 4096
        {
            return Err(());
        }
        let public_key = decode_base64_array::<32>(public_key_base64)?;
        let verifying_key = VerifyingKey::from_bytes(&public_key).map_err(|_| ())?;
        let client = Client::builder()
            .timeout(timeout)
            .https_only(!allow_loopback_http)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| ())?;
        Ok(Self {
            endpoint,
            namespace: namespace.to_string(),
            key_id: key_id.to_string(),
            verifying_key,
            bearer_token: Arc::from(bearer_token),
            client,
        })
    }

    fn append(
        &self,
        request: &LiveRunAnchorAppendRequest,
    ) -> Result<LiveRunAnchorReceipt, ProductError> {
        if request.namespace != self.namespace {
            return Err(anchor_error("live_run_audit_anchor_namespace"));
        }
        let response = self
            .client
            .post(self.url("compare-and-append")?)
            .header(AUTHORIZATION, self.authorization_header()?)
            .header("Idempotency-Key", &request.idempotency_key)
            .json(request)
            .send()
            .map_err(|_| anchor_error("live_run_audit_anchor_transport"))?;
        if response.status().as_u16() == 409 {
            return Err(product_error(
                ProductErrorKind::LiveConflict,
                "live_run_audit_anchor_revision",
            ));
        }
        let receipt = parse_response(response)?;
        validate_receipt_contract(
            &receipt,
            request,
            &self.namespace,
            &self.key_id,
            &self.verifying_key,
        )?;
        Ok(receipt)
    }

    fn latest(&self) -> Result<Option<LiveRunAnchorReceipt>, ProductError> {
        let response = self
            .client
            .get(self.url("latest")?)
            .header(AUTHORIZATION, self.authorization_header()?)
            .send()
            .map_err(|_| anchor_error("live_run_audit_anchor_transport"))?;
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        let receipt = parse_response(response)?;
        let request = LiveRunAnchorAppendRequest::new(
            &self.namespace,
            &receipt.run_id,
            LiveRunAnchorRevision::new(receipt.revision, receipt.workspace_revision),
            receipt.state_sha256.clone(),
            receipt.commit_sha256.clone(),
            receipt.previous_receipt_sha256.clone(),
            receipt.anchored_at_unix_ms,
        );
        validate_receipt_contract(
            &receipt,
            &request,
            &self.namespace,
            &self.key_id,
            &self.verifying_key,
        )?;
        Ok(Some(receipt))
    }

    fn url(&self, operation: &str) -> Result<Url, ProductError> {
        let mut url = self.endpoint.clone();
        {
            let mut segments = url
                .path_segments_mut()
                .map_err(|()| anchor_error("live_run_audit_anchor_endpoint"))?;
            segments.pop_if_empty();
            segments.extend(["anchors", &self.namespace, "workspace", operation]);
        }
        Ok(url)
    }

    fn authorization_header(&self) -> Result<HeaderValue, ProductError> {
        HeaderValue::from_str(&format!("Bearer {}", self.bearer_token))
            .map_err(|_| anchor_error("live_run_audit_anchor_token"))
    }
}

fn parse_response(response: Response) -> Result<LiveRunAnchorReceipt, ProductError> {
    if !response.status().is_success() {
        return Err(anchor_error("live_run_audit_anchor_response"));
    }
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if !content_type
        .split(';')
        .next()
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("application/json"))
    {
        return Err(anchor_error("live_run_audit_anchor_content_type"));
    }
    if response
        .content_length()
        .is_some_and(|length| length > LIVE_RUN_ANCHOR_MAX_RESPONSE_BYTES)
    {
        return Err(anchor_error("live_run_audit_anchor_response_size"));
    }
    let mut raw = Vec::new();
    response
        .take(LIVE_RUN_ANCHOR_MAX_RESPONSE_BYTES + 1)
        .read_to_end(&mut raw)
        .map_err(|_| anchor_error("live_run_audit_anchor_response"))?;
    if raw.len() as u64 > LIVE_RUN_ANCHOR_MAX_RESPONSE_BYTES {
        return Err(anchor_error("live_run_audit_anchor_response_size"));
    }
    serde_json::from_slice(&raw).map_err(|_| anchor_error("live_run_audit_anchor_receipt"))
}

fn validate_receipt_contract(
    receipt: &LiveRunAnchorReceipt,
    request: &LiveRunAnchorAppendRequest,
    namespace: &str,
    key_id: &str,
    verifying_key: &VerifyingKey,
) -> Result<(), ProductError> {
    if receipt.schema_version != LIVE_RUN_ANCHOR_RECEIPT_SCHEMA_VERSION
        || receipt.namespace != namespace
        || receipt.namespace != request.namespace
        || receipt.run_id != request.run_id
        || receipt.revision != request.revision
        || receipt.workspace_revision != request.workspace_revision
        || receipt.state_sha256 != request.state_sha256
        || receipt.commit_sha256 != request.commit_sha256
        || receipt.previous_receipt_sha256 != request.previous_receipt_sha256
        || receipt.key_id != key_id
        || !valid_identifier(&receipt.receipt_id)
        || !valid_sha256_ref(&receipt.state_sha256)
        || !valid_sha256_ref(&receipt.commit_sha256)
        || receipt
            .previous_receipt_sha256
            .as_deref()
            .is_some_and(|value| !valid_sha256_ref(value))
        || receipt.anchored_at_unix_ms == 0
        || receipt.anchored_at_unix_ms < request.observed_at_unix_ms
        || receipt.anchored_at_unix_ms
            > unix_time_ms().saturating_add(LIVE_RUN_ANCHOR_CLOCK_SKEW_MS)
    {
        return Err(anchor_error("live_run_audit_anchor_receipt"));
    }
    let signature_bytes = decode_base64_array::<64>(&receipt.signature_base64)
        .map_err(|()| anchor_error("live_run_audit_anchor_signature"))?;
    let signature = Signature::from_bytes(&signature_bytes);
    verifying_key
        .verify_strict(canonical_receipt(receipt, false).as_bytes(), &signature)
        .map_err(|_| anchor_error("live_run_audit_anchor_signature"))
}

fn canonical_receipt(receipt: &LiveRunAnchorReceipt, include_signature: bool) -> String {
    let previous = receipt.previous_receipt_sha256.as_deref().unwrap_or("-");
    let base = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        receipt.schema_version,
        receipt.namespace,
        receipt.run_id,
        receipt.revision,
        receipt.workspace_revision,
        receipt.state_sha256,
        receipt.commit_sha256,
        previous,
        receipt.anchored_at_unix_ms,
        receipt.key_id,
        receipt.receipt_id,
    );
    if include_signature {
        format!("{base}\n{}", receipt.signature_base64)
    } else {
        base
    }
}

fn decode_base64_array<const N: usize>(value: &str) -> Result<[u8; N], ()> {
    let decoded = STANDARD.decode(value).map_err(|_| ())?;
    if STANDARD.encode(&decoded) != value {
        return Err(());
    }
    decoded.try_into().map_err(|_| ())
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_sha256_ref(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hash| {
        hash.len() == 64
            && hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    })
}

fn unix_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn anchor_error(field: &'static str) -> ProductError {
    product_error(ProductErrorKind::LiveExecutionFailed, field)
}

pub(super) fn anchor_config_refs() -> Vec<String> {
    vec![
        LIVE_RUN_ANCHOR_ENDPOINT_ENV.to_string(),
        LIVE_RUN_ANCHOR_NAMESPACE_ENV.to_string(),
        LIVE_RUN_ANCHOR_KEY_ID_ENV.to_string(),
        LIVE_RUN_ANCHOR_PUBLIC_KEY_ENV.to_string(),
        LIVE_RUN_ANCHOR_TOKEN_ENV.to_string(),
    ]
}

#[cfg(test)]
impl MemoryAnchor {
    fn append(
        &self,
        request: &LiveRunAnchorAppendRequest,
    ) -> Result<LiveRunAnchorReceipt, ProductError> {
        if request.namespace != self.namespace {
            return Err(anchor_error("live_run_audit_anchor_namespace"));
        }
        let mut receipts = self
            .receipts
            .lock()
            .map_err(|_| anchor_error("live_run_audit_anchor_lock"))?;
        if let Some(existing) = receipts.get(request.workspace_revision as usize) {
            validate_receipt_contract(
                existing,
                request,
                &self.namespace,
                &self.key_id,
                &self.signing_key.verifying_key(),
            )?;
            return Ok(existing.clone());
        }
        let expected_revision = receipts.len() as u64;
        let expected_previous = receipts.last().map(LiveRunAnchorReceipt::sha256);
        if request.workspace_revision != expected_revision
            || request.previous_receipt_sha256 != expected_previous
        {
            return Err(product_error(
                ProductErrorKind::LiveConflict,
                "live_run_audit_anchor_revision",
            ));
        }
        let mut receipt = LiveRunAnchorReceipt {
            schema_version: LIVE_RUN_ANCHOR_RECEIPT_SCHEMA_VERSION.to_string(),
            namespace: self.namespace.clone(),
            run_id: request.run_id.clone(),
            revision: request.revision,
            workspace_revision: request.workspace_revision,
            state_sha256: request.state_sha256.clone(),
            commit_sha256: request.commit_sha256.clone(),
            previous_receipt_sha256: request.previous_receipt_sha256.clone(),
            anchored_at_unix_ms: receipts
                .last()
                .map_or(request.observed_at_unix_ms, |receipt| {
                    receipt.anchored_at_unix_ms.max(request.observed_at_unix_ms)
                }),
            key_id: self.key_id.clone(),
            receipt_id: format!(
                "receipt-{}-{}-{}",
                request.workspace_revision, request.run_id, request.revision
            ),
            signature_base64: String::new(),
        };
        receipt.signature_base64 = STANDARD.encode(
            self.signing_key
                .sign(canonical_receipt(&receipt, false).as_bytes())
                .to_bytes(),
        );
        receipts.push(receipt.clone());
        Ok(receipt)
    }

    fn latest(&self) -> Result<Option<LiveRunAnchorReceipt>, ProductError> {
        Ok(self
            .receipts
            .lock()
            .map_err(|_| anchor_error("live_run_audit_anchor_lock"))?
            .last()
            .cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::Write as _, net::TcpListener, thread};
    use tempfile::tempdir;

    fn loopback_client(
        address: std::net::SocketAddr,
        timeout: Duration,
    ) -> LiveRunAuditAnchorClient {
        let signing_key = SigningKey::from_bytes(&[9_u8; 32]);
        LiveRunAuditAnchorClient::External(Box::new(
            ExternalAnchorConfig::new_with_policy_and_timeout(
                &format!("http://{address}/v1"),
                "ntpro-live-test",
                "test-ed25519-http",
                &STANDARD.encode(signing_key.verifying_key().to_bytes()),
                "test-token".to_string(),
                true,
                timeout,
            )
            .unwrap(),
        ))
    }

    fn http_test_request(name: &str) -> LiveRunAnchorAppendRequest {
        LiveRunAnchorAppendRequest::new(
            "ntpro-live-test",
            name,
            LiveRunAnchorRevision::new(0, 0),
            format!("sha256:{}", "1".repeat(64)),
            format!("sha256:{}", "2".repeat(64)),
            None,
            unix_time_ms(),
        )
    }

    #[test]
    fn runtime_authority_requires_the_signed_latest_external_receipt() {
        let client = LiveRunAuditAnchorClient::memory_for_test();
        let state_raw = br#"{"lifecycle":"starting"}"#;
        let observed_at = unix_time_ms();
        let request = LiveRunAnchorAppendRequest::new(
            client.namespace().unwrap(),
            "live-candidate-runtime-authority",
            LiveRunAnchorRevision::new(0, 0),
            prefixed_sha256(state_raw),
            format!("sha256:{}", "2".repeat(64)),
            None,
            observed_at,
        );
        let receipt = client.append(&request).unwrap();
        let receipt_raw = serde_json::to_vec(&receipt).unwrap();
        let receipt_sha256 = receipt.sha256();
        let authority = RuntimeAuthorityBinding {
            run_id: &request.run_id,
            revision: request.revision,
            state_raw,
            commit_sha256: &request.commit_sha256,
            receipt_raw: &receipt_raw,
            expected_receipt_sha256: &receipt_sha256,
            observed_at_unix_ms: observed_at,
        };
        validate_runtime_authority_with_client(&client, &authority).unwrap();

        let mut tampered_state = state_raw.to_vec();
        tampered_state.push(b' ');
        assert!(
            validate_runtime_authority_with_client(
                &client,
                &RuntimeAuthorityBinding {
                    state_raw: &tampered_state,
                    ..authority
                },
            )
            .is_err()
        );
    }

    #[test]
    fn runtime_authority_claim_is_external_single_use() {
        let client = LiveRunAuditAnchorClient::memory_for_test();
        let run_id = "live-candidate-runtime-claim";
        let observed_at = unix_time_ms();
        let starting_request = LiveRunAnchorAppendRequest::new(
            client.namespace().unwrap(),
            run_id,
            LiveRunAnchorRevision::new(3, 0),
            format!("sha256:{}", "1".repeat(64)),
            format!("sha256:{}", "2".repeat(64)),
            None,
            observed_at,
        );
        let starting_receipt = client.append(&starting_request).unwrap();
        let starting_receipt_raw = serde_json::to_vec_pretty(&starting_receipt).unwrap();
        let temp = tempdir().unwrap();
        let artifacts_root = temp.path().join("artifacts");
        let candidate_root = artifacts_root.join("live-runs").join(run_id);
        let runtime_root = artifacts_root.join("live-market-data-runtime").join(run_id);
        fs::create_dir_all(&candidate_root).unwrap();
        fs::create_dir_all(&runtime_root).unwrap();
        fs::write(
            artifacts_root.join(LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE),
            &starting_receipt_raw,
        )
        .unwrap();
        let starting_receipt_sha256 = starting_receipt.sha256();
        let source_manifest_sha256 = format!("sha256:{}", "3".repeat(64));
        let execution_admission_sha256 = format!("sha256:{}", "4".repeat(64));
        let runtime_config_sha256 = format!("sha256:{}", "5".repeat(64));
        let claim = LiveExecutionRuntimeClaim {
            candidate_root: &candidate_root,
            run_id,
            control_state_revision: 3,
            starting_receipt_raw: &starting_receipt_raw,
            expected_starting_receipt_sha256: &starting_receipt_sha256,
            source_manifest_sha256: &source_manifest_sha256,
            execution_admission_sha256: &execution_admission_sha256,
            runtime_config_sha256: &runtime_config_sha256,
            runtime_artifact_root: &runtime_root,
            claimed_at_unix_ms: observed_at,
        };

        claim_runtime_authority_with_client(&client, &claim).unwrap();
        assert!(
            candidate_root
                .join(LIVE_EXECUTION_RUNTIME_CLAIM_FILE)
                .is_file()
        );
        assert!(
            candidate_root
                .join(LIVE_EXECUTION_RUNTIME_CLAIM_RECEIPT_FILE)
                .is_file()
        );
        let latest = client.latest().unwrap().unwrap();
        let local_head: LiveRunAnchorReceipt = serde_json::from_slice(
            &fs::read(artifacts_root.join(LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE)).unwrap(),
        )
        .unwrap();
        assert_eq!(local_head, latest);
        assert!(claim_runtime_authority_with_client(&client, &claim).is_err());

        fs::remove_file(candidate_root.join(LIVE_EXECUTION_RUNTIME_CLAIM_FILE)).unwrap();
        fs::remove_file(candidate_root.join(LIVE_EXECUTION_RUNTIME_CLAIM_RECEIPT_FILE)).unwrap();
        fs::write(
            artifacts_root.join(LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE),
            &starting_receipt_raw,
        )
        .unwrap();
        let error = claim_runtime_authority_with_client(&client, &claim)
            .unwrap_err()
            .to_string();
        assert!(error.contains("starting authority is no longer current"));
    }

    #[test]
    fn runtime_control_result_is_anchored_before_local_publication() {
        let client = LiveRunAuditAnchorClient::memory_for_test();
        let run_id = "live-candidate-control-result";
        let observed_at = unix_time_ms();
        let starting_request = LiveRunAnchorAppendRequest::new(
            client.namespace().unwrap(),
            run_id,
            LiveRunAnchorRevision::new(7, 0),
            format!("sha256:{}", "1".repeat(64)),
            format!("sha256:{}", "2".repeat(64)),
            None,
            observed_at,
        );
        let starting_receipt = client.append(&starting_request).unwrap();
        let starting_receipt_raw = serde_json::to_vec_pretty(&starting_receipt).unwrap();
        let temp = tempdir().unwrap();
        let artifacts_root = temp.path().join("artifacts");
        let candidate_root = artifacts_root.join("live-runs").join(run_id);
        fs::create_dir_all(&candidate_root).unwrap();
        fs::write(
            artifacts_root.join(LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE),
            &starting_receipt_raw,
        )
        .unwrap();
        fs::write(
            candidate_root.join("state-head.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "run_id": run_id,
                "revision": 7
            }))
            .unwrap(),
        )
        .unwrap();
        let result_raw = br#"{"action":"reconcile","status":"reconciled"}"#;
        let request_sha256 = format!("sha256:{}", "3".repeat(64));
        let result = LiveExecutionControlResultAnchor {
            candidate_root: &candidate_root,
            run_id,
            action: "reconcile",
            result_raw,
            request_sha256: &request_sha256,
            completed_at_unix_ms: observed_at,
        };

        anchor_runtime_control_result_with_client(&client, &result).unwrap();

        assert_eq!(
            fs::read(candidate_root.join("execution-reconcile-result.json")).unwrap(),
            result_raw
        );
        let receipt: LiveRunAnchorReceipt = serde_json::from_slice(
            &fs::read(candidate_root.join("execution-reconcile-result-receipt.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(receipt.state_sha256, prefixed_sha256(result_raw));
        assert_eq!(receipt.commit_sha256, request_sha256);
        assert_eq!(client.latest().unwrap(), Some(receipt.clone()));
        let local_head: LiveRunAnchorReceipt = serde_json::from_slice(
            &fs::read(artifacts_root.join(LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE)).unwrap(),
        )
        .unwrap();
        assert_eq!(local_head, receipt);

        fs::remove_file(candidate_root.join("execution-reconcile-result-receipt.json")).unwrap();
        fs::write(
            artifacts_root.join(LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE),
            &starting_receipt_raw,
        )
        .unwrap();
        anchor_runtime_control_result_with_client(&client, &result).unwrap();
        assert!(
            candidate_root
                .join("execution-reconcile-result-receipt.json")
                .is_file()
        );
        anchor_runtime_control_result_with_client(&client, &result).unwrap();

        let result_receipt: LiveRunAnchorReceipt = serde_json::from_slice(
            &fs::read(candidate_root.join("execution-reconcile-result-receipt.json")).unwrap(),
        )
        .unwrap();
        let later_request = LiveRunAnchorAppendRequest::new(
            client.namespace().unwrap(),
            run_id,
            LiveRunAnchorRevision::new(8, result_receipt.workspace_revision + 1),
            format!("sha256:{}", "4".repeat(64)),
            format!("sha256:{}", "5".repeat(64)),
            Some(result_receipt.sha256()),
            observed_at + 1,
        );
        let later_receipt = client.append(&later_request).unwrap();
        fs::write(
            artifacts_root.join(LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE),
            serde_json::to_vec_pretty(&later_receipt).unwrap(),
        )
        .unwrap();
        anchor_runtime_control_result_with_client(&client, &result).unwrap();

        fs::write(
            candidate_root.join("execution-reconcile-result.json"),
            br#"{"action":"cancel","status":"tampered"}"#,
        )
        .unwrap();
        assert!(anchor_runtime_control_result_with_client(&client, &result).is_err());
    }

    #[test]
    fn unreceipted_local_control_result_cannot_create_an_external_anchor() {
        let client = LiveRunAuditAnchorClient::memory_for_test();
        let run_id = "live-candidate-unreceipted-result";
        let observed_at = unix_time_ms();
        let starting_request = LiveRunAnchorAppendRequest::new(
            client.namespace().unwrap(),
            run_id,
            LiveRunAnchorRevision::new(7, 0),
            format!("sha256:{}", "1".repeat(64)),
            format!("sha256:{}", "2".repeat(64)),
            None,
            observed_at,
        );
        let starting_receipt = client.append(&starting_request).unwrap();
        let temp = tempdir().unwrap();
        let artifacts_root = temp.path().join("artifacts");
        let candidate_root = artifacts_root.join("live-runs").join(run_id);
        fs::create_dir_all(&candidate_root).unwrap();
        fs::write(
            artifacts_root.join(LIVE_RUN_WORKSPACE_ANCHOR_HEAD_FILE),
            serde_json::to_vec_pretty(&starting_receipt).unwrap(),
        )
        .unwrap();
        fs::write(
            candidate_root.join("state-head.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "run_id": run_id,
                "revision": 7
            }))
            .unwrap(),
        )
        .unwrap();
        let result_raw = br#"{"action":"reconcile","status":"reconciled"}"#;
        fs::write(
            candidate_root.join("execution-reconcile-result.json"),
            result_raw,
        )
        .unwrap();
        let request_sha256 = format!("sha256:{}", "3".repeat(64));
        let result = LiveExecutionControlResultAnchor {
            candidate_root: &candidate_root,
            run_id,
            action: "reconcile",
            result_raw,
            request_sha256: &request_sha256,
            completed_at_unix_ms: observed_at,
        };

        let error = anchor_runtime_control_result_with_client(&client, &result)
            .unwrap_err()
            .to_string();
        assert!(error.contains("not externally anchored"));
        assert_eq!(client.latest().unwrap(), Some(starting_receipt));
        assert!(
            !candidate_root
                .join("execution-reconcile-result-receipt.json")
                .exists()
        );
    }

    fn serve_raw_once(
        listener: TcpListener,
        response: Vec<u8>,
        delay: Duration,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 4096];
            let _ = stream.read(&mut request);
            thread::sleep(delay);
            let _ = stream.write_all(&response);
        })
    }

    #[test]
    fn memory_anchor_is_monotonic_idempotent_and_signed() {
        let client = LiveRunAuditAnchorClient::memory_for_test();
        let request = LiveRunAnchorAppendRequest::new(
            client.namespace().unwrap(),
            "live-candidate-1",
            LiveRunAnchorRevision::new(0, 0),
            format!("sha256:{}", "1".repeat(64)),
            format!("sha256:{}", "2".repeat(64)),
            None,
            unix_time_ms(),
        );
        let receipt = client.append(&request).unwrap();
        client.validate_receipt(&receipt, &request).unwrap();
        assert_eq!(client.append(&request).unwrap(), receipt);
        assert_eq!(client.latest().unwrap(), Some(receipt.clone()));

        let skipped = LiveRunAnchorAppendRequest::new(
            client.namespace().unwrap(),
            "live-candidate-1",
            LiveRunAnchorRevision::new(2, 2),
            format!("sha256:{}", "3".repeat(64)),
            format!("sha256:{}", "4".repeat(64)),
            Some(receipt.sha256()),
            unix_time_ms(),
        );
        assert!(client.append(&skipped).is_err());
    }

    #[test]
    fn receipt_signature_or_identity_drift_fails_closed() {
        let client = LiveRunAuditAnchorClient::memory_for_test();
        let request = LiveRunAnchorAppendRequest::new(
            client.namespace().unwrap(),
            "live-candidate-2",
            LiveRunAnchorRevision::new(0, 0),
            format!("sha256:{}", "a".repeat(64)),
            format!("sha256:{}", "b".repeat(64)),
            None,
            unix_time_ms(),
        );
        let mut receipt = client.append(&request).unwrap();
        receipt.state_sha256 = format!("sha256:{}", "c".repeat(64));
        assert!(client.validate_receipt(&receipt, &request).is_err());
    }

    #[test]
    fn external_configuration_rejects_non_https_and_partial_values() {
        assert!(
            ExternalAnchorConfig::new(
                "http://example.com/v1",
                "ntpro-live",
                "key-1",
                &STANDARD.encode([1_u8; 32]),
                "token".to_string(),
            )
            .is_err()
        );
        assert!(
            ExternalAnchorConfig::new(
                "https://audit.example.com/v1",
                "invalid/namespace",
                "key-1",
                &STANDARD.encode([1_u8; 32]),
                "token".to_string(),
            )
            .is_err()
        );
    }

    #[test]
    fn external_anchor_rejects_redirects() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut raw = [0_u8; 4096];
            let read = stream.read(&mut raw).unwrap();
            assert!(String::from_utf8_lossy(&raw[..read]).starts_with("POST "));
            write!(
                stream,
                "HTTP/1.1 307 Temporary Redirect\r\nLocation: http://{address}/redirected\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            )
            .unwrap();
        });
        let signing_key = SigningKey::from_bytes(&[8_u8; 32]);
        let config = ExternalAnchorConfig::new_with_policy(
            &format!("http://{address}/v1"),
            "ntpro-live-test",
            "test-ed25519-redirect",
            &STANDARD.encode(signing_key.verifying_key().to_bytes()),
            "test-token".to_string(),
            true,
        )
        .unwrap();
        let client = LiveRunAuditAnchorClient::External(Box::new(config));
        let request = LiveRunAnchorAppendRequest::new(
            "ntpro-live-test",
            "live-candidate-redirect",
            LiveRunAnchorRevision::new(0, 0),
            format!("sha256:{}", "1".repeat(64)),
            format!("sha256:{}", "2".repeat(64)),
            None,
            unix_time_ms(),
        );
        assert!(client.append(&request).is_err());
        server.join().unwrap();
    }

    #[test]
    fn external_anchor_rejects_non_success_response() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = serve_raw_once(
            listener,
            b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                .to_vec(),
            Duration::ZERO,
        );
        let error = loopback_client(address, Duration::from_secs(1))
            .append(&http_test_request("live-candidate-http-500"))
            .unwrap_err();
        assert_eq!(error.field, "live_run_audit_anchor_response");
        server.join().unwrap();
    }

    #[test]
    fn external_anchor_rejects_oversized_response() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            LIVE_RUN_ANCHOR_MAX_RESPONSE_BYTES + 1
        )
        .into_bytes();
        let server = serve_raw_once(listener, response, Duration::ZERO);
        let error = loopback_client(address, Duration::from_secs(1))
            .append(&http_test_request("live-candidate-http-oversized"))
            .unwrap_err();
        assert_eq!(error.field, "live_run_audit_anchor_response_size");
        server.join().unwrap();
    }

    #[test]
    fn external_anchor_rejects_unknown_receipt_fields() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let request = http_test_request("live-candidate-http-unknown");
        let signing_key = SigningKey::from_bytes(&[9_u8; 32]);
        let mut receipt = LiveRunAnchorReceipt {
            schema_version: LIVE_RUN_ANCHOR_RECEIPT_SCHEMA_VERSION.to_string(),
            namespace: request.namespace.clone(),
            run_id: request.run_id.clone(),
            revision: request.revision,
            workspace_revision: request.workspace_revision,
            state_sha256: request.state_sha256.clone(),
            commit_sha256: request.commit_sha256.clone(),
            previous_receipt_sha256: None,
            anchored_at_unix_ms: request.observed_at_unix_ms,
            key_id: "test-ed25519-http".to_string(),
            receipt_id: "receipt-live-candidate-http-unknown-0".to_string(),
            signature_base64: String::new(),
        };
        receipt.signature_base64 = STANDARD.encode(
            signing_key
                .sign(canonical_receipt(&receipt, false).as_bytes())
                .to_bytes(),
        );
        let mut receipt_value = serde_json::to_value(receipt).unwrap();
        receipt_value["unexpected"] = serde_json::json!(true);
        let body = serde_json::to_vec(&receipt_value).unwrap();
        let mut response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .into_bytes();
        response.extend_from_slice(&body);
        let server = serve_raw_once(listener, response, Duration::ZERO);
        let error = loopback_client(address, Duration::from_secs(1))
            .append(&request)
            .unwrap_err();
        assert_eq!(error.field, "live_run_audit_anchor_receipt");
        server.join().unwrap();
    }

    #[test]
    fn external_anchor_times_out_fail_closed() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = serve_raw_once(listener, Vec::new(), Duration::from_millis(100));
        let error = loopback_client(address, Duration::from_millis(20))
            .append(&http_test_request("live-candidate-http-timeout"))
            .unwrap_err();
        assert_eq!(error.field, "live_run_audit_anchor_transport");
        server.join().unwrap();
    }

    #[test]
    fn external_http_contract_posts_and_reads_signed_receipt_on_loopback_only() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let signing_key = SigningKey::from_bytes(&[9_u8; 32]);
        let public_key = STANDARD.encode(signing_key.verifying_key().to_bytes());
        let request = LiveRunAnchorAppendRequest::new(
            "ntpro-live-test",
            "live-candidate-http",
            LiveRunAnchorRevision::new(0, 0),
            format!("sha256:{}", "1".repeat(64)),
            format!("sha256:{}", "2".repeat(64)),
            None,
            unix_time_ms(),
        );
        let mut receipt = LiveRunAnchorReceipt {
            schema_version: LIVE_RUN_ANCHOR_RECEIPT_SCHEMA_VERSION.to_string(),
            namespace: request.namespace.clone(),
            run_id: request.run_id.clone(),
            revision: request.revision,
            workspace_revision: request.workspace_revision,
            state_sha256: request.state_sha256.clone(),
            commit_sha256: request.commit_sha256.clone(),
            previous_receipt_sha256: None,
            anchored_at_unix_ms: request.observed_at_unix_ms,
            key_id: "test-ed25519-http".to_string(),
            receipt_id: "receipt-live-candidate-http-0".to_string(),
            signature_base64: String::new(),
        };
        receipt.signature_base64 = STANDARD.encode(
            signing_key
                .sign(canonical_receipt(&receipt, false).as_bytes())
                .to_bytes(),
        );
        let response_body = serde_json::to_vec(&receipt).unwrap();
        let expected_request = request.clone();
        let server = thread::spawn(move || {
            for expected_method in ["POST", "GET"] {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut raw = Vec::new();
                let mut buffer = [0_u8; 4096];
                let header_end = loop {
                    let read = stream.read(&mut buffer).unwrap();
                    assert!(read > 0);
                    raw.extend_from_slice(&buffer[..read]);
                    if let Some(index) = raw.windows(4).position(|window| window == b"\r\n\r\n") {
                        break index + 4;
                    }
                };
                let headers = String::from_utf8(raw[..header_end].to_vec()).unwrap();
                assert!(headers.starts_with(expected_method));
                assert!(
                    headers
                        .to_ascii_lowercase()
                        .contains("authorization: bearer test-token")
                );
                if expected_method == "POST" {
                    let content_length = headers
                        .lines()
                        .find_map(|line| {
                            line.to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .map(|value| value.trim().parse::<usize>().unwrap())
                        })
                        .unwrap();
                    while raw.len() - header_end < content_length {
                        let read = stream.read(&mut buffer).unwrap();
                        raw.extend_from_slice(&buffer[..read]);
                    }
                    let posted: LiveRunAnchorAppendRequest =
                        serde_json::from_slice(&raw[header_end..header_end + content_length])
                            .unwrap();
                    assert_eq!(posted, expected_request);
                }
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    response_body.len()
                )
                .unwrap();
                stream.write_all(&response_body).unwrap();
            }
        });

        let config = ExternalAnchorConfig::new_with_policy(
            &format!("http://{address}/v1"),
            "ntpro-live-test",
            "test-ed25519-http",
            &public_key,
            "test-token".to_string(),
            true,
        )
        .unwrap();
        let client = LiveRunAuditAnchorClient::External(Box::new(config));
        assert_eq!(client.append(&request).unwrap(), receipt);
        assert_eq!(client.latest().unwrap(), Some(receipt));
        server.join().unwrap();
    }
}
