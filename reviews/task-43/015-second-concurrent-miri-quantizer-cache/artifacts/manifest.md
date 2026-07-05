# Packet 015 — second concurrent miri_ test (quantizer cache)

## Head

- Task bucket: `reviews/task-43/`
- Packet path: `reviews/task-43/015-second-concurrent-miri-quantizer-cache/`
- Validation head SHA: `a079f1e8bc10d4137ce0e65adf3aef77a04542d1` (main)
- Branch: `main`
- Surface under validation: pure-Rust quantizer cache
  (`src/quant/prod.rs:72-127` — `OnceLock<Mutex<HashMap<_, Arc<_>>>>`
  backing `ProdQuantizer::cached`/`cached_with_presence`).
- Storage format / fixture: N/A — pure-Rust unit test, no on-disk format.
- Rerank mode / lane: N/A — concurrency safety contract, not a benchmark.
- Surface isolation: one-process, no PostgreSQL backend; single test
  process per Miri invocation.

## Test added

- `src/quant/prod.rs:2390` —
  `miri_quantizer_cache_concurrent_init_under_contention`.

## Artifacts

### miri-stacked-borrows.log

- Command:
  `PATH=$HOME/.rustup/toolchains/nightly-aarch64-apple-darwin/bin:$PATH
   RUSTUP_TOOLCHAIN=nightly cargo miri test --lib
   --no-default-features --features pg18
   -- miri_quantizer_cache_concurrent_init_under_contention`
- Timestamp: 2026-05-25
- Result:
  `test quant::prod::tests::miri_quantizer_cache_concurrent_init_under_contention ... ok`
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;
   1898 filtered out; finished in 53.17s`
- Aliasing model: default (Stacked Borrows).

### miri-tree-borrows.log

- Command: same as above with `MIRIFLAGS="-Zmiri-tree-borrows"` prepended.
- Timestamp: 2026-05-25
- Result:
  `test quant::prod::tests::miri_quantizer_cache_concurrent_init_under_contention ... ok`
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;
   1898 filtered out; finished in 53.17s`
- Aliasing model: Tree Borrows. Agrees with Stacked Borrows (no
  divergence to triage).

### miri-many-seeds-16.log

- Command: same as Stacked Borrows with
  `MIRIFLAGS="-Zmiri-many-seeds=0..16"` prepended.
- Timestamp: 2026-05-25
- Seed budget: 16 (smoke verification only; the canonical lane
  `make miri-many-seeds` uses `0..128` and picks this test up
  automatically via the `--lib -- miri_` selector).
- Result: see file. Exit code 0 indicates every seed schedule produced
  the same `1 passed; 0 failed` outcome.

### cargo-test-macos-dyld-blocker.log

- Command:
  `cargo test --lib --no-default-features --features pg18
   -- miri_quantizer_cache_concurrent_init_under_contention`
- Timestamp: 2026-05-25
- Result: compile success, runtime aborts at dyld load with
  `symbol not found in flat namespace '_BufferBlocks'`. This is the
  known deferred macOS pgrx-test blocker (memory:
  `feedback_dyld_buffer_blocks_known`); Miri is the validation surface
  for this test on macOS.

## Key result lines cited by request.md

- `1 passed; 0 failed; 0 ignored; 0 measured; 1898 filtered out` —
  appears in `miri-stacked-borrows.log` and `miri-tree-borrows.log`.
- `finished in 53.17s` — appears in both single-seed logs (matching
  the existing
  `miri_parallel_worker_slots_are_unique_under_threaded_contention`
  cost profile from packet 014).
- `EXIT=0` (with all 16 seeds reporting `1 passed`) — appears at the
  bottom of `miri-many-seeds-16.log`.
- `symbol not found in flat namespace '_BufferBlocks'` — documents
  the deferred macOS blocker in `cargo-test-macos-dyld-blocker.log`.
