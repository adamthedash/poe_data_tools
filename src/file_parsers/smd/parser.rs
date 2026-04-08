use anyhow::anyhow;
use winnow::{
    Parser,
    binary::{le_f32, le_i8, le_u8, le_u16, le_u32, length_take},
    combinator::{cond, dispatch, empty, fail, repeat, seq},
    error::ContextError,
    token::take,
};

use super::types::*;
use crate::file_parsers::{
    VersionedResult, VersionedResultExt,
    dolm::{
        parser::{dolm, index_buffer},
        types::DolmVertex,
    },
    shared::winnow::{WinnowParser, le_f16, repeat_array, take_array},
};

fn v3_section<'a>() -> impl WinnowParser<&'a [u8], (u8, Section, [f32; 6])> {
    let parser = |input: &mut &[u8]| -> winnow::Result<_> {
        let (vertex_format, num_shapes, _bytes_name_section, bbox, dolm) =
            (le_u8, le_u16, le_u32, repeat_array(le_f32), dolm()).parse_next(input)?;

        let name_lengths: Vec<_> = repeat(num_shapes as usize, le_u32).parse_next(input)?;

        let shape_names = name_lengths
            .into_iter()
            .map(|len| take(len).try_map(String::from_utf16le).parse_next(input))
            .collect::<winnow::Result<_>>()?;

        Ok((
            vertex_format,
            Section::V3(V3Section { dolm, shape_names }),
            bbox,
        ))
    };

    winnow::trace!("v3_section", parser)
}

struct V2ShapeExtents {
    name_length: u32,
    triangle_index: u32,
}

fn vertex<'a>(vertex_format: u8) -> impl WinnowParser<&'a [u8], DolmVertex> {
    let parser = seq!(DolmVertex {
        pos: repeat_array(le_f32),
        normal: repeat_array(le_i8),
        tangent: repeat_array(le_i8),
        tex_coord0: repeat_array(le_f16).map(Some),
        skin_bones: take_array().map(Some),
        skin_weights: take_array().map(Some),
        skin_extra: cond((vertex_format >> 1) & 1 == 1, take_array()),
        tex_coord1: cond(vertex_format & 1 == 1, repeat_array(le_f16)),
        tail_float: empty::<_, ContextError>.value(None),
        extra_vformat_6: empty::<_, ContextError>.value(None),
    });

    winnow::trace!("vertex", parser)
}

fn v2_section<'a>(version: u8) -> impl WinnowParser<&'a [u8], (u8, Section, [f32; 6])> {
    let parser = move |input: &mut &[u8]| -> winnow::Result<_> {
        let (
            num_triangles,
            num_vertices,
            vertex_format,
            num_shapes,
            _bytes_name_section,
            bbox,
            c04_2,
        ) = (
            le_u32,
            le_u32,
            le_u8,
            le_u16,
            le_u32,
            repeat_array(le_f32),
            cond(version == 2, le_u32),
        )
            .parse_next(input)?;

        let shape_extents: Vec<_> = repeat(
            num_shapes as usize,
            (le_u32, le_u32).map(|(name_length, triangle_index)| V2ShapeExtents {
                name_length,
                triangle_index,
            }),
        )
        .parse_next(input)?;

        let shape_extents = shape_extents
            .into_iter()
            .map(|s| {
                let name = take(s.name_length)
                    .try_map(String::from_utf16le)
                    .parse_next(input)?;

                Ok(ShapeExtents {
                    name,
                    triangle_index: s.triangle_index,
                })
            })
            .collect::<winnow::Result<Vec<_>>>()?;

        let index_buffer = index_buffer(num_vertices, num_triangles).parse_next(input)?;

        let vertex_buffer =
            repeat(num_vertices as usize, vertex(vertex_format)).parse_next(input)?;

        Ok((
            vertex_format,
            Section::V2(V2Section {
                c04_2,
                shape_extents,
                index_buffer,
                vertex_buffer,
            }),
            bbox,
        ))
    };

    winnow::trace!("v2_section", parser)
}

fn v2_tail<'a>(version: u8) -> impl WinnowParser<&'a [u8], Tail> {
    let parser = move |input: &mut &[u8]| {
        let [
            num_ellipsoids,
            num_skinned_vertices,
            num_sv_refs1,
            num_sv_refs2,
        ] = repeat_array(le_u32).parse_next(input)?;

        let (skinned_vertices, sv_refs1, ellipsoids, sv_refs2) = (
            repeat(num_skinned_vertices as usize, skinned_vertex()),
            repeat(num_sv_refs1 as usize, le_u32),
            repeat(num_ellipsoids as usize, ellipsoid(version)),
            repeat(num_sv_refs2 as usize, le_u32),
        )
            .parse_next(input)?;

        Ok(Tail {
            tail_version: 2,
            ellipsoids,
            spheres: vec![],
            sphere_connections: vec![],
            skinned_vertices,
            t3s: vec![],
            sv_refs1,
            sv_refs2,
        })
    };

    winnow::trace!("v2_tail", parser)
}

fn v3_tail<'a>(version: u8) -> impl WinnowParser<&'a [u8], Tail> {
    let parser = move |input: &mut &[u8]| {
        let [
            num_ellipsoids,
            num_spheres,
            num_sphere_connections,
            num_t3s,
            num_skinned_vertices,
            num_sv_refs1,
            num_sv_refs2,
        ] = repeat_array(le_u32).parse_next(input)?;

        let (
            ellipsoids,
            spheres,
            sphere_connections,
            t3s,
            skinned_vertices, //
            sv_refs1,
            sv_refs2,
        ) = (
            repeat(num_ellipsoids as usize, ellipsoid(version)),
            repeat(num_spheres as usize, sphere(version)),
            repeat(num_sphere_connections as usize, sphere_connection()),
            repeat(num_t3s as usize, empty),
            repeat(num_skinned_vertices as usize, skinned_vertex()),
            repeat(num_sv_refs1 as usize, le_u32),
            repeat(num_sv_refs2 as usize, le_u32),
        )
            .parse_next(input)?;

        Ok(Tail {
            tail_version: 3,
            ellipsoids,
            spheres,
            sphere_connections,
            skinned_vertices,
            t3s,
            sv_refs1,
            sv_refs2,
        })
    };

    winnow::trace!("v3_tail", parser)
}

fn v4_tail<'a>(version: u8) -> impl WinnowParser<&'a [u8], Tail> {
    let parser = move |input: &mut &[u8]| {
        let [
            num_ellipsoids,
            num_spheres,
            num_sphere_connections,
            num_t3s,
            num_skinned_vertices,
            num_sv_refs1,
            num_sv_refs2,
        ] = repeat_array(le_u32).parse_next(input)?;

        let (
            skinned_vertices, //
            sv_refs1,
            ellipsoids,
            spheres,
            sphere_connections,
            t3s,
            sv_refs2,
        ) = (
            repeat(num_skinned_vertices as usize, skinned_vertex()),
            repeat(num_sv_refs1 as usize, le_u32),
            repeat(num_ellipsoids as usize, ellipsoid(version)),
            repeat(num_spheres as usize, sphere(version)),
            repeat(num_sphere_connections as usize, sphere_connection()),
            repeat(num_t3s as usize, empty),
            repeat(num_sv_refs2 as usize, le_u32),
        )
            .parse_next(input)?;

        Ok(Tail {
            tail_version: 4,
            ellipsoids,
            spheres,
            sphere_connections,
            skinned_vertices,
            t3s,
            sv_refs1,
            sv_refs2,
        })
    };

    winnow::trace!("v4_tail", parser)
}

fn ellipsoid<'a>(version: u8) -> impl WinnowParser<&'a [u8], Ellipsoid> {
    let parser = seq!(Ellipsoid {
        floats: repeat_array(le_f32),
        unk1: le_u32,
        name: cond(
            version >= 3,
            length_take(le_u32).try_map(String::from_utf16le)
        )
    });

    winnow::trace!("ellipsoid", parser)
}

fn skinned_vertex<'a>() -> impl WinnowParser<&'a [u8], SkinnedVertex> {
    let parser = seq!(SkinnedVertex {
        pos: repeat_array(le_f32),
        unk1: repeat_array(le_u32),
        unk2: repeat_array(le_f32),
    });

    winnow::trace!("skinned_vertex", parser)
}

fn sphere<'a>(version: u8) -> impl WinnowParser<&'a [u8], Sphere> {
    let parser = seq!(Sphere {
        centre: repeat_array(le_f32),
        radius: le_f32,
        unk1: le_u32,
        name: cond(
            version >= 3,
            length_take(le_u32).try_map(String::from_utf16le)
        )
    });

    winnow::trace!("sphere", parser)
}

fn sphere_connection<'a>() -> impl WinnowParser<&'a [u8], SphereConnection> {
    let parser = seq!(SphereConnection {
        s0_index: le_u32,
        s1_index: le_u32,
    });

    winnow::trace!("sphere_connection", parser)
}

pub fn parse_smd(mut contents: &[u8]) -> VersionedResult<SMDFile> {
    let version =
        le_u8(&mut contents).map_err(|e: ContextError| anyhow!("Failed to parse file: {e:?}"))?;

    let ((vertex_format, section, bbox), tail) = (
        dispatch! {
            empty.value(version);
            ..3 => v2_section(version),
            3.. => v3_section(),
        },
        dispatch! {
            le_u32;
            2 => v2_tail(version),
            3 => v3_tail(version),
            4 => v4_tail(version),
            _ => fail,
        },
    )
        .parse_next(&mut contents)
        .map_err(|e: ContextError| anyhow!("Failed to parse file: {e:?}"))
        .with_version(Some(version as u32))?;

    Ok(SMDFile {
        version,
        vertex_format,
        section,
        bbox,
        tail,
    })
    .with_version(Some(version as u32))
}
