use anyhow::anyhow;
use winnow::{
    Parser,
    binary::{le_f32, le_u8, le_u16, le_u32, length_repeat},
    combinator::{cond, dispatch, empty, repeat, seq},
    error::ContextError,
    token::take,
};

use super::types::*;
use crate::file_parsers::{
    VersionedResult, VersionedResultExt,
    dolm::parser::{dolm, index_buffer},
    shared::winnow::{WinnowParser, le_f16, repeat_array, take_array},
};

struct UnresolvedShape {
    name: u32,
    material: u32,
    triangle_index: u32,
}

fn v8_vertex<'a>(vertex_format: Option<u32>) -> impl WinnowParser<&'a [u8], Vertex> {
    (
        repeat_array(le_f32),
        take_array(),
        repeat_array(le_f16),
        cond(vertex_format.unwrap_or_default() == 1, repeat_array(le_f16)),
    )
        .map(|(pos, unk, uv, uv2)| Vertex { pos, unk, uv, uv2 })
}

struct UnresolvedSubcomponent {
    unk1: u8,
    num_d1s: u8,
    tag: u32,
}

fn v8_section<'a>(
    version: u8,
    header: &Header,
) -> impl WinnowParser<&'a [u8], (Section, Vec<UnresolvedShape>, Vec<UnresolvedSubcomponent>)> {
    let (num_triangles, num_vertices) = header.num_t_v.expect("V8 should always have these counts");

    let parser = move |input: &mut &[u8]| {
        let vertex_format = cond(version >= 8, le_u32).parse_next(input)?;

        let (shapes, d0s, index_buffer, vertex_buffer) = (
            repeat(
                header.num_shapes as usize,
                repeat_array(le_u32).map(|[name, material, triangle_index]| UnresolvedShape {
                    name,
                    material,
                    triangle_index,
                }),
            ),
            repeat(
                header.num_sucomponents as usize,
                seq!(UnresolvedSubcomponent {
                    unk1: le_u8,
                    num_d1s: le_u8,
                    tag: le_u32,
                }),
            ),
            index_buffer(num_vertices, num_triangles),
            repeat(num_vertices as usize, v8_vertex(vertex_format)),
        )
            .parse_next(input)?;

        Ok((
            Section::V8(V8Section {
                vertex_format,
                index_buffer,
                vertex_buffer,
            }),
            shapes,
            d0s,
        ))
    };

    winnow::trace!("v8_section", parser)
}

fn v9_section<'a>(
    header: &Header,
) -> impl WinnowParser<&'a [u8], (Section, Vec<UnresolvedShape>, Vec<UnresolvedSubcomponent>)> {
    let parser = (
        dolm().map(Section::V9),
        repeat(
            header.num_shapes as usize,
            repeat_array(le_u32).map(|[name, material]| UnresolvedShape {
                name,
                material,
                triangle_index: 0,
            }),
        ),
        repeat(
            header.num_sucomponents as usize,
            seq!(UnresolvedSubcomponent {
                unk1: le_u8,
                num_d1s: le_u8,
                tag: le_u32,
            }),
        ),
    );

    winnow::trace!("v9_section", parser)
}

fn string_table<'a>() -> impl WinnowParser<&'a [u8], String> {
    winnow::trace!(
        "string_table",
        length_repeat(le_u32, le_u16).try_map(|chars: Vec<_>| String::from_utf16(&chars))
    )
}

/// Read a string from the string table
fn read_string(table: &str, offset: usize) -> Result<String, String> {
    if offset >= table.len() {
        return Err(format!(
            "String table index out of bounds: {}, len: {}",
            offset,
            table.len()
        ));
    }

    let Some(length) = table.chars().skip(offset).position(|c| c == '\0') else {
        return Err("No null terminator found".to_owned());
    };

    let string = table.chars().skip(offset).take(length).collect::<String>();

    Ok(string)
}

struct Header {
    num_t_v: Option<(u32, u32)>,
    num_shapes: u16,
    num_sucomponents: u8,
    _num_d1s: u16,
    num_d3s: u8,
}

fn header<'a>(version: u8) -> impl WinnowParser<&'a [u8], Header> {
    (
        cond(version < 9, (le_u32, le_u32)),
        le_u16,
        le_u8,
        le_u16,
        le_u8,
    )
        .map(|(num_t_v, num_shapes, num_d0s, _num_d1s, num_d3s)| Header {
            num_t_v,
            num_shapes,
            num_sucomponents: num_d0s,
            _num_d1s,
            num_d3s,
        })
}

pub fn parse_fmt(mut contents: &[u8]) -> VersionedResult<FMTFile> {
    let version =
        le_u8(&mut contents).map_err(|e: ContextError| anyhow!("Failed to parse file: {e:?}"))?;

    let header = header(version)
        .parse_next(&mut contents)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))
        .with_version(Some(version as u32))?;

    let (bbox, (section, shapes, subcomponents)) = (
        repeat_array(le_f32), //
        {
            let header = &header;
            dispatch! {
                empty.value(version);
                ..9 => v8_section(version, header),
                9.. => v9_section(header),
            }
        },
    )
        .parse_next(&mut contents)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))
        .with_version(Some(version as u32))?;

    let d1s: Vec<_> = subcomponents
        .iter()
        .map(|s| repeat(s.num_d1s as usize, take_array::<12, _>()).parse_next(&mut contents))
        .collect::<winnow::Result<Vec<_>>>()
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))
        .with_version(Some(version as u32))?;

    let d3_width = match version {
        ..1 => 45_usize,
        1..3 => 45,
        3..4 => 70,
        4..6 => 78,
        6..7 => 83,
        7.. => 87,
    };
    let (d3s, string_table) = (
        repeat(
            header.num_d3s as usize,
            take(d3_width).map(|b: &[u8]| b.to_vec()),
        ), //
        string_table(),
    )
        .parse(contents)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))
        .with_version(Some(version as u32))?;

    // Resolve shape strings
    let shapes = shapes
        .into_iter()
        .map(|s| {
            let shape = Shape {
                name: read_string(&string_table, s.name as usize)?,
                material: read_string(&string_table, s.material as usize)?,
                triangle_start: s.triangle_index,
            };

            Ok(shape)
        })
        .collect::<Result<Vec<_>, String>>()
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))
        .with_version(Some(version as u32))?;

    // Resolve subcomponents
    let subcomponents = subcomponents
        .into_iter()
        .zip(d1s)
        .map(|(s, d1s)| {
            let s = Subcomponent {
                unk1: s.unk1,
                d1s,
                tag: read_string(&string_table, s.tag as usize)?,
            };
            Ok(s)
        })
        .collect::<Result<Vec<_>, String>>()
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))
        .with_version(Some(version as u32))?;

    let fmt_file = FMTFile {
        version,
        bbox,
        section,
        shapes,
        subcomponents,
        d3s,
        string_table,
    };

    Ok(fmt_file).with_version(Some(version as u32))
}
