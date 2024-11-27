run:
	cargo build

install:
	cargo build --release
	cp target/release/yazi ~/.local/bin/yazi

