use std::path::PathBuf;

pub fn steam_folder_search(patch: &str) -> Option<PathBuf> {
    let home = dirs::home_dir().unwrap();
    let game = match patch {
        "1" => "Path of Exile",
        "2" => "Path of Exile 2",
        _ => return None,
    };
    [
        home.join(".local/share/Steam/steamapps/common"),
        home.join("Library/Application Support/Steam/steamapps/common"),
        PathBuf::from("C:\\Program Files (x86)\\Grinding Gear Games"),
        PathBuf::from("/mnt/e/SteamLibrary/steamapps/common"),
    ]
    .iter()
    .map(|p| p.join(game))
    .find(|p| p.exists())
}
