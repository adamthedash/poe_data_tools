#!/bin/bash
OUT_FOLDER="/mnt/e/programming/data/poe/file_dumps"

# Extract the dat64 files from the CDN
# cargo run --release --bin poe_files -- extract "$OUT_FOLDER" "*.datc64"

# Extract the dds files from $STEAM_FOLDER
# cargo run --release --bin poe_files -- --steam extract "$OUT_FOLDER" "*.dds"

# Use imageMagick to turn all the dds files into png
PNG_FOLDER="/mnt/e/programming/data/poe/dds_pngs"
find "$OUT_FOLDER" -type f -name "*.dds" | while read -r dds_path; do
  # Figure out the paths to use
  png_path="$PNG_FOLDER/${dds_path#$OUT_FOLDER/}"
  png_path="${png_path%.dds}.png"
  echo "$dds_path $png_path"
  # Convert in parallel
done | parallel --bar --no-notice --colsep ' ' 'mkdir -p "$(dirname {2})"; magick "{1}" "{2}"'
