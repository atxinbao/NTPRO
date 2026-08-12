//! Live Run 外部单调审计锚点客户端与签名回执合同。

#[cfg(test)]
use std::sync::Mutex;
use std::{
    fmt,
    io::Read,
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
