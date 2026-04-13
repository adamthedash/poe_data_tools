use anyhow::anyhow;
use winnow::{
    Parser,
    ascii::{dec_uint, float, space1},
    binary::length_repeat,
    combinator::{cond, preceded as P},
    token::literal,
};

use super::types::*;
use crate::file_parsers::{
    VersionedResult, VersionedResultExt,
    shared::{
        lift::{SliceParser, lift},
        winnow::{
            WinnowParser, filename, optional_filename, quoted, quoted_str, repeat_array, uint,
            version_line,
        },
    },
};

fn materials<'a>() -> impl SliceParser<'a, &'a str, Vec<Material>> {
    let parser = length_repeat(
        lift(P("Materials ", uint)),
        lift(
            (
                quoted('"').and_then(optional_filename("mat")), //
                P(space1, uint),
            )
                .map(|(mat_file, unk1)| Material { mat_file, unk1 }),
        ),
    );

    winnow::trace!("materials", parser)
}

fn bone_group<'a>() -> impl WinnowParser<&'a str, BoneGroup> {
    winnow::trace!(
        "bone_group",
        (
            quoted_str, //
            P(
                space1,
                length_repeat(
                    dec_uint::<_, u32, _>, //
                    P(space1, quoted_str),
                ),
            ),
        )
            .map(|(name, bones)| BoneGroup { name, bones })
    )
}

fn bone_groups<'a>() -> impl SliceParser<'a, &'a str, Vec<BoneGroup>> {
    winnow::trace!(
        "bone_groups",
        length_repeat(
            lift(P(
                (literal("BoneGroups"), space1), //
                dec_uint::<_, u32, _>,
            )), //
            lift(bone_group()),
        )
    )
}

pub fn parse_sm_str(contents: &str) -> VersionedResult<SMFile> {
    let lines = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>();
    let mut lines = lines.as_slice();

    let version = lift(version_line())
        .parse_next(&mut lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    let mut parser = (
        lift(P("SkinnedMeshData ", quoted('"').and_then(filename("smd")))), //
        materials(),
        cond(
            version >= 5,
            lift(P("BoundingBox", repeat_array(P(space1, float)))),
        ),
        cond(version >= 6, bone_groups()),
    )
        .map(|(smd_file, materials, bbox, bone_groups)| SMFile {
            version,
            smd_file,
            materials,
            bbox,
            bone_groups,
        });

    parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))
        .with_version(Some(version))
}
