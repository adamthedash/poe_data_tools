use anyhow::anyhow;
use nom::{
    Parser,
    character::complete::{space1, u32 as U},
    combinator::{all_consuming, opt},
    multi::many0,
    sequence::preceded,
};

use crate::file_parsers::{
    FileParser,
    gft::types::{GFTFile, GenFile, Section},
    line_parser::{
        MultilineParser, Result as LResult, nom_adapter, single_line, take_forever, take_many,
    },
    shared::{quoted_str, unquoted_str, utf16_bom_to_string, version_line},
};

pub mod types;

pub struct GFTParser;

impl FileParser for GFTParser {
    type Output = GFTFile;

    fn parse(&self, bytes: &[u8]) -> anyhow::Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        let parsed =
            parse_gft_str(&contents).map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

        Ok(parsed)
    }
}

fn file<'a>() -> impl MultilineParser<'a, GenFile> {
    let line_parser = |line| {
        let (line, weight) = U(line)?;

        let (line, path) = preceded(space1, quoted_str)(line)?;

        let (line, rotations) = all_consuming(many0(preceded(space1, unquoted_str)))(line)?;

        let gen_file = GenFile {
            weight,
            path,
            rotations,
        };

        Ok((line, gen_file))
    };

    single_line(nom_adapter(line_parser))
}

fn section<'a>(version: u32) -> impl MultilineParser<'a, Section> {
    move |lines| {
        let (lines, (name, uint1)) =
            single_line(nom_adapter(quoted_str.and(opt(preceded(space1, U)))))(lines)?;

        let (lines, _num_files) = if version == 1 {
            let (lines, num_files) = single_line(nom_adapter(U))(lines)?;

            (lines, Some(num_files))
        } else {
            (lines, None)
        };

        let (lines, files) = take_many(file())(lines)?;

        let section = Section { name, files, uint1 };

        Ok((lines, section))
    }
}

fn parse_gft_str(contents: &str) -> LResult<GFTFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();

    let (lines, version) = version_line()(&lines)?;

    let (lines, _num_sections) = if version == 1 {
        let (lines, num_sections) = single_line(nom_adapter(U))(lines)?;

        (lines, Some(num_sections))
    } else {
        (lines, None)
    };

    let (lines, sections) = take_forever(section(version))(lines)?;
    assert!(lines.is_empty(), "File not fully consumed: {:#?}", lines);

    let gft_file = GFTFile { version, sections };

    Ok(gft_file)
}
