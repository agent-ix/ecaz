.PHONY: fmt fmt-check lint test pg-test deny audit-unsafe build install clean

## Format all source files
fmt:
	cargo fmt --all

## Check formatting without modifying files
fmt-check:
	cargo fmt --all -- --check

## Run Clippy (deny warnings)
lint:
	cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings

## Run unit tests (no Postgres required)
test:
	cargo test

## Run pgrx integration tests (requires: cargo pgrx init)
pg-test:
	cargo pgrx test

## Check dependency licenses
deny:
	cargo deny check licenses

## Verify all unsafe blocks have nearby SAFETY comments
audit-unsafe:
	bash scripts/check_unsafe_comments.sh

## Build release shared library
build:
	cargo build --release

## Install into local Postgres (requires sudo)
install:
	cargo pgrx install --sudo --release

## Remove build artifacts
clean:
	cargo clean
