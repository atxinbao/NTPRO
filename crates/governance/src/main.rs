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

use clap::{Parser, Subcommand, ValueEnum};
use ntpro_governance::{
    golden_trace::{replay_trace, validate_release_scope, validate_trace},
    read_model::validate_read_model_schema,
};

#[derive(Debug, Parser)]
#[command(version, about = "Rust-only NTPRO repository governance tooling")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validates or replays one golden trace JSONL file.
    GoldenTrace {
        trace: PathBuf,
        #[arg(long, value_enum, default_value_t = GoldenTraceMode::ValidateOnly)]
        mode: GoldenTraceMode,
        #[arg(long, env = "GOLDEN_TRACE_REPLAY_COMMAND", default_value = "")]
        replay_command: String,
    },
    /// Validates the release replay/scope manifest against golden traces.
    GoldenTraceReleaseScope {
        #[arg(
            long,
            default_value = "docs/rust-cutover/golden_trace/RELEASE_REPLAY_SCOPE.json"
        )]
        manifest: PathBuf,
        #[arg(long, default_value = "tests/golden/*.jsonl")]
        trace_glob: String,
    },
    /// Validates v0.21 read-model snapshots and fail-closed boundaries.
    ReadModelSchema {
        #[arg(
            long,
            default_value = "docs/rust-cutover/release/v0_21_0_unified_read_model_schema.json"
        )]
        schema: PathBuf,
        #[arg(long, default_value = "tests/golden/**/*.jsonl")]
        trace_glob: String,
    },
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
enum GoldenTraceMode {
    #[default]
    ValidateOnly,
    Replay,
}

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::GoldenTrace {
            trace,
            mode,
            replay_command,
        } => {
            let rows = validate_trace(&trace)?;
            match mode {
                GoldenTraceMode::ValidateOnly => {
                    println!("valid trace: {} ({} rows)", trace.display(), rows.len());
                }
                GoldenTraceMode::Replay => {
                    if replay_command.is_empty() {
                        anyhow::bail!(
                            "replay mode requires --replay-command or GOLDEN_TRACE_REPLAY_COMMAND"
                        );
                    }
                    replay_trace(&trace, &replay_command, &rows)?;
                    println!("replay ok: {} ({} cases)", trace.display(), rows.len());
                }
            }
        }
        Command::GoldenTraceReleaseScope {
            manifest,
            trace_glob,
        } => {
            let counts = validate_release_scope(&manifest, &trace_glob)?;
            println!(
                "golden trace release scope ok: {} cases, {} executable replay, {} validator executable replay, {} schema-only scoped",
                counts.total(),
                counts.executable_replay,
                counts.validator_executable_replay,
                counts.schema_only_scoped
            );
        }
        Command::ReadModelSchema { schema, trace_glob } => {
            let count = validate_read_model_schema(&schema, &trace_glob)?;
            println!(
                "v211_read_model_schema_boundary status=ok validated_read_model_snapshots={count} negative_mutations=8 additional_properties=false boundary_flags=strict"
            );
        }
    }
    Ok(())
}
