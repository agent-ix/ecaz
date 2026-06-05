# Artifact Manifest

- Head SHA: `e2b843774a65906e32198c6bfe44b6e0291dc4f4`
- Task bucket: `reviews/task-65b`
- Packet path: `reviews/task-65b/012-diskann-loader-timing`
- Timestamp: `2026-06-05T04:18:13Z`
- Lane / fixture / storage format / rerank mode: not applicable; compile and parser-validation packet only.
- Isolation: no table/index build was run; no shared-table or one-index-per-table surface was exercised.

## Artifacts

### `artifacts/cargo-fmt-check.log`

- Command: `cargo fmt --check > reviews/task-65b/012-diskann-loader-timing/artifacts/cargo-fmt-check.log 2>&1`
- Result: passed.
- Key lines: rustfmt emitted stable-channel warnings for unstable `imports_granularity` and `group_imports` settings; command exited 0.

### `artifacts/cargo-check-ecaz-pg18.log`

- Command: `cargo check -p ecaz --lib --no-default-features --features pg18 > reviews/task-65b/012-diskann-loader-timing/artifacts/cargo-check-ecaz-pg18.log 2>&1`
- Result: passed.
- Key line: `Finished dev profile [unoptimized + debuginfo] target(s) in 11.06s`.

### `artifacts/cargo-check-ecaz-cli.log`

- Command: `cargo check -p ecaz-cli > reviews/task-65b/012-diskann-loader-timing/artifacts/cargo-check-ecaz-cli.log 2>&1`
- Result: passed.
- Key lines:
  - `warning: field path is never read`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 31.27s`

### `artifacts/cargo-test-suite-parser.log`

- Command: `cargo test -p ecaz-cli parses_ec_diskann_build_timing_rows > reviews/task-65b/012-diskann-loader-timing/artifacts/cargo-test-suite-parser.log 2>&1`
- Result: passed.
- Key lines:
  - `test commands::bench::suite::tests::parses_ec_diskann_build_timing_rows ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 404 filtered out; finished in 0.00s`
