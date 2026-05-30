# Task 67 Packet 040 Artifact Manifest

- head SHA at packet creation: `8f3b2b1b8411bd0f25bc6e20f4f5e202edfd2531`
- task bucket: `reviews/task-67/040-msrv-clippy-avx512-rabitq`
- timestamp: `2026-05-30T18:10:59Z`
- lane: RaBitQ AVX-512 / CI formatter unblocker
- fixture / storage format / rerank mode: not applicable
- isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `cargo-clippy-local.log`

- command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- result: passed locally on macOS/aarch64
- key lines cited by `request.md`: `Finished dev profile [unoptimized + debuginfo] target(s) in 39.26s`

### `git-diff-check.log`

- command: `git diff --check`
- result: passed
- key lines cited by `request.md`: no output

### `cargo-fmt-check.log`

- command: `cargo fmt --all -- --check`
- result: failed locally because local `cargo fmt` used `rustfmt 1.9.0`, which disagrees with CI's Rust 1.95 formatter on the `load.rs` wrapping
- key lines cited by `request.md`: local formatter wants multi-line wrapping, while CI job `78667910979` wanted the arguments on one line
