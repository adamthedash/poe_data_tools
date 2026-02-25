use anyhow::{Result, anyhow};
use winnow::{
    Parser,
    binary::{le_f32, le_i32, le_u8, le_u32, le_u64, length_repeat},
    combinator::{cond, dispatch, empty, fail},
};

use super::types::*;
use crate::file_parsers::shared::winnow::{TraceHelper, WinnowParser};

fn connection<'a>(poe_version: u32) -> impl WinnowParser<&'a [u8], Connection> {
    (
        le_u32, //
        cond(poe_version == 2, le_i32),
    )
        .map(|(passive_id, curvature)| Connection {
            passive_id,
            curvature,
        })
        .trace("connection")
}

fn passive<'a>(poe_version: u32) -> impl WinnowParser<&'a [u8], Passive> {
    (
        le_u32,
        le_i32,
        le_u32,
        length_repeat(le_u32, connection(poe_version)),
    )
        .map(|(id, orbit, orbit_position, connections)| Passive {
            id,
            orbit,
            orbit_position,
            connections,
        })
        .trace("passive")
}

fn group<'a>(poe_version: u32) -> impl WinnowParser<&'a [u8], Group> {
    (
        le_f32,
        le_f32,
        le_u32,
        le_u32,
        le_u8,
        length_repeat(le_u32, passive(poe_version)),
    )
        .map(|(x, y, flags, unk1, unk2, passives)| Group {
            x,
            y,
            flags,
            unk1,
            unk2,
            passives,
        })
        .trace("group")
}

pub fn parse_psg_bytes(contents: &[u8], poe_version: u32) -> Result<PSGFile> {
    let mut parser = (
        le_u8,
        le_u8,
        length_repeat(le_u8, le_u8),
        length_repeat(
            le_u32,
            dispatch! {
                empty.value(poe_version);
                1 => le_u32.map(u64::from),
                2 => le_u64,
                _ => fail,
            },
        ),
        length_repeat(le_u32, group(poe_version)),
    )
        .map(
            |(version, graph_type, passives_per_orbit, root_passives, groups)| PSGFile {
                version,
                root_passives,
                groups,
                graph_type,
                passives_per_orbit,
            },
        );

    parser
        .parse(contents)
        .map_err(|e| anyhow!("Failed to parse file: {e:?}"))
}
