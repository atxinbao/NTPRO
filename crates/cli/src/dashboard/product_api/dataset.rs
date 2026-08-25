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

//! 策略版本兼容的本地历史数据集产品合同。

use std::fs;

use axum::{
    Json,
    extract::{Path as AxumPath, RawQuery, State, rejection::PathRejection},
};
use serde::{Deserialize, Serialize};

use crate::catalog_dataset::{
    LocalQuoteDatasetInspection, PRODUCT_CATALOG_DIRECTORY, inspect_local_quote_datasets,
};

use super::{
    ApiResult, DashboardServerState, PRODUCT_API_CONTRACT_VERSION, ProductError, ProductErrorKind,
    ProductReadOnlyBoundaries, ProductSource, mvp_workspace_root, product_error,
    product_error_response, product_request_id, reject_detail_query, strategy_version,
    unix_time_ms, validate_identifier,
};

const DATASET_LIST_SCHEMA_VERSION: &str = "ntpro.product_api.dataset_list.response.v1";
pub(super) const MIN_PRODUCT_BACKTEST_QUOTES: usize = 30;
pub(super) const MAX_PRODUCT_BACKTEST_QUOTES: usize = 1_000_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(super) struct ProductDataset {
    dataset_id: String,
    data_ref: String,
    data_type: String,
    storage_format: String,
    instrument_id: String,
    venue: String,
    venue_ref: String,
    record_count: usize,
    start_time_ns: String,
    end_time_ns: String,
    file_count: usize,
    size_bytes: u64,
    data_sha256: String,
    source: ProductSource,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(in crate::dashboard) struct DatasetListResponse {
    schema_version: String,
    contract_version: String,
    request_id: String,
    data: Vec<ProductDataset>,
    boundaries: ProductReadOnlyBoundaries,
}

#[derive(Debug, Deserialize)]
pub(in crate::dashboard) struct DatasetPath {
    strategy_id: String,
    version_id: String,
}

#[derive(Clone, Debug)]
pub(super) struct ValidatedProductDataset {
    pub(super) inspection: LocalQuoteDatasetInspection,
    pub(super) venue_ref: String,
}

pub(in crate::dashboard) async fn compatible_dataset_list_api(
    State(state): State<DashboardServerState>,
    dataset_path: Result<AxumPath<DatasetPath>, PathRejection>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<DatasetListResponse> {
    let request_id = product_request_id();
    let path = dataset_path.map(|AxumPath(path)| path).map_err(|_| {
        product_error_response(
            &product_error(ProductErrorKind::BadRequest, "dataset_path"),
            &request_id,
        )
    })?;
    let worker_state = state.clone();
    let worker_request_id = request_id.clone();
    let result = tokio::task::spawn_blocking(move || {
        reject_detail_query(raw_query.as_deref())?;
        validate_identifier("strategy_id", &path.strategy_id)
            .map_err(|_| product_error(ProductErrorKind::BadRequest, "strategy_id"))?;
        strategy_version::validate_requested_version_id("version_id", &path.version_id)?;
        let source = super::load_product_catalog_source(&worker_state, unix_time_ms())?;
        if source.strategy.strategy_id != path.strategy_id {
            return Err(product_error(ProductErrorKind::NotFound, "strategy_id"));
        }
        let version = strategy_version::load_product_strategy_version(&source, unix_time_ms())?;
        if version.strategy_version_id() != path.version_id {
            return Err(product_error(
                ProductErrorKind::VersionNotFound,
                "version_id",
            ));
        }
        let datasets = list_compatible_datasets(&worker_state, &version)?;
        Ok(DatasetListResponse {
            schema_version: DATASET_LIST_SCHEMA_VERSION.to_string(),
            contract_version: PRODUCT_API_CONTRACT_VERSION.to_string(),
            request_id: worker_request_id,
            data: datasets,
            boundaries: ProductReadOnlyBoundaries::enforced(),
        })
    })
    .await
    .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "dataset_worker"))
    .and_then(|result| result);
    result
        .map(Json)
        .map_err(|error| product_error_response(&error, &request_id))
}

pub(super) fn resolve_product_dataset(
    state: &DashboardServerState,
    version: &strategy_version::ProductStrategyVersion,
    data_ref: &str,
    data_sha256: &str,
) -> Result<ValidatedProductDataset, ProductError> {
    let datasets = load_compatible_dataset_inspections(state, version)?;
    let mut matches = datasets
        .into_iter()
        .filter(|dataset| dataset.inspection.data_ref() == data_ref);
    let dataset = matches
        .next()
        .ok_or_else(|| product_error(ProductErrorKind::BadRequest, "data_ref"))?;
    if matches.next().is_some() || dataset.inspection.data_sha256 != data_sha256 {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "data_sha256",
        ));
    }
    Ok(dataset)
}

fn list_compatible_datasets(
    state: &DashboardServerState,
    version: &strategy_version::ProductStrategyVersion,
) -> Result<Vec<ProductDataset>, ProductError> {
    load_compatible_dataset_inspections(state, version).map(|datasets| {
        datasets
            .into_iter()
            .map(|dataset| {
                let inspection = dataset.inspection;
                ProductDataset {
                    dataset_id: inspection.dataset_id(),
                    data_ref: inspection.data_ref(),
                    data_type: "quote_tick".to_string(),
                    storage_format: "parquet".to_string(),
                    instrument_id: inspection.instrument_id.clone(),
                    venue: inspection.venue.clone(),
                    venue_ref: dataset.venue_ref,
                    record_count: inspection.record_count,
                    start_time_ns: inspection.start_time_ns.to_string(),
                    end_time_ns: inspection.end_time_ns.to_string(),
                    file_count: inspection.file_count,
                    size_bytes: inspection.size_bytes,
                    data_sha256: inspection.data_sha256,
                    source: ProductSource {
                        source_type: "local_parquet_catalog".to_string(),
                        freshness_status: "verified".to_string(),
                        source_refs: vec![format!(
                            "workspace://{PRODUCT_CATALOG_DIRECTORY}/data/quotes/{}",
                            inspection.instrument_id
                        )],
                    },
                }
            })
            .collect()
    })
}

fn load_compatible_dataset_inspections(
    state: &DashboardServerState,
    version: &strategy_version::ProductStrategyVersion,
) -> Result<Vec<ValidatedProductDataset>, ProductError> {
    if !version
        .data_types()
        .iter()
        .any(|value| value == "quote_tick")
    {
        return Err(product_error(
            ProductErrorKind::BoundaryViolation,
            "strategy_version_data_types",
        ));
    }
    let workspace = mvp_workspace_root(&state.registry_path)?;
    let catalog_root = workspace.join(PRODUCT_CATALOG_DIRECTORY);
    if !catalog_root.exists() {
        return Err(product_error(
            ProductErrorKind::SourceUnavailable,
            "local_data_catalog",
        ));
    }
    let metadata = fs::symlink_metadata(&catalog_root)
        .map_err(|_| product_error(ProductErrorKind::SourceUnavailable, "local_data_catalog"))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(product_error(
            ProductErrorKind::SourceInvalid,
            "local_data_catalog",
        ));
    }
    let inspections = inspect_local_quote_datasets(&catalog_root)
        .map_err(|_| product_error(ProductErrorKind::SourceInvalid, "local_data_catalog"))?;
    let mut compatible = Vec::new();
    for inspection in inspections {
        if !version
            .data_symbols()
            .iter()
            .any(|symbol| symbol == &inspection.instrument_id)
        {
            continue;
        }
        if !product_backtest_quote_count_supported(inspection.record_count) {
            continue;
        }
        let venue_ref = version
            .data_venues()
            .iter()
            .find(|venue| historical_venue_matches_strategy_venue(&inspection.venue, venue))
            .map(|venue| format!("venue://simulated/{venue}"))
            .ok_or_else(|| product_error(ProductErrorKind::SourceInvalid, "dataset_venue"))?;
        compatible.push(ValidatedProductDataset {
            inspection,
            venue_ref,
        });
    }
    compatible.sort_by(|left, right| {
        left.inspection
            .instrument_id
            .cmp(&right.inspection.instrument_id)
    });
    Ok(compatible)
}

fn historical_venue_matches_strategy_venue(historical: &str, strategy: &str) -> bool {
    strategy == historical
        || strategy
            .strip_suffix("_TESTNET")
            .is_some_and(|base| base == historical)
        || strategy
            .strip_suffix("_SANDBOX")
            .is_some_and(|base| base == historical)
}

fn product_backtest_quote_count_supported(record_count: usize) -> bool {
    (MIN_PRODUCT_BACKTEST_QUOTES..=MAX_PRODUCT_BACKTEST_QUOTES).contains(&record_count)
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_PRODUCT_BACKTEST_QUOTES, MIN_PRODUCT_BACKTEST_QUOTES,
        historical_venue_matches_strategy_venue, product_backtest_quote_count_supported,
    };

    #[test]
    fn product_dataset_compatibility_is_explicit_and_bounded() {
        assert!(historical_venue_matches_strategy_venue(
            "BINANCE", "BINANCE"
        ));
        assert!(historical_venue_matches_strategy_venue(
            "BINANCE",
            "BINANCE_TESTNET"
        ));
        assert!(historical_venue_matches_strategy_venue(
            "BINANCE",
            "BINANCE_SANDBOX"
        ));
        assert!(!historical_venue_matches_strategy_venue(
            "BINANCE",
            "KRAKEN_TESTNET"
        ));
        assert_eq!(MIN_PRODUCT_BACKTEST_QUOTES, 30);
        assert_eq!(MAX_PRODUCT_BACKTEST_QUOTES, 1_000_000);
        assert!(!product_backtest_quote_count_supported(29));
        assert!(product_backtest_quote_count_supported(30));
        assert!(product_backtest_quote_count_supported(1_000_000));
        assert!(!product_backtest_quote_count_supported(1_000_001));
    }
}
