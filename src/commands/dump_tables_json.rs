use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::BufWriter,
    path::Path,
};

use anyhow::{Context, Result, bail, ensure};
use glob::{MatchOptions, Pattern};
use winnow::Parser;

use crate::{
    VERBOSE,
    bundle_fs::FS,
    commands::Patch,
    dat::{
        ivy_schema::{DatTableSchema, Enumeration, SchemaCollection, fetch_schema, load_schema},
        parser::create_parser,
    },
    file_parsers::{
        FileParser,
        dat::{DatParser, types::DatFile},
    },
};

fn resolve_enum(schema: &Enumeration) -> Vec<serde_json::Value> {
    std::iter::repeat_n(serde_json::Value::Null, schema.indexing)
        .chain(schema.enumerators.iter().map(|e| match e {
            Some(value) => serde_json::Value::String(value.clone()),
            None => serde_json::Value::Null,
        }))
        .collect()
}

pub enum TableResolutionStatus {
    /// Non-reference columns resolved
    /// Non-ref keys resolved
    Resolving {
        keys: Option<Vec<serde_json::Value>>,
    },
    /// All columns resolved
    Resolved {
        keys: Option<Vec<serde_json::Value>>,
        table: Vec<serde_json::Value>,
    },
}

impl TableResolutionStatus {
    pub fn keys(&self) -> Option<&Vec<serde_json::Value>> {
        match self {
            TableResolutionStatus::Resolving { keys } => keys.as_ref(),
            TableResolutionStatus::Resolved { keys, .. } => keys.as_ref(),
        }
    }
}

/// Attempt to resolve a table without recursing
/// If any children have not yet been partially resolved, then this table cannot yet be resolved
fn try_resolve_table(
    tables: &HashMap<String, TableResolutionStatus>,
    contents: &DatFile,
    schema: &DatTableSchema,
) -> TableResolutionStatus {
    let DatFile {
        rows,
        variable_data,
    } = contents;

    // FIXME: Figure out a way to give variable section to the parser without leaking it to a
    //          'static lifetime
    let variable_section = Box::leak(Box::new(variable_data.clone()));
    // (At least partially) resolve table
    let parsed = {
        let mut parser = create_parser(tables, variable_section, schema);

        rows.iter()
            .map(|row| parser.parse(row).unwrap_or(serde_json::Value::Null))
            .collect::<Vec<_>>()
    };

    // Extract keys, this should be valid in most cases, but if key column is a reference, it may
    // or may not be resolved yet. This might be a problem. Ideally all keys should be plain types
    let keys_columns = schema.primary_keys().collect::<Vec<_>>();
    let keys_columns = if keys_columns.is_empty() {
        if let Some(col_name) = schema.column_names().next() {
            // Fall back to first column as key for tables without any key
            log::debug!(
                "No keys for table {:?}, falling back to first column: {:?}",
                schema.name,
                col_name
            );
            vec![col_name]
        } else {
            // Table has 0 columns, so can't fall back to anything
            vec![]
        }
    } else {
        keys_columns
    };
    let keys = (!keys_columns.is_empty()).then(|| {
        // Try get the corresponding values for them
        parsed
            .iter()
            .map(|row| {
                let keys = keys_columns
                    .iter()
                    .map(|k| row.get(k).unwrap_or(&serde_json::Value::Null).clone())
                    .collect::<Vec<_>>();

                // If there's multiple primary keys, use a list
                match keys.len() {
                    0 => unreachable!(),
                    1 => keys[0].clone(),
                    _ => serde_json::Value::Array(keys),
                }
            })
            .collect::<Vec<_>>()
    });

    if schema
        .columns
        .iter()
        .filter_map(|c| c.get_ref())
        .all(|c| tables.contains_key(&c.to_lowercase()))
    {
        // All children are at least partially resolved, so this table should be fully resolved
        TableResolutionStatus::Resolved {
            keys,
            table: parsed,
        }
    } else {
        // At least one child has not yet been seen (i.e. keys not yet resolved), so this table
        // needs to be re-resolved layer
        TableResolutionStatus::Resolving { keys }
    }
}

/// Recursively resolve a table and all children
fn resolve_table(
    fs: &mut FS,
    schemas: &SchemaCollection,
    tables: &mut HashMap<String, TableResolutionStatus>,
    table_name: &str,
    version: &Patch,
) -> anyhow::Result<()> {
    // Load up this file's contents
    let filename = match version.major() {
        1 => format!("data/{}.datc64", table_name),
        2 => format!("data/balance/{}.datc64", table_name),
        _ => unreachable!("Invalid major version"),
    };
    let bytes = fs.read(&filename).context("Failed to read file contents")?;
    let schema = schemas
        .tables
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(table_name))
        .context("Failed to find schema for table")?;

    let contents = DatParser
        .parse(&bytes)
        .as_anyhow()
        .context("Failed to parse dat file")?;

    // Resolve this table once
    let parsed = try_resolve_table(tables, &contents, schema);
    match &parsed {
        TableResolutionStatus::Resolving { .. } => {
            log::debug!("Partially resolved table: {:?}", table_name)
        }
        TableResolutionStatus::Resolved { .. } => {
            log::debug!("Fully resolved table: {:?}", table_name)
        }
    }

    tables.insert(table_name.to_lowercase(), parsed);

    // Resolve all children
    let unresolved_children = schema
        .references()
        .filter(|child| !tables.contains_key(&child.to_lowercase()))
        .collect::<HashSet<_>>();
    for child in &unresolved_children {
        resolve_table(fs, schemas, tables, child, version)?;
    }

    let has_self_ref = schema
        .references()
        .any(|r| r.eq_ignore_ascii_case(table_name));

    if !has_self_ref && unresolved_children.is_empty() {
        // All children resolved, so don't need a 2nd pass
        return Ok(());
    }

    // 2nd pass at parent table to resolve child & self references
    log::debug!("2nd pass: {:?}", table_name);
    let parsed = try_resolve_table(tables, &contents, schema);
    match &parsed {
        TableResolutionStatus::Resolving { .. } => {
            log::debug!("Partially resolved table: {:?}", table_name)
        }
        TableResolutionStatus::Resolved { .. } => {
            log::debug!("Fully resolved table: {:?}", table_name)
        }
    }
    tables.insert(table_name.to_lowercase(), parsed);

    Ok(())
}

pub fn dump_tables(
    fs: &mut FS,
    patterns: &[Pattern],
    cache_dir: &Path,
    output_folder: &Path,
    version: &Patch,
    schema: Option<impl AsRef<Path>>,
) -> Result<()> {
    for pattern in patterns {
        ensure!(
            pattern.as_str().ends_with(".datc64"),
            "Only .datc64 table export is supported."
        );
    }

    // Load schema: todo: Get this from Ivy's CDN / cache it
    let schemas = if let Some(path) = schema {
        load_schema(path.as_ref()).context("Failed to load schema file")?
    } else {
        fetch_schema(cache_dir).context("Failed to fetch schema file")?
    }
    .filter_version(version);

    // Remove tables that don't exist in the index
    // This happens when the schema is out of sync
    let schemas = SchemaCollection {
        tables: schemas
            .tables
            .iter()
            .filter(|s| {
                let filename = match version.major() {
                    1 => format!("data/{}.datc64", s.name),
                    2 => format!("data/balance/{}.datc64", s.name),
                    _ => unreachable!("Invalid major version"),
                };

                // Check that this file can be read
                let res = fs.read(&filename);
                if res.is_err() {
                    log::debug!("File not in index: {}", filename);
                }

                res.is_ok()
            })
            .cloned()
            .collect(),
        enumerations: schemas.enumerations.clone(),
    };

    let mut resolved = HashMap::new();

    // Resolve enums first as they have no dependencies
    schemas.enumerations.iter().for_each(|e| {
        let e_resolved = resolve_enum(e);
        resolved.insert(
            e.name.to_lowercase(),
            TableResolutionStatus::Resolved {
                keys: Some(e_resolved.clone()),
                table: e_resolved.clone(),
            },
        );
    });

    let schema_names = schemas
        .tables
        .iter()
        .map(|t| t.name.to_lowercase())
        .collect::<Vec<_>>();

    // Filter list of files we're going to extract
    let filenames = fs
        .list()
        // Filter on glob
        .filter(|filename| {
            patterns.iter().any(|pattern| {
                pattern.matches_with(
                    filename,
                    MatchOptions {
                        require_literal_separator: true,
                        ..Default::default()
                    },
                )
            })
        })
        // Skip files we can't process
        .filter(|filename| {
            let path = Path::new(filename);
            let table_name = path.file_stem().unwrap().to_str().unwrap().to_lowercase();

            let keep = schema_names.contains(&table_name);

            if !keep {
                log::warn!("Skipping {:?}, schema not found", path);
            }

            keep
        })
        .collect::<Vec<_>>();

    filenames
        .into_iter()
        .map(|filename| -> anyhow::Result<_> {
            let path = Path::new(&filename);
            let table_name = path.file_stem().unwrap().to_str().unwrap().to_lowercase();

            resolve_table(fs, &schemas, &mut resolved, &table_name, version)
                .context("Failed to resolve table")?;
            let TableResolutionStatus::Resolved { table: json, .. } = &resolved[&table_name] else {
                bail!("Table not fully resolved after 2nd pass: {:?}", table_name);
            };

            // Save out
            let output_path = output_folder.join(path).with_added_extension("json");
            fs::create_dir_all(output_path.parent().unwrap())
                .context("Failed to create output folder")?;

            let mut out =
                BufWriter::new(File::create(&output_path).context("Failed to create output file")?);
            serde_json::to_writer_pretty(&mut out, json).context("Failed to serialize json")?;

            Ok(filename)
        })
        .for_each(|result| match result {
            Ok(filename) => log::info!("Extracted file: {}", filename),
            Err(e) => {
                let error_message = if *VERBOSE.get().unwrap() {
                    format!("{e:?}")
                } else {
                    format!("{e}")
                };
                log::error!("Failed to extract file: {error_message}");
            }
        });

    Ok(())
}
