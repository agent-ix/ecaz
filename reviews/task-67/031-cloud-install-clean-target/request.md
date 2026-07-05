# Task 67 Review Request: Cloud install clean target option

## Summary

This checkpoint adds `ecaz cloud install --clean-cargo-target`, which runs `cargo clean` in the remote `/var/lib/pgsql/build/ecaz` checkout before `cargo pgrx install`.

The option is off by default. It is intended for benchmark hosts whose retained Cargo build cache has filled the disk and blocked an extension rebuild. The generated script cleans before the pgrx extension build and leaves the normal default install path unchanged.

## Why

Packet 029's bf16-on install retried with `--skip-cli-build`, but the remote host still failed during the bf16 extension compile:

`rustc-LLVM ERROR: IO failure on output stream: No space left on device`

The failed command was still inside `cargo pgrx install --features rabitq-bf16`, so the remaining blocker is stale remote build artifacts, not the CLI rebuild. This option provides a scoped cleanup path for build artifacts before retrying the extension install.

## Code Under Review

- `crates/ecaz-cloud/src/commands/install.rs`
- code commit: `25f2b4754ac0b4a88365f30dddc530058165feb2`

## Validation

Packet-local logs are under `artifacts/local/`; see `artifacts/manifest.md`.

- `cargo fmt --check`: passed
- `cargo test -p ecaz-cloud install_script_ --lib`: passed, 4 tests
- `cargo build -p ecaz-cli`: passed, with existing `LoadedDistributedPlacementConfig.path` warning

## Notes

No benchmark measurements are claimed in this packet. This is a support checkpoint for the pending packet 029 bf16 on/off SQL measurement.
