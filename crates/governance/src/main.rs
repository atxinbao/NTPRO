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
    io::{self, Read},
    path::PathBuf,
};

use clap::{Parser, Subcommand, ValueEnum};
use ntpro_governance::{
    backend_freeze::{BackendFreezeConfig, validate_backend_freeze},
    control_plane::validate_control_plane_retirement,
    docs_examples::validate_docs_examples,
    golden_trace::{replay_trace, validate_release_scope, validate_trace},
    historical_release::validate_historical_release_retirement,
    read_model::validate_read_model_schema,
    release_publication::{
        ReleaseBindingConfig, release_body_hash_report, timestamp_ge, validate_release_binding,
    },
    release_surface::{ReleaseSurfaceConfig, validate_release_surface},
    rust_examples::validate_rust_examples,
    zero_python::validate_zero_python_closeout,
};

#[derive(Debug, Parser)]
#[command(version, about = "Rust-only NTPRO repository governance tooling")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validates the immutable v0.32.0 backend freeze baseline.
    BackendFreeze {
        #[arg(
            long,
            default_value = "docs/rust-cutover/governance/backend_freeze_registry.json"
        )]
        registry: PathBuf,
        #[arg(
            long,
            default_value = "docs/rust-cutover/release/v0_32_0_release_manifest.json"
        )]
        release_manifest: PathBuf,
        #[arg(
            long,
            default_value = "docs/rust-cutover/governance/backend_freeze_policy.md"
        )]
        policy: PathBuf,
        #[arg(long, default_value = "README.md")]
        readme: PathBuf,
        #[arg(long, default_value = "ROADMAP.md")]
        roadmap: PathBuf,
        #[arg(long, default_value = "docs/versioning.md")]
        versioning: PathBuf,
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        negative_selftest: bool,
    },
    /// Validates retirement of legacy Python control-plane tooling.
    ControlPlaneRetirement,
    /// Validates the retained Rust docs and examples governance surface.
    DocsExamples,
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
    /// Validates retirement and Git recovery of historical release executables.
    HistoricalReleaseRetirement {
        #[arg(
            long,
            default_value = "docs/rust-cutover/governance/historical_release_executable_retirement.json"
        )]
        manifest: PathBuf,
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        negative_selftest: bool,
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
    /// Validates the current public release wording and source surface.
    ReleaseSurface {
        #[arg(long, default_value = "v0.32.0")]
        current_version: String,
        #[arg(long, default_value = "ntpro-rust-only-v0.32.0")]
        current_tag: String,
        #[arg(long, default_value = "backend-freeze-governance")]
        governance_track: String,
        #[arg(long, default_value = "v0.33.0+")]
        next_capability: String,
        #[arg(long, default_value = "v0.32.0 Backend Production Closeout")]
        current_capability: String,
        #[arg(long, default_value_t = false)]
        allow_missing_tag: bool,
    },
    /// Validates current release publish-after-gate source evidence.
    ReleasePublishBinding {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        closeout: PathBuf,
        #[arg(long)]
        version: String,
        #[arg(long)]
        tag: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        gate_run_id: u64,
        #[arg(long)]
        tag_sha: String,
    },
    /// Succeeds when the first RFC3339 timestamp is not before the second.
    TimestampGe {
        #[arg(long)]
        left: String,
        #[arg(long)]
        right: String,
    },
    /// Reports normalized and raw SHA-256 values for a release body and notes.
    ReleaseBodyHash {
        #[arg(long)]
        notes: PathBuf,
    },
    /// Validates canonical Rust example paths, TOML, and README references.
    RustExamples,
    /// Validates the repository-wide zero-Python tooling closeout.
    ZeroPythonCloseout {
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        negative_selftest: bool,
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
        Command::BackendFreeze {
            registry,
            release_manifest,
            policy,
            readme,
            roadmap,
            versioning,
            negative_selftest,
        } => {
            let counts = validate_backend_freeze(
                &BackendFreezeConfig {
                    registry,
                    release_manifest,
                    policy,
                    readme,
                    roadmap,
                    versioning,
                },
                negative_selftest,
            )?;
            println!(
                "backend_freeze_baseline=pass tag={} commit={} boundaries={} source_hashes={}",
                counts.tag, counts.commit, counts.boundaries, counts.source_hashes
            );
            if negative_selftest {
                println!(
                    "backend_freeze_negative_selftest=pass cases={}",
                    counts.negative_cases
                );
            }
        }
        Command::ControlPlaneRetirement => {
            let counts = validate_control_plane_retirement()?;
            println!(
                "control_plane_retirement=pass retired_tools={} authority_files={} inventory_rows={}",
                counts.retired_tools, counts.authority_files, counts.inventory_rows
            );
        }
        Command::DocsExamples => {
            let counts = validate_docs_examples()?;
            println!(
                "docs_examples_governance=pass markdown_files={} local_links={} image_links={} integration_pages={} python_fences_classified={} concept_pages={} tutorial_assets={}",
                counts.markdown_files,
                counts.local_links,
                counts.image_links,
                counts.integration_pages,
                counts.python_fences_classified,
                counts.concept_pages,
                counts.tutorial_assets
            );
        }
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
        Command::HistoricalReleaseRetirement {
            manifest,
            negative_selftest,
        } => {
            let counts = validate_historical_release_retirement(&manifest, negative_selftest)?;
            println!(
                "historical_release_retirement=pass retired={} python_tooling={} tags={} restored_blobs={} negative_cases={}",
                counts.retired,
                counts.tooling,
                counts.tags,
                counts.restored_blobs,
                counts.negative_cases
            );
        }
        Command::ReadModelSchema { schema, trace_glob } => {
            let count = validate_read_model_schema(&schema, &trace_glob)?;
            println!(
                "v211_read_model_schema_boundary status=ok validated_read_model_snapshots={count} negative_mutations=8 additional_properties=false boundary_flags=strict"
            );
        }
        Command::ZeroPythonCloseout { negative_selftest } => {
            let counts = validate_zero_python_closeout(negative_selftest)?;
            println!(
                "zero_python_closeout=pass tracked_files={} active_scripts={} workflow_actions={} historical_docs={} negative_cases={}",
                counts.tracked_files,
                counts.active_scripts,
                counts.workflow_actions,
                counts.historical_docs,
                counts.negative_cases
            );
        }
        Command::ReleaseSurface {
            current_version,
            current_tag,
            governance_track,
            next_capability,
            current_capability,
            allow_missing_tag,
        } => {
            validate_release_surface(&ReleaseSurfaceConfig {
                current_version,
                current_tag,
                governance_track,
                next_capability,
                current_capability,
                allow_missing_tag,
            })?;
            println!("release_surface_current_guard=pass");
        }
        Command::ReleasePublishBinding {
            manifest,
            closeout,
            version,
            tag,
            name,
            gate_run_id,
            tag_sha,
        } => {
            let result = validate_release_binding(&ReleaseBindingConfig {
                manifest,
                closeout,
                version,
                tag,
                name,
                gate_run_id,
                tag_sha,
            })?;
            println!(
                "release_publish_after_gate_current_binding=pass release_tag={} release_gate_run_id={} tag_sha={} historical_fixture_only_current_release_proof_allowed=false negative_selftest=1 negative_cases={}",
                result.release_tag,
                result.release_gate_run_id,
                result.tag_sha,
                result.negative_cases
            );
        }
        Command::TimestampGe { left, right } => {
            ensure_timestamp_order(&left, &right)?;
        }
        Command::ReleaseBodyHash { notes } => {
            let mut release_json = String::new();
            io::stdin().read_to_string(&mut release_json)?;
            println!("{}", release_body_hash_report(&release_json, &notes)?);
        }
        Command::RustExamples => {
            let counts = validate_rust_examples()?;
            println!(
                "rust_examples_integrity=pass required_paths={} toml_files={} readme_paths={}",
                counts.required_paths, counts.toml_files, counts.readme_paths
            );
        }
    }
    Ok(())
}

fn ensure_timestamp_order(left: &str, right: &str) -> anyhow::Result<()> {
    if !timestamp_ge(left, right)? {
        anyhow::bail!("timestamp is before required boundary: {left} < {right}");
    }
    Ok(())
}
