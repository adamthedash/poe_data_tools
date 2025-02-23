use nom::{
    bytes::complete::take,
    multi::count,
    number::complete::{le_f32, le_u32, le_u64, u8},
    IResult,
};

#[derive(Debug)]
pub struct Connection {
    pub id: u32,
    pub radius: u32,
}

fn parse_connection(input: &[u8]) -> IResult<&[u8], Connection> {
    let (input, id) = le_u32(input)?;
    let (input, radius) = le_u32(input)?;

    let connection = Connection { id, radius };

    Ok((input, connection))
}

#[derive(Debug)]
pub struct Passive {
    pub id: u32,
    pub radius: u32,
    pub position: u32,
    pub connections: Vec<Connection>,
}

fn parse_passive(input: &[u8]) -> IResult<&[u8], Passive> {
    let (input, id) = le_u32(input)?;
    let (input, radius) = le_u32(input)?;
    let (input, position) = le_u32(input)?;

    let (input, num_connections) = le_u32(input)?;
    let (input, connections) = count(parse_connection, num_connections as usize)(input)?;

    let passive = Passive {
        id,
        radius,
        position,
        connections,
    };

    Ok((input, passive))
}

#[derive(Debug)]
pub struct Group {
    pub x: f32,
    pub y: f32,
    pub flags: u32,
    pub unk1: u32,
    pub unk2: u8,

    pub passives: Vec<Passive>,
}

fn parse_group(input: &[u8]) -> IResult<&[u8], Group> {
    let (input, x) = le_f32(input)?;
    let (input, y) = le_f32(input)?;
    let (input, flags) = le_u32(input)?;
    let (input, unk1) = le_u32(input)?;
    let (input, unk2) = u8(input)?;

    let (input, num_passives) = le_u32(input)?;
    let (input, passives) = count(parse_passive, num_passives as usize)(input)?;

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

#[derive(Debug)]
pub struct PassiveSkillGraph<'a> {
    pub version: u8,
    pub unk1: &'a [u8],
    pub passives: Vec<u64>,
    pub groups: Vec<Group>,
}

/// https://gist.github.com/qcrist/3078c2bbc55401d911583819a65e8bf9
fn parse_psg(input: &[u8]) -> IResult<&[u8], PassiveSkillGraph> {
    let (input, version) = u8(input)?;
    assert_eq!(version, 3, "Only PSG version 3 supported.");

    let (input, unk1) = take(12_usize)(input)?;

    let (input, num_passives) = le_u32(input)?;
    let (input, passives) = count(le_u64, num_passives as usize)(input)?;

    let (input, num_groups) = le_u32(input)?;
    let (input, groups) = count(parse_group, num_groups as usize)(input)?;

    let psg = PassiveSkillGraph {
        version,
        unk1,
        passives,
        groups,
    };

    Ok((input, psg))
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use anyhow::Context;

    use super::parse_psg;
    use crate::dat::ivy_schema::fetch_schema;

    #[test]
    fn test() {
        // passiveskills.csv
        // - Ascendency -> foreign ID
        // - Stats -> foreign ID
        // - Name
        // - Icon -> dds to png

        let path = "/home/adam/poe_data/raw/metadata/passiveskillgraph.psg";
        //let path = "/home/adam/poe_data/raw/metadata/atlasskillgraphs/atlasskillgraph.psg";
        //let path =
        //    "/home/adam/poe_data/raw/metadata/alternateskillgraphs/royalepassiveskillgraph.psg";

        let bytes = std::fs::read(path).unwrap();
        let (_input, psg) = parse_psg(&bytes).unwrap();
        println!("{:#?}", psg);

        //psg.groups.iter().for_each(|g| {
        //    g.passives.iter().for_each(|p| {
        //        p.connections.iter().for_each(|c| {
        //            println!("{} -> {}", p.id, c.id);
        //        })
        //    })
        //});
    }

    #[test]
    fn test2() {
        let cache_dir = PathBuf::from_str("").unwrap();
        // Load schema: todo: Get this from Ivy's CDN / cache it
        let _schemas = fetch_schema(&cache_dir)
            .context("Failed to fetch schema file")
            .unwrap();
    }
}
