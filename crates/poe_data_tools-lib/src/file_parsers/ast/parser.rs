use annotated_parser::{
    ForwardRef, ParserAdapter,
    parsers::{EoF, TakeArray, TakeVec},
    prelude::*,
};

use super::types::*;
use crate::file_parsers::{
    bundle::parser::bundle,
    error::{AsParseError, ParseResultEx, Result},
    shared::annotated_parser::{U8Parser, take_arr_u8},
};

fn header() -> impl U8Parser<Output = Header> {
    (
        u8::LE.trace("version"),
        u8::LE.trace("num_bones"),
        u8::LE.trace("unk1"),
        u16::LE.trace("num_animations"),
        u8::LE.trace("unk3"),
        u8::LE.trace("unk4"),
        u8::LE.trace("num_lights"),
    )
        .map_silent(
            |(version, num_bones, unk1, num_animations, unk3, unk4, num_lights)| Header {
                version,
                num_bones,
                unk1,
                num_animations,
                unk3,
                unk4,
                num_lights,
            },
        )
}

fn bone(version: ForwardRef<u8>) -> impl U8Parser<Output = Bone> {
    let name_length = u8::LE.trace("name_length").store();
    let name = TakeVec::new(name_length.output())
        .try_map(String::from_utf8)
        .trace("name");

    (
        u8::LE.map(|i| (i < 255).then_some(i)).trace("sibling"),
        u8::LE.map(|i| (i < 255).then_some(i)).trace("child"),
        f32::LE.repeat::<4>().repeat::<4>().trace("transform"),
        name_length,
        u8::LE.run_if(version.map(|v| *v >= 8)).trace("unk1"),
        name,
    )
        .map_silent(
            |(sibling, child, transform, name_length, unk1, name)| Bone {
                sibling,
                child,
                transform,
                name_length,
                unk1,
                name,
            },
        )
}

fn light(version: ForwardRef<u8>) -> impl U8Parser<Output = Light> {
    let name_length = u8::LE.trace("name_length").store();
    let name = TakeVec::new(name_length.output())
        .try_map(String::from_utf8)
        .trace("name");

    (
        name_length,
        TakeArray::<51>,
        take_arr_u8::<4>().run_if(version.map(|v| *v >= 7)),
        take_arr_u8::<4>().run_if(version.map(|v| *v >= 9)),
        name,
    )
        .map_silent(
            |(name_length, unk_bytes1, unk_bytes2, unk_bytes3, name)| Light {
                name_length,
                unk_bytes1,
                unk_bytes2,
                unk_bytes3,
                name,
            },
        )
}

/// Track data for v8+
fn track(version: ForwardRef<u8>) -> impl U8Parser<Output = Track> {
    let header = (
        u8::LE.trace("unk1").run_if(version.map(|v| *v < 8)),
        u32::LE.trace("bone_index"),
        u32::LE.trace("num_scales"),
        u32::LE.trace("num_rotations"),
        u32::LE.trace("num_positions"),
        u32::LE.trace("num_unk2"),
        u32::LE.trace("num_unk3"),
        u32::LE.trace("num_unk4"),
    )
        .map_silent(
            |(
                unk1,
                index,
                num_scales,
                num_rotations,
                num_positions,
                num_unk2,
                num_unk3,
                num_unk4,
            )| TrackHeader {
                unk1,
                index,
                num_scales,
                num_rotations,
                num_positions,
                num_unk2,
                num_unk3,
                num_unk4,
            },
        )
        .trace("header")
        .store();

    let header_out = header.output();

    (
        header,
        take_arr_u8::<8>()
            .trace("unk1")
            .run_if(version.map(|v| *v >= 8)),
        f32::LE
            .repeat::<4>()
            .repeat_vec(header_out.map(|h| h.num_scales))
            .trace("scales"),
        f32::LE
            .repeat::<5>()
            .repeat_vec(header_out.map(|h| h.num_rotations))
            .trace("rotations"),
        f32::LE
            .repeat::<4>()
            .repeat_vec(header_out.map(|h| h.num_positions))
            .trace("positions"),
        f32::LE
            .repeat::<4>()
            .repeat_vec(header_out.map(|h| h.num_unk2))
            .trace("unk2s"),
        f32::LE
            .repeat::<5>()
            .repeat_vec(header_out.map(|h| h.num_unk3))
            .trace("unk3s"),
        f32::LE
            .repeat::<4>()
            .repeat_vec(header_out.map(|h| h.num_unk4))
            .trace("unk4s"),
    )
        .map_silent(
            |(header, unk1, scales, rotations, positions, unk2s, unk3s, unk4s)| Track {
                header,
                unk1,
                scales,
                rotations,
                positions,
                unk2s,
                unk3s,
                unk4s,
            },
        )
        .trace("track")
}

fn animation(version: ForwardRef<u8>) -> impl U8Parser<Output = Animation> {
    let num_tracks = u8::LE.store().trace("num_tracks");
    let unk1 = u8::LE.store().trace("unk1");

    let name_length = u8::LE.trace("name_length").store();
    let name = TakeVec::new(name_length.output())
        .try_map(String::from_utf8)
        .trace("name");

    let parent_name_length = u8::LE.trace("parent_name_length").store();
    let parent_name = TakeVec::new(parent_name_length.output())
        .try_map(String::from_utf8)
        .trace("parent_name");

    let data_location = (
        u32::LE.trace("offset"), //
        u32::LE.trace("length"),
    )
        .map(|(offset, length)| DataLocation { offset, length })
        .run_if(version.map(|v| *v >= 8));

    let data = track(version.clone())
        .repeat_vec(num_tracks.output())
        .run_if(version.map(|v| *v < 8));

    (
        num_tracks,
        unk1,
        u8::LE.trace("framerate"),
        u8::LE.trace("unk2"),
        u8::LE.run_if(version.map(|v| *v >= 10)).trace("unk3"),
        name_length,
        parent_name_length.run_if(version.map(|v| *v >= 11)),
        data_location,
        name,
        parent_name.run_if(version.map(|v| *v >= 11)),
        data,
    )
        .map_silent(
            |(
                num_tracks,
                unk1,
                framerate,
                unk2,
                unk3,
                name_length,
                parent_name_length,
                data_location,
                name,
                parent_name,
                data,
            )| {
                Animation {
                    num_tracks,
                    unk1,
                    framerate,
                    unk2,
                    unk3,
                    name_length,
                    parent_name_length,
                    data_location,
                    name,
                    parent_name,
                    data,
                }
            },
        )
}

pub fn ast_parser() -> (impl U8Parser<Output = ASTFile>, ForwardRef<u8>) {
    let header = header().store().trace("header");

    let version = header.output().map(|h| h.version);

    let bones = bone(version.clone())
        .trace("bone")
        .repeat_vec(header.output().map(|h| h.num_bones));
    let lights = light(version.clone())
        .trace("light")
        .repeat_vec(header.output().map(|h| h.num_lights));
    let animations = animation(version.clone())
        .trace("animation")
        .repeat_vec(header.output().map(|h| h.num_animations));

    let bundle = bundle().run_if(version.map(|v| *v >= 8));

    let parser = (header, bones, lights, animations, bundle, EoF).map_silent(
        |(header, bones, lights, animations, bundle, _)| ASTFile {
            header,
            bones,
            lights,
            animations,
            bundle,
        },
    );

    (parser, version)
}

pub fn parse_ast_bytes(mut input: &[u8]) -> Result<ASTFile> {
    let (mut parser, version) = ast_parser();

    let (ast_file, _) = parser
        .parse(&mut input)
        .to_parse_error()
        .with_maybe_version(version.try_get().map(|v| v as u32))?;

    Ok(ast_file)
}
