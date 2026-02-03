use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Direction {
    pub fn diagonals() -> [Self; 4] {
        use Direction::*;
        [SW, SE, NE, NW]
    }

    pub fn cardinal() -> [Self; 4] {
        use Direction::*;
        [N, W, S, E]
    }
}

#[derive(Debug, Serialize)]
pub struct Edge {
    pub direction: Direction,
    pub edge: String,
    pub exit: u32,
    pub virtual_exit: u32,
}

#[derive(Debug, Serialize)]
pub struct Corner {
    pub direction: Direction,
    pub ground: String,
    pub height: u32,
}

#[derive(Debug, Serialize)]
pub struct SlotK {
    pub height: u32,
    pub width: u32,
    pub edges: [Edge; 4],
    pub corners: [Corner; 4],
    pub slot_tag: String,
    pub origin: Direction,
}

#[derive(Debug, Serialize)]
pub enum Slot {
    K(SlotK),
    N,
    // TODO: Thought this was a string index, but doesn't look like it
    F { fill: u32 },
    S,
}

#[derive(Debug, Serialize)]
pub struct PoI {
    pub num1: u32,
    pub num2: u32,
    pub num3: u32,
    pub tag: String,
}

#[derive(Debug, Serialize)]
pub struct Doodad {
    pub num1: u32,
    pub num2: u32,
    pub float1: f32,
    pub rotation: f32,
    pub float2: f32,
    pub num3: u32,
    pub float3: f32,
    pub float4: f32,
    pub num4: u32,
    pub num5: u32,
    pub num6: u32,
    pub num7: u32,
    pub ao_file: String,
    pub stub: String,
}

#[derive(Debug, Serialize)]
pub struct Map {
    pub version: u32,
    pub strings: Vec<String>,
    /// Have seen either 2 or 3 elements here
    pub dimensions: Vec<u32>,
    pub numbers1: Vec<u32>,
    pub tag: String,
    pub numbers2: Vec<u32>,
    pub root_slot: Slot,
    pub numbers3: Vec<Vec<u32>>,
    pub points_of_interest: Vec<Vec<PoI>>,
    pub string1: Option<String>,
    pub grid: Vec<Vec<Slot>>,
    pub doodads: Vec<Doodad>,
    //TODO: interpret
    pub doodad_connections: Vec<String>,
    //TODO: interpret
    pub decals: Vec<String>,
}
