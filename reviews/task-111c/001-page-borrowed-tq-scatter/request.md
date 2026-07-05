# Task 111c Review Request: Page-Borrowed TQ Columnar Scoring

## Summary

This checkpoint starts Task 111c by routing columnar frozen-list TurboQuant scans through page-borrowed payload slices instead of rebuilding a full logical frozen-list byte vector and copying payloads into a contiguous scratch slab.

Code commit under review:

- `11b145d2d3af51106dae03b34c0dac7cccc5d8d8` - `Task 111c: score columnar TQ from pinned pages`

## Changes

- Added `ec_ivf.columnar_page_scatter`, default on, as the Task 111c switch. Disabling it forces the Task 111b logical-copy fallback.
- Added `IvfColumnarFrozenListPinnedPages`, which holds raw column pages locked/pinned and maps logical column offsets to page-local special-area slices.
- Added explicit single-page range validation so borrowed payload and metadata accesses fail if they would cross a raw-page boundary.
- Added a borrowed columnar scan scratch for TQ: gammas and heap TIDs are copied into scan scratch, but payloads are `&[u8]` slices borrowed from pinned raw pages.
- Added `score_turboquant_batch_from_payload_refs`, which builds the existing TQ `CandidateBatch` from borrowed payload references and still uses the current block-kernel path.

## Scope Notes

This is a narrow 111c checkpoint, not the full task:

- Implemented: TurboQuant columnar frozen-list payload borrowing.
- Still fallback: non-TQ codecs, unsupported prepared-query/profile combinations, and `SET ec_ivf.columnar_page_scatter = off`.
- Still copied: small metadata and heap TIDs into scan scratch.
- Not yet done: benchmark packet proving `Columnar Logical Bytes Copied` drops to zero on the TQ columnar path, all-codec scatter coverage, and Task 111d pretransposed canonical geometry.

## Validation

Artifact manifest:

- `reviews/task-111c/001-page-borrowed-tq-scatter/artifacts/manifest.md`

Focused test:

```sh
script -q -c "cargo test columnar_single_page_range --no-default-features --features pg18" reviews/task-111c/001-page-borrowed-tq-scatter/artifacts/cargo-test-columnar-single-page-range.log
```

Result:

```text
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2128 filtered out; finished in 0.00s
```

The test command also compiled the PG18-feature scan code touched by this checkpoint.
