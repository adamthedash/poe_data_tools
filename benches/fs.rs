use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dirs::cache_dir;
use poe_tools::{
    bundle_fs::{from_cdn, from_steam, FS},
    bundle_loader::cdn_base_url,
    steam::steam_folder_search,
};

fn fs_benchmark_steam(c: &mut Criterion) {
    let fs = from_steam(steam_folder_search("2").expect("Can't find steam folder"));
    read_some_files("steam", c, fs);
}
fn fs_benchmark_cdn(c: &mut Criterion) {
    let fs = from_cdn(&cdn_base_url("2"), cache_dir().unwrap().as_path());
    read_some_files("cdn", c, fs);
}

fn fs_load_index(c: &mut Criterion) {
    c.bench_function("load_index", |b| {
        b.iter(|| {
            let _fs = from_cdn(&cdn_base_url("2"), cache_dir().unwrap().as_path());
        });
    });
}

fn read_some_files(source: &str, c: &mut Criterion, mut fs: FS) {
    let glob = glob::Pattern::new("data/skill*.datc64").unwrap();

    let list = fs.list();
    // warm caches
    list.iter().filter(|p| glob.matches(p)).for_each(|p| {
        let _contents = fs.read(p).expect("Failed to read file");
    });

    let mut list = fs.list();
    c.bench_function(format!("read_files_{}", source).as_str(), |b| {
        b.iter(|| {
            black_box(&mut list)
                .iter()
                .filter(|p| glob.matches(p))
                .for_each(|p| {
                    let _contents = fs.read(p).expect("Failed to read file");
                });
        })
    });
}

criterion_group!(small_files, fs_benchmark_cdn, fs_benchmark_steam);
criterion_group!(
    name=index;
    config=Criterion::default().sample_size(10);
    targets=fs_load_index
);
criterion_main!(index, small_files);
