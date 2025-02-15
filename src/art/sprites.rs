use nom::{
    bytes::complete::take,
    multi::count,
    number::complete::{le_u16, le_u32, le_u8},
    IResult,
};

// Take a null-terminated UTF-16 string
fn take_utf16_string(input: &[u8]) -> String {
    let u16_data = input
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes(c.try_into().unwrap()))
        .take_while(|x| *x != 0)
        .collect::<Vec<_>>();

    String::from_utf16(&u16_data).expect("Failed to parse UTF-16 string.")
}

pub fn parse_mtp(bytes: &[u8]) -> IResult<&[u8], ()> {
    // Some bytes at the start
    // Some repeating set of bytes, table maybe?
    // A bunch of utf-16 cstrings
    println!("First 8 bytes: {:?}", &bytes[..8]);
    let (_input, first_bytes) = count(le_u32, 2)(bytes)?;
    println!("First 8 bytes: {:?}", first_bytes);

    let table_rows = first_bytes[1];
    println!("Num rows: {}", table_rows);

    let num_objects = (table_rows / 4) as usize;
    let table_start = 8;

    let string_start = table_start + table_rows as usize * 32 + 4;

    let table_bytes = bytes[table_start..string_start]
        .chunks_exact(32)
        .collect::<Vec<_>>();

    let string_bytes = &bytes[string_start..];
    let mut offset = 0;
    let tdt_paths = (0..num_objects)
        .map(|_| {
            let path = take_utf16_string(&string_bytes[offset..]);
            offset += (path.len() + 1) * 2;
            path
        })
        .collect::<Vec<_>>();

    let dds_path = take_utf16_string(&string_bytes[offset..]);

    tdt_paths.iter().for_each(|p| println!("{:?}", p));
    println!("{:?}", dds_path);

    table_bytes.iter().try_for_each(|&rest| {
        println!("{:?}", rest);
        let (rest, x1) = le_u32(rest)?;
        let (rest, orientation) = le_u32(rest)?;
        let (rest, sprite_width) = le_u32(rest)?;
        let (rest, x3) = le_u16(rest)?;
        let (rest, bytes1) = take(8_usize)(rest)?;
        //let (rest, x4) = le_u32(rest)?;
        let (rest, x5) = le_u8(rest)?;
        let (rest, x6) = le_u16(rest)?;
        println!(
            "{} or {} w {} {:?} {:?} {} {}",
            x1, orientation, sprite_width, x3, bytes1, x5, x6
        );

        println!("{:?}", rest);

        Ok(())
    })?;

    Ok((bytes, ()))
}

#[cfg(test)]
mod tests {
    use crate::art::sprites::parse_mtp;

    #[test]
    fn test() {
        let path = "/home/adam/poe_data/raw/minimap/metadata_terrain_temple_ledge.mtp";
        //let path =
        //    "/home/adam/poe_data/raw/minimap/metadata_terrain_woods_woods_azmerileague_features.mtp";
        //let path = "/home/adam/poe_data/raw/minimap/metadata_terrain_woods_slash_rockpile.mtp";

        let contents = std::fs::read(path).unwrap();
        println!("{}", contents.len());
        parse_mtp(&contents).unwrap();
    }
}
