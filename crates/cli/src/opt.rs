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
}
