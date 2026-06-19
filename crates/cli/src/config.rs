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

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::{
    backtest::validate_minimal_backtest_config_file,
    data::validate_data_catalog_config_file,
    live::{validate_minimal_live_config_file, validate_strategy_node_config_file},
    opt::{ConfigCommand, ConfigKind, ConfigOpt, ConfigValidateOpt},
    sandbox::validate_minimal_sandbox_config_file,
};

pub(crate) fn run_config_command(opt: ConfigOpt) -> anyhow::Result<()> {
    match opt.command {
        ConfigCommand::Validate(validate) => run_config_validate(&validate),
    }
}

fn run_config_validate(opt: &ConfigValidateOpt) -> anyhow::Result<()> {
    validate_workflow_config(opt.kind, &opt.config)?;

    if let Some(output_dir) = &opt.output {
        write_config_validation_artifact(opt.kind, &opt.config, output_dir)?;
    }

    println!(
        "config.validate status=ok kind={} config={}",
        config_kind_label(opt.kind),
        opt.config.display()
    );
    Ok(())
}

fn validate_workflow_config(kind: ConfigKind, path: &Path) -> anyhow::Result<()> {
    match kind {
        ConfigKind::Backtest => validate_minimal_backtest_config_file(path),
        ConfigKind::Sandbox => validate_minimal_sandbox_config_file(path),
        ConfigKind::Live => validate_minimal_live_config_file(path),
        ConfigKind::Data => validate_data_catalog_config_file(path),
        ConfigKind::StrategySession => validate_strategy_node_config_file(path),
    }
}

fn write_config_validation_artifact(
    kind: ConfigKind,
    config: &Path,
    output_dir: &PathBuf,
) -> anyhow::Result<()> {
    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create output dir '{}'", output_dir.display()))?;
    let path = output_dir.join("validation.txt");
    let summary = format!(
        "command=config.validate\nstatus=ok\nkind={}\nconfig={}\n",
        config_kind_label(kind),
        config.display()
    );
    fs::write(&path, summary)
        .with_context(|| format!("failed to write validation artifact '{}'", path.display()))?;
    Ok(())
}

fn config_kind_label(kind: ConfigKind) -> &'static str {
    match kind {
        ConfigKind::Backtest => "backtest",
        ConfigKind::Sandbox => "sandbox",
        ConfigKind::Live => "live",
        ConfigKind::Data => "data",
        ConfigKind::StrategySession => "strategy-session",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ntpro-gh-155-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_config(dir: &Path, content: &str) -> PathBuf {
        let path = dir.join("config.toml");
        fs::write(&path, content).unwrap();
        path
    }

    fn minimal_backtest_config(output_dir: &Path) -> String {
        format!(
            r#"[run]
id = "minimal-backtest-dry-run"
mode = "dry-run"

[data]
source = "synthetic-quotes"
instrument_id = "AUD/USD.SIM"
quotes = 3

[strategy]
name = "no-op"

[output]
dir = "{}"
"#,
            output_dir.display()
        )
    }

    fn minimal_live_config() -> &'static str {
        r#"[run]
id = "live-init-smoke"
mode = "live-init-smoke"
environment = "sandbox"

[system]
trader_id = "LIVE-INIT-001"
node_name = "LiveInitSmoke"

[adapter]
name = "SANDBOX"
kind = "sandbox-simulated-execution"
account_id = "SANDBOX-001"
venue = "SANDBOX"
starting_balances = ["100000 USDT"]

[execution]
order_submission = "disabled"
reconciliation = false
external_venue_connection = false

[shutdown]
mode = "start-stop"
post_stop_delay_secs = 0
connection_timeout_secs = 5
disconnection_timeout_secs = 5
"#
    }

    fn minimal_data_config() -> &'static str {
        r#"[run]
id = "catalog-audit"
mode = "inspect"

[catalog]
path = "catalog/backtests/ema-cross"
protocol = "file"

[[queries]]
data_type = "QuoteTick"
instrument_id = "AUD/USD.SIM"
"#
    }

    fn minimal_strategy_session_config() -> &'static str {
        r#"[node]
node_id = "btc-ema-shadow-001"
mode = "shadow"

[strategy]
strategy_id = "ema-cross-demo"
strategy_package = "builtin"
strategy_runtime = "ema_cross_demo"

[market]
venue = "BINANCE_TESTNET"
symbols = ["BTCUSDT.BINANCE"]
data_mode = "fixture_stream"

[execution]
venue = "BINANCE_TESTNET"
order_submission = "disabled"
external_venue_connection = false

[testnet_order]
enabled = false
mode = "disabled"
manual_gate = "owner-approved-manual"
http_base_url = "https://testnet.binance.vision"
symbol = "BTCUSDT"
instrument_id = "BTCUSDT.BINANCE"
side = "BUY"
order_type = "LIMIT"
time_in_force = "GTC"
price = "1.00"
quantity = "0.00001000"
notional = "0.00001000"
cancel_after_submit_ms = 3000
owner_approval_required = true
manual_env_gate_required = true
production_endpoint_allowed = false
dashboard_order_controls = false

[risk]
kill_switch_enabled = true
kill_switch_active = false
"#
    }

    #[test]
    fn config_validate_backtest_writes_artifact() {
        let dir = temp_dir("backtest");
        let output_dir = dir.join("validation");
        let config = write_config(&dir, &minimal_backtest_config(&dir.join("runs")));

        run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::Backtest,
                config: config.clone(),
                output: Some(output_dir.clone()),
            }),
        })
        .unwrap();

        let artifact = fs::read_to_string(output_dir.join("validation.txt")).unwrap();
        assert!(artifact.contains("command=config.validate"));
        assert!(artifact.contains("status=ok"));
        assert!(artifact.contains("kind=backtest"));
        assert!(artifact.contains(&format!("config={}", config.display())));
    }

    #[test]
    fn config_validate_live_accepts_smoke_config() {
        let dir = temp_dir("live");
        let config = write_config(&dir, minimal_live_config());

        run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::Live,
                config,
                output: None,
            }),
        })
        .unwrap();
    }

    #[test]
    fn config_validate_data_accepts_query_config() {
        let dir = temp_dir("data");
        let config = write_config(&dir, minimal_data_config());

        run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::Data,
                config,
                output: None,
            }),
        })
        .unwrap();
    }

    #[test]
    fn config_validate_strategy_session_accepts_shadow_config() {
        let dir = temp_dir("strategy-session");
        let config = write_config(&dir, minimal_strategy_session_config());

        run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::StrategySession,
                config,
                output: None,
            }),
        })
        .unwrap();
    }

    #[test]
    fn config_validate_strategy_session_rejects_missing_strategy_id() {
        let dir = temp_dir("strategy-session-missing-id");
        let config = write_config(
            &dir,
            r#"[node]
node_id = "btc-ema-shadow-001"
mode = "shadow"

[strategy]

[market]
symbols = ["BTCUSDT.BINANCE"]

[execution]
order_submission = "disabled"

[risk]
kill_switch_enabled = true
kill_switch_active = false
"#,
        );

        let error = run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::StrategySession,
                config,
                output: None,
            }),
        })
        .unwrap_err();
        let error = format!("{error:#}");

        assert!(error.contains("strategy.strategy_id must not be empty"));
    }

    #[test]
    fn config_validate_strategy_session_rejects_order_submission_enabled() {
        let dir = temp_dir("strategy-session-order-enabled");
        let config = write_config(
            &dir,
            &minimal_strategy_session_config().replace(
                r#"order_submission = "disabled""#,
                r#"order_submission = "enabled""#,
            ),
        );

        let error = run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::StrategySession,
                config,
                output: None,
            }),
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("execution.order_submission must be 'disabled'"));
    }

    #[test]
    fn config_validate_strategy_session_rejects_production_testnet_order_endpoint() {
        let dir = temp_dir("strategy-session-production-order-endpoint");
        let config = write_config(
            &dir,
            &minimal_strategy_session_config().replace(
                r#"http_base_url = "https://testnet.binance.vision""#,
                r#"http_base_url = "https://api.binance.com""#,
            ),
        );

        let error = run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::StrategySession,
                config,
                output: None,
            }),
        })
        .unwrap_err()
        .to_string();

        assert!(
            error.contains("testnet_order.http_base_url must be 'https://testnet.binance.vision'")
        );
    }

    #[test]
    fn config_validate_strategy_session_rejects_live_mode() {
        let dir = temp_dir("strategy-session-live");
        let config = write_config(
            &dir,
            &minimal_strategy_session_config().replace(r#"mode = "shadow""#, r#"mode = "live""#),
        );

        let error = run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::StrategySession,
                config,
                output: None,
            }),
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("node.mode must be 'shadow'"));
    }

    #[test]
    fn config_validate_strategy_session_rejects_empty_symbols() {
        let dir = temp_dir("strategy-session-empty-symbols");
        let config = write_config(
            &dir,
            &minimal_strategy_session_config()
                .replace(r#"symbols = ["BTCUSDT.BINANCE"]"#, "symbols = []"),
        );

        let error = run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::StrategySession,
                config,
                output: None,
            }),
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("market.symbols must not be empty"));
    }

    #[test]
    fn config_validate_strategy_session_rejects_multi_symbol_configs() {
        let dir = temp_dir("strategy-session-multi-symbol");
        let config = write_config(
            &dir,
            &minimal_strategy_session_config().replace(
                r#"symbols = ["BTCUSDT.BINANCE"]"#,
                r#"symbols = ["BTCUSDT.BINANCE", "ETHUSDT.BINANCE"]"#,
            ),
        );

        let error = run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::StrategySession,
                config,
                output: None,
            }),
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("market.symbols must contain exactly one symbol"));
    }

    #[test]
    fn config_validate_strategy_session_rejects_missing_kill_switch() {
        let dir = temp_dir("strategy-session-missing-kill-switch");
        let config = write_config(
            &dir,
            r#"[node]
node_id = "btc-ema-shadow-001"
mode = "shadow"

[strategy]
strategy_id = "ema-cross-demo"

[market]
symbols = ["BTCUSDT.BINANCE"]

[execution]
order_submission = "disabled"

[risk]
"#,
        );

        let error = run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::StrategySession,
                config,
                output: None,
            }),
        })
        .unwrap_err();
        let error = format!("{error:#}");

        assert!(
            error.contains(
                "risk.kill_switch_enabled must be true for v0.9.1 shadow strategy sessions"
            )
        );
    }

    #[test]
    fn config_validate_missing_file_reports_read_error() {
        let dir = temp_dir("missing");
        let config = dir.join("missing.toml");

        let error = run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::Data,
                config,
                output: None,
            }),
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("failed to read data config"));
    }

    #[test]
    fn config_validate_data_rejects_missing_queries() {
        let dir = temp_dir("data-missing-queries");
        let config = write_config(
            &dir,
            r#"[run]
id = "catalog-audit"
mode = "inspect"

[catalog]
path = "catalog/backtests/ema-cross"
protocol = "file"
"#,
        );

        let error = run_config_command(ConfigOpt {
            command: ConfigCommand::Validate(ConfigValidateOpt {
                kind: ConfigKind::Data,
                config,
                output: None,
            }),
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("queries must contain at least one data query"));
    }
}
