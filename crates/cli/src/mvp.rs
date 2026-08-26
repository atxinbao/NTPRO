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

//! 单 Supervisor、单节点产品 MVP 的运行编排。

use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    future::Future,
    io::{ErrorKind, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, ensure};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use serde::{Deserialize, Serialize};

use crate::{
    artifacts::{atomic_write_json, remove_file_if_exists},
    backtest::{run_backtest_command, sha256_ref},
    dashboard::run_dashboard_command,
    mvp_contract::{
        MVP_IDENTITY_CONTRACT_PATH, MVP_STATUS_CONTRACT_PATH, MvpIdentityContract,
        MvpStatusContract,
    },
    opt::{
        BacktestCommand, BacktestOpt, BacktestRunOpt, DashboardCommand, DashboardOpt,
        DashboardServeOpt, MvpCommand, MvpOpt, MvpServeOpt,
    },
    supervisor::{
        RegisterNodeRequest, StartNodeRequest, StopNodeRequest, SupervisorProcessState,
        SupervisorRegistryStore,
    },
};

const REGISTRY_PATH: &str = "supervisor/registry.json";
const NODE_ARTIFACT_ROOT: &str = "nodes";
const PRODUCT_BACKTEST_ARTIFACT_ROOT: &str = "artifacts/backtests";
const STRATEGY_VERSION_REGISTRY_PATH: &str = "mvp/strategy_version_registry.json";
const STRATEGY_VERSION_REGISTRY_SCHEMA_VERSION: &str = "ntpro.mvp_strategy_version_registry.v1";
const MIN_STATUS_FRESHNESS_MAX_AGE_MS: u64 = 2_000;
static BASELINE_STAGING_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Deserialize)]
struct MvpBaselineConfigProjection {
    mvp: MvpBaselineSection,
    #[serde(default)]
    product_runs: Vec<MvpBaselineProductRun>,
}

#[derive(Debug, Deserialize)]
struct MvpBaselineSection {
    baseline_backtest_config: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct MvpBaselineProductRun {
    run_id: String,
    environment: String,
    lifecycle: String,
    result_status: String,
    result_ref: Option<String>,
    backtest_result_sha256: Option<String>,
    backtest_details_sha256: Option<String>,
    backtest_analysis_sha256: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MvpStrategyVersionRegistry {
    schema_version: String,
    versions: BTreeMap<String, BTreeMap<String, String>>,
}

impl Default for MvpStrategyVersionRegistry {
    fn default() -> Self {
        Self {
            schema_version: STRATEGY_VERSION_REGISTRY_SCHEMA_VERSION.to_string(),
            versions: BTreeMap::new(),
        }
    }
}

#[derive(Debug)]
struct MvpRuntime {
    store: SupervisorRegistryStore,
    node_id: String,
    registry_path: PathBuf,
    artifact_root: PathBuf,
    identity_contract: MvpIdentityContract,
    identity_contract_path: PathBuf,
    identity_contract_published: bool,
    status_contract_path: PathBuf,
    freshness_max_age_ms: u64,
}

impl MvpRuntime {
    #[cfg(test)]
    fn start(opt: &MvpServeOpt, ntpro_node_bin: PathBuf) -> anyhow::Result<Self> {
        Self::start_with_mode(opt, ntpro_node_bin, true)
    }

    fn prepare(opt: &MvpServeOpt, ntpro_node_bin: PathBuf) -> anyhow::Result<Self> {
        Self::start_with_mode(opt, ntpro_node_bin, false)
    }

    fn start_with_mode(
        opt: &MvpServeOpt,
        ntpro_node_bin: PathBuf,
        start_node: bool,
    ) -> anyhow::Result<Self> {
        ensure!(
            opt.bind.ip().is_loopback(),
            "MVP Dashboard 只能绑定本地 loopback 地址"
        );
        ensure!(opt.bind.port() > 0, "MVP Dashboard 监听端口必须大于 0");
        ensure!(
            opt.config.is_file(),
            "MVP 节点配置文件 '{}' 不存在",
            opt.config.display()
        );
        ensure!(
            ntpro_node_bin.is_file(),
            "ntpro-node 二进制 '{}' 不存在；请先执行 cargo build -p nautilus-cli --bins，或通过 --ntpro-node-bin 指定路径",
            ntpro_node_bin.display()
        );

        let identity_contract = MvpIdentityContract::load(&opt.config, &opt.node_id)?;
        let identity_contract_path = opt.workspace.join(MVP_IDENTITY_CONTRACT_PATH);
        let strategy_version_registry_path = opt.workspace.join(STRATEGY_VERSION_REGISTRY_PATH);
        let status_contract_path = opt.workspace.join(MVP_STATUS_CONTRACT_PATH);
        let strategy_version_registry_update = prepare_strategy_version_registry_update(
            &strategy_version_registry_path,
            &identity_contract_path,
            &identity_contract,
        )?;
        let freshness_max_age_ms = opt
            .node_heartbeat_interval_ms
            .saturating_mul(3)
            .max(MIN_STATUS_FRESHNESS_MAX_AGE_MS);

        let registry_path = opt.workspace.join(REGISTRY_PATH);
        let artifact_root = opt.workspace.join(NODE_ARTIFACT_ROOT).join(&opt.node_id);
        let store = SupervisorRegistryStore::new(&registry_path);
        let registry = store.load()?;
        ensure!(
            registry.nodes.is_empty()
                || (registry.nodes.len() == 1 && registry.nodes.contains_key(&opt.node_id)),
            "MVP 工作区 '{}' 只能包含节点 '{}'",
            opt.workspace.display(),
            opt.node_id
        );
        if let Some(existing) = registry.nodes.get(&opt.node_id) {
            validate_mvp_registry_record_paths(existing, &opt.config, &artifact_root)
                .with_context(|| {
                    format!(
                        "MVP 工作区 '{}' 的节点 '{}' 注册路径不可信",
                        opt.workspace.display(),
                        opt.node_id
                    )
                })?;
            let existing = store.refresh_process_state(&opt.node_id)?;
            ensure!(
                matches!(
                    existing.process.state,
                    SupervisorProcessState::NotStarted | SupervisorProcessState::Stopped
                ),
                "MVP 工作区 '{}' 已有活动或未决节点 '{}'，拒绝清理其合同",
                opt.workspace.display(),
                opt.node_id
            );
        }
        prepare_mvp_backtest_artifact_root(&opt.workspace)?;
        prepare_mvp_baseline_backtest(opt, &identity_contract)?;
        if let Some(registry) = strategy_version_registry_update {
            atomic_write_json(&strategy_version_registry_path, &registry).with_context(|| {
                format!(
                    "写入策略版本注册表 '{}' 失败",
                    strategy_version_registry_path.display()
                )
            })?;
        }
        remove_file_if_exists(&identity_contract_path).with_context(|| {
            format!(
                "清理旧 MVP 身份合同 '{}' 失败",
                identity_contract_path.display()
            )
        })?;
        remove_file_if_exists(&status_contract_path).with_context(|| {
            format!(
                "清理旧 MVP 四轴状态合同 '{}' 失败",
                status_contract_path.display()
            )
        })?;

        let existing_has_active_run = registry.nodes.get(&opt.node_id).is_some_and(|record| {
            record
                .run_ownership
                .values()
                .any(|ownership| ownership.terminal.is_none())
        });
        if existing_has_active_run {
            let existing = registry.nodes.get(&opt.node_id).with_context(|| {
                format!(
                    "MVP 工作区 '{}' 缺少预期节点 '{}'",
                    opt.workspace.display(),
                    opt.node_id
                )
            })?;
            ensure!(
                existing.config_path == opt.config && existing.artifact_root == artifact_root,
                "MVP 工作区 '{}' 的活动 Demo Run 绑定了不同节点配置或工件目录",
                opt.workspace.display()
            );
        } else {
            if registry.nodes.contains_key(&opt.node_id) {
                remove_mvp_runtime_artifacts_nofollow(&opt.workspace, &opt.node_id).with_context(
                    || {
                        format!(
                            "清理 MVP 工作区 '{}' 的节点 '{}' 旧运行时工件失败",
                            opt.workspace.display(),
                            opt.node_id
                        )
                    },
                )?;
            }
            store.register_node(RegisterNodeRequest {
                node_id: opt.node_id.clone(),
                config_path: opt.config.clone(),
                artifact_root: Some(artifact_root.clone()),
            })?;
        }
        let startup_timeout = duration_from_millis("startup_timeout_ms", opt.startup_timeout_ms)?;
        let node_shutdown_timeout =
            duration_from_millis("node_shutdown_timeout_ms", opt.node_shutdown_timeout_ms)?;
        if start_node {
            store.start_node_process(&StartNodeRequest {
                node_id: opt.node_id.clone(),
                ntpro_node_bin,
                startup_timeout,
                node_max_runtime: duration_from_millis(
                    "node_max_runtime_ms",
                    opt.node_max_runtime_ms,
                )?,
                node_heartbeat_interval: duration_from_millis(
                    "node_heartbeat_interval_ms",
                    opt.node_heartbeat_interval_ms,
                )?,
                node_parent_pid: Some(std::process::id()),
                node_shutdown_timeout,
            })?;
        }

        let mut runtime = Self {
            store,
            node_id: opt.node_id.clone(),
            registry_path,
            artifact_root,
            identity_contract,
            identity_contract_path,
            identity_contract_published: false,
            status_contract_path,
            freshness_max_age_ms,
        };
        let startup_result = (|| {
            if start_node {
                runtime.prepare_observability(startup_timeout)?;
            }
            atomic_write_json(&runtime.identity_contract_path, &runtime.identity_contract)
                .with_context(|| {
                    format!(
                        "写入 MVP 身份合同 '{}' 失败",
                        runtime.identity_contract_path.display()
                    )
                })?;
            runtime.identity_contract_published = true;
            runtime
                .write_status_contract()
                .context("初始化 MVP 四轴状态合同失败")?;
            Ok(())
        })();
        if let Err(readiness_error) = startup_result {
            let cleanup_result = runtime.stop(node_shutdown_timeout);
            return match cleanup_result {
                Ok(()) => Err(readiness_error),
                Err(cleanup_error) => Err(readiness_error.context(format!(
                    "MVP 可观测性初始化失败后的节点清理同时失败: {cleanup_error:#}"
                ))),
            };
        }
        Ok(runtime)
    }

    fn prepare_observability(&self, timeout: Duration) -> anyhow::Result<()> {
        let started = Instant::now();
        loop {
            let record = self.store.load()?.nodes.get(&self.node_id).cloned();
            let Some(record) = record else {
                anyhow::bail!("MVP 节点 '{}' 未出现在注册表中", self.node_id);
            };
            if record.metrics_path.exists() {
                self.store
                    .node_metrics(&self.node_id)
                    .context("MVP 节点指标工件无效")?;
                let mut events = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&record.events_log_path)
                    .with_context(|| {
                        format!(
                            "打开 MVP 启动事件日志 '{}' 失败",
                            record.events_log_path.display()
                        )
                    })?;
                writeln!(
                    events,
                    "phase=mvp_start status=ok node_id={} external_venue_connection=false real_orders_submitted=false",
                    self.node_id
                )
                .with_context(|| {
                    format!(
                        "追加 MVP 启动事件 '{}' 失败",
                        record.events_log_path.display()
                    )
                })?;
                return Ok(());
            }
            if started.elapsed() >= timeout {
                anyhow::bail!(
                    "MVP 节点 '{}' 启动后未在 {:?} 内生成指标工件",
                    self.node_id,
                    timeout
                );
            }
            thread::sleep(Duration::from_millis(25));
        }
    }

    fn stop(&self, timeout: Duration) -> anyhow::Result<()> {
        crate::dashboard::shutdown_active_demo_run(&self.registry_path, timeout)
            .context("MVP 退出时收口活动 Demo Run 失败")?;
        let record = self.store.refresh_process_state(&self.node_id)?;
        match record.process.state {
            SupervisorProcessState::Running => {
                self.store.stop_node_process(&StopNodeRequest {
                    node_id: self.node_id.clone(),
                    stop_timeout: timeout,
                })?;
            }
            SupervisorProcessState::NotStarted | SupervisorProcessState::Stopped => {}
            SupervisorProcessState::Stale | SupervisorProcessState::Unknown => {
                anyhow::bail!(
                    "MVP 节点 '{}' 停止时处于 {:?} 状态，需要人工检查 {}",
                    self.node_id,
                    record.process.state,
                    self.artifact_root.display()
                );
            }
        }
        if self.identity_contract_published {
            self.write_status_contract()
                .context("更新 MVP 停止状态合同失败")?;
        }
        Ok(())
    }

    fn write_status_contract(&self) -> anyhow::Result<MvpStatusContract> {
        let identity_error = self.identity_contract_error();
        let status_error = self
            .store
            .refresh_status_from_artifact(&self.node_id)
            .err()
            .map(|error| format!("{error:#}"));
        let metrics_result = self.store.node_metrics(&self.node_id);
        let metrics_error = metrics_result
            .as_ref()
            .err()
            .map(|error| format!("{error:#}"));
        let registry = self.store.load()?;
        let record = registry
            .nodes
            .get(&self.node_id)
            .with_context(|| format!("MVP 节点 '{}' 未出现在注册表中", self.node_id))?;
        let contract = MvpStatusContract::from_runtime(
            &self.identity_contract,
            &self.identity_contract_path,
            &self.registry_path,
            record,
            metrics_result.as_ref().ok(),
            status_error.as_deref(),
            metrics_error.as_deref(),
            identity_error.as_deref(),
            self.freshness_max_age_ms,
        );
        atomic_write_json(&self.status_contract_path, &contract).with_context(|| {
            format!(
                "写入 MVP 四轴状态合同 '{}' 失败",
                self.status_contract_path.display()
            )
        })?;
        Ok(contract)
    }

    fn identity_contract_error(&self) -> Option<String> {
        let raw = match std::fs::read_to_string(&self.identity_contract_path) {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Some("MVP identity contract is missing".to_string());
            }
            Err(error) => {
                return Some(format!("cannot read MVP identity contract: {error}"));
            }
        };
        let published = match serde_json::from_str::<MvpIdentityContract>(&raw) {
            Ok(contract) => contract,
            Err(error) => return Some(format!("invalid MVP identity contract: {error}")),
        };
        (published != self.identity_contract)
            .then(|| "MVP identity contract does not match current runtime identity".to_string())
    }
}

fn prepare_mvp_baseline_backtest(
    opt: &MvpServeOpt,
    identity: &MvpIdentityContract,
) -> anyhow::Result<bool> {
    let raw = fs::read_to_string(&opt.config)
        .with_context(|| format!("读取 MVP 基线配置 '{}' 失败", opt.config.display()))?;
    let projection: MvpBaselineConfigProjection = toml::from_str(&raw)
        .with_context(|| format!("解析 MVP 基线配置 '{}' 失败", opt.config.display()))?;
    let Some(config_ref) = projection.mvp.baseline_backtest_config else {
        return Ok(false);
    };
    let run_id = identity.identities.backtest_run_id.as_str();
    ensure_safe_baseline_run_id(run_id)?;
    let run = projection
        .product_runs
        .iter()
        .find(|run| run.run_id == run_id)
        .with_context(|| format!("MVP 默认基线 Run '{run_id}' 未出现在 product_runs"))?;
    ensure!(
        run.environment == "backtest"
            && run.lifecycle == "completed"
            && run.result_status == "available",
        "MVP 默认基线 Run '{run_id}' 必须声明为 completed/available backtest"
    );
    ensure!(
        run.result_ref.as_deref() == Some(identity.identities.backtest_result_ref.as_str()),
        "MVP 默认基线 Run '{run_id}' 的 result_ref 与身份合同不一致"
    );
    let expected = [
        (
            "summary.json",
            required_baseline_hash(
                run_id,
                "backtest_result_sha256",
                &run.backtest_result_sha256,
            )?,
        ),
        (
            "details.json",
            required_baseline_hash(
                run_id,
                "backtest_details_sha256",
                &run.backtest_details_sha256,
            )?,
        ),
        (
            "analysis.json",
            required_baseline_hash(
                run_id,
                "backtest_analysis_sha256",
                &run.backtest_analysis_sha256,
            )?,
        ),
    ];
    let artifact_root_path = opt.workspace.join(PRODUCT_BACKTEST_ARTIFACT_ROOT);
    let artifact_root = prepare_mvp_backtest_artifact_root(&opt.workspace)?;
    let run_root = artifact_root_path.join(run_id);
    match artifact_root.symlink_metadata(run_id) {
        Ok(metadata) => {
            ensure!(
                metadata.is_dir() && !metadata.file_type().is_symlink(),
                "MVP 默认基线工件路径 '{}' 不是受控目录",
                run_root.display()
            );
            let run_directory = artifact_root.open_dir_nofollow(run_id).with_context(|| {
                format!("打开 MVP 默认基线工件目录 '{}' 失败", run_root.display())
            })?;
            return validate_mvp_baseline_artifacts(&run_directory, &run_root, run_id, &expected)
                .map(|()| false);
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| {
                format!("检查 MVP 默认基线工件路径 '{}' 失败", run_root.display())
            });
        }
    }

    let backtest_config = if config_ref.is_absolute() {
        config_ref
    } else {
        opt.config
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(config_ref)
    };
    ensure!(
        backtest_config.is_file(),
        "MVP 默认基线 Backtest 配置 '{}' 不存在",
        backtest_config.display()
    );
    let sequence = BASELINE_STAGING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let staging_name = format!(".{run_id}.bootstrap-{}-{sequence}", std::process::id());
    let staging = artifact_root_path.join(&staging_name);
    artifact_root
        .create_dir(&staging_name)
        .with_context(|| format!("创建 MVP 默认基线暂存目录 '{}' 失败", staging.display()))?;
    let generation = run_backtest_command(BacktestOpt {
        command: BacktestCommand::Run(BacktestRunOpt {
            config: backtest_config,
            run_id: None,
            output: Some(staging.clone()),
            dry_run: false,
        }),
    })
    .and_then(|()| {
        let staging_directory = artifact_root
            .open_dir_nofollow(&staging_name)
            .with_context(|| format!("打开 MVP 默认基线暂存目录 '{}' 失败", staging.display()))?;
        validate_mvp_baseline_artifacts(&staging_directory, &staging, run_id, &expected)
    })
    .and_then(|()| {
        artifact_root
            .rename(&staging_name, &artifact_root, run_id)
            .with_context(|| format!("发布 MVP 默认基线工件 '{}' 失败", run_root.display()))
    });
    if let Err(error) = generation {
        let cleanup = remove_mvp_baseline_staging(&artifact_root, &staging_name, &staging);
        return match cleanup {
            Ok(()) => Err(error),
            Err(cleanup_error) => Err(error.context(format!(
                "清理失败的默认基线暂存目录同时失败: {cleanup_error:#}"
            ))),
        };
    }
    println!(
        "mvp.baseline_backtest status=ready run_id={run_id} artifact_root={} generated=true external_venue_connection=false real_orders_submitted=false",
        run_root.display()
    );
    Ok(true)
}

fn prepare_mvp_backtest_artifact_root(workspace: &Path) -> anyhow::Result<cap_std::fs::Dir> {
    fs::create_dir_all(workspace)
        .with_context(|| format!("创建 MVP workspace '{}' 失败", workspace.display()))?;
    let workspace_directory =
        cap_std::fs::Dir::open_ambient_dir(workspace, cap_std::ambient_authority())
            .with_context(|| format!("打开 MVP workspace '{}' 失败", workspace.display()))?;
    open_or_create_mvp_directory(&workspace_directory, "catalog", workspace)?;
    let artifacts = open_or_create_mvp_directory(&workspace_directory, "artifacts", workspace)?;
    open_or_create_mvp_directory(&artifacts, "backtests", &workspace.join("artifacts"))
}

fn open_or_create_mvp_directory(
    parent: &cap_std::fs::Dir,
    name: &str,
    parent_path: &Path,
) -> anyhow::Result<cap_std::fs::Dir> {
    match parent.create_dir(name) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "创建 MVP 受控目录 '{}' 失败",
                    parent_path.join(name).display()
                )
            });
        }
    }
    parent.open_dir_nofollow(name).with_context(|| {
        format!(
            "MVP 受控目录 '{}' 不是普通目录或包含符号链接",
            parent_path.join(name).display()
        )
    })
}

fn required_baseline_hash<'a>(
    run_id: &str,
    field: &str,
    value: &'a Option<String>,
) -> anyhow::Result<&'a str> {
    let value = value
        .as_deref()
        .with_context(|| format!("MVP 默认基线 Run '{run_id}' 缺少 {field}"))?;
    ensure!(
        value.len() == 71
            && value.starts_with("sha256:")
            && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit()),
        "MVP 默认基线 Run '{run_id}' 的 {field} 不是 SHA-256 引用"
    );
    Ok(value)
}

fn ensure_safe_baseline_run_id(run_id: &str) -> anyhow::Result<()> {
    ensure!(
        !run_id.is_empty()
            && run_id != "."
            && run_id != ".."
            && run_id
                .bytes()
                .all(|byte| { byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.') }),
        "MVP 默认基线 run_id '{run_id}' 不能作为安全工件目录"
    );
    Ok(())
}

fn validate_mvp_baseline_artifacts(
    run_directory: &cap_std::fs::Dir,
    run_root: &Path,
    run_id: &str,
    expected: &[(&str, &str); 3],
) -> anyhow::Result<()> {
    for (file_name, expected_sha256) in expected {
        let path = run_root.join(file_name);
        let mut options = cap_std::fs::OpenOptions::new();
        options.read(true);
        options.follow(FollowSymlinks::No);
        let mut file = run_directory
            .open_with(file_name, &options)
            .with_context(|| format!("打开 MVP 默认基线工件 '{}' 失败", path.display()))?;
        let metadata = file
            .metadata()
            .with_context(|| format!("检查 MVP 默认基线工件 '{}' 失败", path.display()))?;
        ensure!(
            metadata.is_file(),
            "MVP 默认基线工件 '{}' 不是受控文件",
            path.display()
        );
        let mut raw = Vec::new();
        file.read_to_end(&mut raw)
            .with_context(|| format!("读取 MVP 默认基线工件 '{}' 失败", path.display()))?;
        ensure!(
            sha256_ref(&raw) == *expected_sha256,
            "MVP 默认基线工件 '{}' 的 SHA-256 与节点配置不一致",
            path.display()
        );
        let artifact: serde_json::Value = serde_json::from_slice(&raw)
            .with_context(|| format!("解析 MVP 默认基线工件 '{}' 失败", path.display()))?;
        ensure!(
            artifact.get("run_id").and_then(serde_json::Value::as_str) == Some(run_id),
            "MVP 默认基线工件 '{}' 的 run_id 不一致",
            path.display()
        );
    }
    Ok(())
}

fn remove_mvp_baseline_staging(
    artifact_root: &cap_std::fs::Dir,
    name: &str,
    path: &Path,
) -> anyhow::Result<()> {
    let metadata = match artifact_root.symlink_metadata(name) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("检查默认基线暂存路径 '{}' 失败", path.display()));
        }
    };
    ensure!(
        metadata.file_type().is_dir() && !metadata.file_type().is_symlink(),
        "默认基线暂存路径 '{}' 不是受控目录",
        path.display()
    );
    artifact_root
        .remove_dir_all(name)
        .with_context(|| format!("清理默认基线暂存目录 '{}' 失败", path.display()))
}

fn validate_mvp_registry_record_paths(
    record: &crate::supervisor::SupervisorNodeRecord,
    config_path: &Path,
    artifact_root: &Path,
) -> anyhow::Result<()> {
    ensure!(record.config_path == config_path, "config_path 不匹配");
    ensure!(
        record.artifact_root == artifact_root,
        "artifact_root 不匹配"
    );
    for (field, actual, expected) in [
        ("pid_path", &record.pid_path, artifact_root.join("pid.json")),
        (
            "status_path",
            &record.status_path,
            artifact_root.join("status.json"),
        ),
        (
            "metrics_path",
            &record.metrics_path,
            artifact_root.join("metrics.json"),
        ),
        (
            "stdout_log_path",
            &record.stdout_log_path,
            artifact_root.join("logs/stdout.log"),
        ),
        (
            "stderr_log_path",
            &record.stderr_log_path,
            artifact_root.join("logs/stderr.log"),
        ),
        (
            "events_log_path",
            &record.events_log_path,
            artifact_root.join("logs/events.log"),
        ),
    ] {
        ensure!(actual == &expected, "{field} 不匹配");
    }
    Ok(())
}

fn remove_mvp_runtime_artifacts_nofollow(workspace: &Path, node_id: &str) -> anyhow::Result<()> {
    let workspace = cap_std::fs::Dir::open_ambient_dir(workspace, cap_std::ambient_authority())
        .context("打开 MVP workspace 失败")?;
    let nodes = workspace
        .open_dir_nofollow("nodes")
        .context("打开 MVP nodes 目录失败")?;
    let artifact_root = nodes
        .open_dir_nofollow(node_id)
        .context("打开 MVP node 工件目录失败")?;
    for name in ["pid.json", "status.json", "metrics.json"] {
        match artifact_root.remove_file(name) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error).with_context(|| format!("删除 '{name}' 失败")),
        }
    }
    Ok(())
}

fn prepare_strategy_version_registry_update(
    registry_path: &Path,
    previous_contract_path: &Path,
    current_contract: &MvpIdentityContract,
) -> anyhow::Result<Option<MvpStrategyVersionRegistry>> {
    let previous_contract = if previous_contract_path.is_file() {
        let raw = std::fs::read_to_string(previous_contract_path).with_context(|| {
            format!(
                "读取上一份 MVP 身份合同 '{}' 失败",
                previous_contract_path.display()
            )
        })?;
        Some(
            serde_json::from_str::<MvpIdentityContract>(&raw).with_context(|| {
                format!(
                    "上一份 MVP 身份合同 '{}' 无效，拒绝覆盖",
                    previous_contract_path.display()
                )
            })?,
        )
    } else {
        None
    };
    let has_previous_anchor = previous_contract
        .as_ref()
        .is_some_and(|contract| !contract.identities.strategy_version_content_hash.is_empty());
    let current = &current_contract.identities;
    let has_current_anchor = !current.strategy_version_content_hash.is_empty();
    if !registry_path.is_file() && !has_previous_anchor && !has_current_anchor {
        return Ok(None);
    }

    let mut registry = if registry_path.is_file() {
        let raw = std::fs::read_to_string(registry_path)
            .with_context(|| format!("读取策略版本注册表 '{}' 失败", registry_path.display()))?;
        let registry: MvpStrategyVersionRegistry = serde_json::from_str(&raw)
            .with_context(|| format!("策略版本注册表 '{}' 无效", registry_path.display()))?;
        ensure!(
            registry.schema_version == STRATEGY_VERSION_REGISTRY_SCHEMA_VERSION,
            "策略版本注册表 '{}' schema_version 必须为 '{}'",
            registry_path.display(),
            STRATEGY_VERSION_REGISTRY_SCHEMA_VERSION
        );
        registry
    } else {
        MvpStrategyVersionRegistry::default()
    };

    let mut changed = false;
    if let Some(previous) = previous_contract.as_ref() {
        changed |= register_strategy_version_anchor(&mut registry, previous)?;
    }
    if has_current_anchor {
        changed |= register_strategy_version_anchor(&mut registry, current_contract)?;
    } else if registry
        .versions
        .get(&current.strategy_id)
        .and_then(|versions| versions.get(&current.strategy_version))
        .is_some()
    {
        anyhow::bail!(
            "策略版本 '{}@{}' 已登记内容哈希，当前配置不得移除该锚点",
            current.strategy_id,
            current.strategy_version
        );
    }

    Ok(changed.then_some(registry))
}

fn register_strategy_version_anchor(
    registry: &mut MvpStrategyVersionRegistry,
    contract: &MvpIdentityContract,
) -> anyhow::Result<bool> {
    let identity = &contract.identities;
    if identity.strategy_version_content_hash.is_empty() {
        return Ok(false);
    }
    let versions = registry
        .versions
        .entry(identity.strategy_id.clone())
        .or_default();
    if let Some(registered_hash) = versions.get(&identity.strategy_version) {
        ensure!(
            registered_hash == &identity.strategy_version_content_hash,
            "策略版本 '{}@{}' 已登记为不可变内容；内容哈希从 '{}' 变为 '{}'，请创建新版本号",
            identity.strategy_id,
            identity.strategy_version,
            registered_hash,
            identity.strategy_version_content_hash
        );
        return Ok(false);
    }
    versions.insert(
        identity.strategy_version.clone(),
        identity.strategy_version_content_hash.clone(),
    );
    Ok(true)
}

/// 执行本地单节点 MVP 命令。
///
/// # Errors
///
/// 当配置、节点进程、Dashboard 服务或节点清理失败时返回错误。
pub(crate) async fn run_mvp_command(opt: MvpOpt) -> anyhow::Result<()> {
    match opt.command {
        MvpCommand::Serve(serve) => run_mvp_serve(serve).await,
    }
}

async fn run_mvp_serve(opt: MvpServeOpt) -> anyhow::Result<()> {
    crate::dashboard::validate_strategy_workbench_dist(&opt.strategy_workbench_dist)?;
    let ntpro_node_bin = opt
        .ntpro_node_bin
        .clone()
        .unwrap_or_else(default_ntpro_node_bin_path);
    let stop_timeout =
        duration_from_millis("node_shutdown_timeout_ms", opt.node_shutdown_timeout_ms)?;
    let status_refresh_interval =
        duration_from_millis("node_heartbeat_interval_ms", opt.node_heartbeat_interval_ms)?;
    let runtime = MvpRuntime::prepare(&opt, ntpro_node_bin.clone())?;

    println!(
        "mvp.serve status=ready node_id={} node_process=stopped registry={} artifact_root={} identity_contract={} status_contract={} dashboard_bind={} portal_access=dashboard_bootstrap_output external_venue_connection=false real_orders_submitted=false",
        runtime.node_id,
        runtime.registry_path.display(),
        runtime.artifact_root.display(),
        runtime.identity_contract_path.display(),
        runtime.status_contract_path.display(),
        opt.bind,
    );

    let dashboard = run_dashboard_command(DashboardOpt {
        command: DashboardCommand::Serve(DashboardServeOpt {
            registry: runtime.registry_path.clone(),
            workflow_root: None,
            bind: opt.bind,
            strategy_workbench_dist: opt.strategy_workbench_dist.clone(),
            ntpro_node_bin: Some(ntpro_node_bin),
        }),
    });
    tokio::pin!(dashboard);
    let mut status_refresh = tokio::time::interval(status_refresh_interval);
    status_refresh.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let shutdown_signal = tokio::signal::ctrl_c();
    tokio::pin!(shutdown_signal);

    let serve_result = wait_for_mvp_serve_exit(
        &mut dashboard,
        &mut shutdown_signal,
        &mut status_refresh,
        || runtime.write_status_contract().map(|_| ()),
    )
    .await;
    let stop_result = runtime.stop(stop_timeout).context("MVP 退出时停止节点失败");

    match (serve_result, stop_result) {
        (Err(serve_error), Err(stop_error)) => {
            Err(serve_error.context(format!("节点清理同时失败: {stop_error:#}")))
        }
        (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
        (Ok(()), Ok(())) => {
            println!(
                "mvp.serve status=stopped node_id={} external_venue_connection=false real_orders_submitted=false",
                opt.node_id
            );
            Ok(())
        }
    }
}

async fn wait_for_mvp_serve_exit<Dashboard, Shutdown, Refresh>(
    dashboard: &mut Dashboard,
    shutdown_signal: &mut Shutdown,
    status_refresh: &mut tokio::time::Interval,
    mut refresh_status: Refresh,
) -> anyhow::Result<()>
where
    Dashboard: Future<Output = anyhow::Result<()>> + Unpin,
    Shutdown: Future<Output = std::io::Result<()>> + Unpin,
    Refresh: FnMut() -> anyhow::Result<()>,
{
    loop {
        tokio::select! {
            result = &mut *dashboard => break result.context("MVP Dashboard 服务已退出"),
            result = &mut *shutdown_signal => {
                result.context("等待 MVP Ctrl-C 终止信号失败")?;
                break Ok(());
            }
            _ = status_refresh.tick() => {
                if let Err(error) = refresh_status() {
                    break Err(error.context("刷新 MVP 四轴状态合同失败"));
                }
            }
        }
    }
}

fn duration_from_millis(field: &str, millis: u64) -> anyhow::Result<Duration> {
    ensure!(millis > 0, "{field} 必须大于 0");
    Ok(Duration::from_millis(millis))
}

fn default_ntpro_node_bin_path() -> PathBuf {
    std::env::current_exe().map_or_else(
        |_| PathBuf::from("ntpro-node"),
        |path| {
            let file_name = if cfg!(windows) {
                "ntpro-node.exe"
            } else {
                "ntpro-node"
            };
            path.with_file_name(file_name)
        },
    )
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        net::{IpAddr, Ipv4Addr, SocketAddr},
        path::Path,
        sync::{
            Arc, Barrier, Mutex, MutexGuard,
            atomic::{AtomicUsize, Ordering as AtomicOrdering},
        },
        time::{SystemTime, UNIX_EPOCH},
    };

    #[cfg(unix)]
    use std::os::unix::fs::{PermissionsExt, symlink};

    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Method, Request, StatusCode},
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;
    use crate::supervisor::RegistryArtifactState;

    #[cfg(unix)]
    static MVP_PROCESS_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[cfg(unix)]
    fn process_test_guard() -> MutexGuard<'static, ()> {
        MVP_PROCESS_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn temp_root(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must follow Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "ntpro-mvp-001-{name}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temporary MVP root must be created");
        root
    }

    fn mvp_options(root: &Path, ntpro_node_bin: PathBuf) -> MvpServeOpt {
        let config = root.join("node.toml");
        fs::write(
            &config,
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
"#,
        )
        .expect("fixture config must be written");
        MvpServeOpt {
            config,
            workspace: root.join("workspace"),
            node_id: "mvp-node-001".to_string(),
            bind: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 51_973),
            strategy_workbench_dist: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/strategy-workbench"),
            ntpro_node_bin: Some(ntpro_node_bin),
            startup_timeout_ms: 2_000,
            node_max_runtime_ms: 60_000,
            node_heartbeat_interval_ms: 50,
            node_shutdown_timeout_ms: 2_000,
        }
    }

    fn mvp_product_options(root: &Path, ntpro_node_bin: PathBuf) -> MvpServeOpt {
        let mut opt = mvp_options(root, ntpro_node_bin);
        opt.config = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../configs/nodes/btc-ema-shadow.toml");
        opt.node_id = "mvp-node-001".to_string();
        opt
    }

    async fn mvp_router_json(
        router: &Router,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> (StatusCode, Value) {
        let mut request = Request::builder().method(method).uri(path);
        let request_body = if let Some(body) = body {
            request = request.header("content-type", "application/json");
            Body::from(serde_json::to_vec(body).expect("request body should serialize"))
        } else {
            Body::empty()
        };
        let response = router
            .clone()
            .oneshot(request.body(request_body).expect("request should build"))
            .await
            .expect("router request should complete");
        let status = response.status();
        let raw = to_bytes(response.into_body(), 2 * 1024 * 1024)
            .await
            .expect("response body should be readable");
        let value = serde_json::from_slice(&raw).expect("response should be valid JSON");
        (status, value)
    }

    fn read_json(path: &Path) -> serde_json::Value {
        serde_json::from_str(&fs::read_to_string(path).unwrap_or_else(|error| {
            panic!("fixture '{}' should be readable: {error}", path.display())
        }))
        .unwrap_or_else(|error| {
            panic!("fixture '{}' should be valid JSON: {error}", path.display())
        })
    }

    fn write_json(path: &Path, value: &serde_json::Value) {
        fs::write(
            path,
            serde_json::to_string(value).expect("fixture JSON should serialize"),
        )
        .unwrap_or_else(|error| panic!("fixture '{}' should be writable: {error}", path.display()));
    }

    fn test_unix_time_ms() -> u64 {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must follow Unix epoch")
            .as_millis();
        u64::try_from(millis).unwrap_or(u64::MAX)
    }

    #[tokio::test]
    async fn mvp_serve_exit_keeps_shutdown_listener_across_status_refreshes() {
        let dashboard = std::future::pending::<anyhow::Result<()>>();
        tokio::pin!(dashboard);
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let shutdown_signal = async move {
            shutdown_rx
                .await
                .map_err(|_| std::io::Error::other("test shutdown sender dropped"))
        };
        tokio::pin!(shutdown_signal);
        let mut status_refresh = tokio::time::interval(Duration::from_millis(1));
        status_refresh.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let refresh_count = Arc::new(AtomicUsize::new(0));
        let observed_refresh_count = Arc::clone(&refresh_count);

        let serve = wait_for_mvp_serve_exit(
            &mut dashboard,
            &mut shutdown_signal,
            &mut status_refresh,
            || {
                refresh_count.fetch_add(1, AtomicOrdering::SeqCst);
                Ok(())
            },
        );
        let send_shutdown = async move {
            while observed_refresh_count.load(AtomicOrdering::SeqCst) < 3 {
                tokio::task::yield_now().await;
            }
            shutdown_tx
                .send(())
                .expect("test shutdown receiver should remain registered");
        };

        let (serve_result, ()) = tokio::time::timeout(Duration::from_secs(1), async {
            tokio::join!(serve, send_shutdown)
        })
        .await
        .expect("MVP serve loop should consume shutdown after repeated refreshes");
        serve_result.expect("test shutdown signal should stop the MVP serve loop");
    }

    #[cfg(unix)]
    #[test]
    fn mvp_runtime_starts_and_stops_exactly_one_local_node() {
        let _guard = process_test_guard();
        let root = temp_root("start-stop");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_options(&root, fixture_node.clone());
        let events_path = opt
            .workspace
            .join(NODE_ARTIFACT_ROOT)
            .join(&opt.node_id)
            .join("logs/events.log");
        fs::create_dir_all(
            events_path
                .parent()
                .expect("events log should have a parent"),
        )
        .expect("events log directory should be created");
        fs::write(&events_path, "phase=existing status=ok\n")
            .expect("existing event fixture should be written");

        let runtime = MvpRuntime::start(&opt, fixture_node.clone())
            .expect("MVP runtime should start fixture node");
        let running = runtime
            .store
            .load()
            .expect("registry should load after start");
        assert_eq!(running.nodes.len(), 1);
        let record = running
            .nodes
            .get("mvp-node-001")
            .expect("MVP node should be registered");
        assert_eq!(record.process.state, SupervisorProcessState::Running);
        assert_eq!(
            record.metrics_artifact,
            crate::supervisor::RegistryArtifactState::Available
        );
        assert!(record.events_log_path.is_file());
        let events =
            fs::read_to_string(&events_path).expect("MVP events log should remain readable");
        assert!(events.contains("phase=existing status=ok"));
        assert!(events.contains("phase=mvp_start status=ok"));
        assert!(!record.last_known_status.external_venue_connection);
        assert!(!record.last_known_status.real_orders_submitted);
        let identity_contract: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(&runtime.identity_contract_path)
                .expect("MVP identity contract should be readable"),
        )
        .expect("MVP identity contract should be valid JSON");
        assert_eq!(
            identity_contract["schema_version"],
            crate::mvp_contract::MVP_IDENTITY_CONTRACT_SCHEMA_VERSION
        );
        assert_eq!(identity_contract["identities"]["node_id"], "mvp-node-001");
        assert_eq!(
            identity_contract["identities"]["strategy_instance_id"],
            "strategy-instance-alpha"
        );
        assert_eq!(
            identity_contract["boundaries"]["order_submission_allowed"],
            false
        );
        let status_contract: MvpStatusContract = serde_json::from_str(
            &fs::read_to_string(&runtime.status_contract_path)
                .expect("MVP status contract should be readable"),
        )
        .expect("MVP status contract should be valid JSON");
        assert_eq!(
            status_contract.schema_version,
            crate::mvp_contract::MVP_STATUS_CONTRACT_SCHEMA_VERSION
        );
        assert_eq!(
            status_contract.research.status,
            crate::mvp_contract::MvpResearchStatus::ReferenceBound
        );
        assert_eq!(
            status_contract.research.freshness,
            crate::mvp_contract::MvpStatusFreshness::Unknown
        );
        assert!(
            status_contract
                .research
                .reasons
                .contains(&"research_acceptance_not_claimed".to_string())
        );
        assert_eq!(
            status_contract.runtime.status,
            crate::mvp_contract::MvpRuntimeStatus::Running
        );
        assert_eq!(
            status_contract.technical_health.status,
            crate::mvp_contract::MvpTechnicalHealth::Healthy
        );
        assert_eq!(
            status_contract.trading_readiness.status,
            crate::mvp_contract::MvpTradingReadiness::Blocked
        );
        assert_eq!(
            status_contract.trading_readiness.availability,
            crate::mvp_contract::MvpStatusAvailability::Missing
        );
        assert!(
            status_contract
                .trading_readiness
                .reasons
                .contains(&"missing_unified_read_model".to_string())
        );
        assert!(status_contract.boundaries.read_only_product_contract);
        assert!(
            !status_contract
                .boundaries
                .http_success_implies_technical_health
        );
        assert!(
            !status_contract
                .boundaries
                .process_alive_implies_technical_health
        );
        assert!(
            !status_contract
                .boundaries
                .backtest_reference_implies_research_accepted
        );
        assert!(
            !status_contract
                .boundaries
                .backtest_complete_implies_trading_readiness
        );
        assert!(!status_contract.boundaries.order_submission_allowed);
        assert!(!status_contract.boundaries.order_mutation_allowed);
        assert!(!status_contract.boundaries.automatic_retry_allowed);
        assert!(!status_contract.boundaries.automatic_remediation_allowed);
        assert!(!status_contract.boundaries.external_venue_connection);
        assert!(!status_contract.boundaries.real_orders_submitted);

        runtime
            .stop(Duration::from_secs(2))
            .expect("MVP runtime should stop fixture node");
        let stopped = runtime
            .store
            .load()
            .expect("registry should load after stop");
        assert_eq!(
            stopped.nodes["mvp-node-001"].process.state,
            SupervisorProcessState::Stopped
        );
        let stopped_contract: MvpStatusContract = serde_json::from_str(
            &fs::read_to_string(&runtime.status_contract_path)
                .expect("stopped MVP status contract should be readable"),
        )
        .expect("stopped MVP status contract should be valid JSON");
        assert_eq!(
            stopped_contract.runtime.status,
            crate::mvp_contract::MvpRuntimeStatus::Stopped
        );
        assert_eq!(
            stopped_contract.technical_health.status,
            crate::mvp_contract::MvpTechnicalHealth::NotRunning
        );

        let restarted = MvpRuntime::start(&opt, fixture_node)
            .expect("MVP runtime should restart in the same workspace");
        let restarted_registry = restarted
            .store
            .load()
            .expect("registry should load after restart");
        assert_eq!(restarted_registry.nodes.len(), 1);
        assert_eq!(
            restarted_registry.nodes["mvp-node-001"].process.state,
            SupervisorProcessState::Running
        );
        restarted
            .stop(Duration::from_secs(2))
            .expect("restarted MVP runtime should stop");
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[tokio::test]
    #[allow(
        clippy::await_holding_lock,
        reason = "the shared process-test guard intentionally serializes fixture child processes"
    )]
    async fn mvp_serve_entry_supports_demo_lifecycle_and_same_workspace_restart() {
        let _guard = process_test_guard();
        let root = temp_root("demo-entry-restart");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_product_options(&root, fixture_node.clone());

        let runtime = MvpRuntime::prepare(&opt, fixture_node.clone())
            .expect("official MVP entry should prepare without prestarting the node");
        let prepared = runtime.store.load().expect("prepared registry should load");
        assert_eq!(
            prepared.nodes[&opt.node_id].process.state,
            SupervisorProcessState::NotStarted
        );
        let baseline_root = opt
            .workspace
            .join("artifacts/backtests/ema-cross-btcusdt-baseline-v1");
        let baseline_summary = fs::read(baseline_root.join("summary.json"))
            .expect("MVP prepare should generate the default Backtest baseline");

        let router =
            crate::dashboard::dashboard_router(runtime.registry_path.clone(), fixture_node.clone());
        for suffix in ["", "/metrics", "/report", "/analysis"] {
            let (status, payload) = mvp_router_json(
                &router,
                Method::GET,
                &format!("/api/product/v1/runs/ema-cross-btcusdt-baseline-v1{suffix}"),
                None,
            )
            .await;
            assert_eq!(status, StatusCode::OK, "{suffix}: {payload}");
        }
        let identity = &runtime.identity_contract.identities;
        let create = json!({
            "strategy_id": identity.strategy_id,
            "strategy_version_id": format!("{}@{}", identity.strategy_id, identity.strategy_version),
            "environment": "sandbox",
            "supervisor_node_id": identity.node_id,
            "account_ref": format!("account://sandbox/{}", identity.account_id),
            "venue_ref": format!("venue://sandbox/{}", identity.venue_id),
            "user_confirmed": true
        });
        let (status, created) = mvp_router_json(
            &router,
            Method::POST,
            "/api/product/v1/demo-runs",
            Some(&create),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{created}");
        let run_id = created["data"]["run_id"]
            .as_str()
            .expect("created Demo Run should expose run_id")
            .to_string();
        let action_path = format!("/api/product/v1/demo-runs/{run_id}/actions");
        let start = json!({
            "run_id": run_id,
            "action": "start",
            "user_confirmed": true
        });
        let (status, started) =
            mvp_router_json(&router, Method::POST, &action_path, Some(&start)).await;
        assert_eq!(status, StatusCode::OK, "{started}");
        assert_eq!(started["data"]["current_run"]["lifecycle"], "running");

        let mut stale_status = read_json(&runtime.status_contract_path);
        stale_status["provenance"]["generated_at_unix_ms"] = json!(1);
        write_json(&runtime.status_contract_path, &stale_status);

        runtime
            .stop(Duration::from_secs(2))
            .expect("official MVP exit should stop and terminalize the active Demo Run");
        drop(runtime);

        let restarted = MvpRuntime::prepare(&opt, fixture_node.clone())
            .expect("official MVP entry should reopen the same workspace");
        let restarted_registry = restarted
            .store
            .load()
            .expect("restarted registry should load");
        assert_eq!(
            restarted_registry.nodes[&opt.node_id].process.state,
            SupervisorProcessState::NotStarted
        );
        assert_eq!(
            restarted_registry.nodes[&opt.node_id].status_artifact,
            RegistryArtifactState::Missing
        );
        assert_eq!(
            restarted_registry.nodes[&opt.node_id].metrics_artifact,
            RegistryArtifactState::Missing
        );
        assert!(!restarted_registry.nodes[&opt.node_id].status_path.exists());
        assert!(!restarted_registry.nodes[&opt.node_id].metrics_path.exists());
        let ownership = &restarted_registry.nodes[&opt.node_id].run_ownership[&run_id];
        assert!(ownership.terminal.is_some());
        assert_eq!(
            fs::read(baseline_root.join("summary.json"))
                .expect("restarted MVP baseline should remain readable"),
            baseline_summary,
            "MVP restart must verify instead of rewriting the immutable baseline"
        );

        let mut delayed_status = read_json(&restarted.status_contract_path);
        delayed_status["provenance"]["generated_at_unix_ms"] = json!(1);
        write_json(&restarted.status_contract_path, &delayed_status);

        let restarted_router =
            crate::dashboard::dashboard_router(restarted.registry_path.clone(), fixture_node);
        let (status, detail) = mvp_router_json(
            &restarted_router,
            Method::GET,
            &format!("/api/product/v1/runs/{run_id}"),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{detail}");
        assert_eq!(detail["data"]["lifecycle"], "stopped");
        assert_eq!(detail["data"]["runtime"]["process_state"], "stopped");

        restarted
            .stop(Duration::from_secs(2))
            .expect("restarted MVP runtime should stop cleanly");
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[test]
    fn mvp_prepare_rejects_tampered_existing_baseline_without_overwriting_it() {
        let root = temp_root("tampered-baseline");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_product_options(&root, fixture_node.clone());
        let runtime = MvpRuntime::prepare(&opt, fixture_node.clone())
            .expect("first MVP prepare should generate the baseline");
        runtime
            .stop(Duration::from_secs(2))
            .expect("prepared MVP should stop cleanly");
        drop(runtime);

        let summary = opt
            .workspace
            .join("artifacts/backtests/ema-cross-btcusdt-baseline-v1/summary.json");
        fs::write(&summary, b"{}\n").expect("baseline tamper fixture should be written");
        let error = MvpRuntime::prepare(&opt, fixture_node)
            .expect_err("tampered immutable baseline must block MVP startup");
        assert!(
            format!("{error:#}").contains("SHA-256"),
            "unexpected error: {error:#}"
        );
        assert_eq!(
            fs::read(&summary).expect("tampered baseline should remain present"),
            b"{}\n",
            "MVP prepare must not overwrite an existing baseline"
        );
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn mvp_prepare_rejects_symlinked_existing_baseline_directory() {
        let root = temp_root("symlinked-baseline");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_product_options(&root, fixture_node.clone());
        let runtime = MvpRuntime::prepare(&opt, fixture_node.clone())
            .expect("first MVP prepare should generate the baseline");
        runtime
            .stop(Duration::from_secs(2))
            .expect("prepared MVP should stop cleanly");
        drop(runtime);

        let baseline = opt
            .workspace
            .join("artifacts/backtests/ema-cross-btcusdt-baseline-v1");
        let backing = root.join("baseline-backing");
        fs::rename(&baseline, &backing).expect("baseline fixture should move to backing path");
        symlink(&backing, &baseline).expect("baseline symlink fixture should be created");
        let error = MvpRuntime::prepare(&opt, fixture_node)
            .expect_err("symlinked baseline directory must block MVP startup");
        assert!(
            format!("{error:#}").contains("不是受控目录"),
            "unexpected error: {error:#}"
        );
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[test]
    fn mvp_prepare_rejects_existing_baseline_with_a_missing_artifact() {
        let root = temp_root("missing-baseline-artifact");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_product_options(&root, fixture_node.clone());
        let runtime = MvpRuntime::prepare(&opt, fixture_node.clone())
            .expect("first MVP prepare should generate the baseline");
        runtime
            .stop(Duration::from_secs(2))
            .expect("prepared MVP should stop cleanly");
        drop(runtime);

        let details = opt
            .workspace
            .join("artifacts/backtests/ema-cross-btcusdt-baseline-v1/details.json");
        fs::remove_file(&details).expect("details fixture should be removed");
        let error = MvpRuntime::prepare(&opt, fixture_node)
            .expect_err("missing immutable baseline file must block MVP startup");
        assert!(
            format!("{error:#}").contains("details.json"),
            "unexpected error: {error:#}"
        );
        assert!(
            !details.exists(),
            "MVP prepare must not regenerate a partially present baseline"
        );
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn mvp_prepare_rejects_symlinked_baseline_artifact_file() {
        let root = temp_root("symlinked-baseline-artifact");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_product_options(&root, fixture_node.clone());
        let runtime = MvpRuntime::prepare(&opt, fixture_node.clone())
            .expect("first MVP prepare should generate the baseline");
        runtime
            .stop(Duration::from_secs(2))
            .expect("prepared MVP should stop cleanly");
        drop(runtime);

        let details = opt
            .workspace
            .join("artifacts/backtests/ema-cross-btcusdt-baseline-v1/details.json");
        let backing = root.join("details-backing.json");
        fs::rename(&details, &backing).expect("details fixture should move to backing path");
        let expected = fs::read(&backing).expect("details backing should be readable");
        symlink(&backing, &details).expect("details symlink fixture should be created");
        let error = MvpRuntime::prepare(&opt, fixture_node)
            .expect_err("symlinked baseline file must block MVP startup");
        assert!(
            format!("{error:#}").contains("details.json"),
            "unexpected error: {error:#}"
        );
        assert_eq!(
            fs::read(&backing).expect("external details backing should remain readable"),
            expected,
            "MVP prepare must not modify a symlink target"
        );
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn mvp_prepare_rejects_symlinked_backtest_parent_directories_without_external_writes() {
        for linked_parent in ["artifacts", "backtests"] {
            let root = temp_root(&format!("symlinked-{linked_parent}-parent"));
            let fixture_node = write_fixture_node(&root);
            let opt = mvp_product_options(&root, fixture_node.clone());
            let external = root.join("external-parent");
            fs::create_dir_all(&external).expect("external parent fixture should be created");
            let sentinel = external.join("sentinel.txt");
            fs::write(&sentinel, b"outside-must-not-change\n")
                .expect("external sentinel should be written");
            fs::create_dir_all(&opt.workspace).expect("workspace fixture should be created");
            if linked_parent == "artifacts" {
                symlink(&external, opt.workspace.join("artifacts"))
                    .expect("artifacts symlink fixture should be created");
            } else {
                fs::create_dir_all(opt.workspace.join("artifacts"))
                    .expect("artifacts parent fixture should be created");
                symlink(&external, opt.workspace.join("artifacts/backtests"))
                    .expect("backtests symlink fixture should be created");
            }

            let error = MvpRuntime::prepare(&opt, fixture_node)
                .expect_err("symlinked Backtest parent must block MVP startup");
            assert!(
                format!("{error:#}").contains("符号链接"),
                "unexpected error for {linked_parent}: {error:#}"
            );
            assert_eq!(
                fs::read(&sentinel).expect("external sentinel should remain readable"),
                b"outside-must-not-change\n",
                "MVP prepare must not modify external parent content"
            );
            assert!(
                !external.join("ema-cross-btcusdt-baseline-v1").exists(),
                "MVP prepare must not publish a baseline outside the workspace"
            );
            fs::remove_dir_all(root).expect("temporary MVP root should be removed");
        }
    }

    #[test]
    fn mvp_prepare_ignores_unowned_stale_staging_and_publishes_a_valid_baseline() {
        let root = temp_root("stale-baseline-staging");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_product_options(&root, fixture_node.clone());
        let stale = opt
            .workspace
            .join("artifacts/backtests/.ema-cross-btcusdt-baseline-v1.bootstrap-stale");
        fs::create_dir_all(&stale).expect("stale staging fixture should be created");
        let sentinel = stale.join("sentinel.txt");
        fs::write(&sentinel, b"stale-unowned\n").expect("stale sentinel should be written");

        let runtime = MvpRuntime::prepare(&opt, fixture_node)
            .expect("unowned stale staging must not block a fresh publication");
        runtime
            .stop(Duration::from_secs(2))
            .expect("prepared MVP should stop cleanly");
        assert!(
            opt.workspace
                .join("artifacts/backtests/ema-cross-btcusdt-baseline-v1/summary.json")
                .is_file()
        );
        assert_eq!(
            fs::read(&sentinel).expect("stale sentinel should remain readable"),
            b"stale-unowned\n",
            "MVP prepare must not delete an unowned staging directory"
        );
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[test]
    fn concurrent_baseline_generation_publishes_one_valid_immutable_run() {
        let root = temp_root("concurrent-baseline");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_product_options(&root, fixture_node);
        let identity = MvpIdentityContract::load(&opt.config, &opt.node_id)
            .expect("identity contract fixture should load");
        let barrier = Arc::new(Barrier::new(2));
        let results = std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for _ in 0..2 {
                let barrier = Arc::clone(&barrier);
                let opt = opt.clone();
                let identity = identity.clone();
                handles.push(scope.spawn(move || {
                    barrier.wait();
                    prepare_mvp_baseline_backtest(&opt, &identity)
                }));
            }
            handles
                .into_iter()
                .map(|handle| handle.join().expect("generation thread should not panic"))
                .collect::<Vec<_>>()
        });
        assert_eq!(
            results
                .iter()
                .filter(|result| matches!(result, Ok(true)))
                .count(),
            1,
            "exactly one concurrent generator must publish the absent baseline"
        );
        let run_root = opt
            .workspace
            .join("artifacts/backtests/ema-cross-btcusdt-baseline-v1");
        assert!(run_root.join("summary.json").is_file());
        assert!(run_root.join("details.json").is_file());
        assert!(run_root.join("analysis.json").is_file());
        let staging = fs::read_dir(opt.workspace.join(PRODUCT_BACKTEST_ARTIFACT_ROOT))
            .expect("Backtest artifact root should be readable")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".bootstrap-"))
            .count();
        assert_eq!(
            staging, 0,
            "losing generator must clean its staging directory"
        );
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[test]
    fn mvp_prepare_creates_empty_backtest_root_without_starting_node() {
        let root = temp_root("prepare-product-root");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_options(&root, fixture_node.clone());

        let runtime = MvpRuntime::prepare(&opt, fixture_node)
            .expect("MVP prepare should initialize the product workspace");
        let registry = runtime.store.load().expect("prepared registry should load");

        assert_eq!(
            registry.nodes[&opt.node_id].process.state,
            SupervisorProcessState::NotStarted
        );
        assert!(
            opt.workspace.join(PRODUCT_BACKTEST_ARTIFACT_ROOT).is_dir(),
            "MVP prepare must create the empty Backtest root"
        );
        assert!(
            opt.workspace.join("catalog").is_dir(),
            "MVP prepare must create the empty local data catalog root"
        );

        runtime
            .stop(Duration::from_secs(2))
            .expect("prepared MVP runtime should stop cleanly");
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[test]
    fn mvp_prepare_rejects_registry_runtime_path_drift_before_cleanup() {
        for field in ["pid", "status", "metrics"] {
            let root = temp_root(&format!("registry-{field}-path-drift"));
            let fixture_node = write_fixture_node(&root);
            let opt = mvp_options(&root, fixture_node.clone());
            let runtime = MvpRuntime::prepare(&opt, fixture_node.clone())
                .expect("initial MVP prepare should succeed");
            let store = SupervisorRegistryStore::new(&runtime.registry_path);
            let mut registry = store.load().expect("registry should load for drift test");
            let external = root.join(format!("outside-{field}.json"));
            fs::write(&external, "preserve-me").expect("external sentinel should be written");
            let record = registry
                .nodes
                .get_mut(&opt.node_id)
                .expect("MVP node should exist");
            match field {
                "pid" => record.pid_path = external.clone(),
                "status" => record.status_path = external.clone(),
                "metrics" => record.metrics_path = external.clone(),
                _ => unreachable!(),
            }
            store.save(&registry).expect("drifted registry should save");
            drop(runtime);

            let error = MvpRuntime::prepare(&opt, fixture_node)
                .expect_err("registry path drift must fail before cleanup");
            assert!(format!("{error:#}").contains("注册路径不可信"));
            assert_eq!(
                fs::read_to_string(&external).expect("external sentinel must survive"),
                "preserve-me"
            );
            fs::remove_dir_all(root).expect("temporary MVP root should be removed");
        }
    }

    #[cfg(unix)]
    #[test]
    fn mvp_prepare_rejects_symlinked_artifact_root_before_cleanup() {
        let root = temp_root("registry-artifact-root-symlink");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_options(&root, fixture_node.clone());
        let runtime = MvpRuntime::prepare(&opt, fixture_node.clone())
            .expect("initial MVP prepare should succeed");
        let artifact_root = runtime.artifact_root.clone();
        let original = root.join("original-node-artifacts");
        let external = root.join("external-node-artifacts");
        fs::rename(&artifact_root, &original).expect("node artifacts should move aside");
        fs::create_dir(&external).expect("external sentinel directory should be created");
        for name in ["pid.json", "status.json", "metrics.json"] {
            fs::write(external.join(name), "preserve-me")
                .expect("external sentinel should be written");
        }
        std::os::unix::fs::symlink(&external, &artifact_root)
            .expect("artifact root symlink should be created");
        drop(runtime);

        let error = MvpRuntime::prepare(&opt, fixture_node)
            .expect_err("symlinked artifact root must fail before cleanup");
        assert!(
            format!("{error:#}").contains("打开 MVP node 工件目录失败"),
            "{error:#}"
        );
        for name in ["pid.json", "status.json", "metrics.json"] {
            assert_eq!(
                fs::read_to_string(external.join(name))
                    .expect("external sentinel must remain readable"),
                "preserve-me"
            );
        }
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn mvp_dashboard_bind_failure_leaves_prepared_node_not_started() {
        let root = temp_root("bind-failure");
        let fixture_node = write_fixture_node(&root);
        let mut opt = mvp_options(&root, fixture_node);
        let listener = tokio::net::TcpListener::bind(opt.bind)
            .await
            .expect("test listener should bind");
        opt.bind = listener
            .local_addr()
            .expect("test listener should expose address");

        let error = run_mvp_serve(opt.clone())
            .await
            .expect_err("occupied Dashboard port must fail");
        assert!(format!("{error:#}").contains("failed to bind dashboard server"));

        let store = SupervisorRegistryStore::new(opt.workspace.join(REGISTRY_PATH));
        let registry = store
            .load()
            .expect("registry should load after Dashboard failure");
        assert_eq!(
            registry.nodes["mvp-node-001"].process.state,
            SupervisorProcessState::NotStarted
        );
        drop(listener);
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn mvp_periodic_status_write_failure_leaves_prepared_node_not_started() {
        let root = temp_root("periodic-status-write-failure");
        let fixture_node = write_fixture_node(&root);
        let mut opt = mvp_options(&root, fixture_node);
        opt.node_heartbeat_interval_ms = 500;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("ephemeral test listener should bind");
        opt.bind = listener
            .local_addr()
            .expect("test listener should expose address");
        drop(listener);
        let status_contract_path = opt.workspace.join(MVP_STATUS_CONTRACT_PATH);
        let task = tokio::spawn(run_mvp_serve(opt.clone()));

        let wait_started = Instant::now();
        while !status_contract_path.is_file() {
            assert!(
                wait_started.elapsed() < Duration::from_secs(3),
                "MVP status contract should appear before timeout"
            );
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        fs::remove_file(&status_contract_path)
            .expect("running status contract should be removable");
        fs::create_dir(&status_contract_path)
            .expect("status contract blocker directory should be created");

        let error = tokio::time::timeout(Duration::from_secs(5), task)
            .await
            .expect("MVP serve should stop after refresh failure")
            .expect("MVP serve task should not panic")
            .expect_err("periodic status write failure must fail closed");
        assert!(format!("{error:#}").contains("刷新 MVP 四轴状态合同失败"));
        let registry = SupervisorRegistryStore::new(opt.workspace.join(REGISTRY_PATH))
            .load()
            .expect("registry should load after periodic refresh failure");
        assert_eq!(
            registry.nodes["mvp-node-001"].process.state,
            SupervisorProcessState::NotStarted
        );
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn mvp_stale_contract_cleanup_failure_prevents_node_start() {
        let _guard = process_test_guard();
        let root = temp_root("identity-contract-write-failure");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_options(&root, fixture_node);
        fs::create_dir_all(&opt.workspace).expect("MVP workspace should be created");
        fs::write(opt.workspace.join("mvp"), "blocks contract directory")
            .expect("contract parent blocker should be written");

        let error = MvpRuntime::start(
            &opt,
            opt.ntpro_node_bin
                .clone()
                .expect("fixture node path should be present"),
        )
        .expect_err("stale contract cleanup failure must prevent startup");
        assert!(format!("{error:#}").contains("清理旧 MVP 身份合同"));
        assert!(!opt.workspace.join(REGISTRY_PATH).exists());
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn mvp_missing_metrics_degrades_health_while_process_remains_running() {
        let _guard = process_test_guard();
        let root = temp_root("missing-metrics");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_options(&root, fixture_node.clone());
        let runtime =
            MvpRuntime::start(&opt, fixture_node).expect("MVP runtime should start fixture node");
        let metrics_path = runtime.artifact_root.join("metrics.json");
        fs::remove_file(&metrics_path).expect("metrics fixture should be removed");

        let contract = runtime
            .write_status_contract()
            .expect("missing metrics should produce a fail-closed contract");
        assert_eq!(
            contract.runtime.status,
            crate::mvp_contract::MvpRuntimeStatus::Running
        );
        assert_eq!(
            contract.technical_health.status,
            crate::mvp_contract::MvpTechnicalHealth::Degraded
        );
        assert_eq!(
            contract.technical_health.availability,
            crate::mvp_contract::MvpStatusAvailability::Missing
        );
        assert!(
            contract
                .technical_health
                .reasons
                .contains(&"process_alive_not_sufficient_for_technical_health".to_string())
        );

        runtime
            .stop(Duration::from_secs(2))
            .expect("MVP runtime should stop with missing metrics");
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn mvp_stale_unknown_and_mismatched_generations_never_report_healthy() {
        let _guard = process_test_guard();
        let root = temp_root("artifact-freshness");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_options(&root, fixture_node.clone());
        let runtime =
            MvpRuntime::start(&opt, fixture_node).expect("MVP runtime should start fixture node");
        let status_path = runtime.artifact_root.join("status.json");
        let metrics_path = runtime.artifact_root.join("metrics.json");
        let original_status = read_json(&status_path);
        let original_metrics = read_json(&metrics_path);

        let mut stale_status = original_status.clone();
        let mut stale_metrics = original_metrics.clone();
        stale_status["generated_at"] =
            serde_json::json!({"availability": "available", "value": "1"});
        stale_metrics["generated_at"] =
            serde_json::json!({"availability": "available", "value": "1"});
        write_json(&status_path, &stale_status);
        write_json(&metrics_path, &stale_metrics);
        let stale = runtime
            .write_status_contract()
            .expect("stale artifacts should produce a contract");
        assert_eq!(
            stale.runtime.freshness,
            crate::mvp_contract::MvpStatusFreshness::Stale
        );
        assert_eq!(
            stale.technical_health.status,
            crate::mvp_contract::MvpTechnicalHealth::Degraded
        );
        assert_eq!(
            stale.technical_health.freshness,
            crate::mvp_contract::MvpStatusFreshness::Stale
        );

        let mut unknown_status = original_status.clone();
        let mut unknown_metrics = original_metrics.clone();
        unknown_status["generated_at"] = serde_json::json!({"availability": "unknown"});
        unknown_metrics["generated_at"] = serde_json::json!({"availability": "unknown"});
        write_json(&status_path, &unknown_status);
        write_json(&metrics_path, &unknown_metrics);
        let unknown = runtime
            .write_status_contract()
            .expect("unknown timestamps should produce a contract");
        assert_eq!(
            unknown.technical_health.status,
            crate::mvp_contract::MvpTechnicalHealth::Degraded
        );
        assert_eq!(
            unknown.technical_health.freshness,
            crate::mvp_contract::MvpStatusFreshness::Unknown
        );

        let mut invalid_timestamp_status = original_status.clone();
        let mut invalid_timestamp_metrics = original_metrics.clone();
        invalid_timestamp_status["generated_at"] =
            serde_json::json!({"availability": "available", "value": "not-a-timestamp"});
        invalid_timestamp_metrics["generated_at"] =
            serde_json::json!({"availability": "available", "value": "not-a-timestamp"});
        write_json(&status_path, &invalid_timestamp_status);
        write_json(&metrics_path, &invalid_timestamp_metrics);
        let invalid_timestamp = runtime
            .write_status_contract()
            .expect("invalid timestamps should produce an error contract");
        assert_eq!(
            invalid_timestamp.runtime.status,
            crate::mvp_contract::MvpRuntimeStatus::Unknown
        );
        assert_eq!(
            invalid_timestamp.technical_health.status,
            crate::mvp_contract::MvpTechnicalHealth::Unhealthy
        );
        assert_eq!(
            invalid_timestamp.technical_health.availability,
            crate::mvp_contract::MvpStatusAvailability::Error
        );

        let now = test_unix_time_ms();
        let mut mismatched_status = original_status.clone();
        let mut mismatched_metrics = original_metrics.clone();
        mismatched_status["generated_at"] =
            serde_json::json!({"availability": "available", "value": now.to_string()});
        mismatched_metrics["generated_at"] = serde_json::json!({
            "availability": "available",
            "value": now.saturating_sub(1).to_string()
        });
        write_json(&status_path, &mismatched_status);
        write_json(&metrics_path, &mismatched_metrics);
        let mismatched = runtime
            .write_status_contract()
            .expect("mismatched generations should produce a contract");
        assert_eq!(
            mismatched.technical_health.status,
            crate::mvp_contract::MvpTechnicalHealth::Degraded
        );
        assert!(
            mismatched
                .technical_health
                .reasons
                .contains(&"status_metrics_generation_mismatch".to_string())
        );

        write_json(&status_path, &original_status);
        write_json(&metrics_path, &original_metrics);
        runtime
            .stop(Duration::from_secs(2))
            .expect("MVP runtime should stop after freshness tests");
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn mvp_lifecycle_nested_errors_and_source_boundaries_fail_closed() {
        let _guard = process_test_guard();
        let root = temp_root("runtime-errors");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_options(&root, fixture_node.clone());
        let runtime =
            MvpRuntime::start(&opt, fixture_node).expect("MVP runtime should start fixture node");
        let status_path = runtime.artifact_root.join("status.json");
        let metrics_path = runtime.artifact_root.join("metrics.json");
        let original_status = read_json(&status_path);
        let original_metrics = read_json(&metrics_path);
        let mut error_status = original_status.clone();
        let mut error_metrics = original_metrics.clone();
        error_status["lifecycle_state"] = serde_json::json!("error");
        error_status["execution"]["last_error"] = serde_json::json!("execution failed");
        error_status["risk"]["last_error"] = serde_json::json!("risk failed");
        error_status["external_venue_connection"] = serde_json::json!(true);
        error_metrics["lifecycle_state"] = serde_json::json!("error");
        error_metrics["real_orders_submitted"] = serde_json::json!(true);
        write_json(&status_path, &error_status);
        write_json(&metrics_path, &error_metrics);

        let contract = runtime
            .write_status_contract()
            .expect("runtime errors should produce a fail-closed contract");
        assert_eq!(
            contract.runtime.status,
            crate::mvp_contract::MvpRuntimeStatus::Unknown
        );
        assert_eq!(
            contract.runtime.availability,
            crate::mvp_contract::MvpStatusAvailability::Error
        );
        assert!(
            contract
                .runtime
                .error
                .as_deref()
                .is_some_and(|error| error.contains("lifecycle state is error"))
        );
        assert_eq!(
            contract.technical_health.status,
            crate::mvp_contract::MvpTechnicalHealth::Unhealthy
        );
        let technical_error = contract
            .technical_health
            .error
            .as_deref()
            .expect("nested runtime errors should be projected");
        assert!(technical_error.contains("execution failed"));
        assert!(technical_error.contains("risk failed"));
        assert!(
            contract
                .technical_health
                .reasons
                .contains(&"mvp_trading_boundary_violation".to_string())
        );
        assert!(!contract.boundaries.external_venue_connection);
        assert!(!contract.boundaries.real_orders_submitted);
        assert!(!contract.boundaries.order_submission_allowed);
        assert!(!contract.boundaries.order_mutation_allowed);

        write_json(&status_path, &original_status);
        write_json(&metrics_path, &original_metrics);
        runtime
            .stop(Duration::from_secs(2))
            .expect("MVP runtime should stop after error tests");
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn mvp_invalid_unified_read_model_is_error_but_remains_blocked() {
        let _guard = process_test_guard();
        let root = temp_root("invalid-read-model");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_options(&root, fixture_node.clone());
        let runtime =
            MvpRuntime::start(&opt, fixture_node).expect("MVP runtime should start fixture node");
        let read_model_path = runtime
            .artifact_root
            .join("v0_21/unified_read_model_snapshot.json");
        fs::create_dir_all(
            read_model_path
                .parent()
                .expect("read model fixture should have a parent"),
        )
        .expect("read model fixture directory should be created");
        fs::write(&read_model_path, "not-json")
            .expect("invalid read model fixture should be written");
        let invalid = runtime
            .write_status_contract()
            .expect("invalid read model should produce a contract");
        assert_eq!(
            invalid.trading_readiness.status,
            crate::mvp_contract::MvpTradingReadiness::Blocked
        );
        assert_eq!(
            invalid.trading_readiness.availability,
            crate::mvp_contract::MvpStatusAvailability::Error
        );

        fs::write(&read_model_path, "{}").expect("valid JSON read model fixture should be written");
        let readable = runtime
            .write_status_contract()
            .expect("readable read model should produce a contract");
        assert_eq!(
            readable.trading_readiness.availability,
            crate::mvp_contract::MvpStatusAvailability::Available
        );
        assert_eq!(
            readable.trading_readiness.status,
            crate::mvp_contract::MvpTradingReadiness::Blocked
        );

        runtime
            .stop(Duration::from_secs(2))
            .expect("MVP runtime should stop after read model tests");
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn mvp_stop_writes_final_status_after_identity_file_is_removed() {
        let _guard = process_test_guard();
        let root = temp_root("identity-removed");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_options(&root, fixture_node.clone());
        let runtime =
            MvpRuntime::start(&opt, fixture_node).expect("MVP runtime should start fixture node");
        fs::remove_file(&runtime.identity_contract_path)
            .expect("published identity contract should be removable for the test");

        runtime
            .stop(Duration::from_secs(2))
            .expect("MVP stop should not depend on identity file presence");
        let contract: MvpStatusContract =
            serde_json::from_value(read_json(&runtime.status_contract_path))
                .expect("final status contract should deserialize");
        assert_eq!(
            contract.runtime.status,
            crate::mvp_contract::MvpRuntimeStatus::Stopped
        );
        assert_eq!(
            contract.technical_health.status,
            crate::mvp_contract::MvpTechnicalHealth::Unhealthy
        );
        assert!(!contract.provenance.identity_contract_available);
        assert!(
            contract
                .technical_health
                .error
                .as_deref()
                .is_some_and(|error| error.contains("identity contract is missing"))
        );
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn mvp_second_start_preserves_active_runtime_contracts() {
        let _guard = process_test_guard();
        let root = temp_root("active-runtime-contracts");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_options(&root, fixture_node.clone());
        let runtime = MvpRuntime::start(&opt, fixture_node.clone())
            .expect("first MVP runtime should start fixture node");
        let identity_before = fs::read_to_string(&runtime.identity_contract_path)
            .expect("active identity contract should be readable");
        let status_before = fs::read_to_string(&runtime.status_contract_path)
            .expect("active status contract should be readable");

        let error = MvpRuntime::start(&opt, fixture_node)
            .expect_err("second start must not replace active runtime contracts");
        assert!(format!("{error:#}").contains("已有活动或未决节点"));
        assert_eq!(
            fs::read_to_string(&runtime.identity_contract_path)
                .expect("active identity contract should remain readable"),
            identity_before
        );
        assert_eq!(
            fs::read_to_string(&runtime.status_contract_path)
                .expect("active status contract should remain readable"),
            status_before
        );

        runtime
            .stop(Duration::from_secs(2))
            .expect("first MVP runtime should stop normally");
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn mvp_restart_rejects_same_strategy_version_with_changed_content_hash() {
        let _guard = process_test_guard();
        let root = temp_root("strategy-version-restart-anchor");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_options(&root, fixture_node.clone());
        let config = fs::read_to_string(&opt.config).expect("MVP config should be readable");
        fs::write(
            &opt.config,
            format!(
                "{config}\n[strategy_version]\ncontent_hash = \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"\n"
            ),
        )
        .expect("version hash anchor should be written");

        let runtime = MvpRuntime::start(&opt, fixture_node.clone())
            .expect("first MVP runtime should start with version hash anchor");
        runtime
            .stop(Duration::from_secs(2))
            .expect("first MVP runtime should stop normally");
        let identity_before = fs::read_to_string(&runtime.identity_contract_path)
            .expect("stopped identity contract should remain readable");
        let events_path = runtime.artifact_root.join("logs/events.log");
        let supervisor_before = fs::read_to_string(&runtime.registry_path)
            .expect("stopped supervisor registry should remain readable");
        let events_before =
            fs::read_to_string(&events_path).expect("stopped events should remain readable");
        let version_registry_path = opt.workspace.join(STRATEGY_VERSION_REGISTRY_PATH);
        let version_registry_before = fs::read_to_string(&version_registry_path)
            .expect("strategy version registry should be readable");

        let changed = fs::read_to_string(&opt.config)
            .expect("anchored MVP config should remain readable")
            .replace(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            );
        fs::write(&opt.config, changed).expect("changed version hash should be written");
        let error = MvpRuntime::start(&opt, fixture_node.clone())
            .expect_err("same strategy version with changed hash must fail before restart");
        assert!(format!("{error:#}").contains("已登记为不可变内容"));
        assert_eq!(
            fs::read_to_string(&runtime.identity_contract_path)
                .expect("failed restart must preserve previous identity contract"),
            identity_before
        );
        assert_eq!(
            fs::read_to_string(&runtime.registry_path)
                .expect("failed restart must preserve stopped supervisor registry"),
            supervisor_before
        );
        assert_eq!(
            fs::read_to_string(&events_path)
                .expect("failed restart must not append node start events"),
            events_before
        );
        assert_eq!(
            fs::read_to_string(&version_registry_path)
                .expect("failed restart must preserve version registry"),
            version_registry_before
        );

        let next_version = fs::read_to_string(&opt.config)
            .expect("changed MVP config should remain readable")
            .replace("strategy_version = \"v1\"", "strategy_version = \"v2\"");
        fs::write(&opt.config, next_version).expect("new strategy version should be written");
        let restarted = MvpRuntime::start(&opt, fixture_node)
            .expect("new strategy version identity may register a new content hash");
        assert_eq!(
            restarted.identity_contract.identities.strategy_version,
            "v2"
        );
        restarted
            .stop(Duration::from_secs(2))
            .expect("new strategy version runtime should stop normally");
        let registry = read_json(&version_registry_path);
        assert_eq!(
            registry["versions"]["strategy-alpha"]["v1"],
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(
            registry["versions"]["strategy-alpha"]["v2"],
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        );

        let identity_v2 = fs::read_to_string(&restarted.identity_contract_path)
            .expect("v2 identity contract should remain readable");
        let supervisor_v2 = fs::read_to_string(&restarted.registry_path)
            .expect("v2 supervisor registry should remain readable");
        let events_v2 = fs::read_to_string(&events_path).expect("v2 events should remain readable");
        let version_registry_v2 = fs::read_to_string(&version_registry_path)
            .expect("v1 and v2 registry should remain readable");
        let reverted_version = fs::read_to_string(&opt.config)
            .expect("v2 config should remain readable")
            .replace("strategy_version = \"v2\"", "strategy_version = \"v1\"");
        fs::write(&opt.config, reverted_version)
            .expect("historical version with changed hash should be written");
        let error = MvpRuntime::start(
            &opt,
            opt.ntpro_node_bin
                .clone()
                .expect("fixture node path should remain present"),
        )
        .expect_err("returning to v1 with a different hash must fail before restart");
        assert!(format!("{error:#}").contains("已登记为不可变内容"));
        assert_eq!(
            fs::read_to_string(&restarted.identity_contract_path)
                .expect("historical version failure must preserve v2 identity"),
            identity_v2
        );
        assert_eq!(
            fs::read_to_string(&restarted.registry_path)
                .expect("historical version failure must preserve supervisor registry"),
            supervisor_v2
        );
        assert_eq!(
            fs::read_to_string(&events_path)
                .expect("historical version failure must not append node events"),
            events_v2
        );
        assert_eq!(
            fs::read_to_string(&version_registry_path)
                .expect("historical version failure must preserve version registry"),
            version_registry_v2
        );
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn mvp_invalid_status_is_unhealthy_and_never_running_by_inference() {
        let _guard = process_test_guard();
        let root = temp_root("invalid-status");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_options(&root, fixture_node.clone());
        let runtime =
            MvpRuntime::start(&opt, fixture_node).expect("MVP runtime should start fixture node");
        let status_path = runtime.artifact_root.join("status.json");
        let valid_status = fs::read_to_string(&status_path)
            .expect("valid running status should be readable before corruption");
        fs::write(&status_path, "not-json").expect("status fixture should be corrupted");

        let contract = runtime
            .write_status_contract()
            .expect("invalid status should produce an unhealthy contract");
        assert_eq!(
            contract.runtime.status,
            crate::mvp_contract::MvpRuntimeStatus::Unknown
        );
        assert_eq!(
            contract.runtime.availability,
            crate::mvp_contract::MvpStatusAvailability::Error
        );
        assert_eq!(
            contract.technical_health.status,
            crate::mvp_contract::MvpTechnicalHealth::Unhealthy
        );
        assert_eq!(
            contract.technical_health.availability,
            crate::mvp_contract::MvpStatusAvailability::Error
        );

        fs::write(&status_path, &valid_status).expect("valid running status should be restored");
        let mut registry = runtime
            .store
            .load()
            .expect("registry should load for test cleanup");
        let record = registry
            .nodes
            .get_mut(&runtime.node_id)
            .expect("fixture node should remain registered");
        record.process.state = SupervisorProcessState::Running;
        record.status_artifact = crate::supervisor::RegistryArtifactState::Available;
        record.last_known_status = serde_json::from_str(&valid_status)
            .expect("restored running status should deserialize");
        runtime
            .store
            .save(&registry)
            .expect("restored registry should save");
        runtime
            .stop(Duration::from_secs(2))
            .expect("fixture node should rewrite a valid stopped status");
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn mvp_status_contract_cleanup_failure_prevents_node_start() {
        let root = temp_root("status-contract-write-failure");
        let fixture_node = write_fixture_node(&root);
        let opt = mvp_options(&root, fixture_node);
        fs::create_dir_all(opt.workspace.join(MVP_STATUS_CONTRACT_PATH))
            .expect("status contract blocker directory should be created");

        let error = MvpRuntime::start(
            &opt,
            opt.ntpro_node_bin
                .clone()
                .expect("fixture node path should be present"),
        )
        .expect_err("status contract cleanup failure must prevent startup");
        assert!(format!("{error:#}").contains("清理旧 MVP 四轴状态合同"));
        assert!(!opt.workspace.join(REGISTRY_PATH).exists());
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[test]
    fn mvp_rejects_non_loopback_dashboard_bind_before_starting_node() {
        let root = temp_root("non-loopback");
        let mut opt = mvp_options(&root, root.join("missing-node"));
        opt.bind = "0.0.0.0:5173".parse().expect("bind fixture should parse");

        let error = MvpRuntime::start(&opt, root.join("missing-node"))
            .expect_err("non-loopback MVP bind must fail first");
        assert!(format!("{error:#}").contains("loopback"));
        assert!(!opt.workspace.join(REGISTRY_PATH).exists());
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[test]
    fn mvp_rejects_zero_dashboard_port_before_starting_node() {
        let root = temp_root("zero-port");
        let mut opt = mvp_options(&root, root.join("missing-node"));
        opt.bind = "127.0.0.1:0".parse().expect("bind fixture should parse");

        let error = MvpRuntime::start(&opt, root.join("missing-node"))
            .expect_err("zero MVP Dashboard port must fail first");
        assert!(format!("{error:#}").contains("端口必须大于 0"));
        assert!(!opt.workspace.join(REGISTRY_PATH).exists());
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    fn write_fixture_node(root: &Path) -> PathBuf {
        let path = root.join("fixture-ntpro-node.sh");
        fs::write(
            &path,
            r#"#!/bin/sh
set -eu
node_id=""
output=""
stop_file=""
config=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --run-id) node_id="$2"; shift 2 ;;
    --output) output="$2"; shift 2 ;;
    --stop-file) stop_file="$2"; shift 2 ;;
    --config) config="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$output/logs"
touch "$output/logs/stdout.log" "$output/logs/stderr.log" "$output/logs/events.log"
checksum() {
  if command -v shasum >/dev/null 2>&1; then
    digest="$(shasum -a 256 "$1" | awk '{print $1}')"
  else
    digest="$(sha256sum "$1" | awk '{print $1}')"
  fi
  printf 'sha256:%s' "$digest"
}
byte_len() { wc -c < "$1" | tr -d ' '; }
write_strategy_artifacts() {
  state="$1"
  now_ms="$2"
  artifact_now_ms=$((now_ms + 1))
  strategy_id="$(awk -F'"' '/^strategy_id[[:space:]]*=/{print $2; exit}' "$config")"
  test -n "$strategy_id"
  mkdir -p "$output/strategy"
  : > "$output/strategy/market_events.jsonl"
  : > "$output/strategy/signal.jsonl"
  : > "$output/strategy/order_intent.jsonl"
  : > "$output/strategy/risk_decision.jsonl"
  cat > "$output/strategy/session_status.json" <<EOF
{"schema_version":"ntpro.v09_strategy_session_status.v1","session_id":"$node_id","strategy_id":"$strategy_id","state":"$state","reason":"fixture_runtime","updated_at_unix_ms":$now_ms,"artifacts":{"session_status":"$output/strategy/session_status.json","events":"$output/strategy/events.jsonl","market_status":"$output/strategy/market_status.json","market_events":"$output/strategy/market_events.jsonl","signal":"$output/strategy/signal.jsonl","order_intent":"$output/strategy/order_intent.jsonl","risk_decision":"$output/strategy/risk_decision.jsonl","summary":"$output/strategy/summary.json","simulation_summary":"$output/strategy/simulation_summary.json","simulated_fills":"$output/strategy/simulated_fills.jsonl","simulated_positions":"$output/strategy/simulated_positions.jsonl","equity_curve":"$output/strategy/equity_curve.jsonl","manifest":"$output/strategy/manifest.json"}}
EOF
  cat > "$output/strategy/events.jsonl" <<EOF
{"schema_version":"ntpro.v09_strategy_session_event.v1","event_type":"fixture","session_id":"$node_id","strategy_id":"$strategy_id","previous_state":null,"state":"$state","reason":"fixture_runtime","occurred_at_unix_ms":$now_ms}
EOF
  cat > "$output/strategy/market_status.json" <<EOF
{"schema_version":"ntpro.v09_market_stream_status.v1","session_id":"$node_id","strategy_id":"$strategy_id","connection":"not_configured","state":"$state","source":"fixture_stream","event_count":0,"last_event_at_unix_ms":null,"updated_at_unix_ms":$artifact_now_ms}
EOF
  cat > "$output/strategy/summary.json" <<EOF
{"schema_version":"ntpro.v09_strategy_session_summary.v1","session_id":"$node_id","strategy_id":"$strategy_id","state":"$state","event_count":1,"market_event_count":0,"signal_count":0,"intent_count":0,"risk_decision_count":0,"rejection_count":0,"actual_submission_count":0,"updated_at_unix_ms":$artifact_now_ms}
EOF
  cat > "$output/strategy/simulation_summary.json" <<EOF
{"schema_version":"ntpro.demo_simulation_summary.v1","session_id":"$node_id","strategy_id":"$strategy_id","instrument_id":"BTCUSDT.BINANCE","engine":"nautilus_backtest::engine::BacktestEngine","execution_mode":"simulated","data_sha256":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","parameters":{"trade_size":"1.000000","fast_period":3,"slow_period":5},"fill_count":1,"position_count":1,"equity_point_count":1,"boundaries":{"simulation_only":true,"external_venue_connection":false,"order_submission_allowed":false,"order_mutation_allowed":false,"automatic_retry_allowed":false,"automatic_remediation_allowed":false,"real_orders_submitted":false,"trading_controls_enabled":false}}
EOF
  cat > "$output/strategy/simulated_fills.jsonl" <<EOF
{"schema_version":"ntpro.demo_simulated_fill.v1","session_id":"$node_id","strategy_id":"$strategy_id","simulation_only":true,"trade_id":"trade-demo-001","client_order_id":"order-demo-001","venue_order_id":"simulated-001","position_id":"position-demo-001","side":"SELL","order_type":"MARKET","quantity":"1.000000","price":"100.50","currency":"USDT","liquidity_side":"TAKER","commission":"0.10050000 USDT","ts_event":"${now_ms}000000"}
EOF
  cat > "$output/strategy/simulated_positions.jsonl" <<EOF
{"schema_version":"ntpro.demo_simulated_position.v1","session_id":"$node_id","strategy_id":"$strategy_id","simulation_only":true,"position_id":"position-demo-001","account_id":"BINANCE-001","side":"SHORT","entry_side":"SELL","peak_quantity":"1.000000","buy_quantity":"0.000000","sell_quantity":"1.000000","avg_price_open":"100.5","avg_price_close":null,"realized_return":"0","realized_pnl":null,"trade_count":1,"ts_opened":"${now_ms}000000","ts_closed":null,"duration_ns":"0"}
EOF
  cat > "$output/strategy/equity_curve.jsonl" <<EOF
{"schema_version":"ntpro.demo_equity_point.v1","session_id":"$node_id","strategy_id":"$strategy_id","simulation_only":true,"account_id":"BINANCE-001","currency":"USDT","total":"1000000.00000000 USDT","free":"1000000.00000000 USDT","locked":"0.00000000 USDT","ts_event":"${now_ms}000000"}
EOF
  cat > "$output/strategy/manifest.json.tmp" <<EOF
{"schema_version":"ntpro.v091_strategy_session_manifest.v1","session_id":"$node_id","strategy_id":"$strategy_id","state":"$state","created_at_unix_ms":$now_ms,"updated_at_unix_ms":$now_ms,"artifacts":[
{"name":"session_status","path":"$output/strategy/session_status.json","format":"json","present":true,"record_count":1,"byte_len":$(byte_len "$output/strategy/session_status.json"),"checksum":"$(checksum "$output/strategy/session_status.json")"},
{"name":"events","path":"$output/strategy/events.jsonl","format":"jsonl","present":true,"record_count":1,"byte_len":$(byte_len "$output/strategy/events.jsonl"),"checksum":"$(checksum "$output/strategy/events.jsonl")"},
{"name":"market_status","path":"$output/strategy/market_status.json","format":"json","present":true,"record_count":1,"byte_len":$(byte_len "$output/strategy/market_status.json"),"checksum":"$(checksum "$output/strategy/market_status.json")"},
{"name":"market_events","path":"$output/strategy/market_events.jsonl","format":"jsonl","present":true,"record_count":0,"byte_len":$(byte_len "$output/strategy/market_events.jsonl"),"checksum":"$(checksum "$output/strategy/market_events.jsonl")"},
{"name":"signal","path":"$output/strategy/signal.jsonl","format":"jsonl","present":true,"record_count":0,"byte_len":$(byte_len "$output/strategy/signal.jsonl"),"checksum":"$(checksum "$output/strategy/signal.jsonl")"},
{"name":"order_intent","path":"$output/strategy/order_intent.jsonl","format":"jsonl","present":true,"record_count":0,"byte_len":$(byte_len "$output/strategy/order_intent.jsonl"),"checksum":"$(checksum "$output/strategy/order_intent.jsonl")"},
{"name":"risk_decision","path":"$output/strategy/risk_decision.jsonl","format":"jsonl","present":true,"record_count":0,"byte_len":$(byte_len "$output/strategy/risk_decision.jsonl"),"checksum":"$(checksum "$output/strategy/risk_decision.jsonl")"},
{"name":"summary","path":"$output/strategy/summary.json","format":"json","present":true,"record_count":1,"byte_len":$(byte_len "$output/strategy/summary.json"),"checksum":"$(checksum "$output/strategy/summary.json")"},
{"name":"simulation_summary","path":"$output/strategy/simulation_summary.json","format":"json","present":true,"record_count":1,"byte_len":$(byte_len "$output/strategy/simulation_summary.json"),"checksum":"$(checksum "$output/strategy/simulation_summary.json")"},
{"name":"simulated_fills","path":"$output/strategy/simulated_fills.jsonl","format":"jsonl","present":true,"record_count":1,"byte_len":$(byte_len "$output/strategy/simulated_fills.jsonl"),"checksum":"$(checksum "$output/strategy/simulated_fills.jsonl")"},
{"name":"simulated_positions","path":"$output/strategy/simulated_positions.jsonl","format":"jsonl","present":true,"record_count":1,"byte_len":$(byte_len "$output/strategy/simulated_positions.jsonl"),"checksum":"$(checksum "$output/strategy/simulated_positions.jsonl")"},
{"name":"equity_curve","path":"$output/strategy/equity_curve.jsonl","format":"jsonl","present":true,"record_count":1,"byte_len":$(byte_len "$output/strategy/equity_curve.jsonl"),"checksum":"$(checksum "$output/strategy/equity_curve.jsonl")"}]}
EOF
  mv "$output/strategy/manifest.json.tmp" "$output/strategy/manifest.json"
}
write_status() {
  state="$1"
  previous="$2"
  stops="$3"
  now_ms="$(($(date +%s) * 1000))"
  stopped='{"availability":"unknown"}'
  if [ "$state" = "stopped" ]; then
    stopped="{\"availability\":\"available\",\"value\":\"$now_ms\"}"
  fi
  write_strategy_artifacts "$state" "$now_ms"
  cat > "$output/status.json.tmp" <<EOF
{"schema_version":"ntpro.node_status.v1","node_id":"$node_id","process_mode":"spawned_process","config_path":{"availability":"available","value":"fixture.toml"},"artifact_root":{"availability":"available","value":"$output"},"lifecycle_state":"$state","previous_lifecycle_state":"$previous","data_connection":"not_configured","execution_connection":"disconnected","execution":{"gateway_id":{"availability":"available","value":"SANDBOX"},"connection":"disconnected","started":{"availability":"available","value":false},"account_ref":{"availability":"available","value":"account://sandbox/SANDBOX-001"},"orders_open":{"availability":"available","value":0},"orders_inflight":{"availability":"available","value":0},"orders_closed":{"availability":"available","value":0},"last_report_at":{"availability":"unknown"},"last_reconciliation_at":{"availability":"unknown"},"last_error":null},"risk":{"trading_state":"unknown","health":"unknown","command_count":{"availability":"available","value":0},"event_count":{"availability":"available","value":0},"rejections_total":{"availability":"available","value":0},"last_rejection":null,"last_error":null},"generated_at":{"availability":"available","value":"$now_ms"},"started_at":{"availability":"available","value":"$now_ms"},"stopped_at":$stopped,"last_transition_at":{"availability":"available","value":"$now_ms"},"last_error":null,"external_venue_connection":false,"real_orders_submitted":false}
EOF
  mv "$output/status.json.tmp" "$output/status.json"
  cat > "$output/metrics.json.tmp" <<EOF
{"schema_version":"ntpro.node_metrics.v1","node_id":"$node_id","lifecycle_state":"$state","previous_lifecycle_state":"$previous","process_mode":"spawned_process","uptime_ms":{"availability":"available","value":1},"starts_total":1,"stops_total":$stops,"state_transitions_total":2,"connection_counts":{"data_connected":0,"data_disconnected":0,"data_not_configured":1,"execution_connected":0,"execution_disconnected":1,"execution_not_configured":0},"last_error_summary":null,"generated_at":{"availability":"available","value":"$now_ms"},"started_at":{"availability":"available","value":"$now_ms"},"stopped_at":$stopped,"status_artifact_path":{"availability":"available","value":"$output/status.json"},"stdout_log_path":{"availability":"available","value":"$output/logs/stdout.log"},"stderr_log_path":{"availability":"available","value":"$output/logs/stderr.log"},"events_log_path":{"availability":"available","value":"$output/logs/events.log"},"strategy_signal_count":{"availability":"available","value":0},"strategy_rejection_count":{"availability":"available","value":0},"kill_switch_dry_run":{"artifact_path":{"availability":"available","value":"$output/kill-switch.json"},"artifact_status":{"availability":"available","value":"verified"},"kill_switch_active":{"availability":"available","value":false},"kill_switch_dry_run":{"availability":"available","value":true},"manual_approval_recorded":{"availability":"available","value":false},"approval_state":{"availability":"available","value":"not_approved"},"production_order_submission_allowed":{"availability":"available","value":false},"production_order_mutation_allowed":{"availability":"available","value":false},"production_order_state_reads_allowed":{"availability":"available","value":false},"listen_key_lifecycle_allowed":{"availability":"available","value":false},"production_order_submissions_attempted":{"availability":"available","value":0},"production_orders_submitted":{"availability":"available","value":0},"production_order_mutations_attempted":{"availability":"available","value":0},"production_order_state_reads_attempted":{"availability":"available","value":0},"dashboard_order_controls_enabled":{"availability":"available","value":false},"real_orders_submitted":{"availability":"available","value":false},"network_attempted":{"availability":"available","value":false},"values_are_exchange_truth":{"availability":"available","value":false}},"external_venue_connection":false,"real_orders_submitted":false}
EOF
  mv "$output/metrics.json.tmp" "$output/metrics.json"
}
write_status running starting 0
while [ ! -f "$stop_file" ]; do sleep 0.05; done
write_status stopped running 1
"#,
        )
        .expect("fixture node should be written");
        let mut permissions = fs::metadata(&path)
            .expect("fixture node metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("fixture node should be executable");
        path
    }
}
