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

//! 单节点 MVP 的身份与追溯合同。

use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, ensure};
use serde::{Deserialize, Serialize};

pub(crate) const MVP_IDENTITY_CONTRACT_SCHEMA_VERSION: &str = "ntpro.mvp_identity_contract.v1";
pub(crate) const MVP_IDENTITY_CONTRACT_PATH: &str = "mvp/identity_contract.json";
const SANDBOX_ENVIRONMENT: &str = "sandbox";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MvpIdentityContract {
    pub schema_version: String,
    pub contract_id: String,
    pub identities: MvpIdentitySet,
    pub provenance: MvpIdentityProvenance,
    pub boundaries: MvpIdentityBoundaries,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MvpIdentitySet {
    pub strategy_id: String,
    pub strategy_version: String,
    pub backtest_run_id: String,
    pub backtest_result_ref: String,
    pub node_id: String,
    pub strategy_instance_id: String,
    pub account_id: String,
    pub venue_id: String,
    pub environment: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MvpIdentityProvenance {
    pub config_path: String,
    pub generated_at_unix_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MvpIdentityBoundaries {
    pub read_only_product_contract: bool,
    pub external_venue_connection: bool,
    pub order_submission_allowed: bool,
    pub order_mutation_allowed: bool,
    pub automatic_retry_allowed: bool,
    pub automatic_remediation_allowed: bool,
    pub real_orders_submitted: bool,
}

#[derive(Debug, Deserialize)]
struct IdentityConfigProjection {
    node: IdentityNodeSection,
    strategy: IdentityStrategySection,
    market: IdentityVenueSection,
    execution: IdentityVenueSection,
    mvp: IdentityMvpSection,
}

#[derive(Debug, Deserialize)]
struct IdentityNodeSection {
    node_id: String,
}

#[derive(Debug, Deserialize)]
struct IdentityStrategySection {
    strategy_id: String,
}

#[derive(Debug, Deserialize)]
struct IdentityVenueSection {
    venue: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct IdentityMvpSection {
    strategy_version: String,
    backtest_run_id: String,
    backtest_result_ref: String,
    account_id: String,
    environment: String,
}

impl MvpIdentityContract {
    pub(crate) fn load(config_path: &Path, supervisor_node_id: &str) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(config_path)
            .with_context(|| format!("读取 MVP 身份配置 '{}' 失败", config_path.display()))?;
        let config: IdentityConfigProjection = toml::from_str(&raw).with_context(|| {
            format!(
                "解析 MVP 身份配置 '{}' 失败；mvp serve 要求显式 [mvp] 身份段",
                config_path.display()
            )
        })?;

        let node_id = required("node_id", supervisor_node_id)?;
        let strategy_instance_id = required("node.node_id", &config.node.node_id)?;
        ensure!(
            node_id != strategy_instance_id,
            "Supervisor node_id 与 strategy_instance_id 必须使用不同身份"
        );

        let market_venue = required("market.venue", &config.market.venue)?;
        let execution_venue = required("execution.venue", &config.execution.venue)?;
        ensure!(
            market_venue == execution_venue,
            "market.venue '{market_venue}' 与 execution.venue '{execution_venue}' 不一致"
        );

        let environment = required("mvp.environment", &config.mvp.environment)?;
        ensure!(
            environment == SANDBOX_ENVIRONMENT,
            "mvp.environment 必须为 sandbox，实际为 '{environment}'"
        );

        let identities = MvpIdentitySet {
            strategy_id: required("strategy.strategy_id", &config.strategy.strategy_id)?,
            strategy_version: required("mvp.strategy_version", &config.mvp.strategy_version)?,
            backtest_run_id: required("mvp.backtest_run_id", &config.mvp.backtest_run_id)?,
            backtest_result_ref: required(
                "mvp.backtest_result_ref",
                &config.mvp.backtest_result_ref,
            )?,
            node_id,
            strategy_instance_id,
            account_id: required("mvp.account_id", &config.mvp.account_id)?,
            venue_id: market_venue,
            environment,
        };
        let contract_id = format!(
            "{}:{}:{}",
            identities.node_id, identities.strategy_id, identities.strategy_instance_id
        );

        Ok(Self {
            schema_version: MVP_IDENTITY_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id,
            identities,
            provenance: MvpIdentityProvenance {
                config_path: config_path.display().to_string(),
                generated_at_unix_ms: unix_time_ms(),
            },
            boundaries: MvpIdentityBoundaries {
                read_only_product_contract: true,
                external_venue_connection: false,
                order_submission_allowed: false,
                order_mutation_allowed: false,
                automatic_retry_allowed: false,
                automatic_remediation_allowed: false,
                real_orders_submitted: false,
            },
        })
    }
}

fn required(field: &str, value: &str) -> anyhow::Result<String> {
    let value = value.trim();
    ensure!(!value.is_empty(), "{field} 不能为空");
    Ok(value.to_string())
}

fn unix_time_ms() -> u64 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    u64::try_from(millis).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;

    fn temp_config(name: &str, content: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("ntpro-mvp-contract-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("MVP contract test root should be created");
        let path = root.join("node.toml");
        fs::write(&path, content).expect("MVP contract test config should be written");
        path
    }

    fn valid_config() -> &'static str {
        r#"[node]
node_id = "strategy-instance-alpha"

[strategy]
strategy_id = "strategy-alpha"

[market]
venue = "SANDBOX"

[execution]
venue = "SANDBOX"

[mvp]
strategy_version = "v1"
backtest_run_id = "backtest-alpha-001"
backtest_result_ref = "artifact://backtests/backtest-alpha-001/summary.json"
account_id = "SANDBOX-001"
environment = "sandbox"
"#
    }

    #[test]
    fn mvp_contract_loads_eight_stable_identities_and_closed_boundaries() {
        let path = temp_config("valid", valid_config());
        let contract = MvpIdentityContract::load(&path, "mvp-node-001")
            .expect("valid MVP identity contract should load");

        assert_eq!(
            contract.schema_version,
            MVP_IDENTITY_CONTRACT_SCHEMA_VERSION
        );
        assert_eq!(contract.identities.strategy_id, "strategy-alpha");
        assert_eq!(contract.identities.strategy_version, "v1");
        assert_eq!(contract.identities.backtest_run_id, "backtest-alpha-001");
        assert_eq!(contract.identities.node_id, "mvp-node-001");
        assert_eq!(
            contract.identities.strategy_instance_id,
            "strategy-instance-alpha"
        );
        assert_eq!(contract.identities.account_id, "SANDBOX-001");
        assert_eq!(contract.identities.venue_id, "SANDBOX");
        assert_eq!(contract.identities.environment, "sandbox");
        assert!(contract.boundaries.read_only_product_contract);
        assert!(!contract.boundaries.external_venue_connection);
        assert!(!contract.boundaries.order_submission_allowed);
        assert!(!contract.boundaries.order_mutation_allowed);
        assert!(!contract.boundaries.automatic_retry_allowed);
        assert!(!contract.boundaries.automatic_remediation_allowed);
        assert!(!contract.boundaries.real_orders_submitted);
    }

    #[test]
    fn mvp_contract_rejects_missing_mvp_identity_section() {
        let config = valid_config()
            .split("[mvp]")
            .next()
            .expect("fixture prefix should exist");
        let path = temp_config("missing-mvp", config);
        let error = MvpIdentityContract::load(&path, "mvp-node-001")
            .expect_err("missing MVP identity section must fail closed");
        assert!(format!("{error:#}").contains("missing field `mvp`"));
    }

    #[test]
    fn mvp_contract_rejects_empty_identity() {
        let config =
            valid_config().replace("strategy_version = \"v1\"", "strategy_version = \" \"");
        let path = temp_config("empty", &config);
        let error = MvpIdentityContract::load(&path, "mvp-node-001")
            .expect_err("empty identity must fail closed");
        assert!(format!("{error:#}").contains("mvp.strategy_version 不能为空"));
    }

    #[test]
    fn mvp_contract_rejects_venue_mismatch() {
        let config = valid_config().replace(
            "[execution]\nvenue = \"SANDBOX\"",
            "[execution]\nvenue = \"OTHER\"",
        );
        let path = temp_config("venue-mismatch", &config);
        let error = MvpIdentityContract::load(&path, "mvp-node-001")
            .expect_err("venue mismatch must fail closed");
        assert!(
            format!("{error:#}")
                .contains("market.venue 'SANDBOX' 与 execution.venue 'OTHER' 不一致")
        );
    }

    #[test]
    fn mvp_contract_rejects_non_sandbox_environment() {
        let config =
            valid_config().replace("environment = \"sandbox\"", "environment = \"production\"");
        let path = temp_config("production", &config);
        let error = MvpIdentityContract::load(&path, "mvp-node-001")
            .expect_err("non-sandbox environment must fail closed");
        assert!(format!("{error:#}").contains("mvp.environment 必须为 sandbox"));
    }

    #[test]
    fn mvp_contract_rejects_node_and_strategy_instance_identity_collision() {
        let path = temp_config("identity-collision", valid_config());
        let error = MvpIdentityContract::load(&path, "strategy-instance-alpha")
            .expect_err("identity collision must fail closed");
        assert!(format!("{error:#}").contains("必须使用不同身份"));
    }
}
