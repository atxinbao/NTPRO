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

//! Command-line interface and tools for NTPRO.
//!
//! The `nautilus-cli` crate provides a Rust command-line interface for managing
//! local NTPRO workspaces. Its v0.2.0 product boundary is a local multi-node
//! runtime foundation, not production live trading:
//!
//! - Database initialization and management commands.
//! - PostgreSQL schema setup and maintenance.
//! - Configuration validation and setup utilities.
//! - System administration and operational tools.
//!
//! # NTPRO
//!
//! NTPRO is a Rust-only local runtime foundation for sandbox-first multi-node
//! orchestration. The current public product claim covers local node
//! registration, local `ntpro-node` process control, local status/log/metrics
//! inspection, and sandbox smoke evidence.
//!
//! v0.2.0 does not claim production exchange connectivity, real account
//! connectivity, real order submission, manual order entry, distributed
//! deployment, or prebuilt release artifact delivery.
//!
//! # Feature Flags
//!
//! This crate provides feature flags to control source code inclusion during compilation,
//! depending on the intended use case:
//!
//! - `defi`: Enables DeFi functionality including blockchain data access and pool analysis.

#![warn(rustc::all)]
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(nonstandard_style)]
#![deny(missing_debug_implementations)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(rustdoc::broken_intra_doc_links)]

mod artifacts;
mod backtest;
#[cfg(feature = "defi")]
mod blockchain;
mod config;
pub mod dashboard;
mod data;
mod database;
mod endpoint_classifier;
mod live;
mod mvp;
mod mvp_contract;
pub mod opt;
mod process;
mod sandbox;
pub mod strategy_session;
pub mod supervisor;
mod workflow;
mod workflow_contract;

use std::path::PathBuf;

#[cfg(feature = "defi")]
use crate::blockchain::run_blockchain_command;
use crate::{
    backtest::run_backtest_command,
    config::run_config_command,
    dashboard::run_dashboard_command,
    data::run_data_command,
    database::postgres::run_database_command,
    live::run_live_command,
    mvp::run_mvp_command,
    opt::{Commands, NautilusCli},
    sandbox::run_sandbox_command,
    supervisor::run_supervisor_command,
    workflow::run_workflow_command,
};

pub use live::NtproNodeRunControls;

/// Runs the NTPRO CLI based on the provided options.
///
/// # Errors
///
/// Returns an error if execution of the specified command fails.
pub async fn run(opt: NautilusCli) -> anyhow::Result<()> {
    match opt.command {
        Commands::Backtest(backtest_opt) => run_backtest_command(backtest_opt)?,
        Commands::Sandbox(sandbox_opt) => run_sandbox_command(sandbox_opt)?,
        Commands::Live(live_opt) => run_live_command(live_opt).await?,
        Commands::Data(data_opt) => run_data_command(data_opt)?,
        Commands::Config(config_opt) => run_config_command(config_opt)?,
        Commands::Supervisor(supervisor_opt) => run_supervisor_command(supervisor_opt)?,
        Commands::Dashboard(dashboard_opt) => run_dashboard_command(dashboard_opt).await?,
        Commands::Mvp(mvp_opt) => run_mvp_command(mvp_opt).await?,
        Commands::Workflow(workflow_opt) => run_workflow_command(workflow_opt)?,
        Commands::Database(database_opt) => run_database_command(database_opt).await?,
        #[cfg(feature = "defi")]
        Commands::Blockchain(blockchain_opt) => run_blockchain_command(blockchain_opt).await?,
    }
    Ok(())
}

/// Runs the sandbox-only `ntpro-node` local node entrypoint.
///
/// # Errors
///
/// Returns an error if config validation, node startup, node shutdown, or
/// artifact writing fails.
pub async fn run_ntpro_node(
    config: PathBuf,
    run_id: Option<String>,
    output: Option<PathBuf>,
    stop_file: Option<PathBuf>,
) -> anyhow::Result<()> {
    live::run_ntpro_node(config, run_id, output, stop_file).await
}

/// Runs the sandbox-only `ntpro-node` local node entrypoint with explicit
/// shutdown controls.
///
/// # Errors
///
/// Returns an error if config validation, node startup, node shutdown, or
/// artifact writing fails.
pub async fn run_ntpro_node_with_controls(
    config: PathBuf,
    run_id: Option<String>,
    output: Option<PathBuf>,
    stop_file: Option<PathBuf>,
    controls: NtproNodeRunControls,
) -> anyhow::Result<()> {
    live::run_ntpro_node_with_controls(config, run_id, output, stop_file, controls).await
}
