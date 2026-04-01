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
    bundle_fs::FS,
    commands::Patch,
    dat::{
        ivy_schema::{Enumeration, SchemaCollection, fetch_schema, load_schema},
        parser::create_parser,
    },
    file_parsers::{
        FileParser,
        dat::{DatParser, types::DatFile},
    },
};

fn build_dependency_graph(schemas: &SchemaCollection) -> HashMap<String, Vec<String>> {
    // Tables
    let mut dependencies = schemas
        .tables
        .iter()
        .map(|s| {
            let deps = s
                .columns
                .iter()
                .filter(|c| c.column_type == "foreignrow" || c.column_type == "enumrow")
                .filter_map(|c| c.references.as_ref())
                .map(|r| r.table.to_lowercase())
                .collect();

            (s.name.to_lowercase(), deps)
        })
        .collect::<HashMap<_, _>>();

    // Enums
    dependencies.extend(
        schemas
            .enumerations
            .iter()
            .map(|e| (e.name.to_lowercase(), vec![])),
    );

    dependencies
}

fn resolve_enum(schema: &Enumeration) -> Vec<serde_json::Value> {
    std::iter::repeat_n(serde_json::Value::Null, schema.indexing)
        .chain(schema.enumerators.iter().map(|e| match e {
            Some(value) => serde_json::Value::String(value.clone()),
            None => serde_json::Value::Null,
        }))
        .collect()
}

/// Recursively resolve a table and its dependencies
fn resolve(
    fs: &mut FS,
    schemas: &SchemaCollection,
    resolved: &mut HashMap<String, Vec<serde_json::Value>>,
    resolved_keys: &mut HashMap<String, Vec<serde_json::Value>>,
    deps: &HashMap<String, Vec<String>>,
    table: &str,
    version: &Patch,
) -> anyhow::Result<()> {
    if resolved.contains_key(table) {
        return Ok(());
    }

    // Resolve children
    if let Some(table_deps) = deps.get(table) {
        table_deps.iter().try_for_each(|dep| {
            resolve(fs, schemas, resolved, resolved_keys, deps, dep, version)
                .context("Failed to resolve child table")
        })?;
    } else {
        // NOTE: If there's no dependency entry, assume everything is a-ok (it's probably not).
        // This can happen when the schema refers to a non-existent table
        log::debug!(
            "No dependency entry for table {:?}, assuming already resolved",
            table
        );
        return Ok(());
    }

    // All dependencies resolved, so resolve this one
    let filename = match version.major() {
        1 => format!("data/{}.datc64", table),
        2 => format!("data/balance/{}.datc64", table),
        _ => unreachable!("Invalid major version"),
    };
    let bytes = fs.read(&filename).context("Failed to read file contents")?;
    let schema = schemas
        .tables
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(table))
        .context("Failed to find schema for table")?;

    let DatFile {
        rows,
        variable_data,
    } = DatParser
        .parse(&bytes)
        .as_anyhow()
        .context("Failed to parse dat file")?;

    // FIXME: Figure out a way to give variable section to the parser without leaking it to a
    // 'static lifetime
    let variable_section = Box::leak(Box::new(variable_data));
    let parsed = {
        let mut parser = create_parser(resolved_keys, variable_section, schema);

        rows.iter()
            .map(|row| match parser.parse(row) {
                Ok(key_vals) => key_vals,
                Err(_) => serde_json::Value::Null,
            })
            .collect::<Vec<_>>()
    };
    let parsed = resolved.entry(table.to_owned()).insert_entry(parsed);
    let parsed = parsed.get();

    let keys_columns = schema.primary_keys().collect::<Vec<_>>();
    if !keys_columns.is_empty() {
        // Try get the corresponding values for them
        let keys = parsed
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
            .collect::<Vec<_>>();

        resolved_keys.insert(table.to_owned(), keys);
    }

    log::info!("Resolved table: {:?}", table);

    // Tables with self-references need to be parsed twice
    let has_self_ref = schema.columns.iter().any(|c| c.column_type == "row");
    if has_self_ref {
        log::info!("Resolving self-refs table: {:?}", table);
        let parsed = {
            let mut parser = create_parser(resolved_keys, variable_section, schema);

            rows.iter()
                .map(|row| match parser.parse(row) {
                    Ok(key_vals) => key_vals,
                    Err(_) => serde_json::Value::Null,
                })
                .collect::<Vec<_>>()
        };

        resolved.insert(table.to_owned(), parsed);
    }

    Ok(())
}

/// Checks whether there is a cycle in the dependencies of a table
fn has_cycle<'a>(
    deps: &'a HashMap<String, Vec<String>>,
    table: &'a str,
    seen: &mut Vec<&'a str>,
) -> bool {
    if seen.contains(&table) {
        return true;
    }
    seen.push(table);

    if let Some(children) = deps.get(table)
        && children.iter().any(|c| has_cycle(deps, c, seen))
    {
        return true;
    }
    seen.pop();

    false
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

    // Build dependency graph
    let mut dependencies = build_dependency_graph(&schemas);

    // Remove ones with cycles
    let cycles = dependencies
        .keys()
        .filter(|t| has_cycle(&dependencies, t, &mut vec![]))
        .cloned()
        .collect::<Vec<_>>();
    for t in cycles {
        dependencies.remove(&t);
        log::warn!("Skipping table due to cycle: {:?}", t);
    }

    let mut resolved = HashMap::new();
    let mut resolved_keys = HashMap::new();

    // Resolve enums first as they have no dependencies
    schemas.enumerations.iter().for_each(|e| {
        let e_resolved = resolve_enum(e);
        let name = e.name.to_lowercase();
        resolved.insert(name.clone(), e_resolved.clone());
        resolved_keys.insert(name, e_resolved);
    });

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

            let keep = dependencies.contains_key(&table_name);

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

            resolve(
                fs,
                &schemas,
                &mut resolved,
                &mut resolved_keys,
                &dependencies,
                &table_name,
                version,
            )
            .context("Failed to resolve table")?;
            let json = &resolved[&table_name];

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
