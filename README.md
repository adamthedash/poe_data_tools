# Parsing tools for Path of Exile Bundle files
Tested on linux with the Steam version of PoE, patch 3.25.3.1   

## Dump Paths
Extracts all of the file paths from the PoE steam data bundles.  
Paths will be printed to stdout.  

### Usage
From source (requires Rust to be installed)
```bash
cargo run --release --bin dump_paths -- /path/to/poe/steam_folder
```

Using executable file
```bash
dump_paths /path/to/poe/steam_folder
```

## Dump files
Extracts files from the PoE steam data bundles.  
Files to extract are read from stdin, one per line.  

### Usage
From source (requires Rust to be installed)
```bash
cargo run --release --bin dump_files -- /path/to/poe/steam_folder
```

Using executable file
```bash
dump_files /path/to/poe/steam_folder /path/to/output/folder
```

# Bundle File format
![bundle file format](./images/bundle_spec.png)

# Bundle Index File format
![bundle index file format](./images/bundle_index_spec.png)


**TODO List**
- Group files by their bundle before processing so we don't re-read the bundle a bunch of times
- Directly use Murmur64A as the Hasher for my LUTs, rather than using the hashes as keys with the default Hasher
- Proper error propogation in the lib crate using Anyhow
- Proper documentation for the lib crate
