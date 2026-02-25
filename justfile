build:
	cargo build --release --target x86_64-pc-windows-gnu
	cargo build --release --target x86_64-unknown-linux-gnu

debug VERSION FILE:
	cargo run --release --features winnow/debug -- -p {{VERSION}} translate data{{VERSION}} {{FILE}}

run VERSION FILE:
	cargo run --release -- -p {{VERSION}} translate data{{VERSION}} {{FILE}}

extract VERSION FILE:
	cargo run --release -- -p {{VERSION}} extract data{{VERSION}} {{FILE}}

cat VERSION FILE:
	cargo run --release -- -p {{VERSION}} cat {{FILE}}

list VERSION FILE:
	cargo run --release -- -p {{VERSION}} list {{FILE}}
