use crate::file_parsers::bundle_index::types::PathRep;

// Pull out the path components from the bundle blob into a more usable structure
pub fn parse_paths(path_rep_bytes: &[u8], path_rep: &PathRep) -> ParsedPathRep {
    let full_bytes = &path_rep_bytes
        [path_rep.offset as usize..path_rep.offset as usize + path_rep.size as usize];

    let mut i = 4;

    // Base paths
    let mut bases = vec![];
    loop {
        let idx = u32::from_le_bytes(full_bytes[i..i + 4].try_into().unwrap()) as usize;

        if idx == 0 {
            i += 4;
            break;
        }

        let path = String::from_utf8(
            full_bytes[i + 4..]
                .iter()
                .take_while(|&&b| b != b'\0')
                .copied()
                .collect(),
        )
        .expect("Failed to parse string.");

        i += 4 + path.len() + 1;

        bases.push(PathSegment {
            value: path,
            is_leaf: false,
            parent_index: if idx - 1 < bases.len() {
                Some(idx - 1)
            } else {
                None
            },
        });
    }

    // Leaf paths
    let mut leaves = vec![];
    while i < full_bytes.len() {
        let idx = u32::from_le_bytes(full_bytes[i..i + 4].try_into().unwrap()) as usize;

        let path = String::from_utf8(
            full_bytes[i + 4..]
                .iter()
                .take_while(|&&b| b != b'\0')
                .copied()
                .collect(),
        )
        .expect("Failed to parse string.");

        i += 4 + path.len() + 1;

        leaves.push(PathSegment {
            value: path,
            is_leaf: true,
            parent_index: if idx - 1 < bases.len() {
                Some(idx - 1)
            } else {
                None
            },
        });
    }

    ParsedPathRep { bases, leaves }
}

pub struct ParsedPathRep {
    pub bases: Vec<PathSegment>,
    pub leaves: Vec<PathSegment>,
}

impl ParsedPathRep {
    /// Enumerate all of the full paths
    pub fn get_paths(&self) -> Vec<String> {
        // Enumerate all the strings
        self.leaves
            .iter()
            .map(|l| {
                // todo: try avoiding so much cloning
                let mut path_segments = vec![l.value.clone()];
                let mut parent = l.parent_index;
                while let Some(p) = parent {
                    path_segments.push(self.bases[p].value.clone());
                    parent = self.bases[p].parent_index;
                }

                path_segments.reverse();
                path_segments.concat()
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct PathSegment {
    pub value: String,
    pub is_leaf: bool,
    pub parent_index: Option<usize>,
}
