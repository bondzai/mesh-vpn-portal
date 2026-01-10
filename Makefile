.PHONY: dev build test install clean verify

export PATH := $(HOME)/.cargo/bin:$(PATH)

dev:
	cargo run
	
build:
	cargo build --release

test:
	cargo test

install:
	cargo build
	npm install ws

verify:
	node verify_ws.js

clean:
	cargo clean
	rm -rf node_modules
