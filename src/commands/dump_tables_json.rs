use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufWriter,
    path::Path,
};

use anyhow::{Context, Result, ensure};
use glob::{MatchOptions, Pattern};
use winnow::Parser;

use crate::{
    VERBOSE,
    commands::Patch,
    dat::{
        ivy_schema::{Enumeration, SchemaCollection, fetch_schema, load_schema},
        parser::create_parser,
    },
    file_parsers::{
        FileParser,
        dat::{DatParser, types::DatFile},
    },
    fs::{FS, FileSystem},
};

fn resolve_enum(schema: &Enumeration) -> Vec<serde_json::Value> {
    std::iter::repeat_n(serde_json::Value::Null, schema.indexing)
        .chain(schema.enumerators.iter().map(|e| match e {
            Some(value) => serde_json::Value::String(value.clone()),
            None => serde_json::Value::Null,
        }))
        .collect()
}

type ResolvedKeys = HashMap<String, Option<Vec<serde_json::Value>>>;

/// Depth-first resolution of table keys
fn resolve_keys(
    fs: &mut FS,
    schemas: &SchemaCollection,
    version: &Patch,
    keys: &mut ResolvedKeys,
    table_name: &str,
    resolve_keys_stack: &mut Vec<String>,
) -> anyhow::Result<()> {
    let schema = schemas
        .tables
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(table_name))
        .context("Failed to find schema for table")?;

    let mut keys_columns = schema.primary_keys().collect::<Vec<_>>();
    if keys_columns.is_empty()
        && let Some(col_name) = schema.column_names().next()
    {
        // Fall back to first column as key for tables without any key
        log::debug!(
            "No keys for table {:?}, falling back to first column: {:?}",
            schema.name,
            col_name
        );
        keys_columns.push(col_name);
    }

    let ref_keys = schema
        .enumerate()
        // Select key columns that are references
        .filter_map(|(name, c)| {
            keys_columns
                .contains(&name)
                .then_some(c.get_ref())
                .flatten()
        })
        .map(|s| s.to_lowercase())
        // And only ones that have not yet had their keys resolved
        .filter(|table_name| !keys.contains_key(table_name))
        .collect::<Vec<_>>();

    if !ref_keys.is_empty() {
        log::debug!("Table not yet resolvable: {table_name}");
        // This table is not yet ready to be resolved
        resolve_keys_stack.push(table_name.to_owned());
        resolve_keys_stack.extend(ref_keys);
        return Ok(());
    }

    // All reference keys have been resolved, so this table can be resolved
    // Load up this file's contents
    let filename = match version.major() {
        1 => format!("data/{}.datc64", table_name),
        2 => format!("data/balance/{}.datc64", table_name),
        _ => unreachable!("Invalid major version"),
    };
    let bytes = fs.read(&filename).context("Failed to read file contents")?;
    let contents = DatParser
        .parse(&bytes)
        .as_anyhow()
        .context("Failed to parse dat file")?;

    let DatFile {
        rows,
        variable_data,
    } = contents;

    // FIXME: Figure out a way to give variable section to the parser without leaking it to a
    //          'static lifetime
    let variable_section = Box::leak(Box::new(variable_data.clone()));
    let parsed = {
        let mut parser = create_parser(keys, variable_section, schema);

        rows.iter()
            .map(|row| parser.parse(row).unwrap_or(serde_json::Value::Null))
            .collect::<Vec<_>>()
    };

    // Extract keys from the parsed table
    let key_values = (!keys_columns.is_empty()).then(|| {
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

    log::debug!("Resolved keys for table: {table_name}");
    if keys.insert(table_name.to_owned(), key_values).is_some() {
        unreachable!("Keys already present for {:?}", table_name);
    }

    Ok(())
}

fn resolve_table(
    fs: &mut FS,
    schemas: &SchemaCollection,
    version: &Patch,
    keys: &mut ResolvedKeys,
    table_name: &str,
) -> anyhow::Result<Vec<serde_json::Value>> {
    let schema = schemas
        .tables
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(table_name))
        .context("Failed to find schema for table")?;

    // Start off with all unresolved children in the stack
    let mut resolve_keys_stack = schema
        .references()
        .map(|r| r.to_lowercase())
        .filter(|r| !keys.contains_key(r))
        .collect::<Vec<_>>();

    // Recursively resolve all keys
    while let Some(child) = resolve_keys_stack.pop() {
        // Child may have already been resolved, so check again
        if keys.contains_key(&child) {
            continue;
        }

        resolve_keys(fs, schemas, version, keys, &child, &mut resolve_keys_stack)?;
    }

    // All keys for reference tables have been resolved, so we can now fully resolve this table
    // Load up this file's contents
    let filename = match version.major() {
        1 => format!("data/{}.datc64", table_name),
        2 => format!("data/balance/{}.datc64", table_name),
        _ => unreachable!("Invalid major version"),
    };
    let bytes = fs.read(&filename).context("Failed to read file contents")?;
    let contents = DatParser
        .parse(&bytes)
        .as_anyhow()
        .context("Failed to parse dat file")?;

    let DatFile {
        rows,
        variable_data,
    } = contents;

    // FIXME: Figure out a way to give variable section to the parser without leaking it to a
    //          'static lifetime
    let variable_section = Box::leak(Box::new(variable_data.clone()));
    let parsed = {
        let mut parser = create_parser(keys, variable_section, schema);

        rows.iter()
            .map(|row| parser.parse(row).unwrap_or(serde_json::Value::Null))
            .collect::<Vec<_>>()
    };

    Ok(parsed)
}

fn dump_table(
    fs: &mut FS,
    version: &Patch,
    schemas: &SchemaCollection,
    output_folder: &Path,
    resolved: &mut ResolvedKeys,
    filename: &str,
) -> anyhow::Result<()> {
    let path = Path::new(&filename);
    let table_name = path.file_stem().unwrap().to_str().unwrap().to_lowercase();

    let json = resolve_table(fs, schemas, version, resolved, &table_name)
        .context("Failed to resolve table")?;

    // Save out
    let output_path = output_folder.join(path).with_added_extension("json");
    fs::create_dir_all(output_path.parent().unwrap()).context("Failed to create output folder")?;

    let mut out =
        BufWriter::new(File::create(&output_path).context("Failed to create output file")?);
    serde_json::to_writer_pretty(&mut out, &json).context("Failed to serialize json")?;

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

    // Load schema
    let schemas = if let Some(path) = schema {
        load_schema(path.as_ref()).context("Failed to load schema file")?
    } else {
        fetch_schema(cache_dir).context("Failed to fetch schema file")?
    }
    .filter_version(version);

    let mut resolved = HashMap::new();

    // Resolve enums first as they have no dependencies
    schemas.enumerations.iter().for_each(|e| {
        let e_resolved = resolve_enum(e);
        resolved.insert(e.name.to_lowercase(), Some(e_resolved));
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

    filenames.into_iter().for_each(|filename| {
        let result = dump_table(
            fs,
            version,
            &schemas,
            output_folder,
            &mut resolved,
            &filename,
        );

        if let Err(e) = result {
            let error_message = if *VERBOSE.get().unwrap() {
                format!("{e:?}")
            } else {
                format!("{e}")
            };
            log::error!("Failed to extract file {filename:?}: {error_message}");
        } else {
            log::info!("Extracted file: {}", filename);
        }
    });

    Ok(())
}
