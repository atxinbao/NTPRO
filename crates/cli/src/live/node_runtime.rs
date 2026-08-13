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

//! Live and sandbox node runtime lifecycle.
//!
//! This module owns node start/stop orchestration, heartbeat, status, metrics,
//! and shutdown handling. CLI parsing and historical artifact evaluation remain
//! outside this boundary.

use super::*;

const EXECUTION_STATE_HEAD_SCHEMA_VERSION: &str = "ntpro.product_api.live_run_state_head.v2";
const EXECUTION_CONTROL_STATE_SCHEMA_VERSION: &str = "ntpro.product_api.live_run_state.v2";
const EXECUTION_CONTROL_REQUEST_SCHEMA_VERSION: &str = "ntpro.s3.live_execution_control_request.v1";
const EXECUTION_CONTROL_RESULT_SCHEMA_VERSION: &str = "ntpro.s3.live_execution_control_result.v1";
const EXECUTION_RECONCILE_REQUEST_FILE: &str = "execution-reconcile-request.json";
const EXECUTION_RECONCILE_RESULT_FILE: &str = "execution-reconcile-result.json";
const EXECUTION_RECONCILE_RESULT_RECEIPT_FILE: &str = "execution-reconcile-result-receipt.json";
const EXECUTION_CANCEL_REQUEST_FILE: &str = "execution-cancel-request.json";
const EXECUTION_CANCEL_RESULT_FILE: &str = "execution-cancel-result.json";
const EXECUTION_CANCEL_RESULT_RECEIPT_FILE: &str = "execution-cancel-result-receipt.json";
const EXECUTION_ORDER_STATE_FILE: &str = "execution-order-state.json";
const EXECUTION_ORDER_STATE_SCHEMA_VERSION: &str = "ntpro.s3.live_execution_order_state.v2";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProductionExecutionControlRequest {
    schema_version: String,
    request_id: String,
    action: String,
    run_id: String,
    admission_id: String,
    strategy_version_id: String,
    instrument_id: String,
    client_order_id: String,
    source_order_state_sha256: String,
    owner_confirmed: bool,
    operator_confirmed: bool,
    requested_at_unix_ms: u64,
    expires_at_unix_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProductionExecutionControlResult {
    schema_version: String,
    request_sha256: String,
    request_id: String,
    action: String,
    run_id: String,
    admission_id: String,
    strategy_version_id: String,
    instrument_id: String,
    client_order_id: String,
    venue_order_id: Option<String>,
    status: String,
    exchange_order_status: Option<String>,
    original_quantity: Option<String>,
    filled_quantity: Option<String>,
    remaining_quantity: Option<String>,
    query_attempted: bool,
    cancel_attempted: bool,
    cancel_confirmed: bool,
    automatic_retry_attempted: bool,
    manual_review_required: bool,
    error_code: Option<String>,
    completed_at_unix_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProductionExecutionOrderStateSnapshot {
    schema_version: String,
    admission_id: String,
    strategy_version_id: String,
    instrument_id: String,
    client_order_id: Option<String>,
    venue_order_id: Option<String>,
    original_quantity: String,
    filled_quantity: String,
    remaining_quantity: String,
    status: String,
    terminal: bool,
    new_orders_blocked: bool,
    actual_submission_attempted: bool,
    automatic_retry_attempted: bool,
    cancel_attempted: bool,
    replace_attempted: bool,
    last_error: Option<String>,
    updated_at_unix_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutionControlStateHead {
    schema_version: String,
    run_id: String,
    revision: u64,
    state_sha256: String,
    commit_sha256: String,
    anchor_receipt_sha256: String,
    updated_at_unix_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutionControlState {
    schema_version: String,
    run_id: String,
    source_manifest_sha256: String,
    revision: u64,
    previous_state_sha256: Option<String>,
    lifecycle: String,
    preflight_sha256: Option<String>,
    execution_admission_sha256: Option<String>,
    execution_runtime_config_sha256: Option<String>,
    stop_sha256: Option<String>,
    updated_at_unix_ms: u64,
}

struct ProductionMarketDataRuntimeContext<'a> {
    config_path: &'a Path,
    output_dir: &'a Path,
    run_id: &'a str,
    process_mode: ProcessMode,
    status_path: &'a Path,
    metrics_path: &'a Path,
    stdout_log_path: &'a Path,
    stderr_log_path: &'a Path,
    events_log_path: &'a Path,
    execution_enabled: bool,
}

struct ProductionExecutionControlContext<'a> {
    candidate_root: &'a Path,
    output_dir: &'a Path,
    run_id: &'a str,
    execution: &'a ProductionExecutionSection,
    api_key: &'a str,
    api_secret: &'a str,
}

pub(super) async fn run_production_market_data_node_with_command(
    config_path: &Path,
    requested_run_id: Option<&str>,
    output: Option<&Path>,
    stop_file: Option<&Path>,
    shutdown_controls: NtproNodeRunControls,
) -> anyhow::Result<()> {
    let config = load_production_market_data_node_config(config_path)?;
    let market = &config.live_market_data;
    let run_id = requested_run_id.unwrap_or(&market.node_id);
    validate_non_empty("run_id", run_id)?;
    if run_id != market.node_id {
        anyhow::bail!("production market data run_id must match live_market_data.node_id");
    }
    let api_key = std::env::var(&market.api_key_env)
        .with_context(|| format!("{} is required", market.api_key_env))?;
    let api_secret = std::env::var(&market.api_secret_env)
        .with_context(|| format!("{} is required", market.api_secret_env))?;
    if api_key.trim().is_empty() || api_secret.trim().is_empty() {
        anyhow::bail!("production market data credentials must not be empty");
    }

    let output_dir = output.map_or_else(
        || PathBuf::from("artifacts/live-market-data").join(run_id),
        Path::to_path_buf,
    );
    fs::create_dir_all(output_dir.join("logs"))?;
    if let Some(execution) = &config.live_execution {
        validate_execution_runtime_authority(config_path, &output_dir, run_id, execution)?;
    }
    let status_path = output_dir.join("status.json");
    let metrics_path = output_dir.join("metrics.json");
    let stdout_log_path = output_dir.join("logs/stdout.log");
    let stderr_log_path = output_dir.join("logs/stderr.log");
    let events_log_path = output_dir.join("logs/events.log");
    let context = ProductionMarketDataRuntimeContext {
        config_path,
        output_dir: &output_dir,
        run_id,
        process_mode: ProcessMode::SpawnedProcess,
        status_path: &status_path,
        metrics_path: &metrics_path,
        stdout_log_path: &stdout_log_path,
        stderr_log_path: &stderr_log_path,
        events_log_path: &events_log_path,
        execution_enabled: config.live_execution.is_some(),
    };

    let trader_id = TraderId::from(market.trader_id.as_str());
    let data_config = BinanceDataClientConfig {
        product_types: vec![BinanceProductType::Spot],
        environment: BinanceEnvironment::Live,
        api_key: Some(api_key),
        api_secret: Some(api_secret),
        ws_reconnect_max_attempts: Some(0),
        ..Default::default()
    };
    let instrument_ids = market
        .symbols
        .iter()
        .map(|value| InstrumentId::from_str(value))
        .collect::<Result<Vec<_>, _>>()?;
    let actor = ProductionMarketDataActor::new(ClientId::from("BINANCE"), instrument_ids);
    let (quote_count, trade_count, last_event_unix_ms) = actor.counters();
    let execution_credentials = if let Some(execution) = &config.live_execution {
        let exec_api_key = std::env::var(&execution.api_key_env)
            .with_context(|| format!("{} is required", execution.api_key_env))?;
        let exec_api_secret = std::env::var(&execution.api_secret_env)
            .with_context(|| format!("{} is required", execution.api_secret_env))?;
        if exec_api_key.trim().is_empty() || exec_api_secret.trim().is_empty() {
            anyhow::bail!("production execution credentials must not be empty");
        }
        Some((exec_api_key, exec_api_secret))
    } else {
        None
    };
    let mut builder = LiveNode::builder(trader_id, Environment::Live)?
        .with_name(run_id)
        .with_reconciliation(config.live_execution.is_some())
        .with_shutdown_on_data_disconnect(true)
        .with_timeout_connection(config.shutdown.connection_timeout_secs)
        .with_timeout_disconnection_secs(config.shutdown.disconnection_timeout_secs)
        .with_delay_post_stop_secs(config.shutdown.post_stop_delay_secs)
        .add_data_client(
            Some("BINANCE".to_string()),
            Box::new(BinanceDataClientFactory::new()),
            Box::new(data_config),
        )?;
    if let Some(execution) = &config.live_execution {
        builder = builder.with_risk_engine_config(LiveRiskEngineConfig {
            max_notional_per_order: HashMap::from([(
                execution.instrument_id.clone(),
                execution.max_notional.clone(),
            )]),
            ..Default::default()
        });
        let (exec_api_key, exec_api_secret) = execution_credentials
            .as_ref()
            .context("production execution credentials are unavailable")?;
        builder = builder.add_exec_client(
            Some("BINANCE".to_string()),
            Box::new(BinanceExecutionClientFactory::new()),
            Box::new(BinanceExecClientConfig {
                trader_id,
                account_id: AccountId::from(execution.account_id.as_str()),
                product_types: vec![BinanceProductType::Spot],
                environment: BinanceEnvironment::Live,
                api_key: Some(exec_api_key.clone()),
                api_secret: Some(exec_api_secret.clone()),
                ..Default::default()
            }),
        )?;
    }
    let mut node = builder.build()?;
    node.add_actor(actor)?;
    if let Some(execution) = &config.live_execution {
        node.add_strategy(ProductionSingleShotExecutionStrategy::from_config(
            execution,
            &output_dir,
        )?)?;
    } else if !node.kernel().exec_engine.borrow().client_ids().is_empty() {
        anyhow::bail!("production market data Runtime must not register execution clients");
    }

    let handle = node.handle();
    let run_future = node.run();
    tokio::pin!(run_future);
    let shutdown_signal = wait_for_shutdown_signal();
    tokio::pin!(shutdown_signal);
    let mut started_at: Option<String> = None;
    let mut started_instant: Option<Instant> = None;
    let mut last_heartbeat: Option<Instant> = None;
    let mut last_execution_control_poll: Option<Instant> = None;
    let mut shutdown_reason: Option<ShutdownReason> = None;
    let runtime_result = loop {
        tokio::select! {
            result = &mut run_future => break result,
            result = &mut shutdown_signal, if shutdown_reason.is_none() => {
                shutdown_reason = Some(result?);
                handle.stop();
            }
            () = sleep(SHUTDOWN_POLL_INTERVAL) => {
                if handle.state() == NodeState::Running && started_at.is_none() {
                    let observed_at = now_millis();
                    let observed_instant = Instant::now();
                    write_production_market_data_heartbeat(
                        &context,
                        &observed_at,
                        observed_instant,
                        true,
                    )?;
                    atomic_write_text(
                        &events_log_path,
                        if context.execution_enabled {
                            "phase=start status=ok environment=live data_connection=connected execution_connection=connected subscriptions=quotes,trades order_endpoint_access=true single_shot=true automatic_retry=false\n"
                        } else {
                            "phase=start status=ok environment=live data_connection=connected execution_connection=not_configured subscriptions=quotes,trades order_endpoint_access=false real_orders_submitted=false\n"
                        },
                    )?;
                    started_at = Some(observed_at);
                    started_instant = Some(observed_instant);
                }
                if shutdown_reason.is_none() {
                    shutdown_reason = requested_production_market_data_shutdown(
                        stop_file,
                        shutdown_controls,
                        started_instant,
                    );
                    if shutdown_reason.is_some() {
                        handle.stop();
                    }
                }
                if let (Some(started_at), Some(started_instant)) =
                    (started_at.as_deref(), started_instant)
                    && last_heartbeat
                        .is_none_or(|last| last.elapsed() >= shutdown_controls.heartbeat_interval)
                {
                    write_production_market_data_heartbeat(
                        &context,
                        started_at,
                        started_instant,
                        true,
                    )?;
                    last_heartbeat = Some(Instant::now());
                }
                if context.execution_enabled
                    && last_execution_control_poll.is_none_or(|last| {
                        last.elapsed() >= Duration::from_millis(250)
                    })
                {
                    let execution = config
                        .live_execution
                        .as_ref()
                        .context("production execution control lost its configuration")?;
                    let (exec_api_key, exec_api_secret) = execution_credentials
                        .as_ref()
                        .context("production execution control lost its credentials")?;
                    process_production_execution_controls(&ProductionExecutionControlContext {
                        candidate_root: config_path
                            .parent()
                            .context("live execution config must have a candidate root")?,
                        output_dir: &output_dir,
                        run_id,
                        execution,
                        api_key: exec_api_key,
                        api_secret: exec_api_secret,
                    })
                    .await?;
                    last_execution_control_poll = Some(Instant::now());
                }
            }
        }
    };
    let started_at = started_at
        .context("production market data Runtime exited before reaching connected running state")?;
    let started_instant = started_instant
        .context("production market data Runtime exited before recording its start time")?;
    let stopped_at = now_millis();
    let runtime_error = runtime_result
        .as_ref()
        .err()
        .map(|_| "production market data Runtime exited unexpectedly".to_string());
    let mut status = production_market_data_status(
        &context,
        NodeState::Stopped,
        ConnectionStatus::Disconnected,
        &started_at,
        Some(&stopped_at),
    );
    status.last_error = runtime_error.clone();
    write_status(&status_path, &status)?;
    write_production_market_data_metrics(
        &context,
        &status,
        NodeMetricCounts {
            uptime_ms: Some(millis_to_u64(started_instant.elapsed().as_millis())),
            starts_total: 1,
            stops_total: 1,
            state_transitions_total: 2,
        },
    )?;
    atomic_write_text(
        &output_dir.join("summary.txt"),
        &format!(
            "status={}\nmode={}\nrun_id={run_id}\nenvironment=live\nvenue=BINANCE\nproduct_type=spot\nsymbols={}\nquote_events={}\ntrade_events={}\nlast_market_event_unix_ms={}\ndata_connection=disconnected\nexecution_connection={}\nexecution_client_enabled={}\norder_endpoint_access_allowed={}\norder_submission_allowed={}\nsingle_shot={}\nreal_orders_submitted={}\ncancel_order_allowed=false\nreplace_order_allowed=false\nautomatic_retry_allowed=false\nautomatic_reconnect_allowed=false\nshutdown_reason={}\n",
            if runtime_error.is_some() {
                "error"
            } else {
                "ok"
            },
            if context.execution_enabled {
                "production-single-shot-execution"
            } else {
                "production-market-data"
            },
            market.symbols.join(","),
            quote_count.load(std::sync::atomic::Ordering::Acquire),
            trade_count.load(std::sync::atomic::Ordering::Acquire),
            last_event_unix_ms.load(std::sync::atomic::Ordering::Acquire),
            if context.execution_enabled {
                "disconnected"
            } else {
                "not_configured"
            },
            context.execution_enabled,
            context.execution_enabled,
            context.execution_enabled,
            context.execution_enabled,
            status.real_orders_submitted,
            shutdown_reason.map_or("runtime-error", |reason| reason.label()),
        ),
    )?;
    runtime_result
}

async fn process_production_execution_controls(
    context: &ProductionExecutionControlContext<'_>,
) -> anyhow::Result<()> {
    process_production_execution_control(
        context,
        "reconcile",
        EXECUTION_RECONCILE_REQUEST_FILE,
        "execution-reconcile-attempt.json",
        EXECUTION_RECONCILE_RESULT_FILE,
        EXECUTION_RECONCILE_RESULT_RECEIPT_FILE,
    )
    .await?;
    process_production_execution_control(
        context,
        "cancel",
        EXECUTION_CANCEL_REQUEST_FILE,
        "execution-cancel-attempt.json",
        EXECUTION_CANCEL_RESULT_FILE,
        EXECUTION_CANCEL_RESULT_RECEIPT_FILE,
    )
    .await
}

async fn process_production_execution_control(
    context: &ProductionExecutionControlContext<'_>,
    expected_action: &str,
    request_file: &str,
    attempt_file: &str,
    result_file: &str,
    result_receipt_file: &str,
) -> anyhow::Result<()> {
    let request_path = context.candidate_root.join(request_file);
    if !request_path.exists() {
        return Ok(());
    }
    let request_raw = read_bounded_execution_authority_file(&request_path)?;
    let request: ProductionExecutionControlRequest = serde_json::from_slice(&request_raw)
        .context("live execution control request is invalid")?;
    validate_production_execution_control_request(context, &request, expected_action)?;
    let request_sha256 = execution_sha256_ref(&request_raw);
    let result_path = context.candidate_root.join(result_file);
    let result_receipt_path = context.candidate_root.join(result_receipt_file);
    if result_path.exists() {
        let result_raw = read_bounded_execution_authority_file(&result_path)
            .context("live execution control result is invalid")?;
        let result: ProductionExecutionControlResult = serde_json::from_slice(&result_raw)
            .context("live execution control result is invalid")?;
        if result.schema_version != EXECUTION_CONTROL_RESULT_SCHEMA_VERSION
            || result.request_sha256 != request_sha256
            || result.request_id != request.request_id
            || result.action != expected_action
            || result.run_id != context.run_id
        {
            anyhow::bail!("live execution control result identity is invalid");
        }
        publish_execution_control_result(context, &request_sha256, &result)?;
        return Ok(());
    }
    if result_receipt_path.exists() {
        anyhow::bail!("live execution control result receipt exists without result bytes");
    }
    let attempt_path = context.candidate_root.join(attempt_file);
    if attempt_path.exists() {
        let attempt_raw = read_bounded_execution_authority_file(&attempt_path)?;
        if attempt_raw != request_raw {
            anyhow::bail!("live execution control attempt does not match its request");
        }
        publish_execution_control_result(
            context,
            &request_sha256,
            &interrupted_execution_control_result(&request, &request_sha256),
        )?;
        return Ok(());
    }
    atomic_write_text(
        &attempt_path,
        std::str::from_utf8(&request_raw).context("live execution control request is not UTF-8")?,
    )?;

    let order_state_raw = read_bounded_execution_authority_file(
        &context.output_dir.join(EXECUTION_ORDER_STATE_FILE),
    )?;
    if request.source_order_state_sha256 != execution_sha256_ref(&order_state_raw) {
        publish_execution_control_result(
            context,
            &request_sha256,
            &failed_execution_control_result(
                &request,
                &request_sha256,
                "source_order_state_drift",
                false,
                false,
            ),
        )?;
        return Ok(());
    }
    let order_state: ProductionExecutionOrderStateSnapshot =
        match serde_json::from_slice(&order_state_raw) {
            Ok(order_state) => order_state,
            Err(_) => {
                publish_execution_control_result(
                    context,
                    &request_sha256,
                    &failed_execution_control_result(
                        &request,
                        &request_sha256,
                        "source_order_state_invalid",
                        false,
                        false,
                    ),
                )?;
                return Ok(());
            }
        };
    if !execution_control_source_order_matches(context, &request, &order_state) {
        publish_execution_control_result(
            context,
            &request_sha256,
            &failed_execution_control_result(
                &request,
                &request_sha256,
                "source_order_state_identity_mismatch",
                false,
                false,
            ),
        )?;
        return Ok(());
    }

    let client = match BinanceSpotHttpClient::new(
        BinanceEnvironment::Live,
        get_atomic_clock_realtime(),
        Some(context.api_key.to_string()),
        Some(context.api_secret.to_string()),
        None,
        None,
        Some(10),
        None,
    ) {
        Ok(client) => client,
        Err(_) => {
            publish_execution_control_result(
                context,
                &request_sha256,
                &failed_execution_control_result(
                    &request,
                    &request_sha256,
                    "exchange_client_initialization_failed",
                    false,
                    false,
                ),
            )?;
            return Ok(());
        }
    };
    let instruments = match client.request_instruments().await {
        Ok(instruments) => instruments,
        Err(_) => {
            publish_execution_control_result(
                context,
                &request_sha256,
                &failed_execution_control_result(
                    &request,
                    &request_sha256,
                    "exchange_instrument_query_failed",
                    false,
                    false,
                ),
            )?;
            return Ok(());
        }
    };
    client.cache_instruments(instruments);
    let instrument_id = match InstrumentId::from_str(&request.instrument_id) {
        Ok(instrument_id) => instrument_id,
        Err(_) => {
            publish_execution_control_result(
                context,
                &request_sha256,
                &failed_execution_control_result(
                    &request,
                    &request_sha256,
                    "control_instrument_invalid",
                    false,
                    false,
                ),
            )?;
            return Ok(());
        }
    };
    let client_order_id = ClientOrderId::from(request.client_order_id.as_str());
    let account_id = AccountId::from(context.execution.account_id.as_str());
    let before = match client
        .request_order_status_report(account_id, instrument_id, None, Some(client_order_id))
        .await
    {
        Ok(report) => report,
        Err(_) => {
            publish_execution_control_result(
                context,
                &request_sha256,
                &failed_execution_control_result(
                    &request,
                    &request_sha256,
                    "exchange_order_query_failed",
                    true,
                    false,
                ),
            )?;
            return Ok(());
        }
    };
    let Some(before) = before else {
        publish_execution_control_result(
            context,
            &request_sha256,
            &failed_execution_control_result(
                &request,
                &request_sha256,
                "order_not_found_at_venue",
                true,
                false,
            ),
        )?;
        return Ok(());
    };
    validate_execution_order_report(context, &request, &order_state, &before)?;

    let result = if expected_action == "reconcile" {
        execution_control_result_from_report(
            &request,
            &request_sha256,
            &before,
            "reconciled",
            true,
            false,
            false,
            false,
            None,
        )
    } else if !before.order_status.is_open()
        || matches!(
            before.order_status,
            OrderStatus::PendingCancel | OrderStatus::PendingUpdate
        )
    {
        execution_control_result_from_report(
            &request,
            &request_sha256,
            &before,
            "cancel_not_required_terminal_or_pending",
            true,
            false,
            before.order_status == OrderStatus::Canceled,
            false,
            None,
        )
    } else {
        let venue_order_id = match client
            .cancel_order(
                instrument_id,
                Some(before.venue_order_id),
                Some(client_order_id),
            )
            .await
        {
            Ok(venue_order_id) => venue_order_id,
            Err(_) => {
                publish_execution_control_result(
                    context,
                    &request_sha256,
                    &failed_execution_control_result(
                        &request,
                        &request_sha256,
                        "exchange_cancel_request_failed",
                        true,
                        true,
                    ),
                )?;
                return Ok(());
            }
        };
        let after = match client
            .request_order_status_report(
                account_id,
                instrument_id,
                Some(venue_order_id),
                Some(client_order_id),
            )
            .await
        {
            Ok(report) => report,
            Err(_) => {
                publish_execution_control_result(
                    context,
                    &request_sha256,
                    &failed_execution_control_result(
                        &request,
                        &request_sha256,
                        "exchange_cancel_readback_failed",
                        true,
                        true,
                    ),
                )?;
                return Ok(());
            }
        };
        match after {
            Some(after) => {
                validate_execution_order_report(context, &request, &order_state, &after)?;
                let confirmed = after.order_status == OrderStatus::Canceled;
                execution_control_result_from_report(
                    &request,
                    &request_sha256,
                    &after,
                    if confirmed {
                        "cancel_confirmed"
                    } else {
                        "cancel_sent_readback_pending"
                    },
                    true,
                    true,
                    confirmed,
                    !confirmed,
                    None,
                )
            }
            None => failed_execution_control_result(
                &request,
                &request_sha256,
                "cancel_sent_order_not_found_on_readback",
                true,
                true,
            ),
        }
    };
    publish_execution_control_result(context, &request_sha256, &result)?;
    Ok(())
}

fn publish_execution_control_result(
    context: &ProductionExecutionControlContext<'_>,
    request_sha256: &str,
    result: &ProductionExecutionControlResult,
) -> anyhow::Result<()> {
    let result_raw = serde_json::to_vec_pretty(result)
        .context("live execution control result serialization failed")?;
    crate::dashboard::product_api::live_run_anchor::anchor_runtime_control_result(
        &crate::dashboard::product_api::live_run_anchor::LiveExecutionControlResultAnchor {
            candidate_root: context.candidate_root,
            run_id: context.run_id,
            action: &result.action,
            result_raw: &result_raw,
            request_sha256,
            completed_at_unix_ms: result.completed_at_unix_ms,
        },
    )
}

fn validate_production_execution_control_request(
    context: &ProductionExecutionControlContext<'_>,
    request: &ProductionExecutionControlRequest,
    expected_action: &str,
) -> anyhow::Result<()> {
    let now = current_unix_timestamp_millis();
    let valid_roles = match expected_action {
        "reconcile" => request.owner_confirmed && !request.operator_confirmed,
        "cancel" => request.owner_confirmed && request.operator_confirmed,
        _ => false,
    };
    let valid_hash = request.source_order_state_sha256.len() == 71
        && request.source_order_state_sha256.starts_with("sha256:")
        && request.source_order_state_sha256[7..]
            .bytes()
            .all(|value| value.is_ascii_digit() || (b'a'..=b'f').contains(&value));
    if request.schema_version != EXECUTION_CONTROL_REQUEST_SCHEMA_VERSION
        || request.request_id.trim().is_empty()
        || request.request_id.len() > 128
        || !request
            .request_id
            .bytes()
            .all(|value| value.is_ascii_alphanumeric() || matches!(value, b'-' | b'_'))
        || request.action != expected_action
        || request.run_id != context.run_id
        || request.admission_id != context.execution.admission_id
        || request.strategy_version_id != context.execution.strategy_version_id
        || request.instrument_id != context.execution.instrument_id
        || request.client_order_id.trim().is_empty()
        || !valid_hash
        || !valid_roles
        || request.requested_at_unix_ms > now
        || request.expires_at_unix_ms <= request.requested_at_unix_ms
        || request.expires_at_unix_ms <= now
        || request.expires_at_unix_ms > request.requested_at_unix_ms.saturating_add(5 * 60 * 1_000)
    {
        anyhow::bail!("live execution control request does not match the admitted order");
    }
    Ok(())
}

fn execution_control_source_order_matches(
    context: &ProductionExecutionControlContext<'_>,
    request: &ProductionExecutionControlRequest,
    order: &ProductionExecutionOrderStateSnapshot,
) -> bool {
    let quantities_valid = Decimal::from_str_exact(&order.original_quantity)
        .ok()
        .zip(Decimal::from_str_exact(&order.filled_quantity).ok())
        .zip(Decimal::from_str_exact(&order.remaining_quantity).ok())
        .is_some_and(|((original, filled), remaining)| {
            original > Decimal::ZERO
                && filled >= Decimal::ZERO
                && remaining >= Decimal::ZERO
                && filled + remaining == original
                && original
                    == Decimal::from_str_exact(&context.execution.quantity).unwrap_or_default()
        });
    order.schema_version == EXECUTION_ORDER_STATE_SCHEMA_VERSION
        && order.admission_id == context.execution.admission_id
        && order.strategy_version_id == context.execution.strategy_version_id
        && order.instrument_id == context.execution.instrument_id
        && order.client_order_id.as_deref() == Some(request.client_order_id.as_str())
        && order
            .venue_order_id
            .as_deref()
            .is_none_or(|value| !value.trim().is_empty())
        && quantities_valid
        && order.new_orders_blocked
        && order.actual_submission_attempted
        && !order.automatic_retry_attempted
        && !order.replace_attempted
        && (!order.cancel_attempted || request.action == "cancel")
        && !matches!(order.status.as_str(), "waiting_for_instrument" | "denied")
        && order.updated_at_unix_ms > 0
        && (order.terminal
            == matches!(
                order.status.as_str(),
                "rejected" | "expired" | "filled" | "canceled" | "submission_failed"
            ))
        && order
            .last_error
            .as_deref()
            .is_none_or(|value| !value.trim().is_empty())
}

fn validate_execution_order_report(
    context: &ProductionExecutionControlContext<'_>,
    request: &ProductionExecutionControlRequest,
    source_order: &ProductionExecutionOrderStateSnapshot,
    report: &OrderStatusReport,
) -> anyhow::Result<()> {
    let admitted_quantity = Quantity::from_str(&context.execution.quantity)
        .map_err(|error| anyhow::anyhow!("admitted execution quantity is invalid: {error}"))?;
    if report.account_id != AccountId::from(context.execution.account_id.as_str())
        || report.instrument_id != InstrumentId::from_str(&request.instrument_id)?
        || report.client_order_id != Some(ClientOrderId::from(request.client_order_id.as_str()))
        || source_order
            .venue_order_id
            .as_deref()
            .is_some_and(|value| report.venue_order_id.to_string() != value)
        || report.quantity != admitted_quantity
        || report.filled_qty > report.quantity
    {
        anyhow::bail!(
            "exchange order report identity or quantity does not match the admitted order"
        );
    }
    Ok(())
}

#[expect(clippy::too_many_arguments)]
fn execution_control_result_from_report(
    request: &ProductionExecutionControlRequest,
    request_sha256: &str,
    report: &OrderStatusReport,
    status: &str,
    query_attempted: bool,
    cancel_attempted: bool,
    cancel_confirmed: bool,
    manual_review_required: bool,
    error_code: Option<String>,
) -> ProductionExecutionControlResult {
    ProductionExecutionControlResult {
        schema_version: EXECUTION_CONTROL_RESULT_SCHEMA_VERSION.to_string(),
        request_sha256: request_sha256.to_string(),
        request_id: request.request_id.clone(),
        action: request.action.clone(),
        run_id: request.run_id.clone(),
        admission_id: request.admission_id.clone(),
        strategy_version_id: request.strategy_version_id.clone(),
        instrument_id: request.instrument_id.clone(),
        client_order_id: request.client_order_id.clone(),
        venue_order_id: Some(report.venue_order_id.to_string()),
        status: status.to_string(),
        exchange_order_status: Some(report.order_status.as_ref().to_ascii_lowercase()),
        original_quantity: Some(report.quantity.to_string()),
        filled_quantity: Some(report.filled_qty.to_string()),
        remaining_quantity: Some((report.quantity - report.filled_qty).to_string()),
        query_attempted,
        cancel_attempted,
        cancel_confirmed,
        automatic_retry_attempted: false,
        manual_review_required,
        error_code,
        completed_at_unix_ms: current_unix_timestamp_millis(),
    }
}

fn failed_execution_control_result(
    request: &ProductionExecutionControlRequest,
    request_sha256: &str,
    error_code: &str,
    query_attempted: bool,
    cancel_attempted: bool,
) -> ProductionExecutionControlResult {
    ProductionExecutionControlResult {
        schema_version: EXECUTION_CONTROL_RESULT_SCHEMA_VERSION.to_string(),
        request_sha256: request_sha256.to_string(),
        request_id: request.request_id.clone(),
        action: request.action.clone(),
        run_id: request.run_id.clone(),
        admission_id: request.admission_id.clone(),
        strategy_version_id: request.strategy_version_id.clone(),
        instrument_id: request.instrument_id.clone(),
        client_order_id: request.client_order_id.clone(),
        venue_order_id: None,
        status: "unknown_manual_review".to_string(),
        exchange_order_status: None,
        original_quantity: None,
        filled_quantity: None,
        remaining_quantity: None,
        query_attempted,
        cancel_attempted,
        cancel_confirmed: false,
        automatic_retry_attempted: false,
        manual_review_required: true,
        error_code: Some(error_code.to_string()),
        completed_at_unix_ms: current_unix_timestamp_millis(),
    }
}

fn interrupted_execution_control_result(
    request: &ProductionExecutionControlRequest,
    request_sha256: &str,
) -> ProductionExecutionControlResult {
    failed_execution_control_result(
        request,
        request_sha256,
        "previous_attempt_interrupted_no_retry",
        true,
        request.action == "cancel",
    )
}

fn validate_execution_runtime_authority(
    config_path: &Path,
    output_dir: &Path,
    run_id: &str,
    execution: &ProductionExecutionSection,
) -> anyhow::Result<()> {
    let config_metadata = fs::symlink_metadata(config_path)?;
    if !config_metadata.is_file() || config_metadata.file_type().is_symlink() {
        anyhow::bail!("live execution config must be a regular non-symlink file");
    }
    let config_raw = read_bounded_execution_authority_file(config_path)?;
    let candidate_root = config_path
        .parent()
        .context("live execution config must have a candidate root")?;
    let head_raw = read_bounded_execution_authority_file(&candidate_root.join("state-head.json"))?;
    let head: ExecutionControlStateHead = serde_json::from_slice(&head_raw)
        .context("live execution control state head is invalid")?;
    let state_path = candidate_root.join(format!("state-{:020}.json", head.revision));
    let state_raw = read_bounded_execution_authority_file(&state_path)?;
    let state: ExecutionControlState =
        serde_json::from_slice(&state_raw).context("live execution control state is invalid")?;
    let receipt_raw = read_bounded_execution_authority_file(
        &candidate_root.join(format!("anchor-receipt-{:020}.json", head.revision)),
    )?;
    let expected_output = fs::canonicalize(&execution.runtime_artifact_root)
        .context("live execution admitted Runtime artifact root is unavailable")?;
    let expected_control = fs::canonicalize(&execution.control_artifact_root)
        .context("live execution control artifact root is unavailable")?;
    let actual_control =
        fs::canonicalize(candidate_root).context("live execution candidate root is unavailable")?;
    let actual_output = fs::canonicalize(output_dir)
        .context("live execution Runtime artifact root is unavailable")?;
    let config_sha256 = execution_sha256_ref(&config_raw);
    let valid = head.schema_version == EXECUTION_STATE_HEAD_SCHEMA_VERSION
        && head.run_id == run_id
        && head.state_sha256 == execution_sha256_ref(&state_raw)
        && head.updated_at_unix_ms == state.updated_at_unix_ms
        && state.schema_version == EXECUTION_CONTROL_STATE_SCHEMA_VERSION
        && state.run_id == run_id
        && state.revision == head.revision
        && state.previous_state_sha256.is_some()
        && state.lifecycle == "starting"
        && state.source_manifest_sha256 == execution.source_manifest_sha256
        && state.preflight_sha256.is_some()
        && state.execution_admission_sha256.as_deref()
            == Some(execution.execution_admission_sha256.as_str())
        && state.execution_runtime_config_sha256.as_deref() == Some(config_sha256.as_str())
        && state.stop_sha256.is_none()
        && expected_output == actual_output
        && expected_control == actual_control;
    if !valid {
        anyhow::bail!("live execution Runtime authority does not match the anchored control plane");
    }
    crate::dashboard::product_api::live_run_anchor::validate_runtime_authority(
        run_id,
        state.revision,
        &state_raw,
        &head.commit_sha256,
        &receipt_raw,
        &head.anchor_receipt_sha256,
        state.updated_at_unix_ms,
    )?;
    crate::dashboard::product_api::live_run_anchor::claim_runtime_authority(
        &crate::dashboard::product_api::live_run_anchor::LiveExecutionRuntimeClaim {
            candidate_root,
            run_id,
            control_state_revision: state.revision,
            starting_receipt_raw: &receipt_raw,
            expected_starting_receipt_sha256: &head.anchor_receipt_sha256,
            source_manifest_sha256: &execution.source_manifest_sha256,
            execution_admission_sha256: &execution.execution_admission_sha256,
            runtime_config_sha256: &config_sha256,
            runtime_artifact_root: output_dir,
            claimed_at_unix_ms: current_unix_timestamp_millis(),
        },
    )?;
    Ok(())
}

fn read_bounded_execution_authority_file(path: &Path) -> anyhow::Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 64 * 1024 {
        anyhow::bail!("live execution authority artifact must be a bounded regular file");
    }
    fs::read(path).map_err(Into::into)
}

fn execution_sha256_ref(raw: &[u8]) -> String {
    let value = aws_lc_rs::digest::digest(&aws_lc_rs::digest::SHA256, raw);
    let mut encoded = String::with_capacity(value.as_ref().len() * 2 + 7);
    encoded.push_str("sha256:");
    for byte in value.as_ref() {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

#[cfg(test)]
mod execution_authority_tests {
    use nautilus_core::UnixNanos;
    use nautilus_model::{enums::OrderType, identifiers::VenueOrderId};
    use tempfile::tempdir;

    use super::*;

    fn execution_section(runtime_root: &Path, control_root: &Path) -> ProductionExecutionSection {
        ProductionExecutionSection {
            schema_version: PRODUCTION_EXECUTION_SCHEMA_VERSION.to_string(),
            source_manifest_sha256: format!("sha256:{}", "1".repeat(64)),
            execution_admission_sha256: format!("sha256:{}", "4".repeat(64)),
            runtime_artifact_root: runtime_root.to_path_buf(),
            control_artifact_root: control_root.to_path_buf(),
            risk_policy_ref: format!("risk-config-sha256:{}", "7".repeat(64)),
            owner_authority_ref: "role://institution-owner".to_string(),
            risk_authority_ref: "policy://risk/test-v1".to_string(),
            operator_authority_ref: "role://operations-operator".to_string(),
            admission_id: "admission-authority-test".to_string(),
            strategy_version_id: "ema-cross@v1".to_string(),
            account_id: "BINANCE-001".to_string(),
            instrument_id: "BTCUSDT.BINANCE".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            time_in_force: "GTC".to_string(),
            price: "1.00".to_string(),
            quantity: "0.01".to_string(),
            max_notional: "1.00".to_string(),
            risk_policy_max_notional: "10.00".to_string(),
            expires_at_unix_ms: u64::MAX,
            api_key_env: "NTPRO_BINANCE_LIVE_API_KEY".to_string(),
            api_secret_env: "NTPRO_BINANCE_LIVE_API_SECRET".to_string(),
            owner_confirmed: true,
            risk_confirmed: true,
            operator_confirmed: true,
            kill_switch_active: false,
            single_shot: true,
            cancel_order_allowed: false,
            replace_order_allowed: false,
            automatic_retry_allowed: false,
            automatic_recovery_allowed: false,
        }
    }

    fn control_request(action: &str) -> ProductionExecutionControlRequest {
        let now = current_unix_timestamp_millis();
        ProductionExecutionControlRequest {
            schema_version: EXECUTION_CONTROL_REQUEST_SCHEMA_VERSION.to_string(),
            request_id: format!("{action}-001"),
            action: action.to_string(),
            run_id: "live-candidate-authority-test".to_string(),
            admission_id: "admission-authority-test".to_string(),
            strategy_version_id: "ema-cross@v1".to_string(),
            instrument_id: "BTCUSDT.BINANCE".to_string(),
            client_order_id: "S3LV007-001".to_string(),
            source_order_state_sha256: format!("sha256:{}", "a".repeat(64)),
            owner_confirmed: true,
            operator_confirmed: action == "cancel",
            requested_at_unix_ms: now,
            expires_at_unix_ms: now + 60_000,
        }
    }

    fn partial_fill_report() -> OrderStatusReport {
        OrderStatusReport::new(
            AccountId::from("BINANCE-001"),
            InstrumentId::from("BTCUSDT.BINANCE"),
            Some(ClientOrderId::from("S3LV007-001")),
            VenueOrderId::from("1001"),
            OrderSide::Buy,
            OrderType::Limit,
            TimeInForce::Gtc,
            OrderStatus::PartiallyFilled,
            Quantity::from("0.01000000"),
            Quantity::from("0.00400000"),
            UnixNanos::from(1_000_000_000),
            UnixNanos::from(2_000_000_000),
            UnixNanos::from(3_000_000_000),
            None,
        )
    }

    fn partial_fill_order_state() -> ProductionExecutionOrderStateSnapshot {
        ProductionExecutionOrderStateSnapshot {
            schema_version: EXECUTION_ORDER_STATE_SCHEMA_VERSION.to_string(),
            admission_id: "admission-authority-test".to_string(),
            strategy_version_id: "ema-cross@v1".to_string(),
            instrument_id: "BTCUSDT.BINANCE".to_string(),
            client_order_id: Some("S3LV007-001".to_string()),
            venue_order_id: Some("1001".to_string()),
            original_quantity: "0.01000000".to_string(),
            filled_quantity: "0.00400000".to_string(),
            remaining_quantity: "0.00600000".to_string(),
            status: "partially_filled".to_string(),
            terminal: false,
            new_orders_blocked: true,
            actual_submission_attempted: true,
            automatic_retry_attempted: false,
            cancel_attempted: false,
            replace_attempted: false,
            last_error: None,
            updated_at_unix_ms: 1,
        }
    }

    #[test]
    fn execution_control_roles_are_action_specific_and_fail_closed() {
        let temp = tempdir().unwrap();
        let execution = execution_section(temp.path(), temp.path());
        let context = ProductionExecutionControlContext {
            candidate_root: temp.path(),
            output_dir: temp.path(),
            run_id: "live-candidate-authority-test",
            execution: &execution,
            api_key: "test-key",
            api_secret: "test-secret",
        };
        let reconcile = control_request("reconcile");
        validate_production_execution_control_request(&context, &reconcile, "reconcile").unwrap();
        let mut invalid_reconcile = reconcile;
        invalid_reconcile.operator_confirmed = true;
        assert!(
            validate_production_execution_control_request(
                &context,
                &invalid_reconcile,
                "reconcile"
            )
            .is_err()
        );
        let cancel = control_request("cancel");
        validate_production_execution_control_request(&context, &cancel, "cancel").unwrap();
        let mut invalid_cancel = cancel;
        invalid_cancel.operator_confirmed = false;
        assert!(
            validate_production_execution_control_request(&context, &invalid_cancel, "cancel")
                .is_err()
        );
    }

    #[test]
    fn partial_fill_reconciliation_preserves_quantity_and_never_retries() {
        let request = control_request("reconcile");
        let report = partial_fill_report();
        let result = execution_control_result_from_report(
            &request,
            "sha256:request",
            &report,
            "reconciled",
            true,
            false,
            false,
            false,
            None,
        );
        assert_eq!(result.original_quantity.as_deref(), Some("0.01000000"));
        assert_eq!(result.filled_quantity.as_deref(), Some("0.00400000"));
        assert_eq!(result.remaining_quantity.as_deref(), Some("0.00600000"));
        assert_eq!(
            result.exchange_order_status.as_deref(),
            Some("partially_filled")
        );
        assert!(!result.cancel_attempted);
        assert!(!result.automatic_retry_attempted);
        assert!(!result.manual_review_required);
    }

    #[test]
    fn exchange_report_must_match_admitted_quantity_and_venue_order() {
        let temp = tempdir().unwrap();
        let execution = execution_section(temp.path(), temp.path());
        let context = ProductionExecutionControlContext {
            candidate_root: temp.path(),
            output_dir: temp.path(),
            run_id: "live-candidate-authority-test",
            execution: &execution,
            api_key: "test-key",
            api_secret: "test-secret",
        };
        let request = control_request("cancel");
        let source_order = partial_fill_order_state();
        let report = partial_fill_report();
        validate_execution_order_report(&context, &request, &source_order, &report).unwrap();

        let mut wrong_quantity = partial_fill_report();
        wrong_quantity.quantity = Quantity::from("0.02000000");
        assert!(
            validate_execution_order_report(&context, &request, &source_order, &wrong_quantity)
                .is_err()
        );

        let mut wrong_venue_order = partial_fill_report();
        wrong_venue_order.venue_order_id = VenueOrderId::from("1002");
        assert!(
            validate_execution_order_report(&context, &request, &source_order, &wrong_venue_order)
                .is_err()
        );
    }

    #[test]
    fn interrupted_cancel_is_single_use_and_requires_manual_review() {
        let request = control_request("cancel");
        let result = interrupted_execution_control_result(&request, "sha256:request");
        assert_eq!(result.status, "unknown_manual_review");
        assert_eq!(
            result.error_code.as_deref(),
            Some("previous_attempt_interrupted_no_retry")
        );
        assert!(result.query_attempted);
        assert!(result.cancel_attempted);
        assert!(!result.cancel_confirmed);
        assert!(!result.automatic_retry_attempted);
        assert!(result.manual_review_required);
    }

    #[test]
    fn tampered_runtime_config_is_rejected_before_execution_client_registration() {
        let temp = tempdir().unwrap();
        let candidate = temp.path().join("candidate");
        let output = temp.path().join("runtime");
        fs::create_dir_all(&candidate).unwrap();
        fs::create_dir_all(&output).unwrap();
        let config_path = candidate.join("live-market-data-node.toml");
        fs::write(&config_path, b"original-config").unwrap();
        let config_sha = execution_sha256_ref(b"original-config");
        let state = serde_json::json!({
            "schema_version": EXECUTION_CONTROL_STATE_SCHEMA_VERSION,
            "run_id": "live-candidate-authority-test",
            "source_manifest_sha256": format!("sha256:{}", "1".repeat(64)),
            "revision": 3,
            "previous_state_sha256": format!("sha256:{}", "2".repeat(64)),
            "lifecycle": "starting",
            "preflight_sha256": format!("sha256:{}", "3".repeat(64)),
            "execution_admission_sha256": format!("sha256:{}", "4".repeat(64)),
            "execution_runtime_config_sha256": config_sha,
            "stop_sha256": null,
            "updated_at_unix_ms": 1
        });
        let state_raw = serde_json::to_vec(&state).unwrap();
        fs::write(
            candidate.join("state-00000000000000000003.json"),
            &state_raw,
        )
        .unwrap();
        fs::write(
            candidate.join("anchor-receipt-00000000000000000003.json"),
            b"{}",
        )
        .unwrap();
        fs::write(
            candidate.join("state-head.json"),
            serde_json::to_vec(&serde_json::json!({
                "schema_version": EXECUTION_STATE_HEAD_SCHEMA_VERSION,
                "run_id": "live-candidate-authority-test",
                "revision": 3,
                "state_sha256": execution_sha256_ref(&state_raw),
                "commit_sha256": format!("sha256:{}", "5".repeat(64)),
                "anchor_receipt_sha256": format!("sha256:{}", "6".repeat(64)),
                "updated_at_unix_ms": 1
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(&config_path, b"tampered-config").unwrap();
        let execution = execution_section(&output, &candidate);
        let error = validate_execution_runtime_authority(
            &config_path,
            &output,
            "live-candidate-authority-test",
            &execution,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("does not match the anchored control plane"));
    }
}

fn requested_production_market_data_shutdown(
    stop_file: Option<&Path>,
    controls: NtproNodeRunControls,
    started_instant: Option<Instant>,
) -> Option<ShutdownReason> {
    if stop_file.is_some_and(Path::exists) {
        Some(ShutdownReason::StopFile)
    } else if controls
        .parent_pid
        .is_some_and(|pid| !process_is_alive(pid))
    {
        Some(ShutdownReason::ParentExited)
    } else if controls.max_runtime.is_some_and(|max_runtime| {
        started_instant.is_some_and(|started| started.elapsed() >= max_runtime)
    }) {
        Some(ShutdownReason::MaxRuntime)
    } else if stop_file.is_none() {
        Some(ShutdownReason::StartStop)
    } else {
        None
    }
}

fn write_production_market_data_heartbeat(
    context: &ProductionMarketDataRuntimeContext<'_>,
    started_at: &str,
    started_instant: Instant,
    connected: bool,
) -> anyhow::Result<()> {
    let connection = if connected {
        ConnectionStatus::Connected
    } else {
        ConnectionStatus::Disconnected
    };
    let status =
        production_market_data_status(context, NodeState::Running, connection, started_at, None);
    write_status(context.status_path, &status)?;
    write_production_market_data_metrics(
        context,
        &status,
        NodeMetricCounts {
            uptime_ms: Some(millis_to_u64(started_instant.elapsed().as_millis())),
            starts_total: 1,
            stops_total: 0,
            state_transitions_total: 1,
        },
    )
}

fn production_market_data_status(
    context: &ProductionMarketDataRuntimeContext<'_>,
    state: NodeState,
    data_connection: ConnectionStatus,
    started_at: &str,
    stopped_at: Option<&str>,
) -> NodeStatus {
    let mut status = NodeStatus::from_node_state(context.run_id, state);
    let generated_at = now_millis();
    status.process_mode = context.process_mode;
    status.config_path = SnapshotValue::available(context.config_path.display().to_string());
    status.artifact_root = SnapshotValue::available(context.output_dir.display().to_string());
    status.previous_lifecycle_state = match state {
        NodeState::Stopped => LifecycleStatus::Running,
        _ => LifecycleStatus::Starting,
    };
    status.data_connection = data_connection;
    status.execution_connection = if context.execution_enabled {
        data_connection
    } else {
        ConnectionStatus::NotConfigured
    };
    status.execution = ExecutionStatus {
        gateway_id: if context.execution_enabled {
            SnapshotValue::available("BINANCE".to_string())
        } else {
            SnapshotValue::not_configured()
        },
        connection: status.execution_connection,
        started: SnapshotValue::available(context.execution_enabled && state == NodeState::Running),
        account_ref: if context.execution_enabled {
            SnapshotValue::available("env://NTPRO_BINANCE_LIVE_API_KEY".to_string())
        } else {
            SnapshotValue::not_configured()
        },
        orders_open: SnapshotValue::available(0),
        orders_inflight: SnapshotValue::available(0),
        orders_closed: SnapshotValue::available(0),
        last_report_at: SnapshotValue::not_configured(),
        last_reconciliation_at: SnapshotValue::not_configured(),
        last_error: None,
    };
    status.risk.trading_state = nautilus_live::status::RiskTradingState::Halted;
    status.generated_at = SnapshotValue::available(generated_at.clone());
    status.started_at = SnapshotValue::available(started_at.to_string());
    status.stopped_at = stopped_at.map_or_else(SnapshotValue::unknown, |value| {
        SnapshotValue::available(value.to_string())
    });
    status.last_transition_at = SnapshotValue::available(generated_at);
    status.external_venue_connection = data_connection == ConnectionStatus::Connected;
    status.real_orders_submitted = context.execution_enabled
        && execution_order_state(context.output_dir).is_some_and(|state| {
            matches!(
                state.status.as_str(),
                "submitted"
                    | "accepted"
                    | "rejected"
                    | "expired"
                    | "partially_filled"
                    | "filled"
                    | "canceled"
            )
        });
    status
}

#[derive(Deserialize)]
struct RuntimeExecutionOrderStatus {
    status: String,
}

fn execution_order_state(output_dir: &Path) -> Option<RuntimeExecutionOrderStatus> {
    let path = output_dir.join("execution-order-state.json");
    let metadata = fs::symlink_metadata(&path).ok()?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 64 * 1024 {
        return None;
    }
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

fn write_production_market_data_metrics(
    context: &ProductionMarketDataRuntimeContext<'_>,
    status: &NodeStatus,
    counts: NodeMetricCounts,
) -> anyhow::Result<()> {
    let artifacts = NodeMetricArtifacts {
        status_path: context.status_path.to_path_buf(),
        stdout_log_path: context.stdout_log_path.to_path_buf(),
        stderr_log_path: context.stderr_log_path.to_path_buf(),
        events_log_path: context.events_log_path.to_path_buf(),
        kill_switch_approval_artifact_path: context
            .output_dir
            .join("kill-switch-not-configured.json"),
    };
    write_node_metrics_artifact(
        context.metrics_path,
        &NodeMetrics::from_status(status, &artifacts, counts),
    )
}

pub(super) async fn run_live_run_with_command(
    opt: &LiveRunOpt,
    command_name: &str,
    process_mode: ProcessMode,
    stop_file: Option<&Path>,
    shutdown_controls: NtproNodeRunControls,
) -> anyhow::Result<()> {
    let config = load_minimal_live_config(&opt.config)?;
    let run_id = opt.run_id.as_deref().unwrap_or(config.run.id.as_str());
    validate_non_empty("run_id", run_id)?;

    let output_dir = resolve_output_dir(run_id, opt.output.as_ref(), config.output.as_ref());
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create output dir '{}'", output_dir.display()))?;

    let summary_path = output_dir.join("summary.txt");
    let legacy_events_path = output_dir.join("events.log");
    let events_path = output_dir.join("logs").join("events.log");
    let status_path = output_dir.join("status.json");
    let metrics_path = output_dir.join("metrics.json");
    let stdout_log_path = output_dir.join("logs").join("stdout.log");
    let stderr_log_path = output_dir.join("logs").join("stderr.log");
    if let Some(parent) = events_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log dir '{}'", parent.display()))?;
    }

    let context = LiveRunContext {
        config: &config,
        config_path: &opt.config,
        run_id,
        output_dir: &output_dir,
        process_mode,
        status_path: &status_path,
        metrics_path: &metrics_path,
        stdout_log_path: &stdout_log_path,
        stderr_log_path: &stderr_log_path,
        events_log_path: &events_path,
        stop_file,
        shutdown_controls,
    };
    let smoke = run_live_init_smoke(&context).await?;
    let status = build_node_status(&context, &smoke);
    write_metrics(
        &metrics_path,
        &status,
        &context,
        NodeMetricCounts {
            uptime_ms: Some(smoke.uptime_ms),
            starts_total: 1,
            stops_total: 1,
            state_transitions_total: 2,
        },
    )?;

    let summary = format!(
        "command={command_name}\nstatus=ok\nmode={}\nrun_id={run_id}\nconfig={}\nenvironment={}\nnode_name={}\nprocess_mode={}\nadapter={}\naccount_id={}\nvenue={}\npre_start_state={}\nrunning_state={}\nfinal_state={}\naccount_cached={}\nstatus_artifact={}\nmetrics_artifact={}\nevents_log={}\nexternal_venue_connection=false\nreal_orders_submitted=false\nruntime_status=completed\nshutdown_reason={}\n",
        config.run.mode,
        opt.config.display(),
        config.run.environment,
        node_name(&config),
        process_mode_label(process_mode),
        config.adapter.kind,
        config.adapter.account_id,
        config.adapter.venue,
        smoke.pre_start_state,
        smoke.running_state,
        smoke.final_state,
        smoke.account_cached,
        status_path.display(),
        metrics_path.display(),
        events_path.display(),
        smoke.shutdown_reason.label(),
    );
    atomic_write_text(&summary_path, &summary)
        .with_context(|| format!("failed to write summary '{}'", summary_path.display()))?;

    let status_json = serde_json::to_string_pretty(&status)?;
    atomic_write_text(&status_path, &format!("{status_json}\n"))
        .with_context(|| format!("failed to write status '{}'", status_path.display()))?;

    let event_log = format!(
        "phase=validate_config status=ok\n\
         phase=build_node status=ok node_name={}\n\
         phase=register_adapter status=ok adapter={} venue={}\n\
         phase=start status=ok state={} account_cached={}\n\
         phase=shutdown_trigger status=ok reason={}\n\
         phase=stop status=ok state={} external_venue_connection=false real_orders_submitted=false\n",
        node_name(&config),
        config.adapter.kind,
        config.adapter.venue,
        smoke.running_state,
        smoke.account_cached,
        smoke.shutdown_reason.label(),
        smoke.final_state,
    );
    atomic_write_text(&events_path, &event_log)
        .with_context(|| format!("failed to write events '{}'", events_path.display()))?;
    atomic_write_text(&legacy_events_path, &event_log).with_context(|| {
        format!(
            "failed to write legacy events '{}'",
            legacy_events_path.display()
        )
    })?;

    println!(
        "{command_name} status=ok mode={} run_id={} config={} output={} summary={} events={} status_artifact={} metrics_artifact={} node_name={} adapter={} final_state={} external_venue_connection=false real_orders_submitted=false runtime_status=completed",
        config.run.mode,
        run_id,
        opt.config.display(),
        output_dir.display(),
        summary_path.display(),
        events_path.display(),
        status_path.display(),
        metrics_path.display(),
        node_name(&config),
        config.adapter.kind,
        smoke.final_state,
    );

    Ok(())
}

pub(super) async fn run_strategy_session_node_with_command(
    opt: &LiveRunOpt,
    command_name: &str,
    process_mode: ProcessMode,
    stop_file: Option<&Path>,
    shutdown_controls: NtproNodeRunControls,
) -> anyhow::Result<()> {
    let config = load_strategy_node_config(&opt.config)?;
    let run_id = opt
        .run_id
        .as_deref()
        .unwrap_or(config.node.node_id.as_str());
    validate_non_empty("run_id", run_id)?;

    let output_dir = resolve_output_dir(run_id, opt.output.as_ref(), config.output.as_ref());
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("failed to create output dir '{}'", output_dir.display()))?;

    let summary_path = output_dir.join("summary.txt");
    let legacy_events_path = output_dir.join("events.log");
    let events_path = output_dir.join("logs").join("events.log");
    let status_path = output_dir.join("status.json");
    let metrics_path = output_dir.join("metrics.json");
    let stdout_log_path = output_dir.join("logs").join("stdout.log");
    let stderr_log_path = output_dir.join("logs").join("stderr.log");
    if let Some(parent) = events_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create log dir '{}'", parent.display()))?;
    }

    let started_at = now_millis();
    let started_instant = Instant::now();
    let symbol = config
        .market
        .symbols
        .first()
        .context("market.symbols must not be empty")?;
    let mut session = StrategySession::new(run_id, &config.strategy.strategy_id, &output_dir)?;
    session.set_risk_controls(StrategyRiskControls {
        kill_switch_enabled: config.risk.kill_switch_enabled,
        kill_switch_active: config.risk.kill_switch_active,
    });
    let bars = ema_cross_demo_fixture_bars(symbol);
    let runtime = session.run_ema_cross_demo(&bars)?;
    let counters = runtime.counters;

    let shutdown_reason = wait_for_strategy_shutdown_trigger(
        stop_file,
        shutdown_controls,
        &status_path,
        &metrics_path,
        &stdout_log_path,
        &stderr_log_path,
        &events_path,
        &opt.config,
        &output_dir,
        run_id,
        process_mode,
        &started_at,
        started_instant,
        counters,
    )
    .await?;
    session.stop_after_shutdown(shutdown_reason.label())?;

    let stopped_at = now_millis();
    let uptime_ms = millis_to_u64(started_instant.elapsed().as_millis());
    let status = build_strategy_node_status(
        &StrategyNodeStatusContext {
            config_path: &opt.config,
            output_dir: &output_dir,
            run_id,
            process_mode,
            started_at: &started_at,
            stopped_at: Some(&stopped_at),
            counters,
        },
        NodeState::Stopped,
    );
    atomic_write_json(&status_path, &status)
        .with_context(|| format!("failed to write status '{}'", status_path.display()))?;
    write_strategy_node_metrics(
        &metrics_path,
        &status,
        &StrategyNodeMetricPaths {
            output_dir: &output_dir,
            status_path: &status_path,
            stdout_log_path: &stdout_log_path,
            stderr_log_path: &stderr_log_path,
            events_log_path: &events_path,
        },
        NodeMetricCounts {
            uptime_ms: Some(uptime_ms),
            starts_total: 1,
            stops_total: 1,
            state_transitions_total: 2,
        },
    )?;

    let strategy_summary_path = runtime.summary_artifact.clone();
    let summary = format!(
        "command={command_name}\nstatus=ok\nmode={}\nrun_id={run_id}\nconfig={}\nenvironment=sandbox\nprocess_mode={}\nstrategy_id={}\nstrategy_runtime={}\nmarket_source={}\nmarket_symbol={symbol}\nprocessed_events={}\nmarket_event_count={}\nsignal_count={}\norder_intent_count={}\nrisk_decision_count={}\nrejection_count={}\nactual_submission_count={}\norder_submission_allowed=false\nstatus_artifact={}\nmetrics_artifact={}\nevents_log={}\nsession_status_artifact={}\nsignal_artifact={}\norder_intent_artifact={}\nrisk_decision_artifact={}\nstrategy_summary_artifact={}\nfinal_state=Stopped\nexternal_venue_connection=false\nreal_orders_submitted=false\nruntime_status=completed\nshutdown_reason={}\n",
        config.node.mode,
        opt.config.display(),
        process_mode_label(process_mode),
        config.strategy.strategy_id,
        config
            .strategy
            .strategy_runtime
            .as_deref()
            .unwrap_or(EMA_CROSS_DEMO_STRATEGY),
        config
            .market
            .data_mode
            .as_deref()
            .unwrap_or(FIXTURE_STREAM_DATA_MODE),
        runtime.processed_events,
        counters.market_event_count,
        runtime.signal_count,
        runtime.order_intent_count,
        runtime.risk_decision_count,
        counters.rejection_count,
        counters.actual_submission_count,
        status_path.display(),
        metrics_path.display(),
        events_path.display(),
        session.status().artifacts.session_status,
        runtime.signal_artifact,
        runtime.order_intent_artifact,
        runtime.risk_decision_artifact,
        strategy_summary_path,
        shutdown_reason.label(),
    );
    atomic_write_text(&summary_path, &summary)
        .with_context(|| format!("failed to write summary '{}'", summary_path.display()))?;

    let event_log = format!(
        "phase=validate_config status=ok mode={} strategy_id={}\n\
         phase=strategy_session_start status=ok session_id={run_id} strategy_id={}\n\
         phase=fixture_market_stream status=ok symbol={symbol} processed_events={}\n\
         phase=strategy_loop status=ok signal_count={} order_intent_count={} risk_decision_count={} rejection_count={} actual_submission_count={}\n\
         phase=shutdown_trigger status=ok reason={}\n\
         phase=strategy_session_stop status=ok state=stopped external_venue_connection=false real_orders_submitted=false\n",
        config.node.mode,
        config.strategy.strategy_id,
        config.strategy.strategy_id,
        runtime.processed_events,
        runtime.signal_count,
        runtime.order_intent_count,
        runtime.risk_decision_count,
        counters.rejection_count,
        counters.actual_submission_count,
        shutdown_reason.label(),
    );
    atomic_write_text(&events_path, &event_log)
        .with_context(|| format!("failed to write events '{}'", events_path.display()))?;
    atomic_write_text(&legacy_events_path, &event_log).with_context(|| {
        format!(
            "failed to write legacy events '{}'",
            legacy_events_path.display()
        )
    })?;

    println!(
        "{command_name} status=ok mode={} run_id={} config={} output={} summary={} events={} status_artifact={} metrics_artifact={} strategy_id={} final_state=Stopped external_venue_connection=false real_orders_submitted=false runtime_status=completed",
        config.node.mode,
        run_id,
        opt.config.display(),
        output_dir.display(),
        summary_path.display(),
        events_path.display(),
        status_path.display(),
        metrics_path.display(),
        config.strategy.strategy_id,
    );

    Ok(())
}

#[derive(Debug)]
struct LiveSmokeResult {
    pre_start_state: String,
    running_state: String,
    final_state: String,
    final_node_state: NodeState,
    account_cached: bool,
    started_at: String,
    stopped_at: String,
    uptime_ms: u64,
    shutdown_reason: ShutdownReason,
}

#[derive(Clone, Copy)]
struct LiveRunContext<'a> {
    config: &'a MinimalLiveConfig,
    config_path: &'a Path,
    run_id: &'a str,
    output_dir: &'a Path,
    process_mode: ProcessMode,
    status_path: &'a Path,
    metrics_path: &'a Path,
    stdout_log_path: &'a Path,
    stderr_log_path: &'a Path,
    events_log_path: &'a Path,
    stop_file: Option<&'a Path>,
    shutdown_controls: NtproNodeRunControls,
}

struct StrategyNodeStatusContext<'a> {
    config_path: &'a Path,
    output_dir: &'a Path,
    run_id: &'a str,
    process_mode: ProcessMode,
    started_at: &'a str,
    stopped_at: Option<&'a str>,
    counters: StrategyRuntimeCounters,
}

struct StrategyNodeMetricPaths<'a> {
    output_dir: &'a Path,
    status_path: &'a Path,
    stdout_log_path: &'a Path,
    stderr_log_path: &'a Path,
    events_log_path: &'a Path,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShutdownReason {
    StartStop,
    StopFile,
    Signal,
    MaxRuntime,
    ParentExited,
}

impl ShutdownReason {
    const fn label(self) -> &'static str {
        match self {
            Self::StartStop => "start-stop",
            Self::StopFile => "stop-file",
            Self::Signal => "signal",
            Self::MaxRuntime => "max-runtime",
            Self::ParentExited => "parent-exited",
        }
    }
}

async fn run_live_init_smoke(context: &LiveRunContext<'_>) -> anyhow::Result<LiveSmokeResult> {
    let config = context.config;
    let trader_id = TraderId::from(config.system.trader_id.as_str());
    let account_id = AccountId::from(config.adapter.account_id.as_str());
    let venue = Venue::from(config.adapter.venue.as_str());
    let sandbox_config = SandboxExecutionClientConfig {
        trader_id,
        account_id,
        venue,
        starting_balances: config
            .adapter
            .starting_balances
            .iter()
            .map(|balance| Money::from(balance.as_str()))
            .collect(),
        ..Default::default()
    };

    let mut node = LiveNode::builder(trader_id, Environment::Sandbox)?
        .with_name(node_name(config))
        .with_reconciliation(false)
        .with_load_state(config.system.load_state.unwrap_or(false))
        .with_save_state(config.system.save_state.unwrap_or(false))
        .with_timeout_connection(config.shutdown.connection_timeout_secs)
        .with_timeout_disconnection_secs(config.shutdown.disconnection_timeout_secs)
        .with_delay_post_stop_secs(config.shutdown.post_stop_delay_secs)
        .add_simulated_exec_client(
            Some(config.adapter.name.clone()),
            Box::new(SandboxExecutionClientFactory::new()),
            Box::new(sandbox_config),
        )?
        .build()?;
    let handle = node.handle();

    if node.environment() != Environment::Sandbox {
        anyhow::bail!("live-init-smoke must run in sandbox environment");
    }
    if handle.state() != NodeState::Idle {
        anyhow::bail!("live-init-smoke expected Idle before start");
    }
    let pre_start_state = format!("{:?}", handle.state());

    node.start().await?;
    let started_at = now_millis();
    let started_instant = Instant::now();
    let running_state = format!("{:?}", handle.state());
    let account_cached = node
        .kernel()
        .cache
        .borrow()
        .account_owned(&account_id)
        .is_some();
    if handle.state() != NodeState::Running {
        anyhow::bail!("live-init-smoke expected Running after start");
    }
    if !account_cached {
        anyhow::bail!("live-init-smoke expected sandbox account to be cached");
    }

    let shutdown_reason = wait_for_shutdown_trigger(context, &started_at, started_instant).await?;

    timeout(context.shutdown_controls.shutdown_timeout, node.stop())
        .await
        .with_context(|| {
            format!(
                "ntpro-node shutdown timed out after {} ms",
                millis_to_u64(context.shutdown_controls.shutdown_timeout.as_millis())
            )
        })??;
    let stopped_at = now_millis();
    let uptime_ms = millis_to_u64(started_instant.elapsed().as_millis());
    let final_state = format!("{:?}", handle.state());
    if handle.state() != NodeState::Stopped {
        anyhow::bail!("live-init-smoke expected Stopped after stop");
    }
    let final_node_state = handle.state();

    Ok(LiveSmokeResult {
        pre_start_state,
        running_state,
        final_state,
        final_node_state,
        account_cached,
        started_at,
        stopped_at,
        uptime_ms,
        shutdown_reason,
    })
}

fn build_node_status(context: &LiveRunContext<'_>, smoke: &LiveSmokeResult) -> NodeStatus {
    build_node_status_for_state(
        context,
        smoke.final_node_state,
        LifecycleStatus::Running,
        ConnectionStatus::Disconnected,
        false,
        Some(&smoke.started_at),
        Some(&smoke.stopped_at),
    )
}

fn build_strategy_node_status(
    context: &StrategyNodeStatusContext<'_>,
    state: NodeState,
) -> NodeStatus {
    let mut status = NodeStatus::from_node_state(context.run_id, state);
    let generated_at = now_millis();
    status.process_mode = context.process_mode;
    status.config_path = SnapshotValue::available(context.config_path.display().to_string());
    status.artifact_root = SnapshotValue::available(context.output_dir.display().to_string());
    status.previous_lifecycle_state = LifecycleStatus::Running;
    status.data_connection = match state {
        NodeState::Running => ConnectionStatus::Connected,
        NodeState::Stopped => ConnectionStatus::Disconnected,
        _ => ConnectionStatus::NotConfigured,
    };
    status.execution_connection = ConnectionStatus::NotConfigured;
    status.execution = ExecutionStatus {
        gateway_id: SnapshotValue::not_configured(),
        connection: ConnectionStatus::NotConfigured,
        started: SnapshotValue::available(false),
        account_ref: SnapshotValue::not_configured(),
        orders_open: SnapshotValue::available(0),
        orders_inflight: SnapshotValue::available(0),
        orders_closed: SnapshotValue::available(0),
        last_report_at: SnapshotValue::not_configured(),
        last_reconciliation_at: SnapshotValue::not_configured(),
        last_error: None,
    };
    status.risk.trading_state = nautilus_live::status::RiskTradingState::Halted;
    status.risk.health = nautilus_live::status::HealthStatus::Healthy;
    status.risk.command_count = SnapshotValue::available(context.counters.signal_count);
    status.risk.event_count = SnapshotValue::available(context.counters.risk_decision_count);
    status.risk.rejections_total = SnapshotValue::available(context.counters.rejection_count);
    if context.counters.rejection_count > 0 {
        status.risk.last_rejection = Some("order_submission_disabled".to_string());
    }
    status.generated_at = SnapshotValue::available(generated_at.clone());
    status.started_at = SnapshotValue::available(context.started_at.to_string());
    status.stopped_at = context
        .stopped_at
        .map_or_else(SnapshotValue::unknown, |value| {
            SnapshotValue::available(value.to_string())
        });
    status.last_transition_at = SnapshotValue::available(generated_at);
    status.external_venue_connection = false;
    status.real_orders_submitted = false;
    status
}

fn build_node_status_for_state(
    context: &LiveRunContext<'_>,
    state: NodeState,
    previous_lifecycle_state: LifecycleStatus,
    execution_connection: ConnectionStatus,
    execution_started: bool,
    started_at: Option<&str>,
    stopped_at: Option<&str>,
) -> NodeStatus {
    let config = context.config;
    let mut status = NodeStatus::from_node_state(context.run_id, state);
    let generated_at = now_millis();
    status.process_mode = context.process_mode;
    status.config_path = SnapshotValue::available(context.config_path.display().to_string());
    status.artifact_root = SnapshotValue::available(context.output_dir.display().to_string());
    status.previous_lifecycle_state = previous_lifecycle_state;
    status.data_connection = ConnectionStatus::NotConfigured;
    status.execution_connection = execution_connection;
    status.execution = ExecutionStatus {
        gateway_id: SnapshotValue::available(config.adapter.name.clone()),
        connection: execution_connection,
        started: SnapshotValue::available(execution_started),
        account_ref: SnapshotValue::available("configured".to_string()),
        orders_open: SnapshotValue::unknown(),
        orders_inflight: SnapshotValue::unknown(),
        orders_closed: SnapshotValue::unknown(),
        last_report_at: SnapshotValue::unknown(),
        last_reconciliation_at: SnapshotValue::unknown(),
        last_error: None,
    };
    status.generated_at = SnapshotValue::available(generated_at.clone());
    status.started_at = started_at.map_or_else(SnapshotValue::unknown, |value| {
        SnapshotValue::available(value.to_string())
    });
    status.stopped_at = stopped_at.map_or_else(SnapshotValue::unknown, |value| {
        SnapshotValue::available(value.to_string())
    });
    status.last_transition_at = SnapshotValue::available(generated_at);
    status
}

fn write_status(path: &Path, status: &NodeStatus) -> anyhow::Result<()> {
    let status_json = serde_json::to_string_pretty(status)?;
    atomic_write_text(path, &format!("{status_json}\n"))
        .with_context(|| format!("failed to write status '{}'", path.display()))?;
    Ok(())
}

fn write_metrics(
    path: &Path,
    status: &NodeStatus,
    context: &LiveRunContext<'_>,
    counts: NodeMetricCounts,
) -> anyhow::Result<()> {
    let artifacts = NodeMetricArtifacts {
        status_path: context.status_path.to_path_buf(),
        stdout_log_path: context.stdout_log_path.to_path_buf(),
        stderr_log_path: context.stderr_log_path.to_path_buf(),
        events_log_path: context.events_log_path.to_path_buf(),
        kill_switch_approval_artifact_path: context
            .output_dir
            .join("v0_13")
            .join("kill_switch_approval_artifact.json"),
    };
    let metrics = NodeMetrics::from_status(status, &artifacts, counts);
    write_node_metrics_artifact(path, &metrics)
}

fn write_strategy_node_metrics(
    path: &Path,
    status: &NodeStatus,
    paths: &StrategyNodeMetricPaths<'_>,
    counts: NodeMetricCounts,
) -> anyhow::Result<()> {
    let artifacts = NodeMetricArtifacts {
        status_path: paths.status_path.to_path_buf(),
        stdout_log_path: paths.stdout_log_path.to_path_buf(),
        stderr_log_path: paths.stderr_log_path.to_path_buf(),
        events_log_path: paths.events_log_path.to_path_buf(),
        kill_switch_approval_artifact_path: paths
            .output_dir
            .join("v0_13")
            .join("kill_switch_approval_artifact.json"),
    };
    let metrics = NodeMetrics::from_status(status, &artifacts, counts);
    write_node_metrics_artifact(path, &metrics)
}

async fn wait_for_shutdown_trigger(
    context: &LiveRunContext<'_>,
    started_at: &str,
    started_instant: Instant,
) -> anyhow::Result<ShutdownReason> {
    let Some(stop_file) = context.stop_file else {
        return Ok(ShutdownReason::StartStop);
    };
    let shutdown_signal = wait_for_shutdown_signal();
    tokio::pin!(shutdown_signal);
    let mut last_heartbeat: Option<Instant> = None;

    loop {
        if stop_file.exists() {
            return Ok(ShutdownReason::StopFile);
        }
        if let Some(parent_pid) = context.shutdown_controls.parent_pid
            && !process_is_alive(parent_pid)
        {
            return Ok(ShutdownReason::ParentExited);
        }
        if let Some(max_runtime) = context.shutdown_controls.max_runtime
            && started_instant.elapsed() >= max_runtime
        {
            return Ok(ShutdownReason::MaxRuntime);
        }
        if last_heartbeat
            .is_none_or(|last| last.elapsed() >= context.shutdown_controls.heartbeat_interval)
        {
            write_running_heartbeat(context, started_at, started_instant)?;
            last_heartbeat = Some(Instant::now());
        }

        tokio::select! {
            result = &mut shutdown_signal => return result,
            () = sleep(SHUTDOWN_POLL_INTERVAL) => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn wait_for_strategy_shutdown_trigger(
    stop_file: Option<&Path>,
    shutdown_controls: NtproNodeRunControls,
    status_path: &Path,
    metrics_path: &Path,
    stdout_log_path: &Path,
    stderr_log_path: &Path,
    events_log_path: &Path,
    config_path: &Path,
    output_dir: &Path,
    run_id: &str,
    process_mode: ProcessMode,
    started_at: &str,
    started_instant: Instant,
    counters: StrategyRuntimeCounters,
) -> anyhow::Result<ShutdownReason> {
    let Some(stop_file) = stop_file else {
        return Ok(ShutdownReason::StartStop);
    };
    let shutdown_signal = wait_for_shutdown_signal();
    tokio::pin!(shutdown_signal);
    let mut last_heartbeat: Option<Instant> = None;

    loop {
        if stop_file.exists() {
            return Ok(ShutdownReason::StopFile);
        }
        if let Some(parent_pid) = shutdown_controls.parent_pid
            && !process_is_alive(parent_pid)
        {
            return Ok(ShutdownReason::ParentExited);
        }
        if let Some(max_runtime) = shutdown_controls.max_runtime
            && started_instant.elapsed() >= max_runtime
        {
            return Ok(ShutdownReason::MaxRuntime);
        }
        if last_heartbeat.is_none_or(|last| last.elapsed() >= shutdown_controls.heartbeat_interval)
        {
            let status = build_strategy_node_status(
                &StrategyNodeStatusContext {
                    config_path,
                    output_dir,
                    run_id,
                    process_mode,
                    started_at,
                    stopped_at: None,
                    counters,
                },
                NodeState::Running,
            );
            atomic_write_json(status_path, &status)?;
            write_strategy_node_metrics(
                metrics_path,
                &status,
                &StrategyNodeMetricPaths {
                    output_dir,
                    status_path,
                    stdout_log_path,
                    stderr_log_path,
                    events_log_path,
                },
                NodeMetricCounts {
                    uptime_ms: Some(millis_to_u64(started_instant.elapsed().as_millis())),
                    starts_total: 1,
                    stops_total: 0,
                    state_transitions_total: 1,
                },
            )?;
            last_heartbeat = Some(Instant::now());
        }

        tokio::select! {
            result = &mut shutdown_signal => return result,
            () = sleep(SHUTDOWN_POLL_INTERVAL) => {}
        }
    }
}

fn write_running_heartbeat(
    context: &LiveRunContext<'_>,
    started_at: &str,
    started_instant: Instant,
) -> anyhow::Result<()> {
    let running_status = build_node_status_for_state(
        context,
        NodeState::Running,
        LifecycleStatus::Starting,
        ConnectionStatus::Disconnected,
        true,
        Some(started_at),
        None,
    );
    write_status(context.status_path, &running_status)?;
    write_metrics(
        context.metrics_path,
        &running_status,
        context,
        NodeMetricCounts {
            uptime_ms: Some(millis_to_u64(started_instant.elapsed().as_millis())),
            starts_total: 1,
            stops_total: 0,
            state_transitions_total: 1,
        },
    )
}

async fn wait_for_shutdown_signal() -> anyhow::Result<ShutdownReason> {
    #[cfg(unix)]
    {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .context("failed to register SIGTERM handler for ntpro-node shutdown")?;
        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                result.context("failed to wait for Ctrl-C shutdown signal")?;
                Ok(ShutdownReason::Signal)
            }
            _ = sigterm.recv() => Ok(ShutdownReason::Signal),
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .context("failed to wait for Ctrl-C shutdown signal")?;
        Ok(ShutdownReason::Signal)
    }
}
