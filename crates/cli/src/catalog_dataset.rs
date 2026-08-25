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

//! 本地产品回测数据集的标准目录检查与内容指纹。

use std::{
    collections::BTreeSet,
    fs,
    io::{self, Write},
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, ensure};
use aws_lc_rs::digest::{Context as DigestContext, SHA256};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};
use nautilus_model::{
    data::{HasTsInit, QuoteTick},
    instruments::{Instrument, InstrumentAny},
};
use nautilus_persistence::backend::catalog::ParquetDataCatalog;
use serde::Serialize;

pub(crate) const PRODUCT_CATALOG_DIRECTORY: &str = "catalog";
pub(crate) const PRODUCT_RUN_CATALOG_SNAPSHOT_DIRECTORY: &str = "catalog-snapshot";

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LocalQuoteDatasetInspection {
    pub(crate) catalog_path: PathBuf,
    pub(crate) instrument_id: String,
    pub(crate) venue: String,
    pub(crate) record_count: usize,
    pub(crate) start_time_ns: u64,
    pub(crate) end_time_ns: u64,
    pub(crate) file_count: usize,
    pub(crate) size_bytes: u64,
    pub(crate) data_sha256: String,
    pub(crate) instrument: InstrumentAny,
    pub(crate) quotes: Vec<QuoteTick>,
    catalog_files: Vec<PathBuf>,
}

impl LocalQuoteDatasetInspection {
    #[must_use]
    pub(crate) fn data_ref(&self) -> String {
        format!("dataset://local/quotes/{}", self.instrument_id)
    }

    #[must_use]
    pub(crate) fn dataset_id(&self) -> String {
        format!("local-quotes-{}", &self.data_sha256[7..19])
    }

    pub(crate) fn same_content_as(&self, other: &Self) -> bool {
        self.instrument_id == other.instrument_id
            && self.venue == other.venue
            && self.record_count == other.record_count
            && self.start_time_ns == other.start_time_ns
            && self.end_time_ns == other.end_time_ns
            && self.file_count == other.file_count
            && self.size_bytes == other.size_bytes
            && self.data_sha256 == other.data_sha256
    }
}

/// Copies one validated dataset into a fresh Run-owned catalog and validates the copy.
///
/// The caller must provide a newly created Run directory. Source and destination files are
/// opened without following their final path component, and every intermediate directory is
/// traversed with `open_dir_nofollow`.
pub(crate) fn snapshot_local_quote_dataset(
    source: &LocalQuoteDatasetInspection,
    run_directory: &Dir,
    snapshot_root: &Path,
) -> anyhow::Result<LocalQuoteDatasetInspection> {
    ensure!(
        snapshot_root.is_absolute(),
        "Run catalog snapshot path must be absolute"
    );
    run_directory
        .create_dir(PRODUCT_RUN_CATALOG_SNAPSHOT_DIRECTORY)
        .context("failed to create Run catalog snapshot")?;
    let destination_root = run_directory
        .open_dir_nofollow(PRODUCT_RUN_CATALOG_SNAPSHOT_DIRECTORY)
        .context("failed to open Run catalog snapshot without following links")?;
    let source_root = Dir::open_ambient_dir(&source.catalog_path, cap_std::ambient_authority())
        .context("failed to open validated local catalog")?;

    for source_file in &source.catalog_files {
        let relative = source_file
            .strip_prefix(&source.catalog_path)
            .context("catalog file escaped its validated root before snapshot")?;
        validate_relative_file_path(relative)?;
        copy_file_nofollow(&source_root, &destination_root, relative)?;
    }

    let snapshot = inspect_local_quote_dataset(snapshot_root, &source.instrument_id)
        .context("Run catalog snapshot validation failed")?;
    ensure!(
        snapshot.same_content_as(source),
        "local catalog changed while the immutable Run snapshot was created"
    );
    Ok(snapshot)
}

/// Scans every instrument-backed QuoteTick dataset in a local standard catalog.
///
/// # Errors
///
/// Returns an error when the root is missing, not a normal directory, contains an invalid
/// Parquet dataset, or a listed file escapes the catalog root.
pub(crate) fn inspect_local_quote_datasets(
    catalog_root: &Path,
) -> anyhow::Result<Vec<LocalQuoteDatasetInspection>> {
    let canonical_root = validate_catalog_root(catalog_root)?;
    validate_catalog_tree_nofollow(&canonical_root)?;
    let mut catalog = ParquetDataCatalog::from_uri(
        canonical_root.to_string_lossy().as_ref(),
        None,
        None,
        None,
        None,
    )?;
    let data_types = catalog.list_data_types()?;
    if !data_types.iter().any(|value| value == "quotes") {
        return Ok(Vec::new());
    }

    let instruments = catalog.query_instruments(None)?;
    ensure!(
        !instruments.is_empty(),
        "quote catalog contains no instrument definitions"
    );
    let mut seen = BTreeSet::new();
    let mut datasets = Vec::new();
    for instrument in instruments {
        let instrument_id = instrument.id().to_string();
        ensure!(
            seen.insert(instrument_id.clone()),
            "duplicate instrument definition for '{instrument_id}'"
        );
        if let Some(dataset) =
            inspect_instrument_quotes(&mut catalog, &canonical_root, &instrument, &instrument_id)?
        {
            datasets.push(dataset);
        }
    }
    datasets.sort_by(|left, right| left.instrument_id.cmp(&right.instrument_id));
    Ok(datasets)
}

/// Loads one exact QuoteTick dataset from a local standard catalog.
///
/// # Errors
///
/// Returns an error when the catalog is invalid or the requested instrument has no unique
/// instrument definition and non-empty QuoteTick history.
pub(crate) fn inspect_local_quote_dataset(
    catalog_root: &Path,
    instrument_id: &str,
) -> anyhow::Result<LocalQuoteDatasetInspection> {
    let datasets = inspect_local_quote_datasets(catalog_root)?;
    let mut matches = datasets
        .into_iter()
        .filter(|dataset| dataset.instrument_id == instrument_id);
    let dataset = matches
        .next()
        .with_context(|| format!("no QuoteTick dataset found for '{instrument_id}'"))?;
    ensure!(
        matches.next().is_none(),
        "multiple QuoteTick datasets found for '{instrument_id}'"
    );
    Ok(dataset)
}

fn validate_catalog_root(catalog_root: &Path) -> anyhow::Result<PathBuf> {
    let metadata = fs::symlink_metadata(catalog_root).with_context(|| {
        format!(
            "local catalog root '{}' does not exist",
            catalog_root.display()
        )
    })?;
    ensure!(
        metadata.is_dir() && !metadata.file_type().is_symlink(),
        "local catalog root '{}' must be a normal directory",
        catalog_root.display()
    );
    catalog_root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize '{}'", catalog_root.display()))
}

fn inspect_instrument_quotes(
    catalog: &mut ParquetDataCatalog,
    canonical_root: &Path,
    instrument: &InstrumentAny,
    instrument_id: &str,
) -> anyhow::Result<Option<LocalQuoteDatasetInspection>> {
    catalog.reset_session();
    let quotes = catalog.quote_ticks(Some(vec![instrument_id.to_string()]), None, None)?;
    if quotes.is_empty() {
        return Ok(None);
    }
    ensure!(
        quotes
            .iter()
            .all(|quote| quote.instrument_id.to_string() == instrument_id),
        "QuoteTick dataset '{instrument_id}' contains another instrument"
    );
    ensure!(
        quotes
            .windows(2)
            .all(|pair| pair[0].ts_init() <= pair[1].ts_init()),
        "QuoteTick dataset '{instrument_id}' is not ordered by ts_init"
    );

    let start_time_ns = quotes
        .first()
        .context("non-empty QuoteTick dataset is missing its first item")?
        .ts_init()
        .as_u64();
    let end_time_ns = quotes
        .last()
        .context("non-empty QuoteTick dataset is missing its last item")?
        .ts_init()
        .as_u64();
    ensure!(
        start_time_ns <= end_time_ns,
        "QuoteTick dataset '{instrument_id}' has an invalid time range"
    );

    let mut files =
        catalog.query_files("quotes", Some(vec![instrument_id.to_string()]), None, None)?;
    files.extend(catalog.query_files(
        "instruments",
        Some(vec![instrument_id.to_string()]),
        None,
        None,
    )?);
    files.sort();
    files.dedup();
    ensure!(
        files.len() >= 2,
        "QuoteTick dataset '{instrument_id}' must contain instrument and quote Parquet files"
    );
    let (size_bytes, catalog_files) = local_catalog_files(catalog, canonical_root, &files)?;
    let data_sha256 = dataset_content_sha256(instrument, &quotes)?;

    Ok(Some(LocalQuoteDatasetInspection {
        catalog_path: canonical_root.to_path_buf(),
        instrument_id: instrument_id.to_string(),
        venue: instrument.id().venue.to_string(),
        record_count: quotes.len(),
        start_time_ns,
        end_time_ns,
        file_count: files.len(),
        size_bytes,
        data_sha256,
        instrument: instrument.clone(),
        quotes,
        catalog_files,
    }))
}

fn validate_catalog_tree_nofollow(root: &Path) -> anyhow::Result<()> {
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let mut entries = fs::read_dir(&directory)
            .with_context(|| format!("failed to list catalog directory '{}'", directory.display()))?
            .collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(fs::DirEntry::file_name);
        for entry in entries {
            let path = entry.path();
            let file_type = entry
                .file_type()
                .with_context(|| format!("failed to inspect catalog path '{}'", path.display()))?;
            ensure!(
                !file_type.is_symlink(),
                "catalog path '{}' must not contain symbolic links",
                path.display()
            );
            if file_type.is_dir() {
                pending.push(path);
            } else {
                ensure!(
                    file_type.is_file(),
                    "catalog path '{}' must be a normal file or directory",
                    path.display()
                );
            }
        }
    }
    Ok(())
}

fn local_catalog_files(
    catalog: &ParquetDataCatalog,
    canonical_root: &Path,
    files: &[String],
) -> anyhow::Result<(u64, Vec<PathBuf>)> {
    let mut total = 0_u64;
    let mut canonical_files = Vec::with_capacity(files.len());
    for file in files {
        let reconstructed = PathBuf::from(catalog.reconstruct_full_uri(file));
        validate_catalog_path_components(canonical_root, &reconstructed)?;
        let canonical_file = reconstructed.canonicalize().with_context(|| {
            format!(
                "failed to canonicalize catalog file '{}'",
                reconstructed.display()
            )
        })?;
        ensure!(
            canonical_file.starts_with(canonical_root),
            "catalog file '{}' escapes the local catalog root",
            reconstructed.display()
        );
        let metadata = fs::metadata(&canonical_file).with_context(|| {
            format!(
                "failed to inspect catalog file '{}'",
                canonical_file.display()
            )
        })?;
        total = total
            .checked_add(metadata.len())
            .context("local catalog size overflow")?;
        canonical_files.push(canonical_file);
    }
    canonical_files.sort();
    canonical_files.dedup();
    ensure!(
        canonical_files.len() == files.len(),
        "catalog file list contains aliases or duplicates"
    );
    Ok((total, canonical_files))
}

fn validate_catalog_path_components(root: &Path, path: &Path) -> anyhow::Result<()> {
    let relative = path.strip_prefix(root).with_context(|| {
        format!(
            "catalog file '{}' escapes the local catalog root",
            path.display()
        )
    })?;
    validate_relative_file_path(relative)?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            anyhow::bail!(
                "catalog file '{}' has an invalid path component",
                path.display()
            );
        };
        current.push(name);
        let metadata = fs::symlink_metadata(&current)
            .with_context(|| format!("failed to inspect catalog path '{}'", current.display()))?;
        ensure!(
            !metadata.file_type().is_symlink(),
            "catalog path '{}' must not contain symbolic links",
            current.display()
        );
    }
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("failed to inspect catalog file '{}'", path.display()))?;
    ensure!(
        metadata.is_file(),
        "catalog file '{}' must be a normal file",
        path.display()
    );
    Ok(())
}

fn validate_relative_file_path(path: &Path) -> anyhow::Result<()> {
    ensure!(
        !path.as_os_str().is_empty(),
        "catalog relative path is empty"
    );
    ensure!(
        path.components()
            .all(|component| matches!(component, Component::Normal(_))),
        "catalog relative path '{}' is invalid",
        path.display()
    );
    Ok(())
}

fn copy_file_nofollow(
    source_root: &Dir,
    destination_root: &Dir,
    path: &Path,
) -> anyhow::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let file_name = path
        .file_name()
        .context("catalog snapshot file is missing its name")?;
    let source_parent = open_relative_directory_nofollow(source_root, parent, false)?;
    let destination_parent = open_relative_directory_nofollow(destination_root, parent, true)?;

    let mut source_options = OpenOptions::new();
    source_options.read(true).follow(FollowSymlinks::No);
    let mut source = source_parent
        .open_with(file_name, &source_options)
        .with_context(|| format!("failed to open catalog source '{}'", path.display()))?;

    let mut destination_options = OpenOptions::new();
    destination_options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    let mut destination = destination_parent
        .open_with(file_name, &destination_options)
        .with_context(|| format!("failed to create catalog snapshot '{}'", path.display()))?;
    io::copy(&mut source, &mut destination)
        .with_context(|| format!("failed to copy catalog snapshot '{}'", path.display()))?;
    destination
        .sync_all()
        .with_context(|| format!("failed to sync catalog snapshot '{}'", path.display()))?;
    let mut permissions = destination.metadata()?.permissions();
    permissions.set_readonly(true);
    destination.set_permissions(permissions)?;
    Ok(())
}

fn open_relative_directory_nofollow(root: &Dir, path: &Path, create: bool) -> anyhow::Result<Dir> {
    let mut directory = root.try_clone()?;
    for component in path.components() {
        let Component::Normal(name) = component else {
            anyhow::bail!(
                "catalog directory '{}' has an invalid component",
                path.display()
            );
        };
        if create {
            match directory.create_dir(name) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error.into()),
            }
        }
        directory = directory
            .open_dir_nofollow(name)
            .with_context(|| format!("failed to open catalog directory '{}'", path.display()))?;
    }
    Ok(directory)
}

fn dataset_content_sha256<T: Serialize, U: Serialize>(
    instrument: &T,
    quotes: &U,
) -> anyhow::Result<String> {
    let mut writer = DigestWriter::new();
    writer.write_all(b"ntpro.local_quote_dataset.v1\n")?;
    serde_json::to_writer(&mut writer, instrument)?;
    writer.write_all(b"\n")?;
    serde_json::to_writer(&mut writer, quotes)?;
    let digest = writer.finish();
    Ok(format!("sha256:{}", lowercase_hex(digest.as_ref())))
}

struct DigestWriter {
    context: DigestContext,
}

impl DigestWriter {
    fn new() -> Self {
        Self {
            context: DigestContext::new(&SHA256),
        }
    }

    fn finish(self) -> aws_lc_rs::digest::Digest {
        self.context.finish()
    }
}

impl Write for DigestWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.context.update(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn lowercase_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}
