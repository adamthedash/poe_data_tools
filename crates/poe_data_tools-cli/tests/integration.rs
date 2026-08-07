//!
//! Tests that the CLI "works" at a surface level. Doesn't check for correctness of outputs, just
//! that the CLI workflows have functioned correctly.
//!

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

use assert_cmd::Command;
use tempfile::TempDir;

const PATCH: &str = "1";

const LIST_GLOB: &str = "**/*.datc64";
const CAT_PATH: &str = "data/stats.datc64";
const EXTRACT_PATH: &str = "data/stats.datc64";
const DUMP_TABLES_PATH: &str = "data/stats.datc64";
const DUMP_ART_PATH: &str = "art/2dart/atlas/atlas.dds";
const DUMP_TREES_PATH: &str = "metadata/passiveskillgraph.psg";

// NOTE: Just testing one file type as per-file type tests are better placed in the lib
const TRANSLATE_PATH: &str = "metadata/passiveskillgraph.psg";

/// Process-local singleton
static SHARED_CACHE: OnceLock<PathBuf> = OnceLock::new();
fn shared_cache() -> &'static Path {
    SHARED_CACHE.get_or_init(|| {
        let dir = TempDir::new().expect("failed to create shared cache dir");
        let path = dir.path().to_owned();
        // Leak the TempDir so it lives until the test process exits.
        Box::leak(Box::new(dir));
        path
    })
}

fn base_cmd(cache_dir: &Path) -> Command {
    let mut cmd =
        Command::cargo_bin("poe_data_tools").expect("failed to find poe_data_tools binary");
    cmd.arg("--patch").arg(PATCH);
    cmd.arg("--cache-dir").arg(cache_dir);
    cmd
}

#[test]
fn test_list() {
    let mut cmd = base_cmd(shared_cache());
    cmd.arg("list").arg(LIST_GLOB);
    let result = cmd.assert().success();

    assert!(!result.get_output().stdout.is_empty());
}

#[test]
fn test_cat() {
    let mut cmd = base_cmd(shared_cache());
    cmd.arg("cat").arg(CAT_PATH);
    let result = cmd.assert().success();

    assert!(!result.get_output().stdout.is_empty());
}

#[test]
fn test_extract() {
    let out = TempDir::new().expect("failed to create output dir");

    let mut cmd = base_cmd(shared_cache());
    cmd.arg("extract").arg(out.path()).arg(EXTRACT_PATH);
    cmd.assert().success();

    let expected = out.path().join(EXTRACT_PATH);
    assert!(expected.exists(), "expected extracted file at {expected:?}");
}

#[test]
fn test_dump_tables_csv() {
    let out = TempDir::new().expect("failed to create output dir");

    let mut cmd = base_cmd(shared_cache());
    cmd.arg("dump-tables").arg(out.path()).arg(DUMP_TABLES_PATH);
    cmd.assert().success();

    let expected = out.path().join(DUMP_TABLES_PATH).with_extension("csv");
    assert!(expected.exists(), "expected CSV at {expected:?}");
}

#[test]
fn test_dump_tables_json() {
    let out = TempDir::new().expect("failed to create output dir");

    let mut cmd = base_cmd(shared_cache());
    cmd.arg("dump-tables")
        .arg("--mode")
        .arg("json")
        .arg(out.path())
        .arg(DUMP_TABLES_PATH);
    cmd.assert().success();

    let expected = out.path().join(format!("{DUMP_TABLES_PATH}.json"));
    assert!(expected.exists(), "expected JSON at {expected:?}");
}

#[test]
fn test_dump_art() {
    let out = TempDir::new().expect("failed to create output dir");

    let mut cmd = base_cmd(shared_cache());
    cmd.arg("dump-art").arg(out.path()).arg(DUMP_ART_PATH);
    cmd.assert().success();

    let expected = out.path().join(DUMP_ART_PATH).with_extension("png");
    assert!(expected.exists(), "expected PNG at {expected:?}");
}

#[test]
fn test_dump_trees() {
    let out = TempDir::new().expect("failed to create output dir");

    let mut cmd = base_cmd(shared_cache());
    cmd.arg("dump-trees").arg(out.path()).arg(DUMP_TREES_PATH);
    cmd.assert().success();

    let expected = out.path().join(DUMP_TREES_PATH).with_extension("json");
    assert!(expected.exists(), "expected JSON at {expected:?}");
}

#[test]
fn test_translate() {
    let out = TempDir::new().expect("failed to create output dir");

    let mut cmd = base_cmd(shared_cache());
    cmd.arg("translate").arg(out.path()).arg(TRANSLATE_PATH);
    cmd.assert().success();

    let expected = out.path().join(format!("{TRANSLATE_PATH}.json"));
    assert!(expected.exists(), "expected JSON at {expected:?}");
}
