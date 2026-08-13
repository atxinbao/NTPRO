//! 生产执行准入后的单次订单策略边界。

use super::node_runtime::{execution_sha256_ref, read_bounded_execution_authority_file};
use super::*;

const EXECUTION_STATE_SCHEMA_VERSION: &str = "ntpro.s3.live_execution_order_state.v2";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProductionExecutionOrderState {
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

#[derive(Debug)]
pub(super) struct ProductionSingleShotExecutionStrategy {
    core: StrategyCore,
    admission_id: String,
    strategy_version_id: String,
    instrument_id: InstrumentId,
    side: OrderSide,
    price: Price,
    quantity: Quantity,
    expires_at_unix_ms: u64,
    state_path: PathBuf,
    control_artifact_root: PathBuf,
    submitted: bool,
    client_order_id: Option<String>,
    venue_order_id: Option<String>,
    filled_quantity: Quantity,
}

impl ProductionSingleShotExecutionStrategy {
    pub(super) fn from_config(
        execution: &ProductionExecutionSection,
        output_dir: &Path,
    ) -> anyhow::Result<Self> {
        let instrument_id = InstrumentId::from_str(&execution.instrument_id)?;
        let side = match execution.side.as_str() {
            "BUY" => OrderSide::Buy,
            "SELL" => OrderSide::Sell,
            _ => anyhow::bail!("live_execution.side must be BUY or SELL"),
        };
        let price = Price::from_str(&execution.price).map_err(|error| {
            anyhow::anyhow!("live_execution.price must be a supported Price: {error}")
        })?;
        let quantity = Quantity::from_str(&execution.quantity).map_err(|error| {
            anyhow::anyhow!("live_execution.quantity must be a supported Quantity: {error}")
        })?;
        let strategy_id = StrategyId::from(format!("S3-LIVE-{}", execution.admission_id));
        let state_path = output_dir.join("execution-order-state.json");
        let persisted = load_existing_execution_state(&state_path, execution, &instrument_id)?;
        let submitted = persisted.is_some();
        let client_order_id = persisted
            .as_ref()
            .and_then(|state| state.client_order_id.clone());
        let venue_order_id = persisted
            .as_ref()
            .and_then(|state| state.venue_order_id.clone());
        let filled_quantity = persisted
            .as_ref()
            .map_or_else(
                || Ok(Quantity::zero(quantity.precision)),
                |state| Quantity::from_str(&state.filled_quantity),
            )
            .map_err(|error| anyhow::anyhow!("persisted filled quantity is invalid: {error}"))?;
        Ok(Self {
            core: StrategyCore::new(StrategyConfig {
                strategy_id: Some(strategy_id),
                order_id_tag: Some("S3LV007".to_string()),
                external_order_claims: Some(vec![instrument_id]),
                ..Default::default()
            }),
            admission_id: execution.admission_id.clone(),
            strategy_version_id: execution.strategy_version_id.clone(),
            instrument_id,
            side,
            price,
            quantity,
            expires_at_unix_ms: execution.expires_at_unix_ms,
            state_path,
            control_artifact_root: execution.control_artifact_root.clone(),
            submitted,
            client_order_id,
            venue_order_id,
            filled_quantity,
        })
    }

    fn submit_once(&mut self) -> anyhow::Result<()> {
        if self.submitted {
            return Ok(());
        }
        if current_unix_timestamp_millis() >= self.expires_at_unix_ms {
            self.submitted = true;
            self.write_state(
                "submission_failed",
                true,
                false,
                Some("execution admission expired before submission".to_string()),
            )?;
            return Ok(());
        }
        let client_order_id = deterministic_client_order_id(&self.admission_id);
        let order = self.core.order_factory().limit(
            self.instrument_id,
            self.side,
            self.quantity,
            self.price,
            Some(TimeInForce::Gtc),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(ClientOrderId::from(client_order_id.as_str())),
        );
        self.submitted = true;
        self.client_order_id = Some(client_order_id);
        // 此持久化边界之后的崩溃一律视为交易所结果未知，必须人工对账，不能宣称未发送。
        self.write_state("submission_requested", false, true, None)?;
        if let Err(error) = self.submit_order(order, None, Some(ClientId::from("BINANCE")), None) {
            self.write_state("submission_failed", true, true, Some(error.to_string()))?;
            return Err(error);
        }
        Ok(())
    }

    fn write_state(
        &self,
        status: &str,
        terminal: bool,
        actual_submission_attempted: bool,
        last_error: Option<String>,
    ) -> anyhow::Result<()> {
        let remaining_quantity = self.quantity.saturating_sub(self.filled_quantity);
        let cancel_attempted = self.cancel_request_exists()?;
        atomic_write_json(
            &self.state_path,
            &ProductionExecutionOrderState {
                schema_version: EXECUTION_STATE_SCHEMA_VERSION.to_string(),
                admission_id: self.admission_id.clone(),
                strategy_version_id: self.strategy_version_id.clone(),
                instrument_id: self.instrument_id.to_string(),
                client_order_id: self.client_order_id.clone(),
                venue_order_id: self.venue_order_id.clone(),
                original_quantity: self.quantity.to_string(),
                filled_quantity: self.filled_quantity.to_string(),
                remaining_quantity: remaining_quantity.to_string(),
                status: status.to_string(),
                terminal,
                new_orders_blocked: true,
                actual_submission_attempted,
                automatic_retry_attempted: false,
                cancel_attempted,
                replace_attempted: false,
                last_error,
                updated_at_unix_ms: current_unix_timestamp_millis(),
            },
        )
    }

    fn instrument_ready(&self) -> bool {
        self.cache().instrument(&self.instrument_id).is_some()
    }

    fn cancel_request_exists(&self) -> anyhow::Result<bool> {
        let attempt_path = self
            .control_artifact_root
            .join("execution-cancel-venue-attempt.json");
        if !attempt_path.exists() {
            return Ok(false);
        }
        let attempt = read_bounded_execution_authority_file(&attempt_path)?;
        let request = read_bounded_execution_authority_file(
            &self
                .control_artifact_root
                .join("execution-cancel-request.json"),
        )?;
        if attempt != execution_sha256_ref(&request).as_bytes() {
            anyhow::bail!("live execution cancel venue attempt does not match its request");
        }
        Ok(true)
    }
}

nautilus_strategy!(ProductionSingleShotExecutionStrategy, {
    fn external_order_claims(&self) -> Option<Vec<InstrumentId>> {
        Some(vec![self.instrument_id])
    }

    fn on_order_submitted(&mut self, event: OrderSubmitted) {
        self.client_order_id = Some(event.client_order_id.to_string());
        let _ = self.write_state("submitted", false, true, None);
    }

    fn on_order_accepted(&mut self, event: OrderAccepted) {
        self.client_order_id = Some(event.client_order_id.to_string());
        self.venue_order_id = Some(event.venue_order_id.to_string());
        let _ = self.write_state("accepted", false, true, None);
    }

    fn on_order_rejected(&mut self, event: OrderRejected) {
        self.client_order_id = Some(event.client_order_id.to_string());
        let _ = self.write_state("rejected", true, true, Some(event.reason.to_string()));
    }

    fn on_order_expired(&mut self, event: OrderExpired) {
        self.client_order_id = Some(event.client_order_id.to_string());
        let _ = self.write_state("expired", true, true, None);
    }

    fn on_order_denied(&mut self, event: OrderDenied) {
        self.client_order_id = Some(event.client_order_id.to_string());
        let _ = self.write_state("denied", true, false, Some(event.reason.to_string()));
    }
});

impl DataActor for ProductionSingleShotExecutionStrategy {
    fn on_start(&mut self) -> anyhow::Result<()> {
        Strategy::on_start(self)?;
        if self.submitted {
            return Ok(());
        }
        self.write_state("waiting_for_instrument", false, false, None)?;
        self.subscribe_instrument(self.instrument_id, Some(ClientId::from("BINANCE")), None);
        self.subscribe_quotes(self.instrument_id, Some(ClientId::from("BINANCE")), None);
        if self.instrument_ready() {
            self.submit_once()?;
        }
        Ok(())
    }

    fn on_instrument(&mut self, instrument: &InstrumentAny) -> anyhow::Result<()> {
        if instrument.id() == self.instrument_id {
            self.submit_once()?;
        }
        Ok(())
    }

    fn on_quote(&mut self, quote: &QuoteTick) -> anyhow::Result<()> {
        if quote.instrument_id == self.instrument_id && self.instrument_ready() {
            self.submit_once()?;
        }
        Ok(())
    }

    fn on_order_filled(&mut self, event: &OrderFilled) -> anyhow::Result<()> {
        self.client_order_id = Some(event.client_order_id.to_string());
        self.venue_order_id = Some(event.venue_order_id.to_string());
        let (terminal, filled_quantity) = self
            .cache()
            .order(&event.client_order_id)
            .map_or((false, self.filled_quantity), |order| {
                (order.is_closed(), order.filled_qty())
            });
        self.filled_quantity = filled_quantity;
        self.write_state(
            if terminal {
                "filled"
            } else {
                "partially_filled"
            },
            terminal,
            true,
            None,
        )
    }

    fn on_order_canceled(&mut self, event: &OrderCanceled) -> anyhow::Result<()> {
        self.client_order_id = Some(event.client_order_id.to_string());
        if let Some(venue_order_id) = event.venue_order_id {
            self.venue_order_id = Some(venue_order_id.to_string());
        }
        self.write_state("canceled", true, true, None)
    }

    fn on_stop(&mut self) -> anyhow::Result<()> {
        self.unsubscribe_quotes(self.instrument_id, Some(ClientId::from("BINANCE")), None);
        Ok(())
    }
}

fn deterministic_client_order_id(admission_id: &str) -> String {
    let digest = aws_lc_rs::digest::digest(&aws_lc_rs::digest::SHA256, admission_id.as_bytes());
    let mut encoded = String::from("S3LV007-");
    for byte in digest.as_ref().iter().take(12) {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

fn load_existing_execution_state(
    path: &Path,
    execution: &ProductionExecutionSection,
    instrument_id: &InstrumentId,
) -> anyhow::Result<Option<ProductionExecutionOrderState>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 64 * 1024 {
        anyhow::bail!("existing execution order state is not a bounded regular file");
    }
    let state: ProductionExecutionOrderState = serde_json::from_slice(&fs::read(path)?)
        .context("existing execution order state is invalid")?;
    let valid_client_order_id = state
        .client_order_id
        .as_deref()
        .is_none_or(|value| value == deterministic_client_order_id(&execution.admission_id));
    let quantities_valid = Decimal::from_str_exact(&state.original_quantity)
        .ok()
        .zip(Decimal::from_str_exact(&state.filled_quantity).ok())
        .zip(Decimal::from_str_exact(&state.remaining_quantity).ok())
        .is_some_and(|((original, filled), remaining)| {
            original > Decimal::ZERO
                && filled >= Decimal::ZERO
                && remaining >= Decimal::ZERO
                && filled + remaining == original
                && original == Decimal::from_str_exact(&execution.quantity).unwrap_or_default()
        });
    if state.schema_version != EXECUTION_STATE_SCHEMA_VERSION
        || state.admission_id != execution.admission_id
        || state.strategy_version_id != execution.strategy_version_id
        || state.instrument_id != instrument_id.to_string()
        || !state.new_orders_blocked
        || state.automatic_retry_attempted
        || state.replace_attempted
        || !valid_client_order_id
        || !matches!(
            state.status.as_str(),
            "submission_requested"
                | "submitted"
                | "accepted"
                | "rejected"
                | "denied"
                | "expired"
                | "partially_filled"
                | "filled"
                | "canceled"
                | "submission_failed"
        )
        || (state.cancel_attempted && state.client_order_id.is_none())
        || !quantities_valid
    {
        anyhow::bail!("existing execution order state does not match the admitted single shot");
    }
    Ok(Some(state))
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    fn execution_section() -> ProductionExecutionSection {
        ProductionExecutionSection {
            schema_version: "ntpro.s3.live_execution_node.v1".to_string(),
            source_manifest_sha256: format!("sha256:{}", "1".repeat(64)),
            execution_admission_sha256: format!("sha256:{}", "2".repeat(64)),
            runtime_artifact_root: PathBuf::from("/tmp/ntpro-s3-lv-007-runtime"),
            control_artifact_root: PathBuf::from("/tmp/ntpro-s3-lv-007-control"),
            risk_policy_ref: format!("risk-config-sha256:{}", "3".repeat(64)),
            owner_authority_ref: "role://institution-owner".to_string(),
            risk_authority_ref: "policy://risk/test-v1".to_string(),
            operator_authority_ref: "role://operations-operator".to_string(),
            admission_id: "admission-001".to_string(),
            strategy_version_id: "ema_cross_btcusdt_v1@v1".to_string(),
            account_id: "BINANCE-001".to_string(),
            instrument_id: "BTCUSDT.BINANCE".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            time_in_force: "GTC".to_string(),
            price: "1.00".to_string(),
            quantity: "0.00001000".to_string(),
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

    #[test]
    fn execution_strategy_starts_blocked_until_instrument_is_ready() {
        let temp = tempdir().unwrap();
        let strategy =
            ProductionSingleShotExecutionStrategy::from_config(&execution_section(), temp.path())
                .unwrap();
        assert!(!strategy.submitted);
        assert_eq!(strategy.instrument_id.to_string(), "BTCUSDT.BINANCE");
    }

    #[test]
    fn execution_strategy_rejects_unsupported_side() {
        let temp = tempdir().unwrap();
        let mut section = execution_section();
        section.side = "HOLD".to_string();
        let error = ProductionSingleShotExecutionStrategy::from_config(&section, temp.path())
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("BUY or SELL"));
    }

    #[test]
    fn execution_strategy_rejects_unsupported_fixed_precision() {
        let temp = tempdir().unwrap();
        let mut section = execution_section();
        section.price = "1.0000000000000000000000000001".to_string();
        let error = ProductionSingleShotExecutionStrategy::from_config(&section, temp.path())
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("supported Price"));
    }

    #[test]
    fn execution_strategy_fails_closed_when_admission_expires_before_submission() {
        let temp = tempdir().unwrap();
        let mut section = execution_section();
        section.expires_at_unix_ms = 0;
        let mut strategy =
            ProductionSingleShotExecutionStrategy::from_config(&section, temp.path()).unwrap();

        strategy.submit_once().unwrap();

        let state: serde_json::Value = serde_json::from_slice(
            &std::fs::read(temp.path().join("execution-order-state.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(state["status"], "submission_failed");
        assert_eq!(state["terminal"], true);
        assert_eq!(state["new_orders_blocked"], true);
        assert_eq!(state["actual_submission_attempted"], false);
        assert_eq!(state["client_order_id"], serde_json::Value::Null);
        assert!(strategy.submitted);
    }

    #[test]
    fn execution_strategy_restart_never_replays_persisted_submission_intent() {
        let temp = tempdir().unwrap();
        let section = execution_section();
        let expected_id = deterministic_client_order_id(&section.admission_id);
        atomic_write_json(
            &temp.path().join("execution-order-state.json"),
            &ProductionExecutionOrderState {
                schema_version: EXECUTION_STATE_SCHEMA_VERSION.to_string(),
                admission_id: section.admission_id.clone(),
                strategy_version_id: section.strategy_version_id.clone(),
                instrument_id: section.instrument_id.clone(),
                client_order_id: Some(expected_id.clone()),
                venue_order_id: None,
                original_quantity: section.quantity.clone(),
                filled_quantity: "0".to_string(),
                remaining_quantity: section.quantity.clone(),
                status: "submission_requested".to_string(),
                terminal: false,
                new_orders_blocked: true,
                actual_submission_attempted: true,
                automatic_retry_attempted: false,
                cancel_attempted: false,
                replace_attempted: false,
                last_error: None,
                updated_at_unix_ms: 1,
            },
        )
        .unwrap();

        let mut restarted =
            ProductionSingleShotExecutionStrategy::from_config(&section, temp.path()).unwrap();
        assert!(restarted.submitted);
        assert_eq!(
            restarted.client_order_id.as_deref(),
            Some(expected_id.as_str())
        );
        restarted.submit_once().unwrap();

        let state: ProductionExecutionOrderState = serde_json::from_slice(
            &std::fs::read(temp.path().join("execution-order-state.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(state.status, "submission_requested");
        assert!(state.actual_submission_attempted);
    }

    #[test]
    fn execution_strategy_rejects_tampered_persisted_client_order_id() {
        let temp = tempdir().unwrap();
        let section = execution_section();
        atomic_write_json(
            &temp.path().join("execution-order-state.json"),
            &ProductionExecutionOrderState {
                schema_version: EXECUTION_STATE_SCHEMA_VERSION.to_string(),
                admission_id: section.admission_id.clone(),
                strategy_version_id: section.strategy_version_id.clone(),
                instrument_id: section.instrument_id.clone(),
                client_order_id: Some("attacker-controlled-id".to_string()),
                venue_order_id: None,
                original_quantity: section.quantity.clone(),
                filled_quantity: "0".to_string(),
                remaining_quantity: section.quantity.clone(),
                status: "submitted".to_string(),
                terminal: false,
                new_orders_blocked: true,
                actual_submission_attempted: true,
                automatic_retry_attempted: false,
                cancel_attempted: false,
                replace_attempted: false,
                last_error: None,
                updated_at_unix_ms: 1,
            },
        )
        .unwrap();

        let error = ProductionSingleShotExecutionStrategy::from_config(&section, temp.path())
            .unwrap_err()
            .to_string();
        assert!(error.contains("does not match the admitted single shot"));
    }

    #[test]
    fn execution_strategy_marks_cancel_only_after_a_real_venue_attempt() {
        let temp = tempdir().unwrap();
        let mut section = execution_section();
        section.control_artifact_root = temp.path().join("control");
        fs::create_dir_all(&section.control_artifact_root).unwrap();
        let strategy =
            ProductionSingleShotExecutionStrategy::from_config(&section, temp.path()).unwrap();

        fs::write(
            section
                .control_artifact_root
                .join("execution-cancel-attempt.json"),
            b"approved control request",
        )
        .unwrap();
        fs::write(
            section
                .control_artifact_root
                .join("execution-cancel-request.json"),
            b"approved control request",
        )
        .unwrap();
        assert!(!strategy.cancel_request_exists().unwrap());

        fs::write(
            section
                .control_artifact_root
                .join("execution-cancel-venue-attempt.json"),
            execution_sha256_ref(b"approved control request"),
        )
        .unwrap();
        assert!(strategy.cancel_request_exists().unwrap());

        fs::write(
            section
                .control_artifact_root
                .join("execution-cancel-venue-attempt.json"),
            b"sha256:tampered",
        )
        .unwrap();
        assert!(strategy.cancel_request_exists().is_err());
    }

    #[test]
    fn canceled_event_without_venue_id_preserves_existing_exchange_identity() {
        let temp = tempdir().unwrap();
        let mut strategy =
            ProductionSingleShotExecutionStrategy::from_config(&execution_section(), temp.path())
                .unwrap();
        strategy.venue_order_id = Some("1001".to_string());
        let event = OrderCanceled {
            client_order_id: ClientOrderId::from("S3LV007-001"),
            venue_order_id: None,
            ..Default::default()
        };

        strategy.on_order_canceled(&event).unwrap();

        let state: ProductionExecutionOrderState = serde_json::from_slice(
            &fs::read(temp.path().join("execution-order-state.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(state.venue_order_id.as_deref(), Some("1001"));
    }
}
