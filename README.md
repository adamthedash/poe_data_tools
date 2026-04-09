# PoE Data Tools
Parsing tools for Path of Exile game files. Available as a standalone binary (See releases), and as a rust crate.  

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

From source (requires Rust to be installed)

```bash
cargo run --release --bin poe_data_tools -- --help
```

Using executable file

```bash
poe_data_tools --help
```

As a rust library. Note that due to requiring a few forked dependencies, I can't yet publish it to crates.io. Instead, it must be added as a git dependency.  

```toml
[dependencies]
poe_data_tools = { git = "https://github.com/adamthedash/poe_data_tools.git" }
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

# Format coverage
PoE 1 (patch 3.28.0.7), PoE Data Tools v1.6.0

|Format|Fail|Success|Total|Success %|
|-|-|-|-|-|
|amd   |     0| 19145| 19145|100.00%|
|ao    |     0| 61567| 61567|100.00%|
|arm   |     0| 22463| 22463|100.00%|
|cht   |     0|   602|   602|100.00%|
|clt   |     0|   328|   328|100.00%|
|dct   |     0|   471|   471|100.00%|
|ddt   |     0|   652|   652|100.00%|
|dlp   |     0|    84|    84|100.00%|
|ecf   |     1|   588|   589| 99.83%|
|epk   |     0|  8973|  8973|100.00%|
|et    |     0|  1440|  1440|100.00%|
|fmt   |     0| 33794| 33794|100.00%|
|gcf   |     0|     1|     1|100.00%|
|gft   |     0|   931|   931|100.00%|
|gt    |     0|  1104|  1104|100.00%|
|mat   |     0|143681|143681|100.00%|
|mtd   |     0|   891|   891|100.00%|
|pet   |     1| 37752| 37753|100.00%|
|psg   |     0|     7|     7|100.00%|
|rs    |     0|   989|   989|100.00%|
|tmo   |     0|   155|   155|100.00%|
|toy   |     0|    47|    47|100.00%|
|trl   |     0|  6265|  6265|100.00%|
|tsi   |     0|  1087|  1087|100.00%|
|tst   |     0|   946|   946|100.00%|

<details>
<summary>By file format version</summary>

(fails : successes)
||Unknown|0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31|32|33|34|35|
|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|
|amd|||||0 : 12204|0 : 4685|0 : 2256||||||||||||||||||||||||||||||||
|ao|||||0 : 61567||||||||||||||||||||||||||||||||||
|arm||||||||||||||||0 : 110|0 : 2|0 : 25||0 : 73|0 : 7|0 : 36|0 : 45|0 : 3090|0 : 13|0 : 4||0 : 7690|0 : 1996|0 : 531|0 : 3304|0 : 1697|0 : 2641|0 : 160|0 : 7|0 : 2|0 : 203|0 : 827|
|cht||||0 : 601|0 : 1||||||||||||||||||||||||||||||||||
|clt||||0 : 2|0 : 36|0 : 290|||||||||||||||||||||||||||||||||
|dct||||0 : 471|||||||||||||||||||||||||||||||||||
|ddt|||||0 : 650|0 : 2|||||||||||||||||||||||||||||||||
|dlp|0 : 83|||0 : 1|||||||||||||||||||||||||||||||||||
|ecf|||1 : 588||||||||||||||||||||||||||||||||||||
|epk|0 : 8973||||||||||||||||||||||||||||||||||||||
|et|0 : 1440||||||||||||||||||||||||||||||||||||||
|fmt||||||||0 : 11017|0 : 5415|0 : 2435|0 : 14927||||||||||||||||||||||||||||
|gcf|||0 : 1||||||||||||||||||||||||||||||||||||
|gft|||0 : 2|0 : 929|||||||||||||||||||||||||||||||||||
|gt|0 : 1104||||||||||||||||||||||||||||||||||||||
|mat|0 : 143681||||||||||||||||||||||||||||||||||||||
|mtd|||||||0 : 891||||||||||||||||||||||||||||||||
|pet|0 : 17302|||0 : 3429|0 : 11351|1 : 2839|0 : 2831||||||||||||||||||||||||||||||||
|psg|||||0 : 7||||||||||||||||||||||||||||||||||
|rs||||0 : 989|||||||||||||||||||||||||||||||||||
|tmo|||0 : 155||||||||||||||||||||||||||||||||||||
|toy|||0 : 47||||||||||||||||||||||||||||||||||||
|trl|0 : 3247|||0 : 2206||0 : 812|||||||||||||||||||||||||||||||||
|tsi|0 : 1087||||||||||||||||||||||||||||||||||||||
|tst|0 : 946||||||||||||||||||||||||||||||||||||||

</details>

PoE 2 (patch 4.4.0.10), PoE Data Tools v1.6.0

|Format|Fail|Success|Total|Success %|
|-|-|-|-|-|
|amd   |     0| 16486| 16486|100.00%|
|ao    |     0| 66846| 66846|100.00%|
|arm   |     0| 14766| 14766|100.00%|
|cht   |     1|   248|   249| 99.60%|
|clt   |     0|   208|   208|100.00%|
|dct   |     0|   156|   156|100.00%|
|ddt   |     0|   217|   217|100.00%|
|dlp   |     0|   185|   185|100.00%|
|ecf   |     0|   258|   258|100.00%|
|epk   |     0|  7130|  7130|100.00%|
|et    |     0|  1363|  1363|100.00%|
|fmt   |     0| 25257| 25257|100.00%|
|gft   |     0|   398|   398|100.00%|
|gt    |     0|   647|   647|100.00%|
|mat   |     0|164096|164096|100.00%|
|mtd   |     0|   405|   405|100.00%|
|pet   |     0| 32447| 32447|100.00%|
|psg   |     0|     3|     3|100.00%|
|rs    |     0|   441|   441|100.00%|
|tmo   |     1|   396|   397| 99.75%|
|toy   |     1|   175|   176| 99.43%|
|trl   |     0|  5365|  5365|100.00%|
|tsi   |     0|   463|   463|100.00%|
|tst   |     0|   409|   409|100.00%|

<details>
<summary>By file format version</summary>

(fails : successes)
||Unknown|0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31|32|33|34|35|
|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|
|amd|||0 : 3483|0 : 370|0 : 3175|0 : 5527|0 : 3931||||||||||||||||||||||||||||||||
|ao||||0 : 66846|||||||||||||||||||||||||||||||||||
|arm||||||||||||||||||||0 : 1||0 : 2||0 : 52||||0 : 40|0 : 6|0 : 5|0 : 362|0 : 142|0 : 1218|0 : 125|0 : 12||0 : 907|0 : 11894|
|cht||||1 : 195|0 : 53||||||||||||||||||||||||||||||||||
|clt|||||0 : 13|0 : 195|||||||||||||||||||||||||||||||||
|dct||||0 : 156|||||||||||||||||||||||||||||||||||
|ddt|||||0 : 152|0 : 65|||||||||||||||||||||||||||||||||
|dlp|0 : 11|||0 : 26|0 : 148||||||||||||||||||||||||||||||||||
|ecf|||0 : 258||||||||||||||||||||||||||||||||||||
|epk|0 : 7130||||||||||||||||||||||||||||||||||||||
|et|0 : 1363||||||||||||||||||||||||||||||||||||||
|fmt||||||0 : 1648|0 : 268|0 : 2737|0 : 2506|0 : 819|0 : 17279||||||||||||||||||||||||||||
|gft||||0 : 398|||||||||||||||||||||||||||||||||||
|gt|0 : 647||||||||||||||||||||||||||||||||||||||
|mat|0 : 164096||||||||||||||||||||||||||||||||||||||
|mtd|||||||0 : 405||||||||||||||||||||||||||||||||
|pet|0 : 9730|||0 : 2343|0 : 8066|0 : 4299|0 : 8009||||||||||||||||||||||||||||||||
|psg|||||0 : 3||||||||||||||||||||||||||||||||||
|rs||||0 : 441|||||||||||||||||||||||||||||||||||
|tmo|||1 : 396||||||||||||||||||||||||||||||||||||
|toy|||1 : 175||||||||||||||||||||||||||||||||||||
|trl|0 : 2023|||0 : 1476|0 : 4|0 : 1862|||||||||||||||||||||||||||||||||
|tsi|0 : 463||||||||||||||||||||||||||||||||||||||
|tst|0 : 409||||||||||||||||||||||||||||||||||||||

</details>

# GGPK File Format
![ggpk file format](./images/ggpk_spec.png)

# Bundle File Format
![bundle file format](./images/bundle_spec.png)

# Bundle Index File Format
![bundle index file format](./images/bundle_index_spec.png)

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

## DOLm Sub-File Format
![dolm format](./images/dolm_spec.png)  

## DLP File Format
![dlp format](./images/dlp_spec.png)  

## ECF File Format
![ecf format](./images/ecf_spec.png)  

## ET File Format
![et format](./images/et_spec.png)  

## EPK File Format
![epk format](./images/epk_spec.png)  

## FMT File Format
![fmt format](./images/fmt_spec.png)  

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

## PSG File Format
![psg file format](./images/psg_spec.png)  

## RS File Format
![rs format](./images/rs_spec.png)  

## SMD File Format
![smd format](./images/smd_spec.png)  

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

# TODO List

- Proper documentation for the lib crate
- Swap image.rs version once [DDS support is merged](https://github.com/image-rs/image/pull/2258)

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
