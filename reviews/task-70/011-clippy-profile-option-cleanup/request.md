# Task 70 / Packet 011: Clippy Profile Option Cleanup

## Packet Scope

- Code commit: `1c0de8436a3d800d2d68a1166d4fe6e08691a4ba`
- Review driver: Task 70 exit criterion requiring `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Manifest: `artifacts/manifest.md`

This packet requests review for a small clippy cleanup in the packet-008 frontier profile plumbing.

## Code Change

`src/am/ec_diskann/scan.rs` now matches `Option<&mut FrontierProfile>` with `as_mut()` instead of `as_deref_mut()`. The previous form compiled but tripped `clippy::needless-option-as-deref` under the Task 70 required clippy command.

No behavior changes and no new `unsafe` were introduced.

## Validation

Commands and logs:

- `cargo fmt --check` -> `artifacts/cargo-fmt-check.log`
- `cargo test --lib --no-default-features --features pg18 am::ec_diskann::scan::tests::` -> `artifacts/cargo-test-diskann-scan.log`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` -> `artifacts/cargo-clippy-pg18.log`

The focused scan module passes 20 tests. The Task 70 clippy gate is clean.
