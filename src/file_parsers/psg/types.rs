use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Connection {
    /// Destination node
    pub passive_id: u32,
    /// Curvature of the spline drawn between two nodes
    pub curvature: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct Passive {
    pub id: u32,
    pub orbit: i32,
    /// Clockwise
    pub orbit_position: u32,
    pub connections: Vec<Connection>,
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

#[derive(Debug, Serialize)]
pub struct PSGFile {
    pub version: u8,
    /// 1 == Passive skill tree
    /// 2 == Atlas tree
    pub graph_type: u8,
    pub passives_per_orbit: Vec<u8>,
    pub root_passives: Vec<u64>,
    pub groups: Vec<Group>,
}
