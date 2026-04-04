.PHONY: fmt fmt-check lint test build install clean

## Format all source files
fmt:
	cargo fmt --all

## Check formatting without modifying files
fmt-check:
	cargo fmt --all -- --check

## Run Clippy (deny warnings)
lint:
	cargo clippy --all-targets --all-features -- -D warnings

## Run unit tests (no Postgres required)
test:
	cargo test

## Run pgrx integration tests (requires: cargo pgrx init)
pg-test:
	cargo pgrx test

## Build release shared library
build:
	cargo build --release

## Install into local Postgres (requires sudo)
install:
	cargo pgrx install --sudo --release

## Remove build artifacts
clean:
	cargo clean
