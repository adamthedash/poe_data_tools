# Parsing tools for Path of Exile Bundle files

# Commands

- `list`: List the "virtual" file paths in the bundle
- `extract`: Extract the virtual files as-is, saving them as real files to disk
- `cat`: Dumps the binary contents of a file to stdout
- `dump-art`: Extracts DirectDraw Surface (.dds) files and converts them to PNGs
- `dump-tables`: Extracts data tables (.datc64), applies the [community-curated schemas](https://github.com/poe-tool-dev/dat-schema),
  and saves them out as CSVs where the schema was successfully applied.  
- `dump-tree`: Extracts passive skill trees (player, atlas, ruthless, etc.) to JSON


## Usage

From source (requires Rust to be installed)

```bash
cargo run --release --bin poe_data_tools -- --help
```

Using executable file

```bash
poe_data_tools --help
```

## Globs

Note that commands that take globs use the form that requires `**` to match across directory separators, e.g.

```bash
# all files in all directories (the default)
cargo run --release --bin poe_data_tools -- --patch 2 list '**'
# all .datc64 files in all subdirectories
cargo run --release --bin poe_data_tools -- --patch 2 list '**/*.datc64'
# all files in the art/ directory
cargo run --release --bin poe_data_tools -- --patch 2 list 'art/*'
# all files in the art/ directory and its subdirectories
cargo run --release --bin poe_data_tools -- --patch 2 list 'art/**'
```

# Bundle File format

![bundle file format](./images/bundle_spec.png)

# Bundle Index File format

![bundle index file format](./images/bundle_index_spec.png)

# PassiveSkillGraph (PSG) File Format
![psg file format poe1](./images/psg_spec_poe1.png)  
![psg file format poe2](./images/psg_spec_poe2.png)  

# World Areas (ARM) File Format
![arm format](./images/arm_spec.png)  

**TODO List**

- Proper documentation for the lib crate
- Swap image.rs version once [DDS support is merged](https://github.com/image-rs/image/pull/2258)

# Testing

Tested on linux (WSL) and Windows with the Steam version of PoE 1, and rolling latest patch from the CDN for PoE 2.

# Resources
https://gitlab.com/zao/poe-rs

```
.act = Actor, UCS-2 plain
.ais = AI Script, UCS-2 plain
.amd = Animated Meta Data, UCS-2 plain
.ao = Animated Object, UCS-2 plain
.aoc = Animated Object Controller, UCS-2 plain
.arm = Rooms, UCS-2 plain
.ast = Skeleton, Binary
.bank = FMOD Sound Bank, Binary
.bk2 = Bink Video, Binary
.cht = Chest Data, UCS-2 plain
.dat = Game Data
.dat64 = Game Data 64bits
.dds = Texture, brotli compress or soft link
.ddt = Doodads, UCS-2 plain
.dlp = Doodads, UCS-2 plain
.dgr = Digital Graph Document, UCS-2 plain
.env = Environments, UCS-2 plain
.filter = Loot Filter, UTF-8
.ffx = FFX Render, UCS-2 plain
.fx = Shader, Ascii
.gm = Grandmaster, Binary
.gt = Ground Types, UCS-2 plain
.hlsl = Shader, Ascii
.mat = Material, UCS-2 plain
.ogg = Digital Multimedia, Binary
.ot = Object Type, UCS-2 plain
.otc = Object Type Codes, UCS-2 plain
.pet = Particle Effect, Binary
.pjd = Passive Jewel Data, Binary
.psg = Passive Skill Graphic, Binary
.rs = Room Set, UCS-2 plain
.sm = Skin Mesh, UCS-2 plain
.smd = Skin Mesh Data, binary
.spritefont = Raster Font Data, Binary
.tgt = Tile Group, UCS-2 plain
.txt = Text, UCS-2 plain
.ui = User Interface, UCS-2 plain
```
