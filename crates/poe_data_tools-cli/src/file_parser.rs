use std::{io::BufWriter, path::Path};

use anyhow::{Context, Result};
use poe_data_tools::file_parsers::{FileParser, VersionedFile};
use serde::Serialize;

pub trait FileParserExt {
    /// Parse and serialise to JSON
    fn parse_to_json_file(&self, bytes: &[u8], output_path: &Path) -> Result<()>;
}

impl<P> FileParserExt for P
where
    P: FileParser,
    P::Output: Serialize + VersionedFile,
{
    fn parse_to_json_file(&self, bytes: &[u8], output_path: &Path) -> Result<()> {
        let parsed = self.parse(bytes).context("failed to parse file")?;

        std::fs::create_dir_all(output_path.parent().unwrap())
            .context("Failed to create folder")?;

        let f = std::fs::File::create(output_path)
            .with_context(|| format!("Failed to create file {:?}", output_path))?;
        let f = BufWriter::new(f);

        serde_json::to_writer(f, &parsed).context("Failed to serialise")?;

        Ok(())
    }
}
