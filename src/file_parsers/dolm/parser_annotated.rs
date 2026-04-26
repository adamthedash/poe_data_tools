use annotated_parser::{ForwardRef, parsers::byte::F16LE, prelude::*};

use super::types::*;
use crate::file_parsers::shared::annotated_parser::{U8Parser, take_arr_u8};

fn vertex(format_id: ForwardRef<u32>) -> (impl U8Parser<Output = DolmVertex>, impl Fn()) {
    let (tex_coord0, c0) = F16LE
        .repeat::<2>()
        .configured(format_id.clone().map(|f| (*f >> 3) & 1 == 1));

    let (tail_float, c1) = F16LE
        .repeat::<2>()
        .configured(ForwardRef::with_value(false));

    let (skin_bones, c2) =
        take_arr_u8::<4>().configured(format_id.clone().map(|f| (*f >> 2) & 1 == 1));

    let (skin_weights, c3) =
        take_arr_u8::<4>().configured(format_id.clone().map(|f| (*f >> 2) & 1 == 1));

    let (skin_extra, c4) =
        take_arr_u8::<4>().configured(format_id.clone().map(|f| (*f >> 1) & 1 == 1));

    let (tex_coord1, c5) = F16LE
        .repeat::<2>()
        .configured(format_id.clone().map(|f| *f & 1 == 1));

    let (extra_bit6, c6) =
        take_arr_u8::<4>().configured(format_id.clone().map(|f| (*f >> 6) & 1 == 1));

    let conf = move || {
        c0();
        c1();
        c2();
        c3();
        c4();
        c5();
        c6();
    };

    let parser = (
        f32::LE.repeat::<3>(),
        i8::LE.repeat::<4>(),
        i8::LE.repeat::<4>(),
        tex_coord0,
        tail_float,
        skin_bones,
        skin_weights,
        skin_extra,
        tex_coord1,
        extra_bit6,
    )
        .map_silent(
            |(
                pos,
                normal,
                tangent,
                tex_coord0,
                tail_float,
                skin_bones,
                skin_weights,
                skin_extra,
                tex_coord1,
                extra_vformat_6,
            )| {
                DolmVertex {
                    pos,
                    normal,
                    tangent,
                    tex_coord0,
                    tail_float,
                    skin_bones,
                    skin_weights,
                    skin_extra,
                    tex_coord1,
                    extra_vformat_6,
                }
            },
        )
        .trace("dolm_vertex");

    (parser, conf)
}

pub fn index_buffer(
    num_triangles: ForwardRef<u32>,
    num_vertices: ForwardRef<u32>,
) -> impl U8Parser<Output = IndexBuffer> {
    let num_triangles3 = num_triangles.map(|t| *t * 3);
    (
        u16::LE
            .repeat_vec(num_triangles3.clone())
            .map_silent(IndexBuffer::U16),
        u32::LE
            .repeat_vec(num_triangles3)
            .map_silent(IndexBuffer::U32),
    )
        .dispatch(num_vertices.map(|counts| {
            let index = match counts {
                ..=0xffff => 0,
                0x10000.. => 1,
            };
            Some(index)
        }))
        .trace("index_buffer")
}

pub fn dolm() -> impl U8Parser<Output = Dolm> {
    let c0h = u16::LE.trace("c0h").store();

    let lod_count = u8::LE.store();
    let shape_count = u16::LE.store();
    let vertex_format = u32::LE.trace("vertex_format").store();
    let vertex_format_output = vertex_format.output();
    let (vertex, vertex_conf) = vertex(vertex_format.output());
    let vertex_format = vertex_format.configuring(vertex_conf);

    let shape_extents = u32::LE
        .repeat::<2>()
        .map_silent(|[start_index, count_index]| DolmShapeExtents {
            start_index,
            count_index,
        })
        .repeat_vec(shape_count.output())
        .repeat_vec(lod_count.output());

    let header_lod_extents = u32::LE
        .repeat::<2>()
        .repeat_vec(lod_count.output())
        .trace("lod_extents")
        .store();

    let temp_lod_extents = ForwardRef::<[u32; 2]>::new_source();
    let single_index_buffer = index_buffer(
        temp_lod_extents.clone().map(|[t, _]| *t),
        temp_lod_extents.clone().map(|[_, v]| *v),
    );
    let index_buffers = single_index_buffer
        .parameterize(header_lod_extents.output(), temp_lod_extents)
        .trace("index_buffers");

    let temp_num_vertices = ForwardRef::new_source();
    let vertex_buffer = vertex.repeat_vec(temp_num_vertices.clone());
    let num_vertices = header_lod_extents.output().map(|extents| {
        extents
            .iter()
            .map(|[_, num_vertices]| *num_vertices)
            .collect::<Vec<_>>()
    });
    let vertex_buffers = vertex_buffer
        .parameterize(num_vertices, temp_num_vertices)
        .trace("vertex_buffers");

    let lods = (
        shape_extents, //
        index_buffers,
        vertex_buffers,
    )
        .map_silent(|(shape_extents, index_buffers, vertex_buffers)| {
            shape_extents
                .into_iter()
                .zip(index_buffers)
                .zip(vertex_buffers)
                .map(|((extents, indices), vertices)| Mesh {
                    shape_extents: extents,
                    indices,
                    vertices,
                })
                .collect()
        })
        .trace("lods");

    let extra_vformat_6 = take_arr_u8::<36>()
        .repeat_vec(shape_count.output())
        .run_if(vertex_format_output.clone().map(|v| (*v >> 6) & 1 == 1));

    let extra_vformat_6_c0h_2 = take_arr_u8::<4>().repeat_vec(shape_count.output()).run_if(
        (c0h.output(), vertex_format_output).map(|(c0h, vf)| *c0h == 2 && (*vf >> 6) & 1 == 1),
    );

    let extra_c0h_4 = take_arr_u8::<4>().run_if(
        (c0h.output(), lod_count.output()).map(|(c0h, num_lods)| *c0h == 4 && *num_lods > 0),
    );

    (
        b"DOLm",
        c0h,
        lod_count,
        shape_count,
        vertex_format,
        header_lod_extents,
        lods,
        extra_vformat_6,
        extra_vformat_6_c0h_2,
        extra_c0h_4,
    )
        .map_silent(
            |(
                _magic,
                c0h,
                _lod_count,
                _shape_count,
                vertex_format_id,
                lod_extents,
                lods,
                extra_vformat_6,
                extra_vformat_6_c0h_2,
                extra_c0h_4,
            )| Dolm {
                c0h,
                vertex_format: vertex_format_id,
                lod_extents,
                lods,
                extra_vformat_6,
                extra_vformat_6_c0h_2,
                extra_c0h_4,
            },
        )
        .trace("dolm")
}
