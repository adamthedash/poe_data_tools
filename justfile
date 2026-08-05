build:
	cargo build --release --target x86_64-pc-windows-gnu
	cargo build --release --target x86_64-unknown-linux-gnu

coverage:
	cargo bench --bench coverage

bench_cdn:
	rm -rf scratch/.cache
	time cargo bench --bench cdn

debug VERSION FILE:
	cargo run --release --features debug -- -p {{VERSION}} translate data{{VERSION}} {{FILE}}

run VERSION FILE:
	cargo run --release -- -p {{VERSION}} translate data{{VERSION}} {{FILE}}

extract VERSION FILE:
	cargo run --release -- -p {{VERSION}} extract data{{VERSION}} {{FILE}}

cat VERSION FILE:
	cargo run --release -- -p {{VERSION}} cat {{FILE}}

list VERSION FILE:
	cargo run --release -- -p {{VERSION}} list {{FILE}}

# Create a new tag & push it (here just for my own reference)
# tag TAG REVISION:
# 	jj tag set -r {{REVISION}} {{TAG}}
# 	jj git push --tag {{TAG}}

# publish:
# 	cargo publish -p poe_data_tools
# 	cargo publish -p poe_data_tools-cli

