#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader, path::Path, process::Command};

    use ddsfile::{Caps, DataFormat, Dds, FourCC};

    #[test]
    fn test_load_art() {
        let path = "/home/adam/poe_tools_data/extract/art/textures/pet/chicken/chickenns.dds";

        let result = Command::new("../../external/texconv")
            .args("--help")
            .output()
            .expect("failed");
        println!("{:?}", result);
    }
}
