# Task 67 Review Request: Cloud install skip CLI build

## Summary

This checkpoint adds `ecaz cloud install --skip-cli-build` so the AWS install wrapper can rebuild/reinstall the PostgreSQL extension with extra extension features without also rebuilding and copying `/usr/local/bin/ecaz`.

The default install path is unchanged. Without the flag, the generated script still runs `cargo build --release -p ecaz-cli` and `sudo install -Dm755 ... /usr/local/bin/ecaz`. With the flag, it still runs `cargo pgrx install`, restarts PostgreSQL, and verifies the extension SQL state, but omits the CLI rebuild/install commands.

## Why

Slice I bf16 validation needs to flip the extension feature set on the Intel AWS host. The bf16-enabled extension build/install succeeded during packet 029 setup, but the wrapper then failed while rebuilding the CLI with `No space left on device`. The benchmark runner can use the already-installed remote CLI, so the extension-only install path unblocks the bf16-on SQL measurement without broad remote cleanup.

## Code Under Review

- `crates/ecaz-cloud/src/commands/install.rs`
- code commit: `ad1aec9f251799ddf47ff94c792e02c4dfd5b0df`

## Validation

Packet-local logs are under `artifacts/local/`; see `artifacts/manifest.md`.

- `cargo fmt --check`: passed
- `cargo test -p ecaz-cloud install_script_ --lib`: passed, 3 tests
- `cargo build -p ecaz-cli`: passed, with existing `LoadedDistributedPlacementConfig.path` warning

## Notes

No benchmark measurements are claimed in this packet. This is a support checkpoint for the pending packet 029 bf16 on/off SQL measurement.
