use winnow::{
    Parser,
    binary::{le_f32, le_i8, le_u8, le_u16, le_u32},
    combinator::{cond, repeat, seq},
    token::literal,
};

use super::types::*;
use crate::file_parsers::{
    fmt::parser::index_buffer,
    shared::winnow::{WinnowParser, le_f16, repeat_array, take_array},
};

struct Header {
    c0h: u16,
    num_lods: u8,
    num_shapes: u16,
    vertex_format: u32,
}

fn header<'a>() -> impl WinnowParser<&'a [u8], Header> {
    let parser = seq!(Header {
        _: literal(b"DOLm"),
        c0h: le_u16,
        num_lods: le_u8,
        num_shapes: le_u16,
        vertex_format: le_u32,
    });

    winnow::trace!("dolm_header", parser)
}

fn vertex<'a>(vertex_format: u32) -> impl WinnowParser<&'a [u8], DolmVertex> {
    let parser = seq!(DolmVertex {
        pos: repeat_array(le_f32),
        normal: repeat_array(le_i8),
        tangent: repeat_array(le_i8),
        tex_coord0: cond((vertex_format >> 3) & 1 == 1, repeat_array(le_f16)),
        tail_float: cond(false, repeat_array(le_f16)),
        skin_bones: cond((vertex_format >> 2) & 1 == 1, take_array()),
        skin_weights: cond((vertex_format >> 2) & 1 == 1, take_array()),
        skin_extra: cond((vertex_format >> 1) & 1 == 1, take_array()),
        tex_coord1: cond(vertex_format & 1 == 1, repeat_array(le_f16)),
        extra_vformat_6: cond((vertex_format >> 6) & 1 == 1, take_array()),
    });

    winnow::trace!("vertex", parser)
}

pub fn dolm<'a>() -> impl WinnowParser<&'a [u8], Dolm> {
    let parser = |input: &mut &[u8]| {
        let header = header().parse_next(input)?;

        let lod_extents: Vec<_> =
            repeat(header.num_lods as usize, repeat_array(le_u32)).parse_next(input)?;

        let shape_extents: Vec<_> = repeat(
            header.num_lods as usize,
            repeat(
                header.num_shapes as usize,
                repeat_array(le_u32).map(|[start_index, count_index]| DolmShapeExtents {
                    start_index,
                    count_index,
                }),
            ),
        )
        .parse_next(input)?;

        let index_buffers: Vec<_> = lod_extents
            .iter()
            .map(|[num_triangles, num_vertices]| {
                index_buffer(*num_vertices, *num_triangles).parse_next(input)
            })
            .collect::<winnow::Result<Vec<_>>>()?;

        let vertex_buffers: Vec<_> = lod_extents
            .iter()
            .map(|[_, num_vertices]| {
                repeat(*num_vertices as usize, vertex(header.vertex_format)).parse_next(input)
            })
            .collect::<winnow::Result<Vec<_>>>()?;

        let lods = shape_extents
            .into_iter()
            .zip(index_buffers)
            .zip(vertex_buffers)
            .map(|((e, i), v)| Mesh {
                shape_extents: e,
                indices: i,
                vertices: v,
            })
            .collect();

        let (extra_vformat_6, extra_c0h_4) = (
            cond(
                (header.vertex_format >> 6) & 1 == 1,
                repeat(header.num_shapes as usize, take_array::<36, _>()),
            ),
            cond(header.c0h == 4, take_array::<4, _>()),
        )
            .parse_next(input)?;

        Ok(Dolm {
            c0h: header.c0h,
            vertex_format: header.vertex_format,
            lod_extents,
            lods,
            extra_vformat_6,
            extra_c0h_4,
        })
    };

    winnow::trace!("dolm", parser)
}
