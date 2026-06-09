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
use nautilus_common::logging::ensure_logging_initialized;

/// Runs one local sandbox-only NTPRO node start/stop smoke.
#[derive(Debug, Parser)]
#[command(
    version,
    about = "Run a local sandbox-only NTPRO node process with no external venue access or real orders"
)]
struct NtproNodeCli {
    /// Path to the Rust live-init smoke config file.
    #[arg(long)]
    config: PathBuf,
    /// Optional owner-visible run identifier.
    #[arg(long)]
    run_id: Option<String>,
    /// Optional directory for node run artifacts.
    #[arg(long)]
    output: Option<PathBuf>,
    /// Optional file path the node watches before stopping.
    #[arg(long)]
    stop_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    ensure_logging_initialized();

    let opt = NtproNodeCli::parse();
    if let Err(e) =
        nautilus_cli::run_ntpro_node(opt.config, opt.run_id, opt.output, opt.stop_file).await
    {
        log::error!("Error executing ntpro-node: {e}");
        eprintln!("Error executing ntpro-node: {e}");
        std::process::exit(1);
    }
}
