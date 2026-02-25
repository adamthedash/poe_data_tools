use bytes::Bytes;

#[derive(Debug)]
pub struct BundleInfo {
    pub name: String,
    pub uncompressed_size: u32,
}

#[derive(Debug)]
pub struct FileInfo {
    pub hash: u64,
    pub bundle_index: u32,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug)]
pub struct PathRep {
    pub hash: u64,
    pub offset: u32,
    pub size: u32,
    pub recursive_size: u32,
}

#[derive(Debug)]
pub struct BundleIndexFile {
    pub bundles: Vec<BundleInfo>,
    pub files: Vec<FileInfo>,
    pub paths: Vec<PathRep>,
    pub path_rep_bundle: Bytes,
}
