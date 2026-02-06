use std::collections::HashMap;

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
    pub edge: Option<String>,
    pub exit: u32,
    pub virtual_exit: u32,
}

#[derive(Debug, Serialize)]
pub struct Corner {
    pub direction: Direction,
    pub ground: Option<String>,
    pub height: i32,
}

#[derive(Debug, Serialize)]
pub struct SlotK {
    pub height: u32,
    pub width: u32,
    pub edges: [Edge; 4],
    pub corners: [Corner; 4],
    pub slot_tag: Option<String>,
    pub origin: Direction,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Default)]
#[serde(tag = "kind", content = "data")]
pub enum Slot {
    K(SlotK),
    #[default]
    N,
    F {
        fill: Option<String>,
    },
    S,
    O,
}

#[derive(Debug, Serialize)]
pub struct PoI {
    pub num1: u32,
    pub num2: u32,
    pub num3: f32,
    pub tag: String,
}

#[derive(Debug, Serialize)]
pub struct Doodad {
    pub x: u32,
    pub y: u32,

    pub float_pairs: Option<Vec<(f32, f32)>>,

    pub radians1: f32,

    pub trig1: Option<f32>,
    pub trig2: Option<f32>,
    pub trig3: Option<f32>,
    pub trig4: Option<f32>,

    pub bool1: bool,
    pub bool2: Option<bool>,

    pub floats: Vec<f32>,

    pub scale: f32,
    pub ao_file: String,
    pub stub: String,

    pub key_values: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct DoodadConnection {
    pub from: u32,
    pub to: u32,
    pub tag: String,
}

#[derive(Debug, Serialize)]
pub struct Decal {
    pub float1: f32,
    pub float2: f32,
    pub float3: f32,
    pub uint1: Option<u32>,
    pub float4: f32,
    pub atlas_file: String,
    pub tag: String,
}

#[derive(Debug, Serialize)]
pub struct Zone {
    pub name: String,
    pub x_min: i32,
    pub y_min: i32,
    pub x_max: i32,
    pub y_max: i32,
    pub string1: Option<String>,
    pub env_file: Option<String>,
    pub uint1: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct Thingy {
    pub et_file: Option<String>,
    pub int: i32,
    pub bool1: Option<bool>,
    pub bool2: Option<bool>,
    pub bool3: Option<bool>,
}

#[derive(Debug, Serialize, Default)]
pub struct Dimension {
    pub side_length: u32,
    pub uint1: Option<u32>,
}

#[derive(Debug, Serialize, Default)]
pub struct Map {
    pub version: u32,
    pub strings: Vec<String>,
    pub dimensions: Dimension,
    pub numbers1: Vec<u32>,
    pub tag: String,
    pub bools: Vec<bool>,
    pub root_slot: Slot,
    pub thingies: Vec<Thingy>,
    pub points_of_interest: Vec<Vec<PoI>>,
    pub string1: Option<String>,
    pub grid: Vec<Vec<Slot>>,
    pub doodads: Vec<Doodad>,
    pub doodad_connections: Vec<DoodadConnection>,
    pub decals: Vec<Decal>,
    pub boss_lines: Option<Vec<Vec<String>>>,
    pub zones: Option<Vec<Zone>>,
    pub tags: Option<Vec<String>>,
    pub trailing: Option<Vec<u32>>,
}
