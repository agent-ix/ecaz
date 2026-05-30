# Task 67 Packet 014 Artifact Manifest

- Head SHA: `59e899a41205825ef44f0925bea06701bc3353f3`
- Task bucket: `reviews/task-67/`
- Packet path: `reviews/task-67/014-rabitq-runtime-test-lane/`
- Timestamp: `2026-05-30T02:32:30Z`
- Lane: local RaBitQ runtime test lane restoration
- Fixture: Rust unit-test runtime on local Intel x86_64 AVX2+FMA host
- Storage format: not applicable
- Rerank mode: not applicable
- Surface isolation: not applicable; no benchmark or SQL surface was run

## Artifacts

### `validation.log`

- Command: `cargo fmt`
- Command: `cargo test -p ecaz task67_sum_query_dequant_for_test_scaffold_matches_scalar_when_available -- --nocapture`
- Command: `cargo test -p ecaz quant::rabitq -- --nocapture`
- Command: `git diff --check`
- Result: all passed.
- Key lines cited by `request.md`:
  - `test quant::rabitq::tests::task67_sum_query_dequant_for_test_scaffold_matches_scalar_when_available ... ok`
  - `test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 1888 filtered out; finished in 0.17s`

## Limitations

- The host does not expose AVX-512, so this packet does not satisfy AVX-512,
  AVX-512 BF16, recall, benchmark, or Slice J throughput gates.
- The standalone PostgreSQL stubs are loader/runtime support for plain Rust
  tests only. Backend-only symbols still panic if those paths are invoked
  outside PostgreSQL.
