use std::path::Path;

use poe_data_tools::{
    Patch,
    fs::{
        FileSystem,
        cdn::{CDNFS, cdn_base_url},
    },
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    let (chrome_layer, _guard) = tracing_chrome::ChromeLayerBuilder::default().build();
    tracing_subscriber::registry()
        // .with(tracing_tracy::TracyLayer::default())
        .with(chrome_layer)
        .init();

    let version = Patch::One;
    let cache_dir = Path::new("../../scratch/.cache");
    let base_url = cdn_base_url(cache_dir, version.as_str()).unwrap();
    let fs = CDNFS::new(&base_url, cache_dir).unwrap();

    let files = fs.list().collect::<Vec<_>>();

    fs.batch_read(&files[..10000]).for_each(|(filename, _res)| {
        println!("{filename}");
    });
}
