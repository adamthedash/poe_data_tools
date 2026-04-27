use annotated_parser::{
    AnnotationMode, ForwardRef,
    parsers::{EoF, TakeArray, TakeVec, byte::F16LE},
    prelude::*,
};

use super::types::*;
use crate::file_parsers::{
    VersionedResult, VersionedResultExt,
    dolm::{
        parser_annotated::{dolm, index_buffer},
        types::DolmVertex,
    },
    shared::annotated_parser::{ToAnyhow, U8Parser},
};

#[derive(Debug, Clone)]
struct GeomInfo {
    num_shapes: u16,
    num_vertices: u32,
    num_triangles: u32,
}

fn geom_info() -> impl U8Parser<Output = GeomInfo> {
    (u16::LE, u32::LE, u32::LE)
        .map_silent(|(num_shapes, num_vertices, num_triangles)| GeomInfo {
            num_shapes,
            num_vertices,
            num_triangles,
        })
        .trace("geom_info")
}

fn shape() -> impl U8Parser<Output = ShapeExtentsV8> {
    let ordinal = u16::LE.map_silent(Some);

    (ordinal, f32::LE.repeat::<6>(), u32::LE, u32::LE)
        .map_silent(|(ordinal, bbox, index_base, index_count)| ShapeExtentsV8 {
            ordinal,
            bbox,
            index_base,
            index_count,
        })
        .trace("shape")
}

fn ground_vertex(
    version: ForwardRef<u8>,
    vertex_format: ForwardRef<u8>,
) -> (impl U8Parser<Output = DolmVertex>, impl Fn()) {
    let (tex_coord0, c0) = F16LE
        .repeat::<2>()
        .configured((version.clone(), vertex_format.clone()).map(|(v, vf)| *v >= 4 && *vf == 3));
    let (tail_float, c2) = F16LE
        .repeat::<2>()
        .configured((version.clone(), vertex_format.clone()).map(|(v, vf)| *v >= 4 && *vf == 2));
    let (tex_coord1, c1) = F16LE
        .repeat::<2>()
        .configured((version.clone(), vertex_format.clone()).map(|(v, vf)| *v >= 4 && *vf == 3));

    let conf = move || {
        c0();
        c1();
        c2();
    };

    let parser = (
        f32::LE.repeat::<3>(),
        i8::LE.repeat::<4>(),
        i8::LE.repeat::<4>(),
        tex_coord0,
        tail_float,
        tex_coord1,
    )
        .map_silent(
            |(pos, normal, tangent, tex_coord0, tail_float, tex_coord1)| DolmVertex {
                pos,
                normal,
                tangent,
                tex_coord0,
                tail_float,
                skin_bones: None,
                skin_weights: None,
                skin_extra: None,
                tex_coord1,
                extra_vformat_6: None,
            },
        )
        .trace("ground_vertex");

    (parser, conf)
}

fn main_vertex() -> impl U8Parser<Output = DolmVertex> {
    (
        f32::LE.repeat::<3>(),
        i8::LE.repeat::<4>(),
        i8::LE.repeat::<4>(),
        F16LE.repeat::<2>(),
    )
        .map_silent(|(pos, normal, tangent, tex_coord0)| DolmVertex {
            pos,
            normal,
            tangent,
            tex_coord0: Some(tex_coord0),
            tail_float: None,
            skin_bones: None,
            skin_weights: None,
            skin_extra: None,
            tex_coord1: None,
            extra_vformat_6: None,
        })
        .trace("main_vertex")
}

fn mesh(
    version: ForwardRef<u8>,
    vertex_format: ForwardRef<u8>,
    geom_info: ForwardRef<GeomInfo>,
    is_main_mesh: ForwardRef<bool>,
) -> (impl U8Parser<Output = Mesh>, impl Fn()) {
    // Shape extents
    let shapes = shape().repeat_vec(geom_info.clone().map(|g| g.num_shapes));

    // Vertex buffer
    let (ground_vertex, c1) = ground_vertex(version, vertex_format);
    let vertex_buffer = (
        ground_vertex,
        main_vertex(), //
    )
        .dispatch(is_main_mesh.map(|m| Some(*m as usize)))
        .repeat_vec(geom_info.clone().map(|g| g.num_vertices));

    // Index buffer
    let index_buffer = index_buffer(
        geom_info.clone().map(|g| g.num_triangles),
        geom_info.map(|g| g.num_vertices),
    );

    let parser = (shapes, vertex_buffer, index_buffer)
        .map_silent(|(shape_extents, vertices, indices)| Mesh {
            shape_extents,
            indices,
            vertices,
        })
        .trace("mesh");

    (parser, c1)
}

fn v8_section(version: ForwardRef<u8>) -> impl U8Parser<Output = V8Section> {
    let v8_extra_header = u32::LE.run_if(version.clone().map(|v| *v == 8));

    let geom_infos = geom_info().repeat::<2>().store();
    let vertex_format = u8::LE.store();
    let num_tail_entries = u8::LE.store();

    let geom_info = ForwardRef::new_source();
    let is_main_mesh = ForwardRef::new_source();
    let (mesh, mesh_conf) = mesh(
        version.clone(),
        vertex_format.output(),
        geom_info.clone(),
        is_main_mesh.clone(),
    );
    let meshes = mesh.parameterize(
        (
            geom_infos.output().map(|x| x.to_vec()),
            ForwardRef::with_value(vec![true, false]),
        ),
        (geom_info, is_main_mesh),
    );

    let tail_entries = TakeVec::new(version.map(|v| match v {
        ..3 => 70,
        3 => 74,
        4 => 78,
        5..7 => 83,
        7.. => 87,
    }))
    .repeat_vec(num_tail_entries.output());

    let vertex_format = vertex_format.configuring(mesh_conf);

    (
        geom_infos,
        vertex_format,
        num_tail_entries,
        v8_extra_header,
        meshes,
        tail_entries,
    )
        .map_silent(
            |(_, vertex_format, _, extra_header, meshes, tail_entries)| V8Section {
                vertex_format,
                extra_header,
                meshes,
                tail_entries,
            },
        )
        .trace("v8_section")
}

fn tail_entry() -> impl U8Parser<Output = TailEntry> {
    (u32::LE, f32::LE.repeat::<12>(), u32::LE, TakeArray::<31>)
        .map_silent(|(uint1, floats, uint2, bytes)| TailEntry {
            uint1,
            floats,
            uint2,
            bytes,
        })
        .trace("tail_entry")
}

fn v9_section(version: ForwardRef<u8>) -> impl U8Parser<Output = V9Section> {
    // header
    let num_shapes = u16::LE.store().trace("num_shapes");
    let extra_u16 = u16::LE.trace("extra_u16");
    let extra_u8 = u8::LE.trace("extra_u8");
    let tail_count = u8::LE.trace("tail_count").store();

    let dolm = dolm().store().trace("dolm");

    let is_main_mesh = ForwardRef::new_source();
    let ordinal = (
        u16::LE.map_silent(|x| x as u32), //
        u32::LE,
    )
        .dispatch((version, is_main_mesh.clone()).map(|(v, m)| {
            let index = match (*v, *m) {
                (9, _) => 0,
                (_, false) => 0,
                _ => 1,
            };
            Some(index)
        }))
        .trace("ordinal");
    let bbox = f32::LE.repeat::<6>().trace("bbox");

    let shape_bounds = (ordinal, bbox)
        .map_silent(|(ordinal, bbox)| ShapeExtentsV9 { ordinal, bbox })
        .repeat_vec(
            // TODO: just use num_shapes above?
            dolm.output()
                .map(|d| d.lods.first().map(|m| m.shape_extents.len()).unwrap_or(0)),
        );

    // Different ordinal data type depending on whether it's main or ground mesh
    let geom_parsers = (dolm, shape_bounds)
        .map_silent(|(dolm, shape_extents)| Geometry {
            dolm,
            shape_extents,
        })
        .parameterize(ForwardRef::with_value(vec![true, false]), is_main_mesh)
        .trace("geometries");

    let tail = tail_entry().repeat_vec(tail_count.output());

    (
        num_shapes,
        extra_u16,
        extra_u8,
        tail_count,
        geom_parsers,
        tail,
    )
        .map_silent(|(_, extra_u16, extra_u8, _, geometries, tail)| V9Section {
            extra_u16,
            extra_u8,
            geometries,
            tail,
        })
        .trace("v9_section")
}

pub fn tgm_parser() -> (impl U8Parser<Output = TGMFile>, ForwardRef<u32>) {
    let version = u8::LE.store();
    let section = (
        v8_section(version.output()).map_silent(Section::V8), //
        v9_section(version.output()).map_silent(Section::V9),
    )
        .dispatch(version.output().map(|v| {
            let index = if *v < 9 { 0 } else { 1 };
            Some(index)
        }));

    let version_u32 = version.output().map(|v| *v as u32);

    let parser = (
        version, //
        f32::LE.repeat::<6>(),
        section,
        EoF,
    )
        .map_silent(|(version, bbox, _section, _)| TGMFile { version, bbox })
        .trace("tgm_file");

    (parser, version_u32)
}

pub fn parse_tgm_bytes(mut input: &[u8]) -> VersionedResult<TGMFile> {
    let (mut parser, version) = tgm_parser();

    let res = parser.parse_with(&mut input, AnnotationMode::FAIL);
    let version = *version.try_get();

    res.to_anyhow().with_version(version)
}
