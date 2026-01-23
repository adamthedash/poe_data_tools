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

impl Connection {
    fn parse_poe1(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, id) = le_u32(input)?;

        let connection = Connection {
            passive_id: id,
            curvature: 0,
        };

        Ok((input, connection))
    }

    fn parse_poe2(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, id) = le_u32(input)?;
        let (input, radius) = le_i32(input)?;

        let connection = Connection {
            passive_id: id,
            curvature: radius,
        };

        Ok((input, connection))
    }
}

#[derive(Debug, Serialize)]
pub struct Passive {
    pub id: u32,
    pub orbit: i32,
    /// Clockwise
    pub orbit_position: u32,
    pub connections: Vec<Connection>,
}

impl Passive {
    fn parse_poe1(input: &[u8]) -> IResult<&[u8], Self> {
        Self::parse(input, Connection::parse_poe1)
    }

    fn parse_poe2(input: &[u8]) -> IResult<&[u8], Self> {
        Self::parse(input, Connection::parse_poe2)
    }

    fn parse(
        input: &[u8],
        connection_parser: fn(&[u8]) -> IResult<&[u8], Connection>,
    ) -> IResult<&[u8], Self> {
        let (input, id) = le_u32(input)?;
        let (input, radius) = le_i32(input)?;
        let (input, position) = le_u32(input)?;

        let (input, num_connections) = le_u32(input)?;
        let (input, connections) = count(connection_parser, num_connections as usize)(input)?;

        let passive = Passive {
            id,
            orbit: radius,
            orbit_position: position,
            connections,
        };

        Ok((input, passive))
    }
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
    /// 5 == ? PoE1 Atlas skill tree
    /// 6 == ? PoE1 Atlas skill tree
    pub unk1: u32,
    /// 1 == Cluster jewel
    pub unk2: u8,

    pub passives: Vec<Passive>,
}

impl Group {
    fn parse(
        input: &[u8],
        passive_parser: fn(&[u8]) -> IResult<&[u8], Passive>,
    ) -> IResult<&[u8], Self> {
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

impl PassiveSkillGraph {
    pub fn parse_poe1(input: &[u8]) -> IResult<&[u8], Self> {
        Self::parse(input, Passive::parse_poe1, |input| {
            le_u32(input).map(|(input, id)| (input, id as u64))
        })
    }

    pub fn parse_poe2(input: &[u8]) -> IResult<&[u8], Self> {
        Self::parse(input, Passive::parse_poe2, |input| le_u64(input))
    }

    fn parse(
        input: &[u8],
        passive_parser: fn(&[u8]) -> IResult<&[u8], Passive>,
        passive_id_parser: fn(&[u8]) -> IResult<&[u8], u64>,
    ) -> IResult<&[u8], Self> {
        let (input, version) = u8(input)?;
        assert_eq!(version, 3, "Only PSG version 3 supported.");

        let (input, graph_type) = u8(input)?;

        let (input, num_orbits) = u8(input)?;
        let (input, skills_per_orbit) = count(u8, num_orbits as usize)(input)?;

        let (input, num_passives) = le_u32(input)?;
        let (input, passives) = count(passive_id_parser, num_passives as usize)(input)?;

        let (input, num_groups) = le_u32(input)?;

        let (input, groups) =
            count(|x| Group::parse(x, passive_parser), num_groups as usize)(input)?;

        let psg = PassiveSkillGraph {
            version,
            root_passives: passives,
            groups,
            graph_type,
            passives_per_orbit: skills_per_orbit,
        };

        Ok((input, psg))
    }
}
