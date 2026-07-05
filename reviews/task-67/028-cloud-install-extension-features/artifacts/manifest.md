# Task 67 Packet 028 Manifest

- Task bucket: `reviews/task-67/`
- Packet path: `reviews/task-67/028-cloud-install-extension-features/`
- Head SHA: `567c2a8fb6c6d56fe664ef59b9e833f541b24dd8`
- Timestamp: 2026-05-30
- Scope: `ecaz cloud install` operator support for feature-gated extension builds
- Task slice support: enables Slice I bf16 host validation by allowing
  `--extension-feature rabitq-bf16`

## Code Change

`crates/ecaz-cloud/src/commands/install.rs` now accepts repeatable
`--extension-feature <FEATURE>` arguments. The remote install script appends a
single Cargo `--features '<joined features>'` argument only when at least one
extra feature is requested.

Default behavior remains unchanged when no extra feature is provided.

## Commands

```sh
cargo fmt --check
cargo test -p ecaz-cloud install_script_ --lib
```

## Artifact Inventory

- `artifacts/local/cargo-fmt-check.log`
- `artifacts/local/ecaz-cloud-install-script-tests.log`

## Key Results

- `cargo fmt --check` passed, with the repo's existing stable-rustfmt warnings.
- `cargo test -p ecaz-cloud install_script_ --lib` passed: 2 tests passed.
