# Task 111h Packet 010 Artifact Manifest

- Head SHA: `fbddda0935a8640facc892a2e02a70a89178e38a`
- Task bucket: `reviews/task-111h/`
- Packet path: `reviews/task-111h/010-packed-rerank-mixed-fallback/`
- Timestamp: `2026-06-20T05:54:41Z`
- Lane / fixture / storage format / rerank mode: PG18 focused pgrx fixture;
  `storage_format = 'coarse_rerank'`; `rerank_placement = 'index'`;
  `rerank_format = 'f16'`; mixed direct/missing packed group TID debug scan.
- Surface isolation: isolated one-index-per-table SQL fixture; no shared-table
  benchmark surface.

## Artifacts

### `cargo-check-pg18-final.log`

- Command: `script -q -e -c "cargo check --no-default-features --features pg18" reviews/task-111h/010-packed-rerank-mixed-fallback/artifacts/cargo-check-pg18-final.log`
- Result: passed.
- Key lines:
  - `Checking ecaz v0.1.1 (/home/peter/dev/ecaz)`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 12.70s`
  - `COMMAND_EXIT_CODE="0"`

### `cargo-pgrx-test-pg18-mixed-fallback-pass5.log`

- Command: `script -q -e -c "cargo pgrx test pg18 test_ec_ivf_index_placement_mixed_fallback_chain" reviews/task-111h/010-packed-rerank-mixed-fallback/artifacts/cargo-pgrx-test-pg18-mixed-fallback-pass5.log`
- Result: passed.
- Key lines:
  - `test tests::pg_test_ec_ivf_index_placement_mixed_fallback_chain ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2202 filtered out; finished in 45.10s`
  - `COMMAND_EXIT_CODE="0"`
