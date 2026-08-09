//! 不可变策略版本资源的只读投影与 HTTP 合同。

use std::{cmp::Ordering, collections::BTreeSet};

use axum::{
    Json,
    extract::{Path as AxumPath, RawQuery, State, rejection::PathRejection},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::*;

const STRATEGY_VERSION_LIST_SCHEMA_VERSION: &str =
    "ntpro.product_api.strategy_version_list.response.v1";
const STRATEGY_VERSION_DETAIL_SCHEMA_VERSION: &str =
    "ntpro.product_api.strategy_version_detail.response.v1";
const VERSION_CURSOR_PREFIX: &str = "strategy-version-v1-";

const STRATEGY_VERSION_SNAPSHOT_SCHEMA_VERSION: &str =
    "ntpro.backtest_strategy_version_snapshot.v1";

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(super) struct ProductStrategyVersion {
    strategy_version_id: String,
    strategy_id: String,
    version: String,
    content_hash: String,
    code_ref: String,
    parameter_schema: Value,
    data_requirements: StrategyVersionDataRequirements,
    risk_config: StrategyVersionRiskConfig,
    status: StrategyVersionStatus,
    created_at_unix_ms: u64,
    source: ProductSource,
}

impl ProductStrategyVersion {
    pub(super) fn strategy_version_id(&self) -> &str {
        &self.strategy_version_id
    }

    pub(super) fn strategy_id(&self) -> &str {
        &self.strategy_id
    }

    pub(super) const fn created_at_unix_ms(&self) -> u64 {
        self.created_at_unix_ms
    }

    pub(super) fn data_venues(&self) -> &[String] {
        &self.data_requirements.venues
    }

    pub(super) fn data_symbols(&self) -> &[String] {
        &self.data_requirements.symbols
    }

    pub(super) fn content_hash(&self) -> &str {
        &self.content_hash
    }

    pub(super) fn parameter_const_u64(&self, name: &str) -> Option<u64> {
        self.parameter_schema
            .get("properties")?
            .get(name)?
            .get("const")?
            .as_u64()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProductStrategyVersionSnapshot {
    schema_version: String,
    strategy_version: ProductStrategyVersionConfig,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
struct StrategyVersionDataRequirements {
    venues: Vec<String>,
    symbols: Vec<String>,
    data_types: Vec<String>,
    timeframes: Vec<String>,
    deterministic_replay_required: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
struct StrategyVersionRiskConfig {
    risk_profile_ref: String,
    kill_switch_required: bool,
    external_venue_connection_default: bool,
    order_submission_default: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum StrategyVersionStatus {
    Registered,
}

#[derive(Debug, Deserialize)]
struct ProductVersionConfigProjection {
    strategy_version: ProductStrategyVersionConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ProductStrategyVersionConfig {
    strategy_version_id: String,
    strategy_id: String,
    version: String,
    content_hash: String,
    code_ref: String,
    parameter_schema: Value,
    data_requirements: StrategyVersionDataRequirements,
    risk_config: StrategyVersionRiskConfig,
    status: StrategyVersionStatus,
    created_at_unix_ms: u64,
}

#[derive(Serialize)]
struct StrategyVersionHashMaterial<'a> {
    strategy_version_id: &'a str,
    strategy_id: &'a str,
    version: &'a str,
    code_ref: &'a str,
    parameter_schema: &'a Value,
    data_requirements: &'a StrategyVersionDataRequirements,
    risk_config: &'a StrategyVersionRiskConfig,
    status: StrategyVersionStatus,
    created_at_unix_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct StrategyVersionListResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: Vec<ProductStrategyVersion>,
    page: ProductPage,
    boundaries: ProductReadOnlyBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct StrategyVersionDetailResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: ProductStrategyVersion,
    boundaries: ProductReadOnlyBoundaries,
}

#[derive(Debug, Deserialize)]
pub(in crate::dashboard) struct StrategyVersionPath {
    strategy_id: String,
    version_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StrategyVersionListQuery {
    limit: usize,
    cursor: Option<String>,
    sort: StrategyVersionSort,
    order: SortOrder,
    status: Option<StrategyVersionStatus>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StrategyVersionSort {
    StrategyVersionId,
    Version,
    CreatedAt,
}

pub(in crate::dashboard) async fn strategy_version_list_api(
    State(state): State<DashboardServerState>,
    strategy_path: Result<AxumPath<String>, PathRejection>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<StrategyVersionListResponse> {
    let request_id = product_request_id();
    let strategy_id = strategy_path
        .map(|AxumPath(strategy_id)| strategy_id)
        .map_err(|_| {
            product_error_response(
                &product_error(ProductErrorKind::BadRequest, "strategy_id"),
                &request_id,
            )
        })?;
    let result = validate_requested_identifier("strategy_id", &strategy_id)
        .and_then(|()| parse_strategy_version_list_query(raw_query.as_deref()))
        .and_then(|query| {
            let source = load_product_source(&state, unix_time_ms())?;
            ensure_strategy_matches(&source, &strategy_id)?;
            let version = load_product_strategy_version(&source, unix_time_ms())?;
            project_strategy_version_list(version, &query, request_id.clone())
        });
    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

pub(in crate::dashboard) async fn strategy_version_detail_api(
    State(state): State<DashboardServerState>,
    version_path: Result<AxumPath<StrategyVersionPath>, PathRejection>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<StrategyVersionDetailResponse> {
    let request_id = product_request_id();
    let path = version_path.map(|AxumPath(path)| path).map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "strategy_version_path"),
            &request_id,
        )
    })?;
    let result = reject_detail_query(raw_query.as_deref()).and_then(|()| {
        validate_requested_identifier("strategy_id", &path.strategy_id)?;
        validate_requested_version_id("version_id", &path.version_id)?;
        let source = load_product_source(&state, unix_time_ms())?;
        ensure_strategy_matches(&source, &path.strategy_id)?;
        let version = load_product_strategy_version(&source, unix_time_ms())?;
        if version.strategy_version_id != path.version_id || version.strategy_id != path.strategy_id
        {
            return Err(product_error(
                ProductErrorKind::VersionNotFound,
                "version_id",
            ));
        }
        Ok(StrategyVersionDetailResponse {
            schema_version: STRATEGY_VERSION_DETAIL_SCHEMA_VERSION.to_string(),
            contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
            request_id: request_id.clone(),
            data: version,
            boundaries: ProductReadOnlyBoundaries::enforced(),
        })
    });
    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

fn ensure_strategy_matches(
    source: &ValidatedProductSource,
    strategy_id: &str,
) -> Result<(), ProductError> {
    if source.strategy.strategy_id != strategy_id {
        return Err(product_error(ProductErrorKind::NotFound, "strategy_id"));
    }
    Ok(())
}

pub(super) fn load_product_strategy_version(
    source: &ValidatedProductSource,
    now_unix_ms: u64,
) -> Result<ProductStrategyVersion, ProductError> {
    validate_sha256_hash(
        "strategy_version_content_hash",
        &source.identity.identities.strategy_version_content_hash,
    )?;
    let projection: ProductVersionConfigProjection = toml::from_str(&source.raw_config)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "strategy_version_config"))?;
    let config = projection.strategy_version;
    validate_strategy_version_config(&config, source, now_unix_ms)?;
    Ok(ProductStrategyVersion {
        strategy_version_id: config.strategy_version_id,
        strategy_id: config.strategy_id,
        version: config.version,
        content_hash: config.content_hash,
        code_ref: config.code_ref,
        parameter_schema: config.parameter_schema,
        data_requirements: config.data_requirements,
        risk_config: config.risk_config,
        status: config.status,
        created_at_unix_ms: config.created_at_unix_ms,
        source: ProductSource {
            source_type: "strategy_version_manifest".to_string(),
            freshness_status: "fresh".to_string(),
            source_refs: vec![
                MVP_IDENTITY_CONTRACT_PATH.to_string(),
                MVP_STATUS_CONTRACT_PATH.to_string(),
                format!("node-config:{}#strategy_version", source.config_name),
            ],
        },
    })
}

fn validate_strategy_version_config(
    config: &ProductStrategyVersionConfig,
    source: &ValidatedProductSource,
    now_unix_ms: u64,
) -> Result<(), ProductError> {
    validate_strategy_version_definition(config, now_unix_ms)?;
    let expected_id = strategy_version_resource_id(&config.strategy_id, &config.version);
    if config.strategy_id != source.strategy.strategy_id
        || config.strategy_id != source.identity.identities.strategy_id
        || config.version != source.identity.identities.strategy_version
        || config.content_hash != source.identity.identities.strategy_version_content_hash
        || config.strategy_version_id != expected_id
        || source.strategy.default_version_id != expected_id
        || config.created_at_unix_ms < source.strategy.created_at_unix_ms
        || config.created_at_unix_ms > source.strategy.updated_at_unix_ms
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "strategy_version_ownership",
        ));
    }
    Ok(())
}

fn validate_strategy_version_definition(
    config: &ProductStrategyVersionConfig,
    now_unix_ms: u64,
) -> Result<(), ProductError> {
    validate_identifier("strategy_version_strategy_id", &config.strategy_id)?;
    validate_identifier("strategy_version", &config.version)?;
    validate_version_resource_id("strategy_version_id", &config.strategy_version_id)?;
    let expected_id = strategy_version_resource_id(&config.strategy_id, &config.version);
    if config.strategy_version_id != expected_id {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "strategy_version_ownership",
        ));
    }
    validate_text("strategy_version_code_ref", &config.code_ref, 512)?;
    if !valid_immutable_code_ref(&config.code_ref) {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "strategy_version_code_ref",
        ));
    }
    validate_parameter_schema(&config.parameter_schema)?;
    validate_data_requirements(&config.data_requirements)?;
    validate_risk_config(&config.risk_config)?;
    if config.status != StrategyVersionStatus::Registered {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "strategy_version_status",
        ));
    }
    if config.created_at_unix_ms == 0
        || config.created_at_unix_ms > now_unix_ms.saturating_add(MAX_CLOCK_SKEW_MS)
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "strategy_version_created_at",
        ));
    }
    validate_sha256_hash("strategy_version_content_hash", &config.content_hash)?;
    let actual_hash = strategy_version_content_hash(config)?;
    if config.content_hash != actual_hash {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "strategy_version_content_hash",
        ));
    }
    Ok(())
}

pub(super) fn serialize_strategy_version_snapshot(
    version: &ProductStrategyVersion,
) -> Result<Vec<u8>, ProductError> {
    let snapshot = ProductStrategyVersionSnapshot {
        schema_version: STRATEGY_VERSION_SNAPSHOT_SCHEMA_VERSION.to_string(),
        strategy_version: ProductStrategyVersionConfig {
            strategy_version_id: version.strategy_version_id.clone(),
            strategy_id: version.strategy_id.clone(),
            version: version.version.clone(),
            content_hash: version.content_hash.clone(),
            code_ref: version.code_ref.clone(),
            parameter_schema: version.parameter_schema.clone(),
            data_requirements: version.data_requirements.clone(),
            risk_config: version.risk_config.clone(),
            status: version.status,
            created_at_unix_ms: version.created_at_unix_ms,
        },
    };
    serde_json::to_string_pretty(&snapshot)
        .map(|value| format!("{value}\n").into_bytes())
        .map_err(|_| {
            product_error(
                ProductErrorKind::ExecutionFailed,
                "strategy_version_snapshot",
            )
        })
}

pub(super) fn deserialize_strategy_version_snapshot(
    raw: &[u8],
    source_ref: String,
    now_unix_ms: u64,
) -> Result<ProductStrategyVersion, ProductError> {
    let snapshot: ProductStrategyVersionSnapshot = serde_json::from_slice(raw)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "strategy_version_snapshot"))?;
    if snapshot.schema_version != STRATEGY_VERSION_SNAPSHOT_SCHEMA_VERSION {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "strategy_version_snapshot",
        ));
    }
    let config = snapshot.strategy_version;
    validate_strategy_version_definition(&config, now_unix_ms)?;
    Ok(ProductStrategyVersion {
        strategy_version_id: config.strategy_version_id,
        strategy_id: config.strategy_id,
        version: config.version,
        content_hash: config.content_hash,
        code_ref: config.code_ref,
        parameter_schema: config.parameter_schema,
        data_requirements: config.data_requirements,
        risk_config: config.risk_config,
        status: config.status,
        created_at_unix_ms: config.created_at_unix_ms,
        source: ProductSource {
            source_type: "backtest_strategy_version_snapshot".to_string(),
            freshness_status: "immutable".to_string(),
            source_refs: vec![source_ref],
        },
    })
}

fn valid_immutable_code_ref(value: &str) -> bool {
    let Some(reference) = value.strip_prefix("git://NTPRO@") else {
        return false;
    };
    let Some((revision, target)) = reference.split_once('/') else {
        return false;
    };
    revision.len() == 40
        && revision
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        && target == "crates/cli/src/strategy_session.rs#ema_cross_demo"
}

pub(super) fn validate_parameter_schema(schema: &Value) -> Result<(), ProductError> {
    jsonschema::draft202012::meta::validate(schema).map_err(|_| {
        product_error(
            ProductErrorKind::SourceInvalid,
            "strategy_version_parameter_schema",
        )
    })?;
    let object = schema.as_object().ok_or_else(|| {
        product_error(
            ProductErrorKind::SourceInvalid,
            "strategy_version_parameter_schema",
        )
    })?;
    let properties = object
        .get("properties")
        .and_then(Value::as_object)
        .filter(|properties| !properties.is_empty())
        .ok_or_else(|| {
            product_error(
                ProductErrorKind::SourceInvalid,
                "strategy_version_parameter_schema",
            )
        })?;
    let required = object
        .get("required")
        .and_then(Value::as_array)
        .filter(|required| !required.is_empty())
        .ok_or_else(|| {
            product_error(
                ProductErrorKind::SourceInvalid,
                "strategy_version_parameter_schema",
            )
        })?;
    if object.get("$schema").and_then(Value::as_str)
        != Some("https://json-schema.org/draft/2020-12/schema")
        || object.get("type").and_then(Value::as_str) != Some("object")
        || object.get("additionalProperties").and_then(Value::as_bool) != Some(false)
    {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "strategy_version_parameter_schema",
        ));
    }
    let mut unique = BTreeSet::new();
    for field in required {
        let field = field
            .as_str()
            .filter(|field| properties.contains_key(*field));
        let Some(field) = field else {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "strategy_version_parameter_schema",
            ));
        };
        if !unique.insert(field) {
            return Err(product_error(
                ProductErrorKind::SourceInvalid,
                "strategy_version_parameter_schema",
            ));
        }
    }
    if properties.values().any(|property| {
        property
            .as_object()
            .and_then(|value| value.get("type"))
            .and_then(Value::as_str)
            .is_none()
    }) {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "strategy_version_parameter_schema",
        ));
    }
    Ok(())
}

fn validate_data_requirements(
    requirements: &StrategyVersionDataRequirements,
) -> Result<(), ProductError> {
    for (field, values) in [
        ("strategy_version_data_venues", &requirements.venues),
        ("strategy_version_data_symbols", &requirements.symbols),
        ("strategy_version_data_types", &requirements.data_types),
        ("strategy_version_data_timeframes", &requirements.timeframes),
    ] {
        if values.is_empty() || values.len() > 64 {
            return Err(product_error(ProductErrorKind::SourceInvalid, field));
        }
        let mut unique = BTreeSet::new();
        for value in values {
            validate_text(field, value, 128)?;
            if !unique.insert(value) {
                return Err(product_error(ProductErrorKind::SourceInvalid, field));
            }
        }
    }
    if !requirements.deterministic_replay_required {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "strategy_version_deterministic_replay",
        ));
    }
    Ok(())
}

fn validate_risk_config(config: &StrategyVersionRiskConfig) -> Result<(), ProductError> {
    validate_text(
        "strategy_version_risk_config",
        &config.risk_profile_ref,
        256,
    )?;
    if !config.kill_switch_required
        || config.external_venue_connection_default
        || config.order_submission_default
        || config.risk_profile_ref != "node-config:#risk"
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "strategy_version_risk_config",
        ));
    }
    Ok(())
}

fn strategy_version_content_hash(
    config: &ProductStrategyVersionConfig,
) -> Result<String, ProductError> {
    let material = StrategyVersionHashMaterial {
        strategy_version_id: &config.strategy_version_id,
        strategy_id: &config.strategy_id,
        version: &config.version,
        code_ref: &config.code_ref,
        parameter_schema: &config.parameter_schema,
        data_requirements: &config.data_requirements,
        risk_config: &config.risk_config,
        status: config.status,
        created_at_unix_ms: config.created_at_unix_ms,
    };
    let bytes = serde_json::to_vec(&material).map_err(|_| {
        product_error(
            ProductErrorKind::SourceInvalid,
            "strategy_version_content_hash",
        )
    })?;
    Ok(super::super::sha256_bytes(&bytes))
}

pub(super) fn project_strategy_version_list(
    version: ProductStrategyVersion,
    query: &StrategyVersionListQuery,
    request_id: String,
) -> Result<StrategyVersionListResponse, ProductError> {
    let mut versions = vec![version];
    versions.retain(|item| query.status.is_none_or(|status| item.status == status));
    versions.sort_by(|left, right| strategy_version_comparison(left, right, query));
    let start = if let Some(cursor) = query.cursor.as_deref() {
        let cursor_id = decode_version_cursor(cursor)?;
        versions
            .iter()
            .position(|item| item.strategy_version_id == cursor_id)
            .map(|position| position + 1)
            .ok_or_else(|| product_error(ProductErrorKind::BadRequest, "cursor"))?
    } else {
        0
    };
    let end = start.saturating_add(query.limit).min(versions.len());
    let data = versions[start..end].to_vec();
    let has_more = end < versions.len();
    let next_cursor = has_more
        .then(|| {
            data.last()
                .map(|item| encode_version_cursor(&item.strategy_version_id))
        })
        .flatten();
    Ok(StrategyVersionListResponse {
        schema_version: STRATEGY_VERSION_LIST_SCHEMA_VERSION.to_string(),
        contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
        request_id,
        page: ProductPage {
            limit: query.limit,
            returned_count: data.len(),
            next_cursor,
            has_more,
        },
        data,
        boundaries: ProductReadOnlyBoundaries::enforced(),
    })
}

fn strategy_version_comparison(
    left: &ProductStrategyVersion,
    right: &ProductStrategyVersion,
    query: &StrategyVersionListQuery,
) -> Ordering {
    let comparison = match query.sort {
        StrategyVersionSort::StrategyVersionId => {
            left.strategy_version_id.cmp(&right.strategy_version_id)
        }
        StrategyVersionSort::Version => left.version.cmp(&right.version),
        StrategyVersionSort::CreatedAt => left.created_at_unix_ms.cmp(&right.created_at_unix_ms),
    };
    match query.order {
        SortOrder::Asc => comparison,
        SortOrder::Desc => comparison.reverse(),
    }
}

pub(super) fn parse_strategy_version_list_query(
    raw_query: Option<&str>,
) -> Result<StrategyVersionListQuery, ProductError> {
    let values = parse_query_values(raw_query)?;
    for key in values.keys() {
        if !matches!(
            key.as_str(),
            "limit" | "cursor" | "sort" | "order" | "status"
        ) {
            return Err(product_error(ProductErrorKind::BadRequest, key));
        }
    }
    let limit = values
        .get("limit")
        .map_or(Ok(DEFAULT_PAGE_LIMIT), |value| {
            value
                .parse::<usize>()
                .ok()
                .filter(|value| (1..=MAX_PAGE_LIMIT).contains(value))
                .ok_or_else(|| product_error(ProductErrorKind::BadRequest, "limit"))
        })?;
    let cursor = values.get("cursor").cloned();
    if let Some(cursor) = cursor.as_deref() {
        decode_version_cursor(cursor)?;
    }
    let sort = match values.get("sort").map(String::as_str) {
        None | Some("strategy_version_id") => StrategyVersionSort::StrategyVersionId,
        Some("version") => StrategyVersionSort::Version,
        Some("created_at") => StrategyVersionSort::CreatedAt,
        Some(_) => return Err(product_error(ProductErrorKind::BadRequest, "sort")),
    };
    let order = match values.get("order").map(String::as_str) {
        None | Some("asc") => SortOrder::Asc,
        Some("desc") => SortOrder::Desc,
        Some(_) => return Err(product_error(ProductErrorKind::BadRequest, "order")),
    };
    let status = match values.get("status").map(String::as_str) {
        None => None,
        Some("registered") => Some(StrategyVersionStatus::Registered),
        Some(_) => return Err(product_error(ProductErrorKind::BadRequest, "status")),
    };
    Ok(StrategyVersionListQuery {
        limit,
        cursor,
        sort,
        order,
        status,
    })
}

pub(super) fn encode_version_cursor(version_id: &str) -> String {
    let mut encoded = String::with_capacity(VERSION_CURSOR_PREFIX.len() + version_id.len() * 2);
    encoded.push_str(VERSION_CURSOR_PREFIX);
    for byte in version_id.bytes() {
        use std::fmt::Write as _;
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

fn decode_version_cursor(cursor: &str) -> Result<String, ProductError> {
    let encoded = cursor
        .strip_prefix(VERSION_CURSOR_PREFIX)
        .filter(|value| !value.is_empty() && value.len() % 2 == 0)
        .ok_or_else(|| product_error(ProductErrorKind::BadRequest, "cursor"))?;
    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    for pair in encoded.as_bytes().chunks_exact(2) {
        let pair = std::str::from_utf8(pair)
            .map_err(|_| product_error(ProductErrorKind::BadRequest, "cursor"))?;
        bytes.push(
            u8::from_str_radix(pair, 16)
                .map_err(|_| product_error(ProductErrorKind::BadRequest, "cursor"))?,
        );
    }
    let version_id = String::from_utf8(bytes)
        .map_err(|_| product_error(ProductErrorKind::BadRequest, "cursor"))?;
    validate_requested_version_id("cursor", &version_id)?;
    Ok(version_id)
}

fn validate_requested_identifier(field: &str, value: &str) -> Result<(), ProductError> {
    validate_identifier(field, value)
        .map_err(|_| product_error(ProductErrorKind::BadRequest, field))
}

pub(super) fn validate_requested_version_id(field: &str, value: &str) -> Result<(), ProductError> {
    validate_version_resource_id(field, value)
        .map_err(|_| product_error(ProductErrorKind::BadRequest, field))
}

pub(super) fn validate_version_resource_id(field: &str, value: &str) -> Result<(), ProductError> {
    let Some((strategy_id, version)) = value.split_once('@') else {
        return Err(product_error(ProductErrorKind::SourceInvalid, field));
    };
    if value.len() > 257 || version.contains('@') {
        return Err(product_error(ProductErrorKind::SourceInvalid, field));
    }
    validate_identifier(field, strategy_id)?;
    validate_identifier(field, version)
}

#[cfg(test)]
pub(super) fn config_with_computed_version_hash(raw: &str) -> String {
    let projection: ProductVersionConfigProjection =
        toml::from_str(raw).expect("test version config must parse");
    let hash = strategy_version_content_hash(&projection.strategy_version)
        .expect("test version hash must compute");
    if raw.contains("__STRATEGY_VERSION_CONTENT_HASH__") {
        raw.replace("__STRATEGY_VERSION_CONTENT_HASH__", &hash)
    } else {
        raw.replacen(&projection.strategy_version.content_hash, &hash, 1)
    }
}

#[cfg(test)]
pub(super) fn computed_strategy_version_hash(raw: &str) -> String {
    let projection: ProductVersionConfigProjection =
        toml::from_str(raw).expect("test version config must parse");
    strategy_version_content_hash(&projection.strategy_version)
        .expect("test version hash must compute")
}
