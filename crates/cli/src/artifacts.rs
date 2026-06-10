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
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Context;
use serde::Serialize;

static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// # Errors
///
/// Returns an error if `value` cannot be serialized or the target file cannot
/// be atomically persisted.
pub(crate) fn atomic_write_json<T>(path: &Path, value: &T) -> anyhow::Result<()>
where
    T: Serialize,
{
    let raw = serde_json::to_string_pretty(value)?;
    atomic_write_text(path, &format!("{raw}\n"))
}

/// # Errors
///
/// Returns an error if the target directory or temporary file cannot be
/// created, written, synced, or renamed into place.
pub(crate) fn atomic_write_text(path: &Path, contents: &str) -> anyhow::Result<()> {
    atomic_write_bytes(path, contents.as_bytes())
}

/// # Errors
///
/// Returns an error if a file exists and cannot be removed.
pub(crate) fn remove_file_if_exists(path: &Path) -> anyhow::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => {
            sync_parent_dir_best_effort(path);
            Ok(())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => {
            Err(error).with_context(|| format!("failed to remove artifact '{}'", path.display()))
        }
    }
}

fn atomic_write_bytes(path: &Path, contents: &[u8]) -> anyhow::Result<()> {
    if let Some(parent) = parent_dir(path) {
        fs::create_dir_all(parent).with_context(|| {
            format!("failed to create artifact directory '{}'", parent.display())
        })?;
    }

    let temp_path = temporary_path_for(path)?;
    let write_result = write_temp_file(&temp_path, contents)
        .and_then(|()| {
            fs::rename(&temp_path, path).with_context(|| {
                format!(
                    "failed to move temporary artifact '{}' into '{}'",
                    temp_path.display(),
                    path.display()
                )
            })
        })
        .map(|()| sync_parent_dir_best_effort(path));

    if write_result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }

    write_result
}

fn write_temp_file(path: &Path, contents: &[u8]) -> anyhow::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("failed to create temporary artifact '{}'", path.display()))?;
    file.write_all(contents)
        .with_context(|| format!("failed to write temporary artifact '{}'", path.display()))?;
    file.sync_all()
        .with_context(|| format!("failed to sync temporary artifact '{}'", path.display()))?;
    Ok(())
}

fn temporary_path_for(path: &Path) -> anyhow::Result<PathBuf> {
    let file_name = path
        .file_name()
        .with_context(|| format!("artifact path '{}' has no file name", path.display()))?
        .to_string_lossy();
    let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    let temp_name = format!(
        ".{file_name}.tmp.{}.{}.{}",
        std::process::id(),
        millis,
        sequence
    );

    if let Some(parent) = parent_dir(path) {
        Ok(parent.join(&temp_name))
    } else {
        Ok(PathBuf::from(temp_name))
    }
}

fn parent_dir(path: &Path) -> Option<&Path> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
}

fn sync_parent_dir_best_effort(path: &Path) {
    if let Some(parent) = parent_dir(path)
        && let Ok(dir) = OpenOptions::new().read(true).open(parent)
    {
        let _ = dir.sync_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "ntpro-p0-004-artifacts-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn atomic_text_write_replaces_complete_file_without_temp_residue() {
        let root = temp_root("text");
        let path = root.join("status.txt");

        atomic_write_text(&path, "old\n").unwrap();
        atomic_write_text(&path, "new\n").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "new\n");
        let temp_files = fs::read_dir(&root)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp."))
            .count();
        assert_eq!(temp_files, 0);
    }
}
