#!/bin/bash
STEAM_FOLDER="/mnt/e/SteamLibrary/steamapps/common/Path of Exile"
OUT_FOLDER="/mnt/e/programming/data/poe/file_dumps"

# Extract the dat64 files
# cargo run --release --bin dump_paths -- "$STEAM_FOLDER" | grep "\.dat64$" | cargo run --release --bin dump_files -- "$STEAM_FOLDER" "$OUT_FOLDER"

# Extract the dds & header files
# cargo run --release --bin dump_paths -- "$STEAM_FOLDER" | grep -E "\.dds(\.header)?$" | cargo run --release --bin dump_files -- "$STEAM_FOLDER" "$OUT_FOLDER"

# Use imageMagick to turn all the dds files into png
PNG_FOLDER="/mnt/e/programming/data/poe/dds_pngs"
find "$OUT_FOLDER" -type f -name "*.dds" | while read -r dds_path; do
  # Figure out the right paths
  png_path="$PNG_FOLDER/${dds_path#$OUT_FOLDER/}"
  png_path="${png_path%.dds}.png"

  # Create output folder
  mkdir -p "$(dirname "$png_path")"

  # Convert image
  echo "$dds_path -> $png_path"
  magick "$dds_path" "$png_path"
done
