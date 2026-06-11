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
PoE 1 (patch 3.28.0.12), PoE Data Tools v1.7.3

|Format|Fail|Success|Total|Success %|
|-|-|-|-|-|
|amd   |     0| 19427| 19427|100.00%|
|ao    |     0| 62756| 62756|100.00%|
|arm   |     0| 22473| 22473|100.00%|
|cht   |     0|   602|   602|100.00%|
|clt   |     0|   328|   328|100.00%|
|dct   |     0|   471|   471|100.00%|
|ddt   |     0|   652|   652|100.00%|
|dlp   |     0|    84|    84|100.00%|
|ecf   |     1|   590|   591| 99.83%|
|epk   |     0|  9174|  9174|100.00%|
|et    |     0|  1460|  1460|100.00%|
|fmt   |     0| 34384| 34384|100.00%|
|gcf   |     0|     1|     1|100.00%|
|gft   |     0|   936|   936|100.00%|
|gt    |     0|  1117|  1117|100.00%|
|mat   |     0|147060|147060|100.00%|
|mtd   |     0|   896|   896|100.00%|
|pet   |     1| 38356| 38357|100.00%|
|psg   |     0|     7|     7|100.00%|
|rs    |     0|   994|   994|100.00%|
|sm    |     0| 35093| 35093|100.00%|
|smd   |     3| 34310| 34313| 99.99%|
|tgm   |     0|118418|118418|100.00%|
|tgt   |     0| 19924| 19924|100.00%|
|tmo   |     0|   157|   157|100.00%|
|toy   |     0|    47|    47|100.00%|
|trl   |     0|  6310|  6310|100.00%|
|tsi   |     0|  1093|  1093|100.00%|
|tst   |     0|   950|   950|100.00%|

<details>
<summary>By file format version</summary>

By file format version (fails : successes)
||Unknown|0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31|32|33|34|35|
|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|
|amd|||||0 : 12201|0 : 4683|0 : 2543||||||||||||||||||||||||||||||||
|ao||||0 : 121|0 : 62635||||||||||||||||||||||||||||||||||
|arm||||||||||||||||0 : 110|0 : 2|0 : 25||0 : 73|0 : 7|0 : 36|0 : 45|0 : 3090|0 : 13|0 : 4||0 : 7690|0 : 1996|0 : 531|0 : 3304|0 : 1697|0 : 2641|0 : 160|0 : 7|0 : 2|0 : 203|0 : 837|
|cht||||0 : 601|0 : 1||||||||||||||||||||||||||||||||||
|clt||||0 : 2|0 : 36|0 : 290|||||||||||||||||||||||||||||||||
|dct||||0 : 471|||||||||||||||||||||||||||||||||||
|ddt|||||0 : 650|0 : 2|||||||||||||||||||||||||||||||||
|dlp|0 : 83|||0 : 1|||||||||||||||||||||||||||||||||||
|ecf|||1 : 590||||||||||||||||||||||||||||||||||||
|epk|0 : 9174||||||||||||||||||||||||||||||||||||||
|et|0 : 1460||||||||||||||||||||||||||||||||||||||
|fmt||||||||0 : 11016|0 : 5415|0 : 2425|0 : 15528||||||||||||||||||||||||||||
|gcf|||0 : 1||||||||||||||||||||||||||||||||||||
|gft|||0 : 2|0 : 934|||||||||||||||||||||||||||||||||||
|gt|0 : 1117||||||||||||||||||||||||||||||||||||||
|mat|0 : 147060||||||||||||||||||||||||||||||||||||||
|mtd|||||||0 : 896||||||||||||||||||||||||||||||||
|pet|0 : 17303|||0 : 3429|0 : 11351|1 : 2839|0 : 3434||||||||||||||||||||||||||||||||
|psg|||||0 : 7||||||||||||||||||||||||||||||||||
|rs||||0 : 994|||||||||||||||||||||||||||||||||||
|sm|||||||0 : 23884|0 : 11209|||||||||||||||||||||||||||||||
|smd|||0 : 19449|3 : 1938|0 : 12923||||||||||||||||||||||||||||||||||
|tgm||||||||0 : 38433|0 : 12133|0 : 9923|0 : 12197|0 : 2383||0 : 1547|0 : 2451|0 : 39351|||||||||||||||||||||||
|tgt|||||0 : 19924||||||||||||||||||||||||||||||||||
|tmo|||0 : 157||||||||||||||||||||||||||||||||||||
|toy|||0 : 47||||||||||||||||||||||||||||||||||||
|trl|0 : 3247|||0 : 2206||0 : 857|||||||||||||||||||||||||||||||||
|tsi|0 : 1093||||||||||||||||||||||||||||||||||||||
|tst|0 : 950||||||||||||||||||||||||||||||||||||||

## What's Changed
* TGT Files by @adamthedash in https://github.com/adamthedash/poe_data_tools/pull/30

Preparation work towards publishing the project as a library on crates.io:
- Previously used [fork of image-rs](https://github.com/RunDevelopment/image/tree/new-dds-decoder) with DDS support has been migrated to [image-extras](https://github.com/image-rs/image-extras)
- My [fork of winnow](https://github.com/adamthedash/winnow) has been published to crates.io as [adamthedash_winnow](https://crates.io/crates/adamthedash_winnow)
- My new [annotated parser](https://github.com/adamthedash/annotated_parser) library has been published

**Full Changelog**: https://github.com/adamthedash/poe_data_tools/compare/v1.7.2...v1.7.3

</details>

PoE 2 (patch 4.5.1.1.6), PoE Data Tools v1.7.3

|Format|Fail|Success|Total|Success %|
|-|-|-|-|-|
|amd   |     0|  7252|  7252|100.00%|
|ao    |     4| 18047| 18051| 99.98%|
|arm   |     0|  3680|  3680|100.00%|
|cht   |     0|    55|    55|100.00%|
|clt   |     0|    63|    63|100.00%|
|dct   |     0|    82|    82|100.00%|
|ddt   |     0|    59|    59|100.00%|
|dlp   |     0|    53|    53|100.00%|
|ecf   |     0|    73|    73|100.00%|
|epk   |     0|  2038|  2038|100.00%|
|et    |     0|   366|   366|100.00%|
|fmt   |     0|  7415|  7415|100.00%|
|gft   |     0|   126|   126|100.00%|
|gt    |     0|   186|   186|100.00%|
|mat   |     0| 61548| 61548|100.00%|
|mtd   |     0|   110|   110|100.00%|
|pet   |     0|  9057|  9057|100.00%|
|psg   |     0|     2|     2|100.00%|
|rs    |     0|   113|   113|100.00%|
|sm    |     0| 13619| 13619|100.00%|
|smd   |     7|  7222|  7229| 99.90%|
|tgm   |     1| 79839| 79840|100.00%|
|tgt   |     0|  6233|  6233|100.00%|
|tmo   |     1|    78|    79| 98.73%|
|toy   |     0|    40|    40|100.00%|
|trl   |     0|  1388|  1388|100.00%|
|tsi   |     0|   120|   120|100.00%|
|tst   |     0|   111|   111|100.00%|

<details>
<summary>By file format version</summary>

By file format version (fails : successes)
||Unknown|0|1|2|3|4|5|6|7|8|9|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26|27|28|29|30|31|32|33|34|35|
|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|
|amd|||0 : 1609|0 : 175|0 : 1087|0 : 2171|0 : 2210||||||||||||||||||||||||||||||||
|ao|||||4 : 18047||||||||||||||||||||||||||||||||||
|arm||||||||||||||||||||||||0 : 9||||0 : 8|||0 : 96|0 : 27|0 : 335|0 : 67|0 : 5||0 : 234|0 : 2899|
|cht||||0 : 44|0 : 11||||||||||||||||||||||||||||||||||
|clt|||||0 : 3|0 : 60|||||||||||||||||||||||||||||||||
|dct||||0 : 82|||||||||||||||||||||||||||||||||||
|ddt|||||0 : 45|0 : 14|||||||||||||||||||||||||||||||||
|dlp|0 : 2|||0 : 7|0 : 40|0 : 4|||||||||||||||||||||||||||||||||
|ecf|||0 : 73||||||||||||||||||||||||||||||||||||
|epk|0 : 2038||||||||||||||||||||||||||||||||||||||
|et|0 : 366||||||||||||||||||||||||||||||||||||||
|fmt||||||0 : 446|0 : 99|0 : 711|0 : 640|0 : 225|0 : 5294||||||||||||||||||||||||||||
|gft||||0 : 126|||||||||||||||||||||||||||||||||||
|gt|0 : 186||||||||||||||||||||||||||||||||||||||
|mat|0 : 61548||||||||||||||||||||||||||||||||||||||
|mtd|||||||0 : 110||||||||||||||||||||||||||||||||
|pet|0 : 2424|||0 : 577|0 : 1944|0 : 1099|0 : 3013||||||||||||||||||||||||||||||||
|psg|||||0 : 2||||||||||||||||||||||||||||||||||
|rs||||0 : 113|||||||||||||||||||||||||||||||||||
|sm||||||0 : 1123|0 : 1458|0 : 11038|||||||||||||||||||||||||||||||
|smd|||0 : 1378|0 : 80|7 : 5764||||||||||||||||||||||||||||||||||
|tgm||||||0 : 201|1 : 43|0 : 590|0 : 501|0 : 125|0 : 236|0 : 485|0 : 118|0 : 203|0 : 141|0 : 77196|||||||||||||||||||||||
|tgt|||0 : 152||0 : 6081||||||||||||||||||||||||||||||||||
|tmo|1 : 0||0 : 78||||||||||||||||||||||||||||||||||||
|toy|||0 : 40||||||||||||||||||||||||||||||||||||
|trl|0 : 485|||0 : 350|0 : 1|0 : 552|||||||||||||||||||||||||||||||||
|tsi|0 : 120||||||||||||||||||||||||||||||||||||||
|tst|0 : 111||||||||||||||||||||||||||||||||||||||

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

## SM File Format
![sm format](./images/sm_spec.png)  

## SMD File Format
![smd format](./images/smd_spec.png)  

## TGM File Format
![tgm format](./images/tgm_spec.png)  

## TGT File Format
![tgt format](./images/tgt_spec.png)  

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
