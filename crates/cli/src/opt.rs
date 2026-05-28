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

use std::path::PathBuf;

use clap::Parser;

/// Command-line interface for NautilusTrader.
#[derive(Debug, Parser)]
#[clap(version, about, author)]
pub struct NautilusCli {
    #[clap(subcommand)]
    pub command: Commands,
}

/// Available top-level commands for the NautilusTrader CLI.
#[derive(Parser, Debug)]
pub enum Commands {
    Backtest(BacktestOpt),
    Sandbox(SandboxOpt),
    Live(LiveOpt),
    Data(DataOpt),
    Database(DatabaseOpt),
    #[cfg(feature = "defi")]
    Blockchain(BlockchainOpt),
}

/// Backtest operations and validation commands.
#[derive(Parser, Debug)]
#[command(about = "Backtest operations", long_about = None)]
pub struct BacktestOpt {
    #[clap(subcommand)]
    pub command: BacktestCommand,
}

/// Available backtest commands.
#[derive(Parser, Debug, Clone)]
#[command(about = "Backtest operations", long_about = None)]
pub enum BacktestCommand {
    /// Validates a Rust backtest config without running the engine.
    Validate(BacktestValidateOpt),
    /// Runs a Rust backtest from a validated config.
    Run(BacktestRunOpt),
}

/// Backtest validation options.
#[derive(Parser, Debug, Clone)]
pub struct BacktestValidateOpt {
    /// Path to the Rust backtest config file.
    #[arg(long)]
    pub config: PathBuf,
}

/// Backtest run options.
#[derive(Parser, Debug, Clone)]
pub struct BacktestRunOpt {
    /// Path to the Rust backtest config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Optional owner-visible run identifier.
    #[arg(long)]
    pub run_id: Option<String>,
    /// Optional directory for run artifacts.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

/// Sandbox live-node operations and validation commands.
#[derive(Parser, Debug)]
#[command(about = "Sandbox live-node operations", long_about = None)]
pub struct SandboxOpt {
    #[clap(subcommand)]
    pub command: SandboxCommand,
}

/// Available sandbox commands.
#[derive(Parser, Debug, Clone)]
#[command(about = "Sandbox live-node operations", long_about = None)]
pub enum SandboxCommand {
    /// Validates a Rust sandbox config without starting a node.
    Validate(SandboxValidateOpt),
    /// Runs a Rust sandbox live-node flow from a validated config.
    Run(SandboxRunOpt),
}

/// Sandbox validation options.
#[derive(Parser, Debug, Clone)]
pub struct SandboxValidateOpt {
    /// Path to the Rust sandbox config file.
    #[arg(long)]
    pub config: PathBuf,
}

/// Sandbox run options.
#[derive(Parser, Debug, Clone)]
pub struct SandboxRunOpt {
    /// Path to the Rust sandbox config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Optional owner-visible run identifier.
    #[arg(long)]
    pub run_id: Option<String>,
    /// Optional directory for run artifacts.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

/// Live trading operations and validation commands.
#[derive(Parser, Debug)]
#[command(about = "Live trading operations", long_about = None)]
pub struct LiveOpt {
    #[clap(subcommand)]
    pub command: LiveCommand,
}

/// Available live commands.
#[derive(Parser, Debug, Clone)]
#[command(about = "Live trading operations", long_about = None)]
pub enum LiveCommand {
    /// Validates a Rust live config without starting a node.
    Validate(LiveValidateOpt),
    /// Runs a Rust live-node flow from a validated config.
    Run(LiveRunOpt),
}

/// Live validation options.
#[derive(Parser, Debug, Clone)]
pub struct LiveValidateOpt {
    /// Path to the Rust live config file.
    #[arg(long)]
    pub config: PathBuf,
}

/// Live run options.
#[derive(Parser, Debug, Clone)]
pub struct LiveRunOpt {
    /// Path to the Rust live config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Optional owner-visible run identifier.
    #[arg(long)]
    pub run_id: Option<String>,
    /// Optional directory for run artifacts.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

/// Data catalog inspection, validation, and loading commands.
#[derive(Parser, Debug)]
#[command(about = "Data catalog operations", long_about = None)]
pub struct DataOpt {
    #[clap(subcommand)]
    pub command: DataCommand,
}

/// Available data catalog commands.
#[derive(Parser, Debug, Clone)]
#[command(about = "Data catalog operations", long_about = None)]
pub enum DataCommand {
    /// Inspects catalog or source metadata without running a strategy.
    Inspect(DataInspectOpt),
    /// Validates catalog availability and requested data windows.
    Validate(DataValidateOpt),
    /// Loads scoped source data into a configured catalog target.
    Load(DataLoadOpt),
}

/// Data catalog inspection options.
#[derive(Parser, Debug, Clone)]
pub struct DataInspectOpt {
    /// Path to the Rust data/catalog config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Optional directory for inspection artifacts.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

/// Data catalog validation options.
#[derive(Parser, Debug, Clone)]
pub struct DataValidateOpt {
    /// Path to the Rust data/catalog config file.
    #[arg(long)]
    pub config: PathBuf,
}

/// Data catalog load options.
#[derive(Parser, Debug, Clone)]
pub struct DataLoadOpt {
    /// Path to the Rust data/catalog config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Optional owner-visible run identifier.
    #[arg(long)]
    pub run_id: Option<String>,
    /// Optional directory for load artifacts.
    #[arg(long)]
    pub output: Option<PathBuf>,
}

/// Database management options and subcommands.
#[derive(Parser, Debug)]
#[command(about = "Postgres database operations", long_about = None)]
pub struct DatabaseOpt {
    #[clap(subcommand)]
    pub command: DatabaseCommand,
}

/// Configuration parameters for database connection and operations.
#[derive(Parser, Debug, Clone)]
pub struct DatabaseConfig {
    /// Hostname or IP address of the database server.
    #[arg(long)]
    pub host: Option<String>,
    /// Port number of the database server.
    #[arg(long)]
    pub port: Option<u16>,
    /// Username for connecting to the database.
    #[arg(long)]
    pub username: Option<String>,
    /// Name of the database.
    #[arg(long)]
    pub database: Option<String>,
    /// Password for connecting to the database.
    #[arg(long)]
    pub password: Option<String>,
    /// Directory path to the schema files.
    #[arg(long)]
    pub schema: Option<String>,
}

/// Available database management commands.
#[derive(Parser, Debug, Clone)]
#[command(about = "Postgres database operations", long_about = None)]
pub enum DatabaseCommand {
    /// Initializes a new Postgres database with the latest schema.
    Init(DatabaseConfig),
    /// Drops roles, privileges and deletes all data from the database.
    Drop(DatabaseConfig),
}

#[cfg(feature = "defi")]
/// Blockchain management options and subcommands.
#[derive(Parser, Debug)]
#[command(about = "Blockchain operations", long_about = None)]
pub struct BlockchainOpt {
    #[clap(subcommand)]
    pub command: BlockchainCommand,
}

#[cfg(feature = "defi")]
/// Available blockchain management commands.
#[derive(Parser, Debug, Clone)]
#[command(about = "Blockchain operations", long_about = None)]
pub enum BlockchainCommand {
    /// Syncs blockchain blocks.
    SyncBlocks {
        /// The blockchain chain name (case-insensitive). Examples: ethereum, arbitrum, base, polygon, bsc
        #[arg(long)]
        chain: String,
        /// Starting block number to sync from (optional)
        #[arg(long)]
        from_block: Option<u64>,
        /// Ending block number to sync to (optional, defaults to current chain head)
        #[arg(long)]
        to_block: Option<u64>,
        /// Database configuration options
        #[clap(flatten)]
        database: DatabaseConfig,
    },
    /// Sync DEX pools.
    SyncDex {
        /// The blockchain chain name (case-insensitive). Examples: ethereum, arbitrum, base, polygon, bsc
        #[arg(long)]
        chain: String,
        /// The DEX name (case-insensitive). Examples: `UniswapV3`, uniswapv3, `SushiSwapV2`, `PancakeSwapV3`
        #[arg(long)]
        dex: String,
        /// RPC HTTP URL for blockchain calls (optional, falls back to `RPC_HTTP_URL` env var)
        #[arg(long)]
        rpc_url: Option<String>,
        /// Reset sync progress and start from the beginning, ignoring last synced block
        #[arg(long)]
        reset: bool,
        /// Maximum number of Multicall calls per RPC request (optional, defaults to 100)
        #[arg(long)]
        multicall_calls_per_rpc_request: Option<u32>,
        /// Database configuration options
        #[clap(flatten)]
        database: DatabaseConfig,
    },
    /// Analyze a specific DEX pool.
    AnalyzePool {
        /// The blockchain chain name (case-insensitive). Examples: ethereum, arbitrum, base, polygon, bsc
        #[arg(long)]
        chain: String,
        /// The DEX name (case-insensitive). Examples: UniswapV3, uniswapv3, SushiSwapV2, PancakeSwapV3
        #[arg(long)]
        dex: String,
        /// The pool contract address
        #[arg(long)]
        address: String,
        /// Starting block number to sync from (optional)
        #[arg(long)]
        from_block: Option<u64>,
        /// Ending block number to sync to (optional, defaults to current chain head)
        #[arg(long)]
        to_block: Option<u64>,
        /// RPC HTTP URL for blockchain calls (optional, falls back to RPC_HTTP_URL env var)
        #[arg(long)]
        rpc_url: Option<String>,
        /// Reset sync progress and start from the beginning, ignoring last synced block
        #[arg(long)]
        reset: bool,
        /// Maximum number of Multicall calls per RPC request (optional, defaults to 100)
        #[arg(long)]
        multicall_calls_per_rpc_request: Option<u32>,
        /// Database configuration options
        #[clap(flatten)]
        database: DatabaseConfig,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, Parser};

    #[test]
    fn top_level_help_lists_backtest() {
        let help = NautilusCli::command().render_help().to_string();

        assert!(help.contains("backtest"));
        assert!(help.contains("sandbox"));
        assert!(help.contains("live"));
        assert!(help.contains("data"));
    }

    #[test]
    fn backtest_help_lists_validate_and_run() {
        let mut command = NautilusCli::command();
        let backtest = command
            .find_subcommand_mut("backtest")
            .expect("backtest command should exist");
        let help = backtest.render_help().to_string();

        assert!(help.contains("validate"));
        assert!(help.contains("run"));
    }

    #[test]
    fn parses_backtest_validate_config_path() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "backtest",
            "validate",
            "--config",
            "config/backtest.toml",
        ])
        .expect("backtest validate should parse");

        let Commands::Backtest(backtest) = parsed.command else {
            panic!("expected backtest command");
        };
        let BacktestCommand::Validate(validate) = backtest.command else {
            panic!("expected validate command");
        };

        assert_eq!(validate.config, PathBuf::from("config/backtest.toml"));
    }

    #[test]
    fn parses_backtest_run_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "backtest",
            "run",
            "--config",
            "config/backtest.toml",
            "--run-id",
            "ema-cross",
            "--output",
            "runs/ema-cross",
        ])
        .expect("backtest run should parse");

        let Commands::Backtest(backtest) = parsed.command else {
            panic!("expected backtest command");
        };
        let BacktestCommand::Run(run) = backtest.command else {
            panic!("expected run command");
        };

        assert_eq!(run.config, PathBuf::from("config/backtest.toml"));
        assert_eq!(run.run_id.as_deref(), Some("ema-cross"));
        assert_eq!(run.output, Some(PathBuf::from("runs/ema-cross")));
    }

    #[test]
    fn sandbox_help_lists_validate_and_run() {
        let mut command = NautilusCli::command();
        let sandbox = command
            .find_subcommand_mut("sandbox")
            .expect("sandbox command should exist");
        let help = sandbox.render_help().to_string();

        assert!(help.contains("validate"));
        assert!(help.contains("run"));
    }

    #[test]
    fn parses_sandbox_validate_config_path() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "sandbox",
            "validate",
            "--config",
            "config/sandbox.toml",
        ])
        .expect("sandbox validate should parse");

        let Commands::Sandbox(sandbox) = parsed.command else {
            panic!("expected sandbox command");
        };
        let SandboxCommand::Validate(validate) = sandbox.command else {
            panic!("expected validate command");
        };

        assert_eq!(validate.config, PathBuf::from("config/sandbox.toml"));
    }

    #[test]
    fn parses_sandbox_run_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "sandbox",
            "run",
            "--config",
            "config/sandbox.toml",
            "--run-id",
            "sandbox-smoke",
            "--output",
            "runs/sandbox-smoke",
        ])
        .expect("sandbox run should parse");

        let Commands::Sandbox(sandbox) = parsed.command else {
            panic!("expected sandbox command");
        };
        let SandboxCommand::Run(run) = sandbox.command else {
            panic!("expected run command");
        };

        assert_eq!(run.config, PathBuf::from("config/sandbox.toml"));
        assert_eq!(run.run_id.as_deref(), Some("sandbox-smoke"));
        assert_eq!(run.output, Some(PathBuf::from("runs/sandbox-smoke")));
    }

    #[test]
    fn live_help_lists_validate_and_run() {
        let mut command = NautilusCli::command();
        let live = command
            .find_subcommand_mut("live")
            .expect("live command should exist");
        let help = live.render_help().to_string();

        assert!(help.contains("validate"));
        assert!(help.contains("run"));
    }

    #[test]
    fn parses_live_validate_config_path() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "validate",
            "--config",
            "config/live.toml",
        ])
        .expect("live validate should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::Validate(validate) = live.command else {
            panic!("expected validate command");
        };

        assert_eq!(validate.config, PathBuf::from("config/live.toml"));
    }

    #[test]
    fn parses_live_run_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "live",
            "run",
            "--config",
            "config/live.toml",
            "--run-id",
            "live-dry-run",
            "--output",
            "runs/live-dry-run",
        ])
        .expect("live run should parse");

        let Commands::Live(live) = parsed.command else {
            panic!("expected live command");
        };
        let LiveCommand::Run(run) = live.command else {
            panic!("expected run command");
        };

        assert_eq!(run.config, PathBuf::from("config/live.toml"));
        assert_eq!(run.run_id.as_deref(), Some("live-dry-run"));
        assert_eq!(run.output, Some(PathBuf::from("runs/live-dry-run")));
    }

    #[test]
    fn data_help_lists_inspect_validate_and_load() {
        let mut command = NautilusCli::command();
        let data = command
            .find_subcommand_mut("data")
            .expect("data command should exist");
        let help = data.render_help().to_string();

        assert!(help.contains("inspect"));
        assert!(help.contains("validate"));
        assert!(help.contains("load"));
    }

    #[test]
    fn parses_data_inspect_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "data",
            "inspect",
            "--config",
            "config/data.toml",
            "--output",
            "runs/catalog-audit",
        ])
        .expect("data inspect should parse");

        let Commands::Data(data) = parsed.command else {
            panic!("expected data command");
        };
        let DataCommand::Inspect(inspect) = data.command else {
            panic!("expected inspect command");
        };

        assert_eq!(inspect.config, PathBuf::from("config/data.toml"));
        assert_eq!(inspect.output, Some(PathBuf::from("runs/catalog-audit")));
    }

    #[test]
    fn parses_data_validate_config_path() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "data",
            "validate",
            "--config",
            "config/data.toml",
        ])
        .expect("data validate should parse");

        let Commands::Data(data) = parsed.command else {
            panic!("expected data command");
        };
        let DataCommand::Validate(validate) = data.command else {
            panic!("expected validate command");
        };

        assert_eq!(validate.config, PathBuf::from("config/data.toml"));
    }

    #[test]
    fn parses_data_load_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "data",
            "load",
            "--config",
            "config/data.toml",
            "--run-id",
            "load-quotes",
            "--output",
            "runs/load-quotes",
        ])
        .expect("data load should parse");

        let Commands::Data(data) = parsed.command else {
            panic!("expected data command");
        };
        let DataCommand::Load(load) = data.command else {
            panic!("expected load command");
        };

        assert_eq!(load.config, PathBuf::from("config/data.toml"));
        assert_eq!(load.run_id.as_deref(), Some("load-quotes"));
        assert_eq!(load.output, Some(PathBuf::from("runs/load-quotes")));
    }
}
