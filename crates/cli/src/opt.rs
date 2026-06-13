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

use std::{net::SocketAddr, path::PathBuf};

use clap::{Parser, ValueEnum};

/// Command-line interface for NTPRO.
#[derive(Debug, Parser)]
#[clap(version, about, author)]
pub struct NautilusCli {
    #[clap(subcommand)]
    pub command: Commands,
}

/// Available top-level commands for the NTPRO CLI.
#[derive(Parser, Debug)]
pub enum Commands {
    Backtest(BacktestOpt),
    Sandbox(SandboxOpt),
    Live(LiveOpt),
    Data(DataOpt),
    Config(ConfigOpt),
    Supervisor(SupervisorOpt),
    Dashboard(DashboardOpt),
    Workflow(WorkflowOpt),
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
    /// Runs a Rust backtest smoke path, or metadata-only dry-run with --dry-run.
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
    /// Writes metadata-only dry-run artifacts without starting the backtest engine.
    #[arg(long)]
    pub dry_run: bool,
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
    /// Writes simulation-only sandbox demo artifacts; use live run for LiveNode smoke.
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

/// Local sandbox LiveNode validation and smoke commands.
#[derive(Parser, Debug)]
#[command(
    about = "Local sandbox LiveNode smoke commands (no production venue access)",
    long_about = None
)]
pub struct LiveOpt {
    #[clap(subcommand)]
    pub command: LiveCommand,
}

/// Available local sandbox live commands.
#[derive(Parser, Debug, Clone)]
#[command(
    about = "Local sandbox LiveNode smoke commands (no production venue access)",
    long_about = None
)]
pub enum LiveCommand {
    /// Validates the Rust live-init smoke config.
    Validate(LiveValidateOpt),
    /// Runs a local sandbox LiveNode start/stop smoke path without external venue access or real orders.
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

/// Local supervisor controls for sandbox-only node artifacts and processes.
#[derive(Parser, Debug)]
#[command(
    about = "Local supervisor controls for sandbox-only ntpro-node processes",
    long_about = None
)]
pub struct SupervisorOpt {
    #[clap(subcommand)]
    pub command: SupervisorCommand,
}

/// Available local supervisor commands.
#[derive(Parser, Debug, Clone)]
#[command(
    about = "Local supervisor controls for sandbox-only ntpro-node processes",
    long_about = None
)]
pub enum SupervisorCommand {
    /// Registers or replaces a local sandbox node record.
    Register(SupervisorRegisterOpt),
    /// Lists registered local nodes.
    List(SupervisorListOpt),
    /// Starts a registered local sandbox-only ntpro-node process.
    Start(SupervisorStartOpt),
    /// Stops a registered local ntpro-node process.
    Stop(SupervisorStopOpt),
    /// Pauses a running local sandbox node at the supervisor artifact layer.
    Pause(SupervisorNodeOpt),
    /// Resumes a paused local sandbox node at the supervisor artifact layer.
    Resume(SupervisorNodeOpt),
    /// Records a local sandbox data-source reconnect result.
    ReconnectData(SupervisorNodeOpt),
    /// Records a local sandbox execution-gateway reconnect result.
    ReconnectExecution(SupervisorNodeOpt),
    /// Reads the latest node status artifact.
    Status(SupervisorNodeOpt),
    /// Reads data/execution connection status from node artifacts.
    Connections(SupervisorNodeOpt),
    /// Reads execution summary from node artifacts.
    Execution(SupervisorNodeOpt),
    /// Reads risk summary from node artifacts.
    Risk(SupervisorNodeOpt),
    /// Prints per-node log artifact paths.
    Logs(SupervisorNodeOpt),
    /// Reads minimal per-node metrics JSON.
    Metrics(SupervisorNodeOpt),
}

/// Common supervisor registry option.
#[derive(Parser, Debug, Clone)]
pub struct SupervisorRegistryOpt {
    /// Path to the local supervisor registry JSON file.
    #[arg(long)]
    pub registry: PathBuf,
}

/// Supervisor node registration options.
#[derive(Parser, Debug, Clone)]
pub struct SupervisorRegisterOpt {
    #[clap(flatten)]
    pub registry: SupervisorRegistryOpt,
    /// Stable local node identifier.
    #[arg(long)]
    pub node_id: String,
    /// Path to the Rust live-init smoke config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Optional directory for node artifacts.
    #[arg(long)]
    pub artifact_root: Option<PathBuf>,
}

/// Supervisor node listing options.
#[derive(Parser, Debug, Clone)]
pub struct SupervisorListOpt {
    #[clap(flatten)]
    pub registry: SupervisorRegistryOpt,
}

/// Supervisor node start options.
#[derive(Parser, Debug, Clone)]
pub struct SupervisorStartOpt {
    #[clap(flatten)]
    pub registry: SupervisorRegistryOpt,
    /// Registered local node identifier.
    #[arg(long)]
    pub node_id: String,
    /// Path to the ntpro-node binary.
    #[arg(long)]
    pub ntpro_node_bin: PathBuf,
    /// Startup wait timeout in milliseconds.
    #[arg(long, default_value_t = 5_000)]
    pub startup_timeout_ms: u64,
    /// Maximum runtime passed to the spawned ntpro-node process.
    #[arg(long, default_value_t = 3_600_000)]
    pub node_max_runtime_ms: u64,
    /// Heartbeat interval passed to the spawned ntpro-node process.
    #[arg(long, default_value_t = 1_000)]
    pub node_heartbeat_interval_ms: u64,
    /// Optional parent process PID passed to the spawned ntpro-node process.
    #[arg(long)]
    pub node_parent_pid: Option<u32>,
    /// Shutdown timeout passed to the spawned ntpro-node process.
    #[arg(long, default_value_t = 5_000)]
    pub node_shutdown_timeout_ms: u64,
}

/// Supervisor node stop options.
#[derive(Parser, Debug, Clone)]
pub struct SupervisorStopOpt {
    #[clap(flatten)]
    pub registry: SupervisorRegistryOpt,
    /// Registered local node identifier.
    #[arg(long)]
    pub node_id: String,
    /// Stop wait timeout in milliseconds.
    #[arg(long, default_value_t = 5_000)]
    pub stop_timeout_ms: u64,
}

/// Supervisor single-node query options.
#[derive(Parser, Debug, Clone)]
pub struct SupervisorNodeOpt {
    #[clap(flatten)]
    pub registry: SupervisorRegistryOpt,
    /// Registered local node identifier.
    #[arg(long)]
    pub node_id: String,
}

/// Local dashboard HTTP server commands.
#[derive(Parser, Debug)]
#[command(about = "Local dashboard HTTP server", long_about = None)]
pub struct DashboardOpt {
    #[clap(subcommand)]
    pub command: DashboardCommand,
}

/// Available local dashboard commands.
#[derive(Parser, Debug, Clone)]
#[command(about = "Local dashboard HTTP server", long_about = None)]
pub enum DashboardCommand {
    /// Serves the static dashboard shell and local JSON API.
    Serve(DashboardServeOpt),
}

/// Local dashboard server options.
#[derive(Parser, Debug, Clone)]
pub struct DashboardServeOpt {
    /// Path to the local supervisor registry JSON file.
    #[arg(long)]
    pub registry: PathBuf,
    /// Local loopback address for the dashboard HTTP server.
    #[arg(long, default_value = "127.0.0.1:5173")]
    pub bind: SocketAddr,
    /// Optional path to the local ntpro-node binary used by start controls.
    #[arg(long)]
    pub ntpro_node_bin: Option<PathBuf>,
}

/// Local workflow artifact commands for sandbox/testnet product smokes.
#[derive(Parser, Debug)]
#[command(
    about = "Local sandbox/testnet workflow artifact commands",
    long_about = None
)]
pub struct WorkflowOpt {
    #[clap(subcommand)]
    pub command: WorkflowCommand,
}

/// Available local workflow artifact commands.
#[derive(Parser, Debug, Clone)]
#[command(
    about = "Local sandbox/testnet workflow artifact commands",
    long_about = None
)]
pub enum WorkflowCommand {
    /// Runs a local Binance sandbox/testnet workflow and writes dashboard-readable artifacts.
    Run(WorkflowRunOpt),
}

/// Supported local workflow kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum WorkflowKind {
    /// Local Binance sandbox workflow using checked-in fixtures and mock execution only.
    BinanceSandbox,
    /// Local Binance testnet dry-run workflow; no network or real orders by default.
    BinanceTestnet,
}

/// Supported local workflow run modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum WorkflowRunMode {
    /// Validate config and write local dry-run artifacts only.
    DryRun,
    /// Validate the testnet connectivity contract without opening a network connection.
    ConnectivityProbe,
}

/// Local workflow run options.
#[derive(Parser, Debug, Clone)]
pub struct WorkflowRunOpt {
    /// Workflow kind to run.
    #[arg(long, value_enum, default_value_t = WorkflowKind::BinanceSandbox)]
    pub workflow: WorkflowKind,
    /// Workflow run mode. Testnet modes are offline unless explicitly documented otherwise.
    #[arg(long, value_enum, default_value_t = WorkflowRunMode::DryRun)]
    pub mode: WorkflowRunMode,
    /// Optional workflow config. Required for the Binance testnet workflow.
    #[arg(long)]
    pub config: Option<PathBuf>,
    /// Acknowledge optional testnet-network intent. The current v0.6 gate still records dry-run artifacts only.
    #[arg(long)]
    pub allow_testnet_network: bool,
    /// Owner-visible run identifier used in the artifact directory.
    #[arg(long)]
    pub run_id: Option<String>,
    /// Optional output directory for workflow artifacts.
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
    /// Inspects local data/catalog metadata without running a workflow.
    Inspect(DataInspectOpt),
    /// Validates local data/catalog readability and query shape.
    Validate(DataValidateOpt),
    /// Loads a local QuoteTick fixture into a Rust catalog directory.
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

/// Shared Rust config validation commands.
#[derive(Parser, Debug)]
#[command(about = "Rust config validation", long_about = None)]
pub struct ConfigOpt {
    #[clap(subcommand)]
    pub command: ConfigCommand,
}

/// Available Rust config validation commands.
#[derive(Parser, Debug, Clone)]
#[command(about = "Rust config validation", long_about = None)]
pub enum ConfigCommand {
    /// Validates a Rust workflow config without running the workflow.
    Validate(ConfigValidateOpt),
}

/// Supported workflow config kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ConfigKind {
    Backtest,
    Sandbox,
    Live,
    Data,
}

/// Shared Rust config validation options.
#[derive(Parser, Debug, Clone)]
pub struct ConfigValidateOpt {
    /// Workflow config kind.
    #[arg(long, value_enum)]
    pub kind: ConfigKind,
    /// Path to the Rust config file.
    #[arg(long)]
    pub config: PathBuf,
    /// Optional directory for validation artifacts.
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

    fn render_subcommand_help(path: &[&str]) -> String {
        let mut command = NautilusCli::command();
        let mut current = &mut command;
        for name in path {
            current = current
                .find_subcommand_mut(name)
                .unwrap_or_else(|| panic!("{name} command should exist"));
        }
        current.render_help().to_string()
    }

    #[test]
    fn top_level_help_lists_backtest() {
        let help = NautilusCli::command().render_help().to_string();

        assert!(help.contains("backtest"));
        assert!(help.contains("sandbox"));
        assert!(help.contains("live"));
        assert!(help.contains("data"));
        assert!(help.contains("config"));
        assert!(help.contains("supervisor"));
        assert!(help.contains("dashboard"));
        assert!(help.contains("workflow"));
        assert!(help.contains("database"));
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
            "--dry-run",
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
        assert!(run.dry_run);
    }

    #[test]
    fn backtest_run_help_describes_dry_run_boundary() {
        let help = render_subcommand_help(&["backtest", "run"]);

        assert!(help.contains("Rust backtest smoke path"));
        assert!(help.contains("metadata-only dry-run"));
        assert!(help.contains("--dry-run"));
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
    fn sandbox_run_help_describes_simulated_boundary() {
        let help = render_subcommand_help(&["sandbox", "run"]);

        assert!(help.contains("simulation-only sandbox demo artifacts"));
        assert!(help.contains("use live run for LiveNode smoke"));
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
    fn live_help_describes_live_init_smoke_boundary() {
        let validate_help = render_subcommand_help(&["live", "validate"]);
        let run_help = render_subcommand_help(&["live", "run"]);

        assert!(validate_help.contains("live-init smoke config"));
        assert!(run_help.contains("LiveNode start/stop smoke path"));
        assert!(run_help.contains("without external venue access"));
        assert!(run_help.contains("real orders"));
    }

    #[test]
    fn supervisor_help_lists_local_artifact_controls() {
        let mut command = NautilusCli::command();
        let supervisor = command
            .find_subcommand_mut("supervisor")
            .expect("supervisor command should exist");
        let help = supervisor.render_help().to_string();

        for command in [
            "register",
            "list",
            "start",
            "stop",
            "pause",
            "resume",
            "reconnect-data",
            "reconnect-execution",
            "status",
            "connections",
            "execution",
            "risk",
            "logs",
            "metrics",
        ] {
            assert!(help.contains(command), "{command} should be listed");
        }
    }

    #[test]
    fn supervisor_help_describes_local_boundary() {
        let help = render_subcommand_help(&["supervisor", "start"]);

        assert!(help.contains("registered local sandbox-only ntpro-node process"));
        assert!(help.contains("sandbox-only"));
        assert!(help.contains("--registry"));
        assert!(help.contains("--node-id"));
        assert!(help.contains("--ntpro-node-bin"));
    }

    #[test]
    fn parses_supervisor_register_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "supervisor",
            "register",
            "--registry",
            "runs/supervisor/registry.json",
            "--node-id",
            "sandbox-a",
            "--config",
            "config/live.toml",
            "--artifact-root",
            "runs/supervisor/nodes/sandbox-a",
        ])
        .expect("supervisor register should parse");

        let Commands::Supervisor(supervisor) = parsed.command else {
            panic!("expected supervisor command");
        };
        let SupervisorCommand::Register(register) = supervisor.command else {
            panic!("expected register command");
        };

        assert_eq!(
            register.registry.registry,
            PathBuf::from("runs/supervisor/registry.json")
        );
        assert_eq!(register.node_id, "sandbox-a");
        assert_eq!(register.config, PathBuf::from("config/live.toml"));
        assert_eq!(
            register.artifact_root,
            Some(PathBuf::from("runs/supervisor/nodes/sandbox-a"))
        );
    }

    #[test]
    fn parses_supervisor_start_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "supervisor",
            "start",
            "--registry",
            "runs/supervisor/registry.json",
            "--node-id",
            "sandbox-a",
            "--ntpro-node-bin",
            "target/debug/ntpro-node",
            "--startup-timeout-ms",
            "7500",
        ])
        .expect("supervisor start should parse");

        let Commands::Supervisor(supervisor) = parsed.command else {
            panic!("expected supervisor command");
        };
        let SupervisorCommand::Start(start) = supervisor.command else {
            panic!("expected start command");
        };

        assert_eq!(
            start.registry.registry,
            PathBuf::from("runs/supervisor/registry.json")
        );
        assert_eq!(start.node_id, "sandbox-a");
        assert_eq!(
            start.ntpro_node_bin,
            PathBuf::from("target/debug/ntpro-node")
        );
        assert_eq!(start.startup_timeout_ms, 7_500);
        assert_eq!(start.node_max_runtime_ms, 3_600_000);
        assert_eq!(start.node_heartbeat_interval_ms, 1_000);
        assert_eq!(start.node_parent_pid, None);
        assert_eq!(start.node_shutdown_timeout_ms, 5_000);
    }

    #[test]
    fn parses_supervisor_query_options() {
        for command in [
            "pause",
            "resume",
            "reconnect-data",
            "reconnect-execution",
            "status",
            "connections",
            "execution",
            "risk",
            "logs",
            "metrics",
        ] {
            let parsed = NautilusCli::try_parse_from([
                "nautilus",
                "supervisor",
                command,
                "--registry",
                "runs/supervisor/registry.json",
                "--node-id",
                "sandbox-a",
            ])
            .unwrap_or_else(|_| panic!("supervisor {command} should parse"));

            let Commands::Supervisor(supervisor) = parsed.command else {
                panic!("expected supervisor command");
            };
            match supervisor.command {
                SupervisorCommand::Status(node)
                | SupervisorCommand::Pause(node)
                | SupervisorCommand::Resume(node)
                | SupervisorCommand::ReconnectData(node)
                | SupervisorCommand::ReconnectExecution(node)
                | SupervisorCommand::Connections(node)
                | SupervisorCommand::Execution(node)
                | SupervisorCommand::Risk(node)
                | SupervisorCommand::Logs(node)
                | SupervisorCommand::Metrics(node) => {
                    assert_eq!(
                        node.registry.registry,
                        PathBuf::from("runs/supervisor/registry.json")
                    );
                    assert_eq!(node.node_id, "sandbox-a");
                }
                _ => panic!("expected query command"),
            }
        }
    }

    #[test]
    fn dashboard_help_lists_local_http_server() {
        let mut command = NautilusCli::command();
        let dashboard = command
            .find_subcommand_mut("dashboard")
            .expect("dashboard command should exist");
        let help = dashboard.render_help().to_string();

        assert!(help.contains("serve"));
        assert!(help.contains("Local dashboard HTTP server"));
    }

    #[test]
    fn dashboard_serve_help_describes_local_boundary() {
        let help = render_subcommand_help(&["dashboard", "serve"]);

        assert!(help.contains("static dashboard shell"));
        assert!(help.contains("local JSON API"));
        assert!(help.contains("--registry"));
        assert!(help.contains("--bind"));
    }

    #[test]
    fn parses_dashboard_serve_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "dashboard",
            "serve",
            "--registry",
            "runs/supervisor/registry.json",
            "--bind",
            "127.0.0.1:5174",
        ])
        .expect("dashboard serve should parse");

        let Commands::Dashboard(dashboard) = parsed.command else {
            panic!("expected dashboard command");
        };
        let DashboardCommand::Serve(serve) = dashboard.command;

        assert_eq!(
            serve.registry,
            PathBuf::from("runs/supervisor/registry.json")
        );
        assert_eq!(serve.bind.to_string(), "127.0.0.1:5174");
        assert!(serve.ntpro_node_bin.is_none());
    }

    #[test]
    fn dashboard_serve_defaults_to_loopback() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "dashboard",
            "serve",
            "--registry",
            "runs/supervisor/registry.json",
        ])
        .expect("dashboard serve should parse with default bind");

        let Commands::Dashboard(dashboard) = parsed.command else {
            panic!("expected dashboard command");
        };
        let DashboardCommand::Serve(serve) = dashboard.command;

        assert_eq!(serve.bind.to_string(), "127.0.0.1:5173");
    }

    #[test]
    fn parses_dashboard_serve_ntpro_node_bin() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "dashboard",
            "serve",
            "--registry",
            "runs/supervisor/registry.json",
            "--ntpro-node-bin",
            "target/debug/ntpro-node",
        ])
        .expect("dashboard serve should parse with ntpro-node binary");

        let Commands::Dashboard(dashboard) = parsed.command else {
            panic!("expected dashboard command");
        };
        let DashboardCommand::Serve(serve) = dashboard.command;

        assert_eq!(
            serve.ntpro_node_bin,
            Some(PathBuf::from("target/debug/ntpro-node"))
        );
    }

    #[test]
    fn workflow_help_lists_local_artifact_run() {
        let mut command = NautilusCli::command();
        let workflow = command
            .find_subcommand_mut("workflow")
            .expect("workflow command should exist");
        let help = workflow.render_help().to_string();

        assert!(help.contains("run"));
        assert!(help.contains("Local sandbox/testnet workflow artifact commands"));
    }

    #[test]
    fn workflow_run_help_describes_sandbox_boundary() {
        let help = render_subcommand_help(&["workflow", "run"]);

        assert!(help.contains("local Binance sandbox/testnet workflow"));
        assert!(help.contains("dashboard-readable artifacts"));
        assert!(help.contains("--workflow"));
        assert!(help.contains("--mode"));
        assert!(help.contains("--config"));
        assert!(help.contains("--allow-testnet-network"));
        assert!(help.contains("--run-id"));
        assert!(help.contains("--output"));
    }

    #[test]
    fn parses_workflow_run_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "workflow",
            "run",
            "--workflow",
            "binance-sandbox",
            "--run-id",
            "v05-smoke",
            "--output",
            "runs/workflows/v05-smoke",
        ])
        .expect("workflow run should parse");

        let Commands::Workflow(workflow) = parsed.command else {
            panic!("expected workflow command");
        };
        let WorkflowCommand::Run(run) = workflow.command;

        assert_eq!(run.workflow, WorkflowKind::BinanceSandbox);
        assert_eq!(run.mode, WorkflowRunMode::DryRun);
        assert_eq!(run.config, None);
        assert!(!run.allow_testnet_network);
        assert_eq!(run.run_id.as_deref(), Some("v05-smoke"));
        assert_eq!(run.output, Some(PathBuf::from("runs/workflows/v05-smoke")));
    }

    #[test]
    fn parses_workflow_binance_testnet_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "workflow",
            "run",
            "--workflow",
            "binance-testnet",
            "--mode",
            "connectivity-probe",
            "--config",
            "examples/rust/binance/testnet_dry_run.toml",
            "--allow-testnet-network",
            "--run-id",
            "v06-smoke",
            "--output",
            "runs/workflows/v06-smoke",
        ])
        .expect("workflow run should parse");

        let Commands::Workflow(workflow) = parsed.command else {
            panic!("expected workflow command");
        };
        let WorkflowCommand::Run(run) = workflow.command;

        assert_eq!(run.workflow, WorkflowKind::BinanceTestnet);
        assert_eq!(run.mode, WorkflowRunMode::ConnectivityProbe);
        assert_eq!(
            run.config,
            Some(PathBuf::from("examples/rust/binance/testnet_dry_run.toml"))
        );
        assert!(run.allow_testnet_network);
        assert_eq!(run.run_id.as_deref(), Some("v06-smoke"));
        assert_eq!(run.output, Some(PathBuf::from("runs/workflows/v06-smoke")));
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

    #[test]
    fn data_help_describes_supported_and_deferred_boundaries() {
        let inspect_help = render_subcommand_help(&["data", "inspect"]);
        let validate_help = render_subcommand_help(&["data", "validate"]);
        let load_help = render_subcommand_help(&["data", "load"]);

        assert!(inspect_help.contains("Inspects local data/catalog metadata"));
        assert!(validate_help.contains("Validates local data/catalog readability"));
        assert!(load_help.contains("Loads a local QuoteTick fixture"));
    }

    #[test]
    fn config_help_lists_validate() {
        let mut command = NautilusCli::command();
        let config = command
            .find_subcommand_mut("config")
            .expect("config command should exist");
        let help = config.render_help().to_string();

        assert!(help.contains("validate"));
    }

    #[test]
    fn config_validate_help_lists_kind_config_and_output() {
        let mut command = NautilusCli::command();
        let config = command
            .find_subcommand_mut("config")
            .expect("config command should exist");
        let validate = config
            .find_subcommand_mut("validate")
            .expect("config validate command should exist");
        let help = validate.render_help().to_string();

        assert!(help.contains("--kind"));
        assert!(help.contains("--config"));
        assert!(help.contains("--output"));
    }

    #[test]
    fn config_validate_help_describes_validation_boundary() {
        let help = render_subcommand_help(&["config", "validate"]);

        assert!(help.contains("Validates a Rust workflow config"));
        assert!(help.contains("--kind"));
        assert!(help.contains("--config"));
    }

    #[test]
    fn parses_config_validate_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "config",
            "validate",
            "--kind",
            "backtest",
            "--config",
            "config/backtest.toml",
            "--output",
            "runs/config-validate",
        ])
        .expect("config validate should parse");

        let Commands::Config(config) = parsed.command else {
            panic!("expected config command");
        };
        let ConfigCommand::Validate(validate) = config.command;

        assert_eq!(validate.kind, ConfigKind::Backtest);
        assert_eq!(validate.config, PathBuf::from("config/backtest.toml"));
        assert_eq!(validate.output, Some(PathBuf::from("runs/config-validate")));
    }

    #[test]
    fn database_help_lists_init_and_drop() {
        let mut command = NautilusCli::command();
        let database = command
            .find_subcommand_mut("database")
            .expect("database command should exist");
        let help = database.render_help().to_string();

        assert!(help.contains("init"));
        assert!(help.contains("drop"));
    }

    #[test]
    fn parses_database_init_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "database",
            "init",
            "--host",
            "localhost",
            "--port",
            "5432",
            "--username",
            "ntpro",
            "--database",
            "ntpro",
            "--password",
            "secret",
            "--schema",
            "schema/postgres",
        ])
        .expect("database init should parse");

        let Commands::Database(database) = parsed.command else {
            panic!("expected database command");
        };
        let DatabaseCommand::Init(config) = database.command else {
            panic!("expected init command");
        };

        assert_eq!(config.host.as_deref(), Some("localhost"));
        assert_eq!(config.port, Some(5432));
        assert_eq!(config.username.as_deref(), Some("ntpro"));
        assert_eq!(config.database.as_deref(), Some("ntpro"));
        assert_eq!(config.password.as_deref(), Some("secret"));
        assert_eq!(config.schema.as_deref(), Some("schema/postgres"));
    }

    #[test]
    fn parses_database_drop_options() {
        let parsed = NautilusCli::try_parse_from([
            "nautilus",
            "database",
            "drop",
            "--host",
            "localhost",
            "--port",
            "5432",
            "--username",
            "ntpro",
            "--database",
            "ntpro",
        ])
        .expect("database drop should parse");

        let Commands::Database(database) = parsed.command else {
            panic!("expected database command");
        };
        let DatabaseCommand::Drop(config) = database.command else {
            panic!("expected drop command");
        };

        assert_eq!(config.host.as_deref(), Some("localhost"));
        assert_eq!(config.port, Some(5432));
        assert_eq!(config.username.as_deref(), Some("ntpro"));
        assert_eq!(config.database.as_deref(), Some("ntpro"));
        assert_eq!(config.password, None);
        assert_eq!(config.schema, None);
    }
}
