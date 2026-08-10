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
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, ensure};
use cap_fs_ext::DirExt;
use serde::{Deserialize, Serialize};

use crate::{
    artifacts::{atomic_write_json, remove_file_if_exists},
    dashboard::run_dashboard_command,
    mvp_contract::{
        MVP_IDENTITY_CONTRACT_PATH, MVP_STATUS_CONTRACT_PATH, MvpIdentityContract,
        MvpStatusContract,
    },
    opt::{DashboardCommand, DashboardOpt, DashboardServeOpt, MvpCommand, MvpOpt, MvpServeOpt},
    supervisor::{
        RegisterNodeRequest, StartNodeRequest, StopNodeRequest, SupervisorProcessState,
        SupervisorRegistryStore,
    },
};

const REGISTRY_PATH: &str = "supervisor/registry.json";
const NODE_ARTIFACT_ROOT: &str = "nodes";
const PRODUCT_ARTIFACT_ROOT: &str = "artifacts";
const STRATEGY_VERSION_REGISTRY_PATH: &str = "mvp/strategy_version_registry.json";
const STRATEGY_VERSION_REGISTRY_SCHEMA_VERSION: &str = "ntpro.mvp_strategy_version_registry.v1";
const MIN_STATUS_FRESHNESS_MAX_AGE_MS: u64 = 2_000;

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
        std::fs::create_dir_all(opt.workspace.join(PRODUCT_ARTIFACT_ROOT)).with_context(|| {
            format!(
                "初始化 MVP 产品工件目录 '{}' 失败",
                opt.workspace.join(PRODUCT_ARTIFACT_ROOT).display()
            )
        })?;
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

    let serve_result = loop {
        tokio::select! {
            result = &mut dashboard => break result.context("MVP Dashboard 服务已退出"),
            result = tokio::signal::ctrl_c() => {
                result.context("等待 MVP Ctrl-C 终止信号失败")?;
                break Ok(());
            }
            _ = status_refresh.tick() => {
                if let Err(error) = runtime.write_status_contract() {
                    break Err(error.context("刷新 MVP 四轴状态合同失败"));
                }
            }
        }
    };
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
        sync::{Mutex, MutexGuard},
        time::{SystemTime, UNIX_EPOCH},
    };

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Method, Request, StatusCode},
    };
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;
    use crate::{
        backtest::run_backtest_command,
        opt::{BacktestCommand, BacktestOpt, BacktestRunOpt},
        supervisor::RegistryArtifactState,
    };

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
        run_backtest_command(BacktestOpt {
            command: BacktestCommand::Run(BacktestRunOpt {
                config: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../configs/backtests/ema-cross-btcusdt-product.toml"),
                run_id: None,
                output: Some(
                    opt.workspace
                        .join("artifacts/backtests/ema-cross-btcusdt-baseline-v1"),
                ),
                dry_run: false,
            }),
        })
        .expect("the current StrategyVersion backtest baseline should be created first");

        let runtime = MvpRuntime::prepare(&opt, fixture_node.clone())
            .expect("official MVP entry should prepare without prestarting the node");
        let prepared = runtime.store.load().expect("prepared registry should load");
        assert_eq!(
            prepared.nodes[&opt.node_id].process.state,
            SupervisorProcessState::NotStarted
        );

        let router =
            crate::dashboard::dashboard_router(runtime.registry_path.clone(), fixture_node.clone());
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
while [ "$#" -gt 0 ]; do
  case "$1" in
    --run-id) node_id="$2"; shift 2 ;;
    --output) output="$2"; shift 2 ;;
    --stop-file) stop_file="$2"; shift 2 ;;
    *) shift ;;
  esac
done
mkdir -p "$output/logs"
touch "$output/logs/stdout.log" "$output/logs/stderr.log" "$output/logs/events.log"
write_status() {
  state="$1"
  previous="$2"
  stops="$3"
  now_ms="$(($(date +%s) * 1000))"
  stopped='{"availability":"unknown"}'
  if [ "$state" = "stopped" ]; then
    stopped="{\"availability\":\"available\",\"value\":\"$now_ms\"}"
  fi
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
