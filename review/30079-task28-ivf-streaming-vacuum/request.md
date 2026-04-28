# Task 28 IVF Streaming Vacuum

## Scope

This packet records the A2 code checkpoint for replacing the full-list materialization in IVF vacuum.

Code checkpoint: `7b4e23281acdc9a6a38402a0d45ced2b4e7ca8b9` (`ivf: stream vacuum posting rewrites`).

## What Changed

- Added `page::rewrite_ivf_postings_for_list_blocks`, which walks one posting-list block at a time under an exclusive buffer lock.
- The page visitor decodes one posting tuple at a time, calls the vacuum rewrite callback, and copies the rewritten tuple back into the same slot when needed.
- Changed `vacuum.rs` so `bulkdelete_list_postings` no longer calls `read_ivf_postings_for_list_blocks_with_tids` and no longer materializes an entire list into `Vec<(tid, posting)>`.
- The live/dead count repair behavior is preserved, including empty-list head/tail repair.

## Validation

Focused PG18 validation:

- `cargo fmt --check`
- `cargo test --lib am::ec_ivf::page::tests --no-default-features --features pg18`
- `cargo test --lib am::ec_ivf --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_ivf_vacuum`
- `git diff --check`

## Status

The A2 code path is landed and tested for correctness on focused PG18 regressions.

The required 1M-row wall-time and peak-memory measurement for `nlists in {8,32,64}` is not included in this packet yet. Treat this packet as the code checkpoint; a follow-up measurement packet is still required before marking A2 fully closed under the task-file wording.

No DiskANN work is included.
