# Parsing tools for Path of Exile Bundle files

# Commands
* `list`: List the "virtual" file paths in the bundle
* `extract`: Extract the virtual files as-is, saving them as real files to disk
* `cat`: Dumps the binary contents of a file to stdout
* `dump-art`: Extracts DirectDraw Surface (.dds) files and converts them to PNGs
* `dump-tables`: Extracts data tables (.datc64), applies the [community-curated schemas](https://github.com/poe-tool-dev/dat-schema),
and saves them out as CSVs where the schema was successfully applied.

## Usage

From source (requires Rust to be installed)

```bash
cargo run --release --bin poe_files -- --help
```

Using executable file

```bash
poe_files --help
```

# Bundle File format

![bundle file format](./images/bundle_spec.png)

# Bundle Index File format

![bundle index file format](./images/bundle_index_spec.png)

**TODO List**

- Directly use Murmur64A as the Hasher for my LUTs, rather than using the hashes as keys with the default Hasher
- Proper error propogation in the lib crate using Anyhow
- Proper documentation for the lib crate

# Testing
Tested on linux (WSL) and Windows with the Steam version of PoE 1, and rolling latest patch from the CDN for PoE 2.
