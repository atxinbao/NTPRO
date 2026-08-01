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
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, ensure};

use crate::{
    artifacts::atomic_write_json,
    dashboard::run_dashboard_command,
    mvp_contract::{MVP_IDENTITY_CONTRACT_PATH, MvpIdentityContract},
    opt::{DashboardCommand, DashboardOpt, DashboardServeOpt, MvpCommand, MvpOpt, MvpServeOpt},
    supervisor::{
        RegisterNodeRequest, StartNodeRequest, StopNodeRequest, SupervisorProcessState,
        SupervisorRegistryStore,
    },
};

const REGISTRY_PATH: &str = "supervisor/registry.json";
const NODE_ARTIFACT_ROOT: &str = "nodes";

#[derive(Debug)]
struct MvpRuntime {
    store: SupervisorRegistryStore,
    node_id: String,
    registry_path: PathBuf,
    artifact_root: PathBuf,
    identity_contract_path: PathBuf,
}

impl MvpRuntime {
    fn start(opt: &MvpServeOpt, ntpro_node_bin: PathBuf) -> anyhow::Result<Self> {
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

        store.register_node(RegisterNodeRequest {
            node_id: opt.node_id.clone(),
            config_path: opt.config.clone(),
            artifact_root: Some(artifact_root.clone()),
        })?;
        let startup_timeout = duration_from_millis("startup_timeout_ms", opt.startup_timeout_ms)?;
        let node_shutdown_timeout =
            duration_from_millis("node_shutdown_timeout_ms", opt.node_shutdown_timeout_ms)?;
        store.start_node_process(&StartNodeRequest {
            node_id: opt.node_id.clone(),
            ntpro_node_bin,
            startup_timeout,
            node_max_runtime: duration_from_millis("node_max_runtime_ms", opt.node_max_runtime_ms)?,
            node_heartbeat_interval: duration_from_millis(
                "node_heartbeat_interval_ms",
                opt.node_heartbeat_interval_ms,
            )?,
            node_parent_pid: Some(std::process::id()),
            node_shutdown_timeout,
        })?;

        let runtime = Self {
            store,
            node_id: opt.node_id.clone(),
            registry_path,
            artifact_root,
            identity_contract_path,
        };
        let startup_result = runtime
            .prepare_observability(startup_timeout)
            .and_then(|()| {
                atomic_write_json(&runtime.identity_contract_path, &identity_contract).with_context(
                    || {
                        format!(
                            "写入 MVP 身份合同 '{}' 失败",
                            runtime.identity_contract_path.display()
                        )
                    },
                )
            });
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
        Ok(())
    }
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
    let ntpro_node_bin = opt
        .ntpro_node_bin
        .clone()
        .unwrap_or_else(default_ntpro_node_bin_path);
    let stop_timeout =
        duration_from_millis("node_shutdown_timeout_ms", opt.node_shutdown_timeout_ms)?;
    let runtime = MvpRuntime::start(&opt, ntpro_node_bin.clone())?;

    println!(
        "mvp.serve status=ok node_id={} registry={} artifact_root={} identity_contract={} dashboard_url=http://{}/dashboard external_venue_connection=false real_orders_submitted=false",
        runtime.node_id,
        runtime.registry_path.display(),
        runtime.artifact_root.display(),
        runtime.identity_contract_path.display(),
        opt.bind,
    );

    let dashboard = run_dashboard_command(DashboardOpt {
        command: DashboardCommand::Serve(DashboardServeOpt {
            registry: runtime.registry_path.clone(),
            workflow_root: None,
            bind: opt.bind,
            ntpro_node_bin: Some(ntpro_node_bin),
        }),
    });
    tokio::pin!(dashboard);

    let serve_result = tokio::select! {
        result = &mut dashboard => result.context("MVP Dashboard 服务已退出"),
        result = tokio::signal::ctrl_c() => {
            result.context("等待 MVP Ctrl-C 终止信号失败")?;
            Ok(())
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

    use super::*;

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
            ntpro_node_bin: Some(ntpro_node_bin),
            startup_timeout_ms: 2_000,
            node_max_runtime_ms: 60_000,
            node_heartbeat_interval_ms: 50,
            node_shutdown_timeout_ms: 2_000,
        }
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
    async fn mvp_dashboard_bind_failure_stops_started_node() {
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
            SupervisorProcessState::Stopped
        );
        drop(listener);
        fs::remove_dir_all(root).expect("temporary MVP root should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn mvp_identity_contract_write_failure_stops_started_node() {
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
        .expect_err("identity contract write failure must stop startup");
        assert!(format!("{error:#}").contains("写入 MVP 身份合同"));

        let store = SupervisorRegistryStore::new(opt.workspace.join(REGISTRY_PATH));
        let registry = store
            .load()
            .expect("registry should load after identity contract failure");
        assert_eq!(
            registry.nodes["mvp-node-001"].process.state,
            SupervisorProcessState::Stopped
        );
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
write_status() {
  state="$1"
  previous="$2"
  cat > "$output/status.json.tmp" <<EOF
{"schema_version":"ntpro.node_status.v1","node_id":"$node_id","process_mode":"spawned_process","config_path":{"availability":"available","value":"fixture.toml"},"artifact_root":{"availability":"available","value":"$output"},"lifecycle_state":"$state","previous_lifecycle_state":"$previous","data_connection":"not_configured","execution_connection":"disconnected","execution":{"gateway_id":{"availability":"available","value":"SANDBOX"},"connection":"disconnected","started":{"availability":"available","value":true},"account_ref":{"availability":"available","value":"configured"},"orders_open":{"availability":"unknown"},"orders_inflight":{"availability":"unknown"},"orders_closed":{"availability":"unknown"},"last_report_at":{"availability":"unknown"},"last_reconciliation_at":{"availability":"unknown"},"last_error":null},"risk":{"trading_state":"unknown","health":"unknown","command_count":{"availability":"unknown"},"event_count":{"availability":"unknown"},"rejections_total":{"availability":"unknown"},"last_rejection":null,"last_error":null},"generated_at":{"availability":"unknown"},"started_at":{"availability":"unknown"},"stopped_at":{"availability":"unknown"},"last_transition_at":{"availability":"unknown"},"last_error":null,"external_venue_connection":false,"real_orders_submitted":false}
EOF
  mv "$output/status.json.tmp" "$output/status.json"
}
write_status running starting
cat > "$output/metrics.json" <<EOF
{"schema_version":"ntpro.node_metrics.v1","node_id":"$node_id","lifecycle_state":"running","previous_lifecycle_state":"starting","process_mode":"spawned_process","uptime_ms":{"availability":"available","value":0},"starts_total":1,"stops_total":0,"state_transitions_total":1,"connection_counts":{"data_connected":0,"data_disconnected":0,"data_not_configured":1,"execution_connected":0,"execution_disconnected":1,"execution_not_configured":0},"last_error_summary":null,"generated_at":{"availability":"available","value":"1"},"started_at":{"availability":"available","value":"1"},"stopped_at":{"availability":"unknown"},"status_artifact_path":{"availability":"available","value":"$output/status.json"},"stdout_log_path":{"availability":"available","value":"$output/logs/stdout.log"},"stderr_log_path":{"availability":"available","value":"$output/logs/stderr.log"},"events_log_path":{"availability":"available","value":"$output/logs/events.log"},"external_venue_connection":false,"real_orders_submitted":false}
EOF
while [ ! -f "$stop_file" ]; do sleep 0.05; done
write_status stopped running
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
