// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
// -------------------------------------------------------------------------------------------------

#![cfg(unix)]

use std::{
    env,
    ffi::OsStr,
    fs::{self, OpenOptions},
    io::{self, Write},
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

use sysinfo::{Pid, ProcessStatus, ProcessesToUpdate, Signal, System};
use tokio::{
    process::{Child, Command},
    signal::unix::{SignalKind, signal},
    time::timeout,
};

const LOCK_NAME: &str = ".local-delivery.lock";
const GUARDIAN_MODE: &str = "--ntpro-local-delivery-guardian";

#[derive(Debug)]
struct LauncherError {
    code: i32,
    message: String,
}

impl LauncherError {
    fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

struct RuntimeLock {
    path: PathBuf,
    child_path: PathBuf,
    owner_pid: u32,
}

impl RuntimeLock {
    fn publish_child(&self, child_pid: u32) -> Result<(), LauncherError> {
        atomic_write_pid(&self.child_path, child_pid).map_err(|error| {
            LauncherError::new(74, format!("无法记录 NTPRO 服务进程 {child_pid}：{error}"))
        })
    }

    fn cleanup(&self) {
        if read_pid(&self.path) == Some(self.owner_pid) {
            let _ = fs::remove_file(&self.path);
        }
        let _ = fs::remove_file(&self.child_path);
    }
}

impl Drop for RuntimeLock {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[tokio::main]
async fn main() {
    let args = env::args_os().collect::<Vec<_>>();
    if args.get(1).is_some_and(|arg| arg == GUARDIAN_MODE) {
        let code = run_guardian(&args[2..]);
        std::process::exit(code);
    }

    if let Err(error) = run_launcher().await {
        eprintln!("NTPRO 启动失败：{}", error.message);
        std::process::exit(error.code);
    }
}

async fn run_launcher() -> Result<(), LauncherError> {
    let current_exe = env::current_exe()
        .map_err(|error| LauncherError::new(66, format!("无法定位启动器：{error}")))?;
    let package_root = current_exe
        .parent()
        .ok_or_else(|| LauncherError::new(66, "无法定位交付目录"))?
        .to_path_buf();
    let nautilus_bin = package_root.join("bin/nautilus");
    let node_bin = package_root.join("bin/ntpro-node");
    let node_config = package_root.join("configs/nodes/btc-ema-shadow.toml");
    let backtest_config = package_root.join("configs/backtests/ema-cross-btcusdt-product.toml");
    let frontend_dist = package_root.join("apps/strategy-workbench/dist");

    require_executable(&nautilus_bin, "NTPRO 主程序")?;
    require_executable(&node_bin, "NTPRO 节点程序")?;
    require_file(&node_config, "节点配置")?;
    require_file(&backtest_config, "回测配置")?;
    require_file(&frontend_dist.join("index.html"), "策略工作台页面")?;

    let workspace = workspace_path()?;
    let bind = env::var("NTPRO_BIND").unwrap_or_else(|_| "127.0.0.1:5173".to_string());
    let bind_addr = parse_bind(&bind)?;
    let node_id = env::var("NTPRO_NODE_ID").unwrap_or_else(|_| "mvp-node-001".to_string());
    let startup_timeout_ms = positive_env_u64("NTPRO_STARTUP_TIMEOUT_MS", 10_000)?;
    let node_max_runtime_ms = positive_env_u64("NTPRO_NODE_MAX_RUNTIME_MS", 86_400_000)?;
    let node_shutdown_timeout_ms = positive_env_u64("NTPRO_NODE_SHUTDOWN_TIMEOUT_MS", 10_000)?;
    let service_stop_timeout =
        Duration::from_millis(node_shutdown_timeout_ms.saturating_add(30_000));

    fs::create_dir_all(&workspace).map_err(|error| {
        LauncherError::new(
            73,
            format!("无法创建工作区 {}：{error}", workspace.display()),
        )
    })?;
    let runtime_lock = acquire_lock(&workspace)?;

    if TcpStream::connect_timeout(&bind_addr, Duration::from_millis(250)).is_ok() {
        return Err(LauncherError::new(
            69,
            format!(
                "端口 {} 已被占用。请关闭占用程序，或设置 NTPRO_BIND=127.0.0.1:其他端口。",
                bind_addr.port()
            ),
        ));
    }

    // 在服务启动前注册三种停止信号，避免启动窗口遗漏终止请求。
    let mut interrupt = signal(SignalKind::interrupt())
        .map_err(|error| LauncherError::new(70, format!("无法监听 Ctrl-C：{error}")))?;
    let mut terminate = signal(SignalKind::terminate())
        .map_err(|error| LauncherError::new(70, format!("无法监听 TERM：{error}")))?;
    let mut hangup = signal(SignalKind::hangup())
        .map_err(|error| LauncherError::new(70, format!("无法监听 HUP：{error}")))?;

    let mut guardian = spawn_guardian(&current_exe, &workspace, std::process::id())?;

    println!("NTPRO 正在启动...");
    println!("工作区：{}", workspace.display());
    println!("页面入口：服务就绪后，请打开下方 strategy_workbench_url 的完整地址。");
    println!("停止方式：回到本终端按 Ctrl-C。");

    let mut child = Command::new(&nautilus_bin)
        .arg("mvp")
        .arg("serve")
        .arg("--config")
        .arg(&node_config)
        .arg("--workspace")
        .arg(&workspace)
        .arg("--node-id")
        .arg(&node_id)
        .arg("--bind")
        .arg(&bind)
        .arg("--strategy-workbench-dist")
        .arg(&frontend_dist)
        .arg("--ntpro-node-bin")
        .arg(&node_bin)
        .arg("--startup-timeout-ms")
        .arg(startup_timeout_ms.to_string())
        .arg("--node-max-runtime-ms")
        .arg(node_max_runtime_ms.to_string())
        .arg("--node-shutdown-timeout-ms")
        .arg(node_shutdown_timeout_ms.to_string())
        .spawn()
        .map_err(|error| LauncherError::new(70, format!("无法启动 NTPRO 主程序：{error}")))?;
    let child_pid = child
        .id()
        .ok_or_else(|| LauncherError::new(70, "NTPRO 主程序没有可用 PID"))?;
    if let Err(error) = runtime_lock.publish_child(child_pid) {
        let _ = stop_child(&mut child, child_pid, service_stop_timeout).await;
        let _ = guardian.kill().await;
        let _ = guardian.wait().await;
        return Err(error);
    }

    let exit_status = tokio::select! {
        result = child.wait() => result.map_err(|error| LauncherError::new(70, format!("等待 NTPRO 主程序失败：{error}")))?,
        _ = interrupt.recv() => stop_child(&mut child, child_pid, service_stop_timeout).await?,
        _ = terminate.recv() => stop_child(&mut child, child_pid, service_stop_timeout).await?,
        _ = hangup.recv() => stop_child(&mut child, child_pid, service_stop_timeout).await?,
    };

    let _ = guardian.kill().await;
    let _ = guardian.wait().await;
    runtime_lock.cleanup();

    if !exit_status.success() {
        return Err(LauncherError::new(
            exit_status.code().unwrap_or(70),
            format!(
                "NTPRO 已停止，但启动或运行过程返回错误（状态 {:?}）。节点日志位于 {}/nodes/{}/logs/。",
                exit_status.code(),
                workspace.display(),
                node_id
            ),
        ));
    }

    println!("NTPRO 已安全停止。运行数据保留在：{}", workspace.display());
    Ok(())
}

async fn stop_child(
    child: &mut Child,
    child_pid: u32,
    stop_timeout: Duration,
) -> Result<ExitStatus, LauncherError> {
    send_signal(child_pid, Signal::Interrupt).map_err(|error| {
        LauncherError::new(70, format!("无法向 NTPRO 主程序发送 Ctrl-C：{error}"))
    })?;
    match timeout(stop_timeout, child.wait()).await {
        Ok(result) => result
            .map_err(|error| LauncherError::new(70, format!("等待 NTPRO 安全停止失败：{error}"))),
        Err(_) => {
            let _ = send_signal(child_pid, Signal::Kill);
            let _ = child.wait().await;
            Err(LauncherError::new(
                70,
                format!(
                    "NTPRO 在 {} 秒内未完成安全停止，已强制终止",
                    stop_timeout.as_secs()
                ),
            ))
        }
    }
}

fn spawn_guardian(
    current_exe: &Path,
    workspace: &Path,
    parent_pid: u32,
) -> Result<Child, LauncherError> {
    Command::new(current_exe)
        .arg(GUARDIAN_MODE)
        .arg(parent_pid.to_string())
        .arg(workspace)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| LauncherError::new(70, format!("无法启动 NTPRO 守护进程：{error}")))
}

fn run_guardian(args: &[std::ffi::OsString]) -> i32 {
    let Some(parent_pid) = args
        .first()
        .and_then(|value| value.to_str())
        .and_then(|value| value.parse::<u32>().ok())
    else {
        return 64;
    };
    let Some(workspace) = args.get(1).map(PathBuf::from) else {
        return 64;
    };
    let lock_path = workspace.join(LOCK_NAME);
    let child_path = child_path(&workspace, parent_pid);

    let mut observed_child = None;
    while process_alive(parent_pid) {
        observed_child = read_pid(&child_path)
            .or_else(|| find_nautilus_child(parent_pid))
            .or(observed_child);
        thread::sleep(Duration::from_millis(200));
    }

    let child_pid = read_pid(&child_path).or(observed_child);
    if let Some(child_pid) = child_pid.filter(|pid| process_alive(*pid)) {
        let _ = send_signal(child_pid, Signal::Interrupt);
        if !wait_for_process_exit(child_pid, Duration::from_secs(45)) {
            let _ = send_signal(child_pid, Signal::Kill);
            let _ = wait_for_process_exit(child_pid, Duration::from_secs(2));
        }
    }
    if read_pid(&lock_path) == Some(parent_pid) {
        let _ = fs::remove_file(lock_path);
    }
    let _ = fs::remove_file(child_path);
    0
}

fn workspace_path() -> Result<PathBuf, LauncherError> {
    if let Some(path) = env::var_os("NTPRO_WORKSPACE") {
        return Ok(PathBuf::from(path));
    }
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| LauncherError::new(64, "HOME 未设置，无法确定默认数据目录"))?;
    if cfg!(target_os = "macos") {
        Ok(home.join("Library/Application Support/NTPRO/usable-product-v1"))
    } else {
        let base =
            env::var_os("XDG_DATA_HOME").map_or_else(|| home.join(".local/share"), PathBuf::from);
        Ok(base.join("ntpro/usable-product-v1"))
    }
}

fn parse_bind(value: &str) -> Result<SocketAddr, LauncherError> {
    let address = value
        .parse::<SocketAddr>()
        .map_err(|_| LauncherError::new(64, "NTPRO_BIND 必须是本机地址，例如 127.0.0.1:5173。"))?;
    if address.ip().to_string() != "127.0.0.1" || address.port() == 0 {
        return Err(LauncherError::new(
            64,
            "NTPRO_BIND 必须是本机地址，例如 127.0.0.1:5173。",
        ));
    }
    Ok(address)
}

fn positive_env_u64(name: &str, default: u64) -> Result<u64, LauncherError> {
    let value = match env::var(name) {
        Ok(value) => value
            .parse::<u64>()
            .map_err(|_| LauncherError::new(64, format!("{name} 必须是大于 0 的整数。")))?,
        Err(_) => default,
    };
    if value == 0 {
        return Err(LauncherError::new(
            64,
            format!("{name} 必须是大于 0 的整数。"),
        ));
    }
    Ok(value)
}

fn require_file(path: &Path, label: &str) -> Result<(), LauncherError> {
    if path.is_file() {
        Ok(())
    } else {
        Err(LauncherError::new(
            66,
            format!("缺少{label}：{}。请重新获取完整交付目录。", path.display()),
        ))
    }
}

fn require_executable(path: &Path, label: &str) -> Result<(), LauncherError> {
    use std::os::unix::fs::PermissionsExt;
    if path.is_file()
        && path
            .metadata()
            .is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
    {
        Ok(())
    } else {
        Err(LauncherError::new(
            66,
            format!(
                "缺少可执行的{label}：{}。请重新获取完整交付目录。",
                path.display()
            ),
        ))
    }
}

fn acquire_lock(workspace: &Path) -> Result<RuntimeLock, LauncherError> {
    let owner_pid = std::process::id();
    let lock_path = workspace.join(LOCK_NAME);
    let candidate = workspace.join(format!("{LOCK_NAME}.candidate.{owner_pid}"));
    let _ = fs::remove_file(&candidate);
    write_pid_create_new(&candidate, owner_pid)
        .map_err(|error| LauncherError::new(73, format!("无法创建原子运行锁候选：{error}")))?;

    for attempt in 0..3_u32 {
        match fs::hard_link(&candidate, &lock_path) {
            Ok(()) => {
                let _ = fs::remove_file(&candidate);
                return Ok(RuntimeLock {
                    path: lock_path,
                    child_path: child_path(workspace, owner_pid),
                    owner_pid,
                });
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                let _ = fs::remove_file(&candidate);
                return Err(LauncherError::new(
                    73,
                    format!("无法发布原子运行锁：{error}"),
                ));
            }
        }

        let metadata = fs::symlink_metadata(&lock_path)
            .map_err(|error| LauncherError::new(73, format!("无法检查运行锁：{error}")))?;
        if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
            let _ = fs::remove_file(&candidate);
            return Err(LauncherError::new(
                73,
                format!("运行锁路径不是可信普通文件：{}。", lock_path.display()),
            ));
        }
        let existing_owner = read_pid(&lock_path)
            .ok_or_else(|| LauncherError::new(75, "运行锁内容无效；请保留现场并联系维护人员。"))?;
        if process_alive(existing_owner) {
            let _ = fs::remove_file(&candidate);
            return Err(LauncherError::new(
                73,
                format!(
                    "该工作区已经有一个 NTPRO 实例在运行（PID {existing_owner}）。请使用现有页面，或先在原终端按 Ctrl-C 停止。"
                ),
            ));
        }
        let existing_child_path = child_path(workspace, existing_owner);
        if let Some(existing_child) =
            read_pid(&existing_child_path).filter(|pid| process_alive(*pid))
            && !wait_for_process_exit(existing_child, Duration::from_secs(5))
        {
            let _ = fs::remove_file(&candidate);
            return Err(LauncherError::new(
                75,
                format!(
                    "上次启动器已退出，但 NTPRO 正在安全收口（PID {existing_child}）。请稍等几秒后重试。"
                ),
            ));
        }

        let quarantine = workspace.join(format!("{LOCK_NAME}.stale.{owner_pid}.{attempt}"));
        match fs::rename(&lock_path, &quarantine) {
            Ok(()) => {
                let quarantined_owner = read_pid(&quarantine);
                if quarantined_owner != Some(existing_owner) || process_alive(existing_owner) {
                    let _ = fs::rename(&quarantine, &lock_path);
                    let _ = fs::remove_file(&candidate);
                    return Err(LauncherError::new(
                        75,
                        "运行锁在接管期间发生变化，拒绝继续启动。",
                    ));
                }
                let _ = fs::remove_file(&quarantine);
                let _ = fs::remove_file(existing_child_path);
                println!("NTPRO 检测到上次异常退出，已原子接管失效运行锁。");
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                let _ = fs::remove_file(&candidate);
                return Err(LauncherError::new(
                    75,
                    format!("无法原子接管失效运行锁：{error}"),
                ));
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    let _ = fs::remove_file(candidate);
    Err(LauncherError::new(
        75,
        "另一个启动器正在接管该工作区，请稍等几秒后重试。",
    ))
}

fn child_path(workspace: &Path, owner_pid: u32) -> PathBuf {
    workspace.join(format!(".local-delivery.child.{owner_pid}"))
}

fn write_pid_create_new(path: &Path, pid: u32) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    writeln!(file, "{pid}")?;
    file.sync_all()
}

fn atomic_write_pid(path: &Path, pid: u32) -> io::Result<()> {
    let temporary = path.with_extension(format!("tmp.{}", std::process::id()));
    let _ = fs::remove_file(&temporary);
    write_pid_create_new(&temporary, pid)?;
    fs::rename(temporary, path)
}

fn read_pid(path: &Path) -> Option<u32> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn process_alive(pid: u32) -> bool {
    let sys_pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[sys_pid]), true);
    system.process(sys_pid).is_some_and(|process| {
        !matches!(
            process.status(),
            ProcessStatus::Dead | ProcessStatus::Zombie
        )
    })
}

fn send_signal(pid: u32, signal: Signal) -> io::Result<()> {
    let sys_pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[sys_pid]), true);
    let Some(process) = system.process(sys_pid) else {
        return Ok(());
    };
    match process.kill_with(signal) {
        Some(true) => Ok(()),
        Some(false) => Err(io::Error::other(format!(
            "操作系统拒绝向 PID {pid} 发送 {signal:?}"
        ))),
        None => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("当前平台不支持 {signal:?}"),
        )),
    }
}

fn wait_for_process_exit(pid: u32, wait: Duration) -> bool {
    let started = Instant::now();
    while process_alive(pid) {
        if started.elapsed() >= wait {
            return false;
        }
        thread::sleep(Duration::from_millis(100));
    }
    true
}

fn find_nautilus_child(parent_pid: u32) -> Option<u32> {
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);
    system.processes().values().find_map(|process| {
        (process.parent().map(Pid::as_u32) == Some(parent_pid)
            && process.name() == OsStr::new("nautilus"))
        .then(|| process.pid().as_u32())
    })
}
