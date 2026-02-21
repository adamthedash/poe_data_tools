use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::{dec_int, dec_uint, float, space1},
    binary::length_repeat,
    combinator::{cond, opt, preceded as P, repeat, trace},
    token::literal,
};

use super::types::*;
use crate::file_parsers::{
    lift_winnow::{SliceParser, lift},
    shared::winnow::{
        TraceHelper, WinnowParser, quoted_str, separated_array, unquoted_str, version_line,
    },
};

fn animation_stage<'a>() -> impl SliceParser<'a, &'a str, AnimationStage> {
    (
        lift(quoted_str), //
        lift(dec_uint),
        lift(separated_array(space1, float)),
    )
        .map(|(name, uint1, floats)| AnimationStage {
            name,
            uint1,
            floats,
        })
        .trace("animation_stage")
}

fn bone_rotation<'a>(num_coords: usize) -> impl WinnowParser<&'a str, BoneRotation> {
    trace("bone_rotation", move |input: &mut &str| {
        let bone = quoted_str(input)?;

        let coord_order = P(space1, unquoted_str).parse_next(input)?;

        let coords = repeat(
            num_coords * coord_order.len(), //
            P(space1, dec_int::<_, i32, _>),
        )
        .parse_next(input)?;

        let rotation = BoneRotation {
            bone,
            coord_order,
            coords,
        };

        Ok(rotation)
    })
}

fn bone_rotations<'a>() -> impl SliceParser<'a, &'a str, Vec<BoneRotation>> {
    trace("bone_rotations", |input: &mut &[&str]| {
        let (num_rotations, num_coords): (usize, Option<usize>) =
            lift((dec_uint, opt(P(space1, dec_uint)))).parse_next(input)?;

        let rotations = repeat(
            num_rotations, //
            lift(bone_rotation(num_coords.unwrap_or(0))),
        )
        .parse_next(input)?;

        Ok(rotations)
    })
}

fn group<'a>(version: u32) -> impl SliceParser<'a, &'a str, Group> {
    (
        lift(quoted_str),
        lift(unquoted_str),
        lift(dec_uint),
        length_repeat(
            lift(dec_uint::<_, u32, _>), //
            animation_stage(),
        ),
        lift(length_repeat(
            dec_uint::<_, u32, _>, //
            P(space1, float::<_, f32, _>),
        )),
        cond(version >= 4, bone_rotations()),
        opt((lift(dec_int), lift(dec_int))),
    )
        .map(
            |(
                name,
                animation_type,
                animation_time,
                animation_stages,
                float_group,
                bone_rotations,
                extra_ints,
            )| {
                Group {
                    name,
                    animation_type,
                    animation_time,
                    animation_stages,
                    float_group,
                    bone_rotations,
                    extra_ints,
                }
            },
        )
        .trace("group")
}

fn bone_group<'a>() -> impl WinnowParser<&'a str, BoneGroup> {
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
        .trace("bone_group")
}

fn bone_groups<'a>() -> impl SliceParser<'a, &'a str, Vec<BoneGroup>> {
    length_repeat(
        lift(P(
            (literal("BoneGroups"), space1), //
            dec_uint::<_, u32, _>,
        )), //
        lift(bone_group()),
    )
    .trace("bone_groups")
}

pub fn parse_amd_str(contents: &str) -> Result<AMDFile> {
    let lines = contents
        .lines()
        // TODO: trimming here might be a bad idea
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();
    let mut lines = lines.as_slice();

    let version = lift(version_line())
        .parse_next(&mut lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    let mut parser = (
        length_repeat(
            lift(dec_uint::<_, u32, _>), //
            group(version),
        ),
        cond(version >= 5, bone_groups()),
    )
        .map(|(groups, bone_groups)| AMDFile {
            version,
            groups,
            bone_groups,
        });

    let amd_file = parser
        .parse(lines)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(amd_file)
}
