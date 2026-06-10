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
    thread,
    time::{Duration, Instant},
};

use anyhow::Context;
use sysinfo::{Pid, ProcessesToUpdate, Signal, System};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SignalDelivery {
    ProcessExited,
    Sent,
    Unsupported,
}

#[must_use]
pub(crate) fn process_is_alive(pid: u32) -> bool {
    let sys_pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[sys_pid]), true);
    system.process(sys_pid).is_some()
}

/// # Errors
///
/// Returns an error if the process exists but the operating system rejects the
/// termination signal.
pub(crate) fn send_termination(pid: u32) -> anyhow::Result<SignalDelivery> {
    send_signal(pid, Signal::Term)
}

/// # Errors
///
/// Returns an error if the process exists but the operating system rejects the
/// kill request.
pub(crate) fn send_kill(pid: u32) -> anyhow::Result<SignalDelivery> {
    let sys_pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[sys_pid]), true);
    let Some(process) = system.process(sys_pid) else {
        return Ok(SignalDelivery::ProcessExited);
    };
    if process.kill() {
        Ok(SignalDelivery::Sent)
    } else {
        anyhow::bail!("operating system rejected kill request for pid {pid}");
    }
}

#[must_use]
pub(crate) fn wait_for_process_exit(pid: u32, timeout: Duration) -> bool {
    let started = Instant::now();
    loop {
        if !process_is_alive(pid) {
            return true;
        }
        if started.elapsed() >= timeout {
            return false;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn send_signal(pid: u32, signal: Signal) -> anyhow::Result<SignalDelivery> {
    let sys_pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[sys_pid]), true);
    let Some(process) = system.process(sys_pid) else {
        return Ok(SignalDelivery::ProcessExited);
    };
    match process.kill_with(signal) {
        Some(true) => Ok(SignalDelivery::Sent),
        Some(false) => Err(anyhow::anyhow!(
            "operating system rejected {signal:?} for pid {pid}"
        ))
        .with_context(|| format!("failed to signal local node process {pid}")),
        None => Ok(SignalDelivery::Unsupported),
    }
}
