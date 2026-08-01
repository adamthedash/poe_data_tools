

# PoE Data Tools
Herramientas de análisis para archivos de juego de Path of Exile. Disponibles como binario independiente y como crate de Rust.  

Consulte [aquí](./crates/poe_data_tools-cli) para obtener más información sobre la CLI.  
Consulte [aquí](./crates/poe_data_tools-lib) para obtener más información sobre el crate de Rust.  
Consulte [aquí](FORMATS.md) la base de conocimiento sobre los formatos de archivos de PoE.  

### Aviso sobre IA
Todo aquí ha sido elaborado a mano con cariño, pero no descarto el uso de LLMs en el futuro. Si/cuando esto ocurra, su uso se detallará claramente aquí.

### Lista de Tareas (TODO)
- (lib) Mejor documentación / rediseño de la API pública
- (lib) API asíncrona nativa para el sistema de archivos
- (lib) Mejorar la programación de descargas para `CDNFS::batch_read`
- (repo) Usar dependencias compartidas del workspace, relajar los requisitos de versión para la librería
- (bin) Exportación [gLTF](https://en.wikipedia.org/wiki/GlTF) para archivos de mallas
- (lib) Mover `AnnotatedError` al crate annotated_parser
- (lib) MSRV
- (lib) ¿Wrappers para python/typescript?

### Recursos (principalmente para mí)
https://gitlab.com/zao/poe-rs  
https://gist.github.com/zao/96cb1869db121fbd038f8cd66f7d5736 - para .fmt/tdt/tgm  
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
