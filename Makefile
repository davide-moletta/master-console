.PHONY: build release run check test fmt clippy clean doc lint

build: check fmt
	cargo build

release: check lint
	cargo build --release

clean:
	cargo clean

fmt:
	cargo fmt

clippy:
	cargo clippy -- -D warnings

lint: fmt clippy

check:
	cargo check

run: build
	cargo run -- $(args)

test:
	cargo test

doc:
	cargo doc --open
