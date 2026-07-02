use std::collections::HashMap;

use poe_data_tools::file_parsers::psg::types::Group;
use serde::Serialize;

use crate::tree::passive_info::PassiveSkillInfo;

/// Same format as what's exported by RePoE
#[derive(Debug, Serialize)]
pub struct PassiveSkillGraph {
    pub version: u8,
    /// 1 == Passive skill tree
    /// 2 == Atlas tree
    pub graph_type: u8,
    pub passives_per_orbit: Vec<u8>,
    pub root_passives: Vec<u64>,
    pub groups: Vec<Group>,
    pub passive_info: HashMap<u16, PassiveSkillInfo>,
}
