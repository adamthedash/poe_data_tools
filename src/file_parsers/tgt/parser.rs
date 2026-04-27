use annotated_parser::{
    AnnotationMode, ForwardRef,
    combinators::LengthRepeat,
    parsers::{EoF, str::U32},
    prelude::*,
};
use anyhow::Context;

use super::types::*;
use crate::file_parsers::{
    VersionedResult, VersionedResultExt,
    shared::annotated_parser::{StrParser, ToAnyhow, quoted, whitespace},
};

fn subtile_material_indices(
    size: ForwardRef<[u32; 2]>,
) -> impl StrParser<Output = Vec<Vec<Vec<Index>>>> {
    let length = U32.store();
    let length_out = length.output();
    let values = " ".ignore_then(U32).many().try_map(move |values| {
        let length = *length_out.get() as usize;
        let indices = if values.len() == length * 3 {
            values
                .chunks_exact(3)
                .map(|x| Index {
                    uint1: x[0],
                    uint2: x[1],
                    uint3: Some(x[2]),
                })
                .collect::<Vec<_>>()
        } else if values.len() == length * 2 {
            values
                .chunks_exact(2)
                .map(|x| Index {
                    uint1: x[0],
                    uint2: x[1],
                    uint3: None,
                })
                .collect::<Vec<_>>()
        } else {
            return Err("Bad indices thing");
        };

        Ok(indices)
    });

    let subtile_material_index = length.ignore_then(values);

    ("SubTileMaterialIndices", whitespace())
        .ignore_then(
            subtile_material_index
                .separated_vec(whitespace(), size.map(|[w, _]| *w))
                .separated_vec(whitespace(), size.map(|[_, h]| *h)),
        )
        .trace("subtile_material_indices")
}

fn ground_mask() -> impl StrParser<Output = String> {
    "GroundMask ".ignore_then(quoted())
}

fn v3_section() -> impl StrParser<Output = V3Section> {
    let size = "Size ".ignore_then(U32.separated_arr::<2, _>(" ")).store();

    let normal_materials = "NormalMaterials "
        .ignore_then(LengthRepeat::new(U32, whitespace().ignore_then(quoted())))
        .store();

    let subtile_material_indices = subtile_material_indices(size.output())
        .run_if(normal_materials.output().map(|mats| !mats.is_empty()));

    (
        "SourceScene ".ignore_then(quoted()).optional(),
        size,
        "TileMeshRoot ".ignore_then(quoted()),
        ground_mask().optional(),
        normal_materials,
        "MaterialSlots "
            .ignore_then(LengthRepeat::new(U32, whitespace().ignore_then(quoted())))
            .optional(),
        subtile_material_indices,
    )
        .separated_tuple(whitespace())
        .map_silent(
            |(
                source_scene,
                size,
                tile_mesh_root,
                ground_mask,
                normal_materials,
                material_slots,
                subtile_material_indices,
            )| V3Section {
                source_scene,
                size,
                tile_mesh_root,
                ground_mask,
                normal_materials,
                material_slots,
                subtile_material_indices,
            },
        )
        .trace("v3_section")
}

fn v1_normal_material() -> impl StrParser<Output = V1NormalMaterial> {
    (quoted(), U32)
        .separated_tuple(whitespace())
        .map_silent(|(mat_file, uint)| V1NormalMaterial { mat_file, uint })
        .trace("v1_normal_material")
}

fn v1_section() -> impl StrParser<Output = V1Section> {
    (
        "TileMesh ".ignore_then(quoted()),
        ground_mask().optional(),
        "NormalMaterials ".ignore_then(LengthRepeat::new(
            U32,
            whitespace().ignore_then(v1_normal_material()),
        )),
    )
        .separated_tuple(whitespace())
        .map_silent(|(tile_mesh, ground_mask, normal_materials)| V1Section {
            tile_mesh,
            ground_mask,
            normal_materials,
        })
        .trace("v1_section")
}

pub fn tgt_file() -> (impl StrParser<Output = TGTFile>, ForwardRef<u32>) {
    let version = "version ".ignore_then(U32).store();
    let version_out = version.output();

    let section = (
        v1_section().map_silent(Section::V1), //
        v3_section().map_silent(Section::V3),
    )
        .dispatch(version.output().map(|v| match *v {
            ..3 => Some(0),
            3.. => Some(1),
        }));

    let parser = (version, section, EoF)
        .separated_tuple(whitespace())
        .map_silent(|(version, section, _)| TGTFile { version, section })
        .trace("tgt_file");

    (parser, version_out)
}

pub fn parse_tgt_str(mut contents: &str) -> VersionedResult<TGTFile> {
    let (mut parser, version) = tgt_file();

    parser
        .parse_with(&mut contents, AnnotationMode::FAIL)
        .to_anyhow()
        .context("Failed to parse file")
        .with_version(*version.try_get())
}
