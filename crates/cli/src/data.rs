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
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;

use crate::opt::{DataCommand, DataInspectOpt, DataLoadOpt, DataOpt, DataValidateOpt};

const SUPPORTED_DATA_TYPES: &[&str] = &[
    "Bar",
    "FundingRateUpdate",
    "InstrumentAny",
    "OrderBookDelta",
    "OrderBookDepth10",
    "QuoteTick",
    "TradeTick",
];

#[derive(Debug)]
struct DataCatalogConfig {
    config_path: PathBuf,
    run_id: String,
    run_mode: String,
    catalog_path: PathBuf,
    catalog_protocol: String,
    queries: Vec<DataQuery>,
}

#[derive(Debug)]
struct DataQuery {
    data_type: String,
    identifier: String,
    start_time: Option<String>,
    end_time: Option<String>,
}

#[derive(Debug)]
struct DataPathInspection {
    path: PathBuf,
    kind: &'static str,
    size_bytes: Option<u64>,
    extension: Option<String>,
    entry_count: Option<usize>,
    discovered_entries: Vec<String>,
}

pub(crate) fn run_data_command(opt: DataOpt) -> anyhow::Result<()> {
    match opt.command {
        DataCommand::Inspect(inspect) => run_data_inspect(&inspect),
        DataCommand::Validate(validate) => run_data_validate(&validate),
        DataCommand::Load(load) => run_data_load(&load),
    }
}

pub(crate) fn validate_data_catalog_config_file(path: &Path) -> anyhow::Result<()> {
    load_data_catalog_config(path)?;
    Ok(())
}

fn run_data_inspect(opt: &DataInspectOpt) -> anyhow::Result<()> {
    let config = load_data_catalog_config(&opt.config)?;
    ensure_inspect_or_validate_mode(&config, "data inspect")?;
    let inspection = inspect_catalog_path(&config)?;
    let summary = format_data_summary("data.inspect", &config, &inspection);

    if let Some(output_dir) = &opt.output {
        write_data_artifact(output_dir, "inspection.txt", &summary)?;
    }

    println!("{}", first_summary_line(&summary));
    Ok(())
}

fn run_data_validate(opt: &DataValidateOpt) -> anyhow::Result<()> {
    let config = load_data_catalog_config(&opt.config)?;
    ensure_inspect_or_validate_mode(&config, "data validate")?;
    let inspection = inspect_catalog_path(&config)?;
    let summary = format_data_summary("data.validate", &config, &inspection);

    println!("{}", first_summary_line(&summary));
    Ok(())
}

fn run_data_load(opt: &DataLoadOpt) -> anyhow::Result<()> {
    anyhow::bail!(
        "data load is defined but not implemented yet for config '{}'; see docs/rust-cutover/product/DATA_CATALOG_CLI_CONTRACT.md",
        opt.config.display()
    )
}

fn load_data_catalog_config(path: &Path) -> anyhow::Result<DataCatalogConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read data config '{}'", path.display()))?;
    let value: toml::Value = toml::from_str(&raw)
        .with_context(|| format!("failed to parse data config '{}'", path.display()))?;
    if value.as_table().is_none_or(toml::value::Table::is_empty) {
        anyhow::bail!("data config must contain at least one TOML table");
    }

    let run = required_table(&value, "run")?;
    let run_id = required_string(run, "run.id")?;
    let run_mode = one_of(run, "run.mode", &["inspect", "validate", "load"])?;

    let catalog = required_table(&value, "catalog")?;
    let catalog_path_raw = required_string(catalog, "catalog.path")?;
    let catalog_protocol = string_field(catalog, "catalog.protocol", "file")?;
    let catalog_path = resolve_config_relative_path(path, &catalog_path_raw);

    let queries = value
        .get("queries")
        .and_then(toml::Value::as_array)
        .context("queries must contain at least one data query")?;
    if queries.is_empty() {
        anyhow::bail!("queries must contain at least one data query");
    }

    let mut parsed_queries = Vec::with_capacity(queries.len());
    for (index, query) in queries.iter().enumerate() {
        let table = query
            .as_table()
            .with_context(|| format!("queries[{index}] must be a table"))?;
        parsed_queries.push(parse_data_query(index, table)?);
    }

    Ok(DataCatalogConfig {
        config_path: path.to_path_buf(),
        run_id,
        run_mode,
        catalog_path,
        catalog_protocol,
        queries: parsed_queries,
    })
}

fn parse_data_query(index: usize, table: &toml::value::Table) -> anyhow::Result<DataQuery> {
    let data_type = required_string(table, &format!("queries[{index}].data_type"))?;
    if !SUPPORTED_DATA_TYPES.contains(&data_type.as_str()) {
        anyhow::bail!(
            "queries[{index}].data_type '{data_type}' is not supported by the Rust data CLI"
        );
    }

    let identifier = if let Some(instrument_id) = optional_non_empty_string(table, "instrument_id")
    {
        format!("instrument_id={instrument_id}")
    } else if let Some(bar_type) = optional_non_empty_string(table, "bar_type") {
        format!("bar_type={bar_type}")
    } else {
        anyhow::bail!("queries[{index}] must declare either instrument_id or bar_type");
    };

    let start_time = optional_non_empty_string(table, "start_time");
    let end_time = optional_non_empty_string(table, "end_time");
    if let (Some(start), Some(end)) = (&start_time, &end_time)
        && start >= end
    {
        anyhow::bail!("queries[{index}].start_time must be earlier than queries[{index}].end_time");
    }

    Ok(DataQuery {
        data_type,
        identifier,
        start_time,
        end_time,
    })
}

fn ensure_inspect_or_validate_mode(
    config: &DataCatalogConfig,
    command: &str,
) -> anyhow::Result<()> {
    if config.run_mode == "load" {
        anyhow::bail!("{command} does not support run.mode 'load'");
    }
    Ok(())
}

fn inspect_catalog_path(config: &DataCatalogConfig) -> anyhow::Result<DataPathInspection> {
    let metadata = fs::metadata(&config.catalog_path).with_context(|| {
        format!(
            "catalog path '{}' does not exist or is not readable",
            config.catalog_path.display()
        )
    })?;

    if metadata.is_file() {
        let _file = fs::File::open(&config.catalog_path).with_context(|| {
            format!(
                "catalog file '{}' is not readable",
                config.catalog_path.display()
            )
        })?;
        return Ok(DataPathInspection {
            path: config.catalog_path.clone(),
            kind: "file",
            size_bytes: Some(metadata.len()),
            extension: file_extension(&config.catalog_path),
            entry_count: None,
            discovered_entries: Vec::new(),
        });
    }

    if metadata.is_dir() {
        let mut entries = Vec::new();
        for entry in fs::read_dir(&config.catalog_path).with_context(|| {
            format!(
                "catalog directory '{}' is not readable",
                config.catalog_path.display()
            )
        })? {
            let entry = entry.with_context(|| {
                format!(
                    "failed to read an entry under '{}'",
                    config.catalog_path.display()
                )
            })?;
            entries.push(entry.file_name().to_string_lossy().to_string());
        }
        entries.sort();

        return Ok(DataPathInspection {
            path: config.catalog_path.clone(),
            kind: "directory",
            size_bytes: None,
            extension: None,
            entry_count: Some(entries.len()),
            discovered_entries: entries.into_iter().take(10).collect(),
        });
    }

    anyhow::bail!(
        "catalog path '{}' must be a regular file or directory",
        config.catalog_path.display()
    )
}

fn format_data_summary(
    command: &str,
    config: &DataCatalogConfig,
    inspection: &DataPathInspection,
) -> String {
    let data_types = data_types_summary(config);
    let query_filters = query_filters_summary(config);
    let mut lines = vec![
        format!(
            "{command} status=ok run_id={} config={} catalog={} protocol={} kind={} queries={} data_types={}",
            config.run_id,
            config.config_path.display(),
            inspection.path.display(),
            config.catalog_protocol,
            inspection.kind,
            config.queries.len(),
            data_types
        ),
        format!("command={command}"),
        "status=ok".to_string(),
        format!("run_id={}", config.run_id),
        format!("config={}", config.config_path.display()),
        format!("catalog={}", inspection.path.display()),
        format!("protocol={}", config.catalog_protocol),
        format!("kind={}", inspection.kind),
        format!("queries={}", config.queries.len()),
        format!("data_types={data_types}"),
        format!("query_filters={query_filters}"),
    ];

    if let Some(size_bytes) = inspection.size_bytes {
        lines.push(format!("size_bytes={size_bytes}"));
    }
    if let Some(extension) = &inspection.extension {
        lines.push(format!("extension={extension}"));
    }
    if let Some(entry_count) = inspection.entry_count {
        lines.push(format!("entry_count={entry_count}"));
    }
    if !inspection.discovered_entries.is_empty() {
        lines.push(format!(
            "discovered_entries={}",
            inspection.discovered_entries.join(",")
        ));
    }

    lines.push(String::new());
    lines.join("\n")
}

fn data_types_summary(config: &DataCatalogConfig) -> String {
    let mut data_types = BTreeSet::new();
    for query in &config.queries {
        data_types.insert(query.data_type.as_str());
    }
    data_types.into_iter().collect::<Vec<_>>().join(",")
}

fn query_filters_summary(config: &DataCatalogConfig) -> String {
    config
        .queries
        .iter()
        .map(|query| {
            let mut parts = vec![query.data_type.clone(), query.identifier.clone()];
            if let Some(start_time) = &query.start_time {
                parts.push(format!("start_time={start_time}"));
            }
            if let Some(end_time) = &query.end_time {
                parts.push(format!("end_time={end_time}"));
            }
            parts.join(":")
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn write_data_artifact(output_dir: &Path, file_name: &str, summary: &str) -> anyhow::Result<()> {
    fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create output dir '{}'", output_dir.display()))?;
    let path = output_dir.join(file_name);
    fs::write(&path, summary)
        .with_context(|| format!("failed to write data artifact '{}'", path.display()))?;
    Ok(())
}

fn first_summary_line(summary: &str) -> &str {
    summary.lines().next().unwrap_or(summary)
}

fn resolve_config_relative_path(config_path: &Path, raw_path: &str) -> PathBuf {
    let path = PathBuf::from(raw_path);
    if path.is_absolute() || path.exists() {
        return path;
    }
    config_path
        .parent()
        .map(|parent| parent.join(&path))
        .unwrap_or(path)
}

fn file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| !extension.trim().is_empty())
        .map(str::to_ascii_lowercase)
}

fn required_table<'a>(
    value: &'a toml::Value,
    name: &str,
) -> anyhow::Result<&'a toml::value::Table> {
    value
        .get(name)
        .and_then(toml::Value::as_table)
        .with_context(|| format!("{name} section is required"))
}

fn required_string(table: &toml::value::Table, field: &str) -> anyhow::Result<String> {
    let (_, key) = field
        .rsplit_once('.')
        .with_context(|| format!("invalid field path '{field}'"))?;
    let value = table
        .get(key)
        .and_then(toml::Value::as_str)
        .with_context(|| format!("{field} must be a string"))?;
    if value.trim().is_empty() {
        anyhow::bail!("{field} must not be empty");
    }
    Ok(value.to_string())
}

fn one_of(table: &toml::value::Table, field: &str, allowed: &[&str]) -> anyhow::Result<String> {
    let value = required_string(table, field)?;
    if !allowed.contains(&value.as_str()) {
        anyhow::bail!(
            "{field} must be one of {}, got '{value}'",
            allowed.join(", ")
        );
    }
    Ok(value)
}

fn string_field(table: &toml::value::Table, field: &str, expected: &str) -> anyhow::Result<String> {
    let value = required_string(table, field)?;
    if value != expected {
        anyhow::bail!("{field} must be '{expected}', got '{value}'");
    }
    Ok(value)
}

fn optional_non_empty_string(table: &toml::value::Table, key: &str) -> Option<String> {
    table
        .get(key)
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ntpro-gh-156-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_file(path: &Path, content: &str) {
        fs::write(path, content).unwrap();
    }

    fn data_config(catalog_path: &Path, data_type: &str) -> String {
        format!(
            r#"[run]
id = "catalog-audit"
mode = "inspect"

[catalog]
path = "{}"
protocol = "file"

[[queries]]
data_type = "{}"
instrument_id = "AUD/USD.SIM"
start_time = "2025-01-01T00:00:00Z"
end_time = "2025-01-02T00:00:00Z"
"#,
            catalog_path.display(),
            data_type
        )
    }

    #[test]
    fn data_inspect_file_writes_artifact() {
        let dir = temp_dir("inspect-file");
        let data_file = dir.join("quotes.csv");
        let config = dir.join("data.toml");
        let output_dir = dir.join("out");
        write_file(&data_file, "ts_init,bid,ask\n1,1.0,1.1\n");
        write_file(&config, &data_config(&data_file, "QuoteTick"));

        run_data_command(DataOpt {
            command: DataCommand::Inspect(DataInspectOpt {
                config: config.clone(),
                output: Some(output_dir.clone()),
            }),
        })
        .unwrap();

        let artifact = fs::read_to_string(output_dir.join("inspection.txt")).unwrap();
        assert!(artifact.contains("command=data.inspect"));
        assert!(artifact.contains("status=ok"));
        assert!(artifact.contains("kind=file"));
        assert!(artifact.contains("extension=csv"));
        assert!(artifact.contains("data_types=QuoteTick"));
        assert!(artifact.contains(&format!("config={}", config.display())));
    }

    #[test]
    fn data_validate_file_accepts_supported_data_type() {
        let dir = temp_dir("validate-file");
        let data_file = dir.join("quotes.csv");
        let config = dir.join("data.toml");
        write_file(&data_file, "ts_init,bid,ask\n1,1.0,1.1\n");
        write_file(&config, &data_config(&data_file, "QuoteTick"));

        run_data_command(DataOpt {
            command: DataCommand::Validate(DataValidateOpt { config }),
        })
        .unwrap();
    }

    #[test]
    fn data_validate_directory_reports_entries() {
        let dir = temp_dir("validate-dir");
        let catalog_dir = dir.join("catalog");
        let config = dir.join("data.toml");
        fs::create_dir_all(&catalog_dir).unwrap();
        write_file(&catalog_dir.join("quotes.parquet"), "fixture");
        write_file(&config, &data_config(&catalog_dir, "QuoteTick"));

        let loaded = load_data_catalog_config(&config).unwrap();
        let inspection = inspect_catalog_path(&loaded).unwrap();
        let summary = format_data_summary("data.validate", &loaded, &inspection);

        assert!(summary.contains("kind=directory"));
        assert!(summary.contains("entry_count=1"));
        assert!(summary.contains("discovered_entries=quotes.parquet"));
    }

    #[test]
    fn data_validate_missing_catalog_path_errors() {
        let dir = temp_dir("missing-catalog");
        let config = dir.join("data.toml");
        write_file(&config, &data_config(&dir.join("missing.csv"), "QuoteTick"));

        let error = run_data_command(DataOpt {
            command: DataCommand::Validate(DataValidateOpt { config }),
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("catalog path"));
        assert!(error.contains("does not exist or is not readable"));
    }

    #[test]
    fn data_validate_rejects_unsupported_data_type() {
        let dir = temp_dir("unsupported-type");
        let data_file = dir.join("quotes.csv");
        let config = dir.join("data.toml");
        write_file(&data_file, "ts_init,bid,ask\n1,1.0,1.1\n");
        write_file(&config, &data_config(&data_file, "CustomPythonData"));

        let error = run_data_command(DataOpt {
            command: DataCommand::Validate(DataValidateOpt { config }),
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("CustomPythonData"));
        assert!(error.contains("not supported by the Rust data CLI"));
    }

    #[test]
    fn config_shape_validation_does_not_require_catalog_to_exist() {
        let dir = temp_dir("shape-only");
        let config = dir.join("data.toml");
        write_file(&config, &data_config(&dir.join("missing.csv"), "QuoteTick"));

        validate_data_catalog_config_file(&config).unwrap();
    }
}
