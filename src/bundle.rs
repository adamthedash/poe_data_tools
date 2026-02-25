use std::{fs, path::Path};

use anyhow::{Context, Result};
use bytes::Bytes;
use oozextract::Extractor;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use url::Url;

use crate::{
    bundle_loader::CDNLoader,
    file_parsers::{
        FileParser,
        bundle::{BundleParser, types::BundleFile},
    },
};

impl BundleFile {
    /// Return the entire content of the bundle
    /// todo: decode blocks in parallel
    ///     Also return a result instead of panicing
    pub fn read_all(&self) -> Bytes {
        self.read_range(0, self.head.uncompressed_size as usize)
    }

    pub fn read_range(&self, offset: usize, len: usize) -> Bytes {
        let block_size = self.head.uncompressed_block_granularity as usize;

        // Create a buffer, needs to be block-aligned since we're decoding entire blocks into it
        let block_start = offset / block_size;
        let block_end = (offset + len).div_ceil(block_size);
        let buf_size = (block_end * block_size).min(self.head.uncompressed_size as usize)
            - block_start * block_size;
        let mut buf = vec![0; buf_size];

        // Chunk into slices which can be written to in parallel
        let chunks = buf.chunks_mut(block_size).collect::<Vec<_>>();

        // Decode blocks in parallel
        chunks
            .into_par_iter()
            .zip(&self.blocks[block_start..block_end])
            .for_each(|(chunk, block)| {
                let mut ext = Extractor::new();

                ext.read_from_slice(block, chunk)
                    .context("Failed to decompress bundle block")
                    .unwrap();
            });

        // Grab subset form block aligned buffer
        Bytes::from(buf).slice(offset % block_size..offset % block_size + len)
    }
}

/// Load a bundle file from disk
pub fn load_bundle_content(path: &Path) -> Result<BundleFile> {
    // todo: figure how to properly do error propogation with nom
    let bundle_content = fs::read(path).context("Failed to read bundle file")?;

    let bundle = BundleParser
        .parse(&bundle_content)
        .context("Failed to parse bundle")?;

    Ok(bundle)
}

// Fetch a bundle file from the CDN (or cache)
pub fn fetch_bundle_content(base_url: &Url, cache_dir: &Path, path: &Path) -> Result<BundleFile> {
    let bundle_content = CDNLoader::new(base_url, cache_dir.to_str().unwrap())
        .load(path)
        .context("Failed to load bundle")?;

    let bundle = BundleParser
        .parse(&bundle_content)
        .context("Failed to parse bundle")?;

    Ok(bundle)
}
