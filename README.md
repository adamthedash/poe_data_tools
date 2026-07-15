# PoE Data Tools
Parsing tools for Path of Exile game files. Available as a standalone binary and as a rust crate.  

See [here](./crates/poe_data_tools-cli) for more info on the CLI.  
See [here](./crates/poe_data_tools-lib) for more info on the rust crate.  
See [here](FORMATS.md) for knowledge base of PoE file formats.  

### AI Disclaimer
Everything here has been has been lovingly hand crafted, but I won't rule out the use of LLMs down the line. If/when that happens, its use will be clearly outlined here.

### TODO List
- (lib) Better documentation / public API redesign
- (lib) Anyhow -> structured errors
- (lib) Native async API for filesystem
- (lib) Improve download scheduling for `CDNFS::batch_read`
- (repo) Use shared workspace dependencies, relax version reqs for lib
- (bin) [gLTF](https://en.wikipedia.org/wiki/GlTF) export for mesh files

### Resources (mostly for me)
https://gitlab.com/zao/poe-rs  
https://gist.github.com/zao/96cb1869db121fbd038f8cd66f7d5736 - for .fmt/tdt/tgm  
https://gitlab.com/zao/poe-cpp/-/tree/master/libpoe/poe/format  
https://bitbucket.org/zao/gggtools/src/evolve-ggpkviewer/spec/  
https://github.com/annalithic/poeformats


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
