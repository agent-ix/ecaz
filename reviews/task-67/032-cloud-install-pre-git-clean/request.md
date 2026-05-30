# Task 67 Review Request: Clean target before git reset

## Summary

This checkpoint adjusts `ecaz cloud install --clean-cargo-target` so the remote `cargo clean` runs immediately after `cd /var/lib/pgsql/build/ecaz`, before `git reset --hard`.

## Why

Packet 029's bf16-on retry with `--clean-cargo-target` failed before reaching `cargo clean` because the host was already too full for git to write `.git/index.lock`:

`fatal: sha1 file '/var/lib/pgsql/build/ecaz/.git/index.lock' write error. Out of diskspace`

Cleaning the Cargo target before any git work frees space for the reset/fetch/checkout sequence and then for the extension rebuild.

## Code Under Review

- `crates/ecaz-cloud/src/commands/install.rs`
- code commit: `650fb11c83a26812fa58884911cfbf45dc3dc7cc`

## Validation

Packet-local logs are under `artifacts/local/`; see `artifacts/manifest.md`.

- `cargo fmt --check`: passed
- `cargo test -p ecaz-cloud install_script_ --lib`: passed, 4 tests
- `cargo build -p ecaz-cli`: passed, with existing `LoadedDistributedPlacementConfig.path` warning

## Notes

No benchmark measurements are claimed in this packet. This is a support checkpoint for the pending packet 029 bf16 on/off SQL measurement.
