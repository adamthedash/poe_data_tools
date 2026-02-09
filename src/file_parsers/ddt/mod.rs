pub mod types;

use anyhow::{Result, anyhow};
use nom::{
    Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{space1, u32 as U},
    combinator::opt,
    number::complete::float as F,
    sequence::{Tuple, preceded as P},
};

use crate::file_parsers::{
    FileParser,
    ddt::types::{DDTFile, Group, Line1, Object, Weight},
    line_parser::{
        MultilineParser, NomParser, Result as LResult, nom_adapter, optional, single_line,
        take_forever, take_many,
    },
    shared::{quoted_str, safe_u32, unquoted_str, utf16_bom_to_string, version_line},
};

pub struct DDTParser;

impl FileParser for DDTParser {
    type Output = DDTFile;

    fn parse(&self, bytes: &[u8]) -> Result<Self::Output> {
        let contents = utf16_bom_to_string(bytes)?;

        let parsed =
            parse_ddt_str(&contents).map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

        Ok(parsed)
    }
}

fn line1<'a>() -> impl MultilineParser<'a, Line1> {
    let parser = |line| {
        let (line, scale) = F(line)?;

        let (line, uint1) = opt(P(space1, U))(line)?;
        let (line, uint2) = opt(P(space1, U))(line)?;

        let line1 = Line1 {
            scale,
            uint1,
            uint2,
        };

        Ok((line, line1))
    };

    single_line(nom_adapter(parser))
}

fn group_header<'a>(
    name_parser: impl NomParser<'a, String>,
) -> impl MultilineParser<'a, (String, Option<String>, Option<f32>)> {
    let mut line_parser = (
        name_parser, //
        opt(P(space1, unquoted_str)),
        opt(P(space1, F)),
    );
    let line_parser = move |line| line_parser.parse(line);

    single_line(nom_adapter(line_parser))
}

fn object<'a>() -> impl MultilineParser<'a, Object> {
    let line_parser = |line| {
        let (line, weight) = alt((tag("All").map(|_| Weight::All), F.map(Weight::Float)))(line)?;

        let (line, ao_file) = P(space1, quoted_str)(line)?;

        let (line, uint1) = opt(P(space1, safe_u32))(line)?;

        let (line, d) = opt(P(space1, tag("D").map(String::from)))(line)?;

        let (line, float1) = opt(P(space1, F))(line)?;

        let object = Object {
            weight,
            ao_file,
            d,
            float1,
            uint1,
        };

        Ok((line, object))
    };

    single_line(nom_adapter(line_parser))
}

fn group<'a>(name_parser: impl NomParser<'a, String>) -> impl MultilineParser<'a, Group> {
    let mut group_header = group_header(name_parser);

    move |lines| {
        let (lines, (name, d, float1)) = group_header(lines)?;

        let (lines, objects) = take_many(object())(lines)?;

        let group = Group {
            name,
            d,
            float1,
            objects,
        };

        Ok((lines, group))
    }
}

fn parse_ddt_str(contents: &str) -> LResult<DDTFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();

    let (lines, version) = version_line()(&lines)?;

    let (lines, line1) = line1()(lines)?;

    let (lines, uint1) = optional(single_line(nom_adapter(U)))(lines)?;

    // Default line
    let (lines, default_group) = group(unquoted_str)(lines)?;

    // Rest of groups
    let (lines, mut groups) = take_forever(group(quoted_str))(lines)?;
    groups.insert(0, default_group);

    assert!(lines.is_empty(), "Lines remaining: {:?}", lines);

    let ddt_file = DDTFile {
        version,
        line1,
        uint1,
        groups,
    };

    Ok(ddt_file)
}
