use annotated_parser::{
    ForwardRef, ParserAdapter,
    parsers::{TakeArray, TakeVec},
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
        u8::LE.trace("num_animations"),
        u8::LE.trace("unk2"),
        u8::LE.trace("unk3"),
        u8::LE.trace("unk4"),
        u8::LE.trace("num_lights"),
    )
        .map_silent(
            |(version, num_bones, unk1, num_animations, unk2, unk3, unk4, num_lights)| Header {
                version,
                num_bones,
                unk1,
                num_animations,
                unk2,
                unk3,
                unk4,
                num_lights,
            },
        )
}

fn bone() -> impl U8Parser<Output = Bone> {
    let name_length = u8::LE.trace("name_length").store();
    let name = TakeVec::new(name_length.output())
        .try_map(String::from_utf8)
        .trace("name");

    (
        u8::LE.trace("sibling"),
        u8::LE.trace("child"),
        f32::LE.repeat::<4>().repeat::<4>().trace("transform"),
        name_length,
        u8::LE.trace("unk1"),
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
        TakeArray::<55>,
        take_arr_u8::<4>().run_if(version.map(|v| *v >= 9)),
        name,
    )
        .map_silent(|(name_length, unk_bytes1, unk_bytes2, name)| Light {
            name_length,
            unk_bytes1,
            unk_bytes2,
            name,
        })
}

fn animation(version: ForwardRef<u8>) -> impl U8Parser<Output = Animation> {
    let name_length = u8::LE.trace("name_length").store();
    let name = TakeVec::new(name_length.output())
        .try_map(String::from_utf8)
        .trace("name");

    let parent_name_length = u8::LE.trace("parent_name_length").store();
    let parent_name = TakeVec::new(parent_name_length.output())
        .try_map(String::from_utf8)
        .trace("parent_name");

    (
        u8::LE.trace("num_tracks"),
        u8::LE.trace("unk1"),
        u8::LE.trace("framerate"),
        u8::LE.trace("unk2"),
        u8::LE.run_if(version.map(|v| *v >= 10)).trace("unk3"),
        name_length,
        parent_name_length.run_if(version.map(|v| *v >= 11)),
        u32::LE.trace("offset"),
        u32::LE.trace("size"),
        name,
        parent_name.run_if(version.map(|v| *v >= 11)),
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
                data_offset,
                data_size,
                name,
                parent_name,
            )| Animation {
                num_tracks,
                unk1,
                framerate,
                unk2,
                unk3,
                name_length,
                parent_name_length,
                data_offset,
                data_size,
                name,
                parent_name,
            },
        )
}

pub fn ast_parser() -> (
    impl U8Parser<Output = ASTFile>,
    impl ForwardRefGet<Value = u8>,
) {
    let header = header().store().trace("header");
    let bones = bone()
        .trace("bone")
        .repeat_vec(header.output().map(|h| h.num_bones));
    let lights = light(header.output().map(|h| h.version))
        .trace("light")
        .repeat_vec(header.output().map(|h| h.num_lights));
    let animations = animation(header.output().map(|h| h.version))
        .trace("animation")
        .repeat_vec(header.output().map(|h| h.num_animations));

    let version = header.output().map(|h| h.version);

    let parser = (header, bones, lights, animations, bundle())
        .map_silent(|(header, bones, lights, animations, bundle)| ASTFile {
            header,
            bones,
            lights,
            animations,
            bundle,
        })
        .trace("ast_file");

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
