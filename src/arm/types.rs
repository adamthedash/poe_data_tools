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

#[derive(Debug, Serialize)]
pub enum Slot {
    K(SlotK),
    N,
    F { fill: Option<String> },
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

    pub float_pairs: Vec<(f32, f32)>,

    // Likely yaw/pitch/roll
    // Quaternion? https://gitlab.com/zao/poe-rs/-/blob/master/src/formats/arm/types.rs?ref_type=heads#L82
    pub radians1: f32,
    pub radians2: Option<f32>,
    pub radians3: Option<f32>,

    pub radians4: Option<f32>,
    pub radians5: Option<f32>,
    // pub float1: Option<f32>,
    pub uint3: u32,
    pub uint4: Option<u32>,
    pub floats: Vec<f32>,

    pub scale: f32,
    pub ao_file: String,
    pub stub: String,

    pub key_values: HashMap<String, String>,
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
    pub numbers3: Vec<Vec<i32>>,
    pub points_of_interest: Vec<Vec<PoI>>,
    pub string1: Option<String>,
    pub grid: Vec<Vec<Slot>>,
    pub doodads: Vec<Doodad>,
    //TODO: interpret
    pub doodad_connections: Vec<String>,
    //TODO: interpret
    pub decals: Vec<String>,
    //TODO: interpret
    pub boss_lines: Option<Vec<(Vec<String>, Vec<i32>)>>,
    //TODO: interpret
    pub zones: Option<Vec<String>>,
}
