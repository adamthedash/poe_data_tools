use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    ascii::{dec_int, dec_uint, float, space1},
    binary::length_repeat,
    combinator::{
        alt, dispatch, eof, opt, preceded as P, repeat, repeat_till, separated_pair,
        terminated as T,
    },
    token::{literal, rest},
};

use super::types::*;
use crate::file_parsers::{
    lift_winnow::{SliceParser, lift},
    shared::{
        remove_trailing,
        winnow::{
            TraceHelper, WinnowParser, filename, quoted, quoted_str, repeat_array, unquoted,
            unquoted_str,
        },
    },
};

pub fn render_passes<'a>() -> impl SliceParser<'a, &'a str, Effect> {
    P(
        lift(literal("RenderPasses").map(String::from)),
        (
            lift(literal("{")),
            repeat_till(.., lift(rest), lift(literal("}"))),
        )
            .map(|(b0, (middle, b1)): (&str, (Vec<&str>, &str))| {
                [[b0].as_slice(), &middle, &[b1]].concat().concat()
            }),
    )
    .try_map(|payload| {
        let payload = remove_trailing(&payload);
        serde_json::from_str(&payload)
    })
    .map(Effect::RenderPasses)
    .trace("RenderPasses")
}

pub fn attached_object<'a>() -> impl WinnowParser<&'a str, Effect> {
    (
        quoted_str, //
        P(space1, quoted('"').and_then(filename("ao"))),
        opt(P(space1, literal("\"ignore_errors\""))).map(|x| x.is_some()),
    )
        .map(
            |(attachment_point, ao_file, ignore_errors)| Effect::AttachedObject {
                attachment_point,
                ao_file,
                ignore_errors,
            },
        )
        .trace("AttachedObject")
}

pub fn particle_effect<'a>() -> impl WinnowParser<&'a str, Effect> {
    (
        quoted_str,
        P(space1, quoted('"').and_then(filename("pet"))),
        opt(P(
            (space1, literal("\"limit\"")), //
            repeat_array(P(space1, dec_uint)),
        )),
        opt(P(space1, literal("\"ignore_errors\""))).map(|x| x.is_some()),
    )
        .map(
            |(glob, pet_file, limit, ignore_errors)| Effect::ParticleEffect {
                glob,
                pet_file,
                limit,
                ignore_errors,
            },
        )
        .trace("ParticleEffect")
}

pub fn attached_object_ex<'a>() -> impl WinnowParser<&'a str, Effect> {
    (
        opt(T(quoted_str, space1)),
        length_repeat(
            dec_uint::<_, u32, _>,
            P(
                space1,
                quoted('"').and_then(alt((
                    filename("ao"), //
                    filename("fmt"),
                ))),
            ),
        ),
        repeat_array(P(space1, dec_uint)),
        repeat_array(P(space1, float)),
        (
            opt(P(space1, dec_int)), //
            opt(P(space1, dec_int)),
        )
            .map(|(r0, r1)| [r0, r1]),
        opt(P(space1, literal("\"include_aux\""))).map(|x| x.is_some()),
        opt(P(space1, literal("\"ignore_errors\""))).map(|x| x.is_some()),
        opt(P(space1, literal("\"multi_attach\""))).map(|x| x.is_some()),
    )
        .map(
            |(
                attachment_point,
                files,
                uints1,
                floats,
                rotations,
                include_aux,
                ignore_errors,
                multi_attach,
            )| {
                Effect::AttachedObjectEx {
                    attachment_point,
                    files,
                    uints1,
                    floats,
                    rotations,
                    include_aux,
                    ignore_errors,
                    multi_attach,
                }
            },
        )
        .trace("AttachedObjectEx")
}

pub fn attached_object_bone_index<'a>() -> impl WinnowParser<&'a str, Effect> {
    (
        dec_uint, //
        P(space1, quoted('"').and_then(filename("ao"))),
    )
        .map(|(bone_index, ao_file)| Effect::AttachedObjectBoneIndex {
            bone_index,
            ao_file,
        })
        .trace("AttachedObject")
}

pub fn child_attached_object<'a>() -> impl WinnowParser<&'a str, Effect> {
    (
        quoted('"').and_then(filename("ao")), //
        P((space1, literal("from_bone"), space1), quoted_str),
        P((space1, literal("from_bone_group_index"), space1), dec_uint),
    )
        .map(
            |(ao_file, from_bone, from_bone_group_index)| Effect::ChildAttachedObject {
                ao_file,
                from_bone,
                from_bone_group_index,
            },
        )
}

pub fn trail_effect<'a>() -> impl WinnowParser<&'a str, Effect> {
    (
        quoted_str,
        P(space1, quoted('"').and_then(filename("trl"))),
        opt(P(
            (space1, literal("\"limit\"")), //
            repeat_array(P(space1, dec_uint)),
        )),
        opt(P(space1, literal("\"ignore_errors\""))).map(|x| x.is_some()),
    )
        .map(
            |(glob, trl_file, limit, ignore_errors)| Effect::TrailEffect {
                glob,
                trl_file,
                limit,
                ignore_errors,
            },
        )
        .trace("TrailEffect")
}

pub fn hide_first_pass_after_delay<'a>() -> impl WinnowParser<&'a str, Effect> {
    float
        .map(|delay| Effect::HideFirstPassAfterDelay { delay })
        .trace("HideFirstPassAfterDelay")
}

pub fn hide_first_pass_after_delay_for_duration<'a>() -> impl WinnowParser<&'a str, Effect> {
    separated_pair(float, space1, float)
        .map(|(delay, duration)| Effect::HideFirstPassAfterDelayForDuration { delay, duration })
        .trace("HideFirstPassAfterDelayForDuration")
}

pub fn hide_first_pass_using_epk_parameter<'a>() -> impl WinnowParser<&'a str, Effect> {
    (
        unquoted_str, //
        P(space1, float),
        P(space1, float),
    )
        .map(
            |(parameter, float1, float2)| Effect::HideFirstPassUsingEPKParameter {
                parameter,
                float1,
                float2,
            },
        )
        .trace("HideFirstPassUsingEPKParameter")
}

pub fn hide_first_pass_using_timeline_parameter<'a>() -> impl WinnowParser<&'a str, Effect> {
    (
        unquoted_str, //
        P(space1, float),
        P(space1, float),
    )
        .map(
            |(parameter, float1, float2)| Effect::HideFirstPassUsingTimelineParameter {
                parameter,
                float1,
                float2,
            },
        )
        .trace("HideFirstPassUsingTimelineParameter")
}

pub fn play_misc_effect_pack_after_delay<'a>() -> impl WinnowParser<&'a str, Effect> {
    separated_pair(quoted_str, space1, float)
        .map(|(effect, delay)| Effect::PlayMiscEffectPackAfterDelay { effect, delay })
        .trace("PlayMiscEffectPackAfterDelay")
}

pub fn other_effect<'a>(name: &str) -> impl WinnowParser<&'a str, Effect> {
    rest.map(|rest: &str| Effect::Other {
        name: name.to_string(),
        rest: rest.to_string(),
    })
    .trace("other_effect")
}

pub fn effect<'a>() -> impl SliceParser<'a, &'a str, Effect> {
    alt((
        render_passes(),
        lift(dispatch! {
            T(unquoted(), opt(space1));
            "AttachedObject" => attached_object(),
            "AttachedObjectEx" => attached_object_ex(),
            "AttachedObjectBoneIndex" => attached_object_bone_index(),
            "ChildAttachedObject" => child_attached_object(),
            "ParticleEffect" => particle_effect(),
            "TrailEffect" => trail_effect(),
            "ParentOnlyEffects" => eof.map(|_| Effect::ParentOnlyEffects),
            "ApplyToAllPasses" => eof.map(|_| Effect::ApplyToAllPasses),
            "PlayMiscEffectPackOnEnd" => quoted_str.map(|effect| Effect::PlayMiscEffectPackOnEnd { effect }),
            "PlayMiscEffectPackOnBegin" => quoted_str.map(|effect| Effect::PlayMiscEffectPackOnBegin { effect }),
            "PlayMiscEffectPackAfterDelay" => play_misc_effect_pack_after_delay(),
            "HideFirstPassAfterDelay" => hide_first_pass_after_delay(),
            "HideFirstPassAfterDelayForDuration" => hide_first_pass_after_delay_for_duration(),
            "HideFirstPassUsingEPKParameter" => hide_first_pass_using_epk_parameter(),
            "HideFirstPassUsingTimelineParameter" => hide_first_pass_using_timeline_parameter(),
            name => other_effect(name),
        }),
    ))
    .trace("effect")
}

pub fn parse_epk_str(contents: &str) -> Result<EPKFile> {
    // NOTE: Edge case: sometimes there's an errouneous line break when AttachedObject is the last
    // item
    let contents = contents.replace("\"<root>\" \n", "\"<root>\" ");

    let lines = contents
        .lines()
        .map(|l| l.trim_end())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect::<Vec<_>>();

    let mut parser = repeat(.., effect());

    let epk_file = parser
        .parse(lines.as_slice())
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))?;

    Ok(epk_file)
}
