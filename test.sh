#!/bin/bash
STEAM_FOLDER="/mnt/e/SteamLibrary/steamapps/common/Path of Exile"
OUT_FOLDER="/mnt/e/programming/data/poe/file_dumps"

cargo run --release --bin dump_paths -- "$STEAM_FOLDER" | grep "\.dat64$" | cargo run --release --bin dump_files -- "$STEAM_FOLDER" "$OUT_FOLDER"

