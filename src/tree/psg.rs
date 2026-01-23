use nom::{
    multi::count,
    number::complete::{le_f32, le_i32, le_u32, le_u64, u8},
    IResult,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Connection {
    /// Destination node
    pub passive_id: u32,
    /// Curvature of the spline drawn between two nodes
    pub curvature: i32,
}

fn parse_connection(input: &[u8]) -> IResult<&[u8], Connection> {
    let (input, id) = le_u32(input)?;
    let (input, radius) = le_i32(input)?;

    let connection = Connection {
        passive_id: id,
        curvature: radius,
    };

    Ok((input, connection))
}

#[derive(Debug, Serialize)]
pub struct Passive {
    pub id: u32,
    pub orbit: i32,
    /// Clockwise
    pub orbit_position: u32,
    pub connections: Vec<Connection>,
}

fn parse_passive_poe1(input: &[u8]) -> IResult<&[u8], Passive> {
    let (input, id) = le_u32(input)?;
    let (input, radius) = le_i32(input)?;
    let (input, position) = le_u32(input)?;

    let (input, num_connections) = le_u32(input)?;
    let (input, connections) = count(le_u32, num_connections as usize)(input)?;

    // Cast to common type
    let connections = connections
        .into_iter()
        .map(|id| Connection {
            passive_id: id,
            curvature: 0,
        })
        .collect();

    let passive = Passive {
        id,
        orbit: radius,
        orbit_position: position,
        connections,
    };

    Ok((input, passive))
}

fn parse_passive_poe2(input: &[u8]) -> IResult<&[u8], Passive> {
    let (input, id) = le_u32(input)?;
    let (input, radius) = le_i32(input)?;
    let (input, position) = le_u32(input)?;

    let (input, num_connections) = le_u32(input)?;
    let (input, connections) = count(parse_connection, num_connections as usize)(input)?;

    let passive = Passive {
        id,
        orbit: radius,
        orbit_position: position,
        connections,
    };

    Ok((input, passive))
}

#[derive(Debug, Serialize)]
pub struct Group {
    pub x: f32,
    pub y: f32,
    pub flags: u32,
    /// 0 == ?
    /// 1 == ?
    /// 2 == ? PoE1
    /// 3 == ? PoE2 Mastery group / ascendencies?
    /// 4 == ? PoE2 Possibly non-wheel groups?
    pub unk1: u32,
    /// 1 == Cluster jewel
    pub unk2: u8,

    pub passives: Vec<Passive>,
}

fn parse_group(
    input: &[u8],
    passive_parser: impl FnMut(&[u8]) -> IResult<&[u8], Passive>,
) -> IResult<&[u8], Group> {
    let (input, x) = le_f32(input)?;
    let (input, y) = le_f32(input)?;
    let (input, flags) = le_u32(input)?;
    let (input, unk1) = le_u32(input)?;
    let (input, unk2) = u8(input)?;

    let (input, num_passives) = le_u32(input)?;
    let (input, passives) = count(passive_parser, num_passives as usize)(input)?;

    let group = Group {
        x,
        y,
        flags,
        unk1,
        unk2,
        passives,
    };

    Ok((input, group))
}

#[derive(Debug, Serialize)]
pub struct PassiveSkillGraph {
    pub version: u8,
    /// 1 == Passive skill tree
    /// 2 == Atlas tree
    pub graph_type: u8,
    pub passives_per_orbit: Vec<u8>,
    pub root_passives: Vec<u64>,
    pub groups: Vec<Group>,
}

pub fn parse_psg_poe1(input: &[u8]) -> IResult<&[u8], PassiveSkillGraph> {
    let (input, version) = u8(input)?;
    assert_eq!(version, 3, "Only PSG version 3 supported.");

    let (input, graph_type) = u8(input)?;

    let (input, num_orbits) = u8(input)?;
    let (input, skills_per_orbit) = count(u8, num_orbits as usize)(input)?;

    let (input, num_passives) = le_u32(input)?;
    let (input, passives) = count(le_u32, num_passives as usize)(input)?;

    // Cast to common type
    let passives = passives.into_iter().map(|x| x as u64).collect();

    let (input, num_groups) = le_u32(input)?;

    let (input, groups) =
        count(|x| parse_group(x, parse_passive_poe1), num_groups as usize)(input)?;

    let psg = PassiveSkillGraph {
        version,
        root_passives: passives,
        groups,
        graph_type,
        passives_per_orbit: skills_per_orbit,
    };

    Ok((input, psg))
}

/// https://gist.github.com/qcrist/3078c2bbc55401d911583819a65e8bf9
/// Above seems only valid for PoE 2
pub fn parse_psg_poe2(input: &[u8]) -> IResult<&[u8], PassiveSkillGraph> {
    let (input, version) = u8(input)?;
    assert_eq!(version, 3, "Only PSG version 3 supported.");

    let (input, graph_type) = u8(input)?;

    let (input, num_orbits) = u8(input)?;
    let (input, skills_per_orbit) = count(u8, num_orbits as usize)(input)?;

    let (input, num_passives) = le_u32(input)?;
    let (input, passives) = count(le_u64, num_passives as usize)(input)?;

    let (input, num_groups) = le_u32(input)?;

    let (input, groups) =
        count(|x| parse_group(x, parse_passive_poe2), num_groups as usize)(input)?;

    let psg = PassiveSkillGraph {
        version,
        root_passives: passives,
        groups,
        graph_type,
        passives_per_orbit: skills_per_orbit,
    };

    Ok((input, psg))
}
