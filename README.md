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
