.PHONY: dev build test install clean verify verify-js verify-py test-ws help

export PATH := $(HOME)/.cargo/bin:$(PATH)

help:
	@echo "Available commands:"
	@echo "  make dev        - Run development server"
	@echo "  make build      - Build release binary"
	@echo "  make test       - Run Rust tests"
	@echo "  make test-ws    - Run all WebSocket tests"
	@echo "  make verify-js  - Verify WebSocket with JavaScript"
	@echo "  make verify-py  - Verify WebSocket with Python"
	@echo "  make install    - Install dependencies"
	@echo "  make clean      - Clean build artifacts"

dev:
	cargo run
	
build:
	cargo build --release

test:
	cargo test

install:
	cargo build
	npm install ws

# Run all WebSocket verification tests
test-ws: verify-js verify-py

# Verify WebSocket with JavaScript
verify-js:
	@echo "Running JavaScript WebSocket verification..."
	node tests/verify_ws.js

# Verify WebSocket with Python
verify-py:
	@echo "Running Python WebSocket verification..."
	python3 tests/verify_ws.py

# Alias for backward compatibility
verify: verify-js

clean:
	cargo clean
	rm -rf node_modules
