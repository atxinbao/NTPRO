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
    source: Option<DataLoadSource>,
    mapping: Option<DataLoadMapping>,
    output: Option<DataOutputConfig>,
}

#[derive(Debug)]
struct DataQuery {
    data_type: String,
    identifier: String,
    start_time: Option<String>,
    end_time: Option<String>,
}

#[derive(Debug)]
struct DataLoadSource {
    kind: String,
    path: PathBuf,
    schema: String,
}

#[derive(Debug)]
struct DataLoadMapping {
    data_type: String,
    instrument_id: String,
    timestamp_column: String,
}

#[derive(Debug)]
struct DataOutputConfig {
    dir: Option<PathBuf>,
    write_summary: bool,
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
    let config = load_data_catalog_config(&opt.config)?;
    ensure_load_mode(&config, "data load")?;
    let source = config
        .source
        .as_ref()
        .context("source section is required for data load")?;
    let mapping = config
        .mapping
        .as_ref()
        .context("mapping section is required for data load")?;
    validate_load_scope(source, mapping)?;

    let row_count = count_csv_data_rows(&source.path, &mapping.timestamp_column)?;
    let output_dir = resolve_data_output_dir(&config, opt)?;
    let catalog_file = write_fixture_catalog_copy(&config, source, mapping)?;
    let summary = format_data_load_summary(
        &config,
        source,
        mapping,
        row_count,
        &catalog_file,
        &output_dir,
    );

    if config
        .output
        .as_ref()
        .is_none_or(|output| output.write_summary)
    {
        write_data_artifact(&output_dir, "summary.txt", &summary)?;
    }

    println!("{}", first_summary_line(&summary));
    Ok(())
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

    let parsed_queries = if run_mode == "load" {
        Vec::new()
    } else {
        parse_data_queries(&value)?
    };
    let source = if run_mode == "load" {
        Some(parse_load_source(&value, path)?)
    } else {
        None
    };
    let mapping = if run_mode == "load" {
        Some(parse_load_mapping(&value)?)
    } else {
        None
    };
    let output = if run_mode == "load" {
        Some(parse_output_config(&value, path)?)
    } else {
        None
    };

    Ok(DataCatalogConfig {
        config_path: path.to_path_buf(),
        run_id,
        run_mode,
        catalog_path,
        catalog_protocol,
        queries: parsed_queries,
        source,
        mapping,
        output,
    })
}

fn parse_data_queries(value: &toml::Value) -> anyhow::Result<Vec<DataQuery>> {
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
    Ok(parsed_queries)
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

fn parse_load_source(config: &toml::Value, config_path: &Path) -> anyhow::Result<DataLoadSource> {
    let source = required_table(config, "source")?;
    let kind = string_field(source, "source.kind", "fixture")?;
    let path = resolve_config_relative_path(config_path, &required_string(source, "source.path")?);
    let schema = string_field(source, "source.schema", "quote_tick_csv_v1")?;
    Ok(DataLoadSource { kind, path, schema })
}

fn parse_load_mapping(config: &toml::Value) -> anyhow::Result<DataLoadMapping> {
    let mapping = required_table(config, "mapping")?;
    let data_type = string_field(mapping, "mapping.data_type", "QuoteTick")?;
    let instrument_id = required_string(mapping, "mapping.instrument_id")?;
    let timestamp_column = string_field(mapping, "mapping.timestamp_column", "ts_init")?;
    Ok(DataLoadMapping {
        data_type,
        instrument_id,
        timestamp_column,
    })
}

fn parse_output_config(
    config: &toml::Value,
    config_path: &Path,
) -> anyhow::Result<DataOutputConfig> {
    let Some(output) = config.get("output").and_then(toml::Value::as_table) else {
        return Ok(DataOutputConfig {
            dir: None,
            write_summary: true,
        });
    };
    let dir = optional_non_empty_string(output, "dir")
        .map(|path| resolve_config_relative_path(config_path, &path));
    let write_summary = output
        .get("write_summary")
        .and_then(toml::Value::as_bool)
        .unwrap_or(true);
    Ok(DataOutputConfig { dir, write_summary })
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

fn ensure_load_mode(config: &DataCatalogConfig, command: &str) -> anyhow::Result<()> {
    if config.run_mode != "load" {
        anyhow::bail!("{command} requires run.mode 'load'");
    }
    Ok(())
}

fn validate_load_scope(source: &DataLoadSource, mapping: &DataLoadMapping) -> anyhow::Result<()> {
    if source.kind != "fixture" {
        anyhow::bail!("source.kind must be 'fixture', got '{}'", source.kind);
    }
    if source.schema != "quote_tick_csv_v1" {
        anyhow::bail!(
            "source.schema must be 'quote_tick_csv_v1', got '{}'",
            source.schema
        );
    }
    if mapping.data_type != "QuoteTick" {
        anyhow::bail!(
            "mapping.data_type must be 'QuoteTick', got '{}'",
            mapping.data_type
        );
    }
    if mapping.timestamp_column != "ts_init" {
        anyhow::bail!(
            "mapping.timestamp_column must be 'ts_init', got '{}'",
            mapping.timestamp_column
        );
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

fn format_data_load_summary(
    config: &DataCatalogConfig,
    source: &DataLoadSource,
    mapping: &DataLoadMapping,
    row_count: usize,
    catalog_file: &Path,
    output_dir: &Path,
) -> String {
    let lines = vec![
        format!(
            "data.load status=ok run_id={} config={} catalog={} catalog_file={} source={} data_type={} instrument_id={} rows={} runtime_status=completed",
            config.run_id,
            config.config_path.display(),
            config.catalog_path.display(),
            catalog_file.display(),
            source.path.display(),
            mapping.data_type,
            mapping.instrument_id,
            row_count
        ),
        "command=data.load".to_string(),
        "status=ok".to_string(),
        format!("run_id={}", config.run_id),
        format!("config={}", config.config_path.display()),
        format!("catalog={}", config.catalog_path.display()),
        format!("catalog_file={}", catalog_file.display()),
        format!("source={}", source.path.display()),
        format!("source_kind={}", source.kind),
        format!("source_schema={}", source.schema),
        format!("data_type={}", mapping.data_type),
        format!("instrument_id={}", mapping.instrument_id),
        format!("timestamp_column={}", mapping.timestamp_column),
        format!("rows={row_count}"),
        format!("output_dir={}", output_dir.display()),
        "runtime_status=completed".to_string(),
        "external_adapter=false".to_string(),
        "semantic_decode=false".to_string(),
        String::new(),
    ];
    lines.join("\n")
}

fn resolve_data_output_dir(
    config: &DataCatalogConfig,
    opt: &DataLoadOpt,
) -> anyhow::Result<PathBuf> {
    if let Some(output) = &opt.output {
        return Ok(output.clone());
    }
    if let Some(output) = &config.output
        && let Some(dir) = &output.dir
    {
        return Ok(dir.clone());
    }
    Ok(PathBuf::from("runs").join(&config.run_id))
}

fn write_fixture_catalog_copy(
    config: &DataCatalogConfig,
    source: &DataLoadSource,
    mapping: &DataLoadMapping,
) -> anyhow::Result<PathBuf> {
    fs::create_dir_all(&config.catalog_path).with_context(|| {
        format!(
            "failed to create catalog directory '{}'",
            config.catalog_path.display()
        )
    })?;
    let catalog_file = config
        .catalog_path
        .join(format!("{}.csv", catalog_file_stem(mapping)));
    fs::copy(&source.path, &catalog_file).with_context(|| {
        format!(
            "failed to copy fixture '{}' into catalog file '{}'",
            source.path.display(),
            catalog_file.display()
        )
    })?;
    Ok(catalog_file)
}

fn count_csv_data_rows(path: &Path, timestamp_column: &str) -> anyhow::Result<usize> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read source fixture '{}'", path.display()))?;
    let mut lines = raw.lines().filter(|line| !line.trim().is_empty());
    let header = lines.next().with_context(|| {
        format!(
            "source fixture '{}' must contain a header row",
            path.display()
        )
    })?;
    let columns = header.split(',').map(str::trim).collect::<Vec<_>>();
    if !columns.contains(&timestamp_column) {
        anyhow::bail!(
            "source fixture '{}' must contain timestamp column '{}'",
            path.display(),
            timestamp_column
        );
    }
    let rows = lines.count();
    if rows == 0 {
        anyhow::bail!(
            "source fixture '{}' must contain at least one data row",
            path.display()
        );
    }
    Ok(rows)
}

fn catalog_file_stem(mapping: &DataLoadMapping) -> String {
    format!(
        "{}-{}",
        sanitize_file_component(&mapping.data_type),
        sanitize_file_component(&mapping.instrument_id)
    )
}

fn sanitize_file_component(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    sanitized.trim_matches('_').to_string()
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

    fn load_config(catalog_path: &Path, source_path: &Path, output_dir: &Path) -> String {
        format!(
            r#"[run]
id = "load-quotes"
mode = "load"

[catalog]
path = "{}"
protocol = "file"

[source]
kind = "fixture"
path = "{}"
schema = "quote_tick_csv_v1"

[mapping]
data_type = "QuoteTick"
instrument_id = "AUD/USD.SIM"
timestamp_column = "ts_init"

[output]
dir = "{}"
write_summary = true
"#,
            catalog_path.display(),
            source_path.display(),
            output_dir.display()
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
    fn data_load_fixture_writes_catalog_copy_and_summary() {
        let dir = temp_dir("load-fixture");
        let source_file = dir.join("quotes.csv");
        let catalog_dir = dir.join("catalog");
        let output_dir = dir.join("out");
        let config = dir.join("load.toml");
        write_file(
            &source_file,
            "ts_init,bid,ask,bid_size,ask_size\n1,1.0,1.1,100,100\n2,1.1,1.2,100,100\n",
        );
        write_file(
            &config,
            &load_config(&catalog_dir, &source_file, &output_dir),
        );

        run_data_command(DataOpt {
            command: DataCommand::Load(DataLoadOpt {
                config: config.clone(),
                run_id: None,
                output: None,
            }),
        })
        .unwrap();

        let catalog_file = catalog_dir.join("QuoteTick-AUD_USD_SIM.csv");
        let copied = fs::read_to_string(&catalog_file).unwrap();
        assert!(copied.contains("ts_init,bid,ask"));
        let summary = fs::read_to_string(output_dir.join("summary.txt")).unwrap();
        assert!(summary.contains("command=data.load"));
        assert!(summary.contains("runtime_status=completed"));
        assert!(summary.contains("rows=2"));
        assert!(summary.contains(&format!("catalog_file={}", catalog_file.display())));
        assert!(summary.contains(&format!("config={}", config.display())));
    }

    #[test]
    fn data_load_rejects_unsupported_schema() {
        let dir = temp_dir("load-unsupported-schema");
        let source_file = dir.join("quotes.csv");
        let catalog_dir = dir.join("catalog");
        let output_dir = dir.join("out");
        let config = dir.join("load.toml");
        write_file(&source_file, "ts_init,bid,ask\n1,1.0,1.1\n");
        let raw_config = load_config(&catalog_dir, &source_file, &output_dir)
            .replace("quote_tick_csv_v1", "python_pickle");
        write_file(&config, &raw_config);

        let error = run_data_command(DataOpt {
            command: DataCommand::Load(DataLoadOpt {
                config,
                run_id: None,
                output: None,
            }),
        })
        .unwrap_err()
        .to_string();

        assert!(error.contains("source.schema must be 'quote_tick_csv_v1'"));
        assert!(error.contains("python_pickle"));
    }

    #[test]
    fn config_shape_validation_does_not_require_catalog_to_exist() {
        let dir = temp_dir("shape-only");
        let config = dir.join("data.toml");
        write_file(&config, &data_config(&dir.join("missing.csv"), "QuoteTick"));

        validate_data_catalog_config_file(&config).unwrap();
    }
}
