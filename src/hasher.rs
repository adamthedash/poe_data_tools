//! Hasher for virtual file paths

use std::hash::{BuildHasher, Hasher};

use murmurhash64::murmur_hash64a;

pub struct MurmurHash64A {
    seed: u64,
    data: Vec<u8>,
}

impl MurmurHash64A {
    pub fn new(seed: u64) -> Self {
        MurmurHash64A { seed, data: vec![] }
    }
}

impl Hasher for MurmurHash64A {
    fn finish(&self) -> u64 {
        murmur_hash64a(&self.data, self.seed)
    }

    fn write(&mut self, bytes: &[u8]) {
        self.data.extend(bytes);
    }
}

pub struct BuildMurmurHash64A {
    pub seed: u64,
}

impl BuildHasher for BuildMurmurHash64A {
    type Hasher = MurmurHash64A;

    fn build_hasher(&self) -> Self::Hasher {
        MurmurHash64A::new(self.seed)
    }
}
