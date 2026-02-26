# Parsing tools for Path of Exile Bundle files

# Commands

- `list`: List the "virtual" file paths in the bundle
- `extract`: Extract the virtual files as-is, saving them as real files to disk
- `cat`: Dumps the binary contents of a file to stdout
- `dump-art`: Extracts DirectDraw Surface (.dds) files and converts them to PNGs
- `dump-tables`: Extracts data tables (.datc64), applies the [community-curated schemas](https://github.com/poe-tool-dev/dat-schema),
  and saves them out as CSVs where the schema was successfully applied.  
- `dump-tree`: Extracts passive skill trees (player, atlas, ruthless, etc.) to JSON
- `translate`: Extracts files and converts them to more accessible formats.  


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

# Format coverage
PoE 1 (patch 3.27.0.8.2), PoE Data Tools v1.4.0  

|Format|Fail|Success|Total|Success %|
|-|-|-|-|-|
|amd   |     0| 18362| 18362|100.00%|
|ao    | 50659|  8822| 59481| 14.83%|
|arm   |     0| 22422| 22422|100.00%|
|cht   |     0|   602|   602|100.00%|
|clt   |     0|   328|   328|100.00%|
|dct   |     0|   471|   471|100.00%|
|ddt   |     0|   652|   652|100.00%|
|dlp   |     0|    84|    84|100.00%|
|ecf   |     1|   588|   589| 99.83%|
|epk   |     0|  8709|  8709|100.00%|
|et    |     0|  1435|  1435|100.00%|
|gcf   |     0|     1|     1|100.00%|
|gft   |     0|   922|   922|100.00%|
|gt    |     0|  1094|  1094|100.00%|
|mat   |     0|136006|136006|100.00%|
|mtd   |     0|   882|   882|100.00%|
|pet   |     1| 36946| 36947|100.00%|
|psg   |     0|     7|     7|100.00%|
|rs    |     0|   980|   980|100.00%|
|tmo   |     0|   152|   152|100.00%|
|toy   |     0|    47|    47|100.00%|
|trl   |     0|  6177|  6177|100.00%|
|tsi   |     0|  1078|  1078|100.00%|
|tst   |     0|   937|   937|100.00%|

PoE 2 (patch 4.4.0.6.6), PoE Data Tools v1.4.0  

|Format|Fail|Success|Total|Success %|
|-|-|-|-|-|
|amd   |     0| 16435| 16435|100.00%|
|ao    | 54120| 12412| 66532| 18.66%|
|arm   |     0| 14759| 14759|100.00%|
|cht   |     1|   248|   249| 99.60%|
|clt   |     0|   208|   208|100.00%|
|dct   |     0|   156|   156|100.00%|
|ddt   |     0|   217|   217|100.00%|
|dlp   |     0|   185|   185|100.00%|
|ecf   |     0|   258|   258|100.00%|
|epk   |     0|  7079|  7079|100.00%|
|et    |     0|  1362|  1362|100.00%|
|gft   |     0|   397|   397|100.00%|
|gt    |     0|   646|   646|100.00%|
|mat   |     0|163326|163326|100.00%|
|mtd   |     0|   404|   404|100.00%|
|pet   |     0| 32314| 32314|100.00%|
|psg   |     0|     3|     3|100.00%|
|rs    |     0|   440|   440|100.00%|
|tmo   |     1|   395|   396| 99.75%|
|toy   |     1|   175|   176| 99.43%|
|trl   |     0|  5359|  5359|100.00%|
|tsi   |     0|   462|   462|100.00%|
|tst   |     0|   408|   408|100.00%|


# Bundle File format

![bundle file format](./images/bundle_spec.png)

# Bundle Index File format

![bundle index file format](./images/bundle_index_spec.png)

# PassiveSkillGraph (PSG) File Format
![psg file format poe1](./images/psg_spec_poe1.png)  
![psg file format poe2](./images/psg_spec_poe2.png)  

# World Area related files
![world areas](./images/world_areas.png)  

## AMD File Format
![amd format](./images/amd_spec.png)  

## AO File Format
![ao format](./images/ao_spec.png)  

## ARM File Format
![arm format](./images/arm_spec.png)  

## CHT File Format
![cht format](./images/cht_spec.png)  

## CLT File Format
![clt format](./images/clt_spec.png)  

## DCT File Format
![dct format](./images/dct_spec.png)  

## DDT File Format
![ddt format](./images/ddt_spec.png)  

## DLP File Format
![dlp format](./images/dlp_spec.png)  

## ECF File Format
![ecf format](./images/ecf_spec.png)  

## ET File Format
![et format](./images/et_spec.png)  

## EPK File Format
![epk format](./images/epk_spec.png)  

## GCF File Format
![gcf format](./images/gcf_spec.png)  

## GFT File Format
![gft format](./images/gft_spec.png)  

## GT File Format
![gt format](./images/gt_spec.png)  

## MAT File Format
![mat format](./images/mat_spec.png)  

## MTD File Format
![mtd format](./images/mtd_spec.png)  

## PET File Format
![pet format](./images/pet_spec.png)  

## RS File Format
![rs format](./images/rs_spec.png)  

## TMO File Format
![tmo format](./images/tmo_spec.png)  

## TOY File Format
![toy format](./images/toy_spec.png)  

## TRL File Format
![trl format](./images/trl_spec.png)  

## TSI File Format
![tsi format](./images/tsi_spec.png)  

## TST File Format
![tst format](./images/tst_spec.png)  

**TODO List**

- Proper documentation for the lib crate
- Full migration from nom over to winnow  
- Swap image.rs version once [DDS support is merged](https://github.com/image-rs/image/pull/2258)

# Testing

Tested on linux (WSL) and Windows with the Steam version of PoE 1, and rolling latest patch from the CDN for PoE 2.

# Resources
https://gitlab.com/zao/poe-rs  
https://gist.github.com/zao/96cb1869db121fbd038f8cd66f7d5736 - for .fmt/tdt/tgm  
https://gitlab.com/zao/poe-cpp/-/tree/master/libpoe/poe/format  
https://bitbucket.org/zao/gggtools/src/evolve-ggpkviewer/spec/  


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
