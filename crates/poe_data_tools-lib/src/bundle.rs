use anyhow::Context;
use bytes::Bytes;
use oozextract::Extractor;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use crate::file_parsers::bundle::types::BundleFile;

impl BundleFile {
    /// Return the entire content of the bundle
    pub fn read_all(&self) -> anyhow::Result<Bytes> {
        self.read_range(0, self.head.uncompressed_size as usize)
    }

    pub fn read_range(&self, offset: usize, len: usize) -> anyhow::Result<Bytes> {
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
            .try_for_each(|(chunk, block)| {
                let mut ext = Extractor::new();

                ext.read_from_slice(block, chunk)
                    .map(|_| ())
                    .context("Failed to decompress bundle block")
            })?;

        // Grab subset form block aligned buffer
        let slice = Bytes::from(buf).slice(offset % block_size..offset % block_size + len);
        Ok(slice)
    }
}
