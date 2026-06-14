pub struct GGPKFile {
    pub entries: Vec<Entry>,
}

pub struct Entry {
    pub name: String,
    pub hash: Option<u32>,
    pub sha_digest: [u8; 32],
    pub data: EntryData,
}

pub enum EntryData {
    Dir(Vec<Entry>),
    File { offset: usize, length: usize },
}
