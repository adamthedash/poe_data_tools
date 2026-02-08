pub type RSFile = Vec<Room>;

pub struct Room {
    pub weight: Option<u32>,
    pub arm_file: String,
}
