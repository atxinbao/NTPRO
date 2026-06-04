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
//! The `nautilus-cli` crate provides a command-line interface for managing and
//! operating NTPRO workspaces. It includes tools for database management,
//! system configuration, and operational utilities:
//!
//! - Database initialization and management commands.
//! - PostgreSQL schema setup and maintenance.
//! - Configuration validation and setup utilities.
//! - System administration and operational tools.
//!
//! # NTPRO
//!
//! NTPRO is an open-source, production-grade, Rust-native
//! engine for multi-asset, multi-venue trading systems.
//!
//! The system spans research, deterministic simulation, and live execution within a single
//! event-driven architecture, providing research-to-live semantic parity.
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

mod backtest;
#[cfg(feature = "defi")]
mod blockchain;
mod config;
mod data;
mod database;
pub mod opt;
mod sandbox;

#[cfg(feature = "defi")]
use crate::blockchain::run_blockchain_command;
use crate::{
    backtest::run_backtest_command,
    config::run_config_command,
    data::run_data_command,
    database::postgres::run_database_command,
    opt::{Commands, LiveCommand, LiveOpt, NautilusCli},
    sandbox::run_sandbox_command,
};

/// Runs the NTPRO CLI based on the provided options.
///
/// # Errors
///
/// Returns an error if execution of the specified command fails.
pub async fn run(opt: NautilusCli) -> anyhow::Result<()> {
    match opt.command {
        Commands::Backtest(backtest_opt) => run_backtest_command(backtest_opt)?,
        Commands::Sandbox(sandbox_opt) => run_sandbox_command(sandbox_opt)?,
        Commands::Live(live_opt) => run_live_command(live_opt)?,
        Commands::Data(data_opt) => run_data_command(data_opt)?,
        Commands::Config(config_opt) => run_config_command(config_opt)?,
        Commands::Database(database_opt) => run_database_command(database_opt).await?,
        #[cfg(feature = "defi")]
        Commands::Blockchain(blockchain_opt) => run_blockchain_command(blockchain_opt).await?,
    }
    Ok(())
}

fn run_live_command(opt: LiveOpt) -> anyhow::Result<()> {
    match opt.command {
        LiveCommand::Validate(validate) => anyhow::bail!(
            "live validate is defined but not implemented yet for config '{}'; see docs/rust-cutover/product/LIVE_SANDBOX_CLI_CONTRACT.md",
            validate.config.display()
        ),
        LiveCommand::Run(run) => anyhow::bail!(
            "live run is defined but not implemented yet for config '{}'; see docs/rust-cutover/product/LIVE_SANDBOX_CLI_CONTRACT.md",
            run.config.display()
        ),
    }
}
