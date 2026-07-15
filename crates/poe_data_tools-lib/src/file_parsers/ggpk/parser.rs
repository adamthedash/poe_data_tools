use std::io::{Read, Seek, SeekFrom};

use winnow::{
    Parser,
    binary::{le_u16, le_u32, le_u64},
    combinator::repeat,
    error::ContextError,
    token::take,
};

use super::types::*;
use crate::file_parsers::{
    error::{AsParseError, ParseErrorInner, Result},
    shared::winnow::{WinnowParser, take_array},
};

#[derive(Debug, thiserror::Error)]
enum GGPKError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Winnow(#[from] winnow::error::ContextError),
    #[error("unexpected GGPK chunk tag: {0:?}")]
    UnexpectedTag(Tag),
}

impl<I, E> From<winnow::error::ParseError<I, E>> for GGPKError
where
    E: Into<GGPKError>,
{
    fn from(value: winnow::error::ParseError<I, E>) -> Self {
        value.into_inner().into()
    }
}

impl From<GGPKError> for ParseErrorInner {
    fn from(value: GGPKError) -> Self {
        ParseErrorInner::Other(Box::new(value))
    }
}

#[derive(Debug)]
enum Tag {
    Ggpk,
    PDir,
    File,
    Free,
}

fn tag<'a>() -> impl WinnowParser<&'a [u8], Tag> {
    let parser = take(4_usize).verify_map(|bytes: &[u8]| {
        let tag = match bytes {
            b"GGPK" => Tag::Ggpk,
            b"PDIR" => Tag::PDir,
            b"FILE" => Tag::File,
            b"FREE" => Tag::Free,
            _ => return None,
        };

        Some(tag)
    });

    winnow::trace!("tag", parser)
}

fn chunk_header<'a>() -> impl WinnowParser<&'a [u8], (u32, Tag)> {
    winnow::trace!("chunk_header", (le_u32, tag()))
}

fn pdir<F: Read + Seek>(file: &mut F) -> Result<Entry, GGPKError> {
    let mut buf = [0; 4 + 4 + 32];
    file.read_exact(&mut buf)?;

    let (name_length, num_entries, sha_digest) =
        (le_u32::<_, ContextError>, le_u32, take_array::<32, _>()).parse(buf.as_slice())?;

    let mut buf = vec![
        0;
        name_length as usize * std::mem::size_of::<u16>()
            + num_entries as usize
                * (std::mem::size_of::<u64>() + std::mem::size_of::<u32>())
    ];
    file.read_exact(&mut buf)?;

    let (mut name, entries): (_, Vec<_>) = (
        repeat(name_length as usize, le_u16::<_, ContextError>)
            .try_map(|chars: Vec<_>| String::from_utf16(&chars)),
        repeat(num_entries as usize, (le_u32, le_u64)),
    )
        .parse(buf.as_slice())?;

    name.pop().expect("Name didn't have null terminator");

    let mut buf = [0; 8];
    let entries = entries
        .into_iter()
        .map(|(hash, offset)| -> Result<_, _> {
            file.seek(SeekFrom::Start(offset))?;
            file.read_exact(&mut buf)?;

            let (length, tag) = chunk_header().parse(&buf)?;
            match tag {
                Tag::PDir => pdir(file),
                Tag::File => parse_file(file, length, hash),
                t => Err(GGPKError::UnexpectedTag(t)),
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Entry {
        name,
        hash: None,
        sha_digest,
        data: EntryData::Dir(entries),
    })
}

fn parse_file<F>(file: &mut F, length: u32, hash: u32) -> Result<Entry, GGPKError>
where
    F: Read + Seek,
{
    let start_offset = file.stream_position()? - 8;

    let mut buf = [0; 4 + 32];
    file.read_exact(&mut buf)?;

    let (name_length, sha_digest) =
        (le_u32::<_, ContextError>, take_array::<32, _>()).parse(buf.as_slice())?;

    let mut buf = vec![0; name_length as usize * std::mem::size_of::<u16>()];
    file.read_exact(&mut buf)?;

    let mut name = repeat(name_length as usize, le_u16::<_, ContextError>)
        .try_map(|chars: Vec<_>| String::from_utf16(&chars))
        .parse(buf.as_slice())?;

    name.pop().expect("Name didn't have null terminator");

    let data_offset = file.stream_position()?;
    let header_length = data_offset - start_offset;
    let data_length = length as u64 - header_length;

    Ok(Entry {
        name,
        hash: Some(hash),
        sha_digest,
        data: EntryData::File {
            offset: data_offset as usize,
            length: data_length as usize,
        },
    })
}

fn _parse_ggpk(mut file: impl Read + Seek) -> Result<GGPKFile, GGPKError> {
    let mut buf = [0; 4 + 4 + 4 + 8 + 8];
    file.read_exact(&mut buf)?;
    let ((_length, _tag), _version, entries) = (
        chunk_header().verify(|(_, t)| matches!(t, Tag::Ggpk)),
        le_u32,
        repeat::<_, _, Vec<_>, _, _>(2, le_u64),
    )
        .parse(&buf)?;

    let mut buf = [0; 4 + 4];
    let entries = entries
        .into_iter()
        .map(|offset| -> Result<_, GGPKError> {
            file.seek(SeekFrom::Start(offset))?;
            file.read_exact(&mut buf)?;

            let (_length, tag) = chunk_header().parse(&buf)?;
            let entry = match tag {
                Tag::PDir => pdir(&mut file)?,
                Tag::Free => return Ok(None),
                t => return Err(GGPKError::UnexpectedTag(t)),
            };

            Ok(Some(entry))
        })
        .filter_map(|res| match res {
            Ok(Some(entry)) => Some(Ok(entry)),
            Err(e) => Some(Err(e)),
            // Filter out FREE chunks
            Ok(None) => None,
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(GGPKFile { entries })
}

pub fn parse_ggpk(file: impl Read + Seek) -> Result<GGPKFile> {
    _parse_ggpk(file).to_parse_error()
}
