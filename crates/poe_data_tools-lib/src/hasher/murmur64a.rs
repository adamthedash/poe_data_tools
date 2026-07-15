use std::hash::{BuildHasher, Hasher};

use murmurhash64::murmur_hash64a;

/// Hasher used internally by GGG
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

pub trait BuildHasherEx: BuildHasher {
    /// Hash a string
    // NOTE: This is required because the str::hash hashse a length-prefixed set of bytes.
    // GGG's impl just hashes the bytes
    fn hash_one_str(&self, string: &str) -> u64;
}

impl BuildHasherEx for BuildMurmurHash64A {
    fn hash_one_str(&self, string: &str) -> u64 {
        let mut hasher = self.build_hasher();
        hasher.write(string.as_bytes());
        hasher.finish()
    }
}
