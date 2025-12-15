use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dirs::cache_dir;
use poe_data_tools::{bundle_fs::FS, bundle_loader::cdn_base_url, steam::steam_folder_search};

fn fs_benchmark_steam(c: &mut Criterion) {
    read_some_files("steam", c, steam_fs(), "data/skill*.datc64");
}
fn fs_benchmark_cdn_dats(c: &mut Criterion) {
    read_some_files("cdn_dats", c, cdn_fs(), "data/skill*.datc64");
}

fn fs_benchmark_cdn_small_art(c: &mut Criterion) {
    read_some_files("cdn_small_art", c, cdn_fs(), "minimap/**/*.dds");
}

fn fs_benchmark_cdn_large_art(c: &mut Criterion) {
    read_some_files(
        "cdn_large_art",
        c,
        cdn_fs(),
        "art/textures/interface/2d/2dart/uiimages/login/4k/**/*.dds",
    );
}

fn fs_load_index(c: &mut Criterion) {
    let cache_dir = cache_dir().unwrap().join("poe_data_tools");
    let base_url = cdn_base_url(&cache_dir, "2").expect("Failed to get base url");
    c.bench_function("load_index", |b| {
        b.iter(|| {
            let _fs = FS::from_cdn(&base_url, &cache_dir);
        });
    });
}

fn steam_fs() -> FS {
    FS::from_steam(steam_folder_search("2").expect("Can't find steam folder"))
        .expect("Failed to load file system")
}

fn cdn_fs() -> FS {
    let cache_dir = cache_dir().unwrap().join("poe_data_tools");
    let base_url = cdn_base_url(&cache_dir, "2").expect("Failed to get base url");
    FS::from_cdn(&base_url, &cache_dir).expect("Failed to load file system")
}

fn read_some_files(source: &str, c: &mut Criterion, fs: FS, pattern: &str) {
    let glob = glob::Pattern::new(pattern).unwrap();

    let list: Vec<String> = fs.list().collect();
    // warm caches
    list.iter().filter(|p| glob.matches(p)).for_each(|p| {
        let _contents = fs.read(p).expect("Failed to read file");
    });

    c.bench_function(format!("read_files_{}", source).as_str(), |b| {
        b.iter(|| {
            black_box(&list)
                .iter()
                .filter(|p| glob.matches(p))
                .for_each(|p| {
                    let _contents = fs.read(p).expect("Failed to read file");
                });
        })
    });
}

criterion_group!(
    name=large_files;
    config=Criterion::default().sample_size(10);
    targets=fs_benchmark_cdn_large_art, fs_benchmark_cdn_small_art
);
criterion_group!(small_files, fs_benchmark_cdn_dats, fs_benchmark_steam);
criterion_group!(
    name=index;
    config=Criterion::default().sample_size(10);
    targets=fs_load_index
);
criterion_main!(index, small_files, large_files);
