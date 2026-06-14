# PoE Data Tools
[![LICENSE-MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../../LICENSE-MIT)
[![LICENSE-APACHE](https://img.shields.io/badge/license-Apache_2.0-blue.svg)](../../LICENSE-APACHE)
[![Crates.io Version](https://img.shields.io/crates/v/poe_data_tools-cli.svg)](https://crates.io/crates/poe_data_tools-cli)

A command line utility for working with Path of Exile game files.  

# Commands

- `list`: List the "virtual" file paths in the bundle
- `extract`: Extract the virtual files as-is, saving them as real files to disk
- `cat`: Dumps the binary contents of a file to stdout
- `dump-art`: Extracts DirectDraw Surface (.dds) files and converts them to PNGs
- `dump-tables`: Extracts data tables (.datc64), applies the [community-curated schemas](https://github.com/poe-tool-dev/dat-schema),
  and saves them out to more accessible formats.  
- `dump-tree`: Extracts passive skill trees (player, atlas, ruthless, etc.) to JSON
- `translate`: Extracts files and converts them to more accessible formats.  

## Usage
### Installation
1) Download a pre-built binary (see releases page on GitHub)  

2) Install with `cargo install` (requires rust toolchain):  
```bash
cargo install poe_data_tools-cli
```

3) From source (requires rust toolchain):  
```bash
cargo build --release
```

### Running
1) Using the executable
```bash
poe_data_tools --help
```

2) From source (requires rust toolchain):  
```bash
cargo run --release -- --help
```

## Globs
Many of the commands can take glob patterns to operate over several files at once. Note that the patterns follow the [Unix glob](https://www.man7.org/linux/man-pages/man7/glob.7.html) specification.  
Several patterns can be provided at once.  

```bash
# all files in all directories (the default)
poe_data_tools --patch 2 list '**'
# all .datc64 and .dds files in all subdirectories
poe_data_tools --patch 2 list '**/*.datc64' '**/*.dds'
# all files in the art/ directory
poe_data_tools --patch 2 list 'art/*'
# all files in the art/ directory and its subdirectories
poe_data_tools --patch 2 list 'art/**'
```
