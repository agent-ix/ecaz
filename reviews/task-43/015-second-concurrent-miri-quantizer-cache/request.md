# Task 43 follow-up: second concurrent miri_ test (quantizer cache)

## Scope

Closes reviewer follow-up #1 from the Task 43 closeout audit
(`reviews/task-43/014-final-campaign-audit/feedback/2026-05-18-01-reviewer.md`):

> Second real concurrent `miri_` test under many-seeds (quantizer
> `OnceLock` or another production primitive).

Adds `miri_quantizer_cache_concurrent_init_under_contention` in
`src/quant/prod.rs:2390` alongside the existing
`miri_parallel_worker_slots_are_unique_under_threaded_contention` in
`src/am/common/parallel.rs:1130`. This gives the many-seeds lane a second
real threaded surface that is independent of the parallel-scan descriptor
attachment path: it stresses the `OnceLock<Mutex<HashMap<_, Arc<_>>>>`
cache that backs `ProdQuantizer::cached`/`cached_with_presence` on the
HNSW/DiskANN encode paths.

No production code change; this is a test-only addition.

Validation head: `a079f1e8bc10d4137ce0e65adf3aef77a04542d1` (main).

## What the test asserts

- Four worker threads share the global quantizer cache via `thread::scope`.
- Every worker requests one **shared key** (`(8, 4, 0xCAFE_BABE_FACE_FEED)`)
  and one **private key** keyed off `0xA5A5_5A5A_C3C3_3C3C ^ worker_id`.
- After join:
  - All four shared-key `Arc`s satisfy `Arc::ptr_eq` (canonical init won
    the race, no duplicate `ProdQuantizer::new` survived).
  - Every private-key `Arc` is distinct from the shared one and from every
    other private `Arc` (`Arc::ptr_eq` returns false).
- The distinctive seed namespaces (`0xCAFE_BABE_FACE_FEED` and
  `0xA5A5_5A5A_C3C3_3C3C` prefix) deliberately avoid collision with other
  in-process tests that may also populate the global cache.

## Why this surface

`ProdQuantizer::cached` is on the hot quantizer-build path used by
HNSW/DiskANN encode/recall. Its safety contract has three layered
primitives that interact only under threaded contention:

1. `OnceLock::get_or_init` — must publish the inner `Mutex` exactly once
   regardless of how many threads race the first call.
2. `Mutex::lock` — must serialize access to the `HashMap`.
3. `HashMap::entry(key).or_insert_with(...).clone()` — must produce a
   single canonical `Arc` per key even when multiple threads race the
   same key.

The closeout audit explicitly called this out as one of two recommended
candidates for a second concurrent surface; the other was a SPIRE
coordinator worker primitive, which still depends on the SPIRE
careful-extraction work tracked under campaign-tracker G6.

## Evidence

- `artifacts/miri-stacked-borrows.log`: `cargo miri test --lib ...` under
  default Stacked Borrows. 1 passed, finished in 53.17s.
- `artifacts/miri-tree-borrows.log`: `MIRIFLAGS=-Zmiri-tree-borrows
  cargo miri test --lib ...`. 1 passed.
- `artifacts/miri-many-seeds.log`: `MIRIFLAGS=-Zmiri-many-seeds=0..16
  cargo miri test --lib ...`. Smoke-budget seed sweep; the full
  `0..128` lane will pick this test up automatically through the
  `--lib -- miri_` selector once `make miri-many-seeds` next runs.
- `artifacts/cargo-test-macos-dyld-blocker.log`: `cargo test --lib ...`
  documents the known macOS `_BufferBlocks` dyld blocker (see memory
  `feedback_dyld_buffer_blocks_known`); compile succeeds, runtime is
  validated via Miri instead.
- `artifacts/manifest.md`: command metadata and key result lines.

## Reviewer focus

- The test passes under both Stacked Borrows and Tree Borrows, matching
  the campaign's two-aliasing-model invariant from packet 005/014.
- The many-seeds run picks up the new test through the existing
  `-- miri_` prefix selector — no Makefile changes required.
- Distinctive seed namespaces are sufficient to avoid collisions with
  other tests sharing the global cache.
- Test name follows the existing `miri_<surface>_<contract>` convention
  in `src/quant/prod.rs` and `src/am/common/parallel.rs`.

## Out of scope

- SPIRE careful micro-harness (reviewer follow-up #2): still blocked on
  the path-lift extraction tracked under campaign-tracker G6.
- Property-based fuzz layer over SPIRE serialization rejection paths
  (reviewer follow-up #3): scheduled under Task 46 work.
- Adding new typed-view miri tests for the dsm.rs / buffer_guard.rs
  wrappers landed by Tasks 52/58.1/59: those wrappers call into
  `pg_sys::*` primitives and are not pure-Rust Miri-able without the
  same kind of path-lift extraction the SPIRE careful blockers describe.
