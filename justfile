build:
	cargo build --release --target x86_64-pc-windows-gnu
	cargo build --release --target x86_64-unknown-linux-gnu

debug FILE:
	cargo run --release --features winnow/debug -- -p 1 translate data1 {{FILE}}

run FILE:
	cargo run --release -- -p 1 translate data1 {{FILE}}

cat FILE:
	cargo run --release -- -p 1 cat {{FILE}}

list FILE:
	cargo run --release -- -p 1 list {{FILE}}
