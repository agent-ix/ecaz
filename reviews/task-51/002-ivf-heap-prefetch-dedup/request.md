# Review Request: IVF Heap Rerank Prefetch Dedup

- task: Task 51, IVF + RaBitQ optimization round
- packet: `reviews/task-51/002-ivf-heap-prefetch-dedup`
- code commit: `6c066017d` (`Deduplicate IVF heap rerank prefetch blocks`)
- validation head: `863f8b0c8f9c6e7543e57b4b9929354a86f20f04`
- scope: local `ec_ivf` RaBitQ heap_f32 rerank only

## Summary

The heap_f32 rerank path already sorts the rerank frontier by heap TID before fetching heap rows. This change reuses that ordering to build a distinct heap-block prefetch list once, then passes only those unique blocks into PG18 `read_stream` / pre-PG18 `PrefetchBuffer`.

This does not change candidate scoring, rerank order, emitted TIDs, or EXPLAIN counter semantics. It only avoids duplicate prefetch entries when several rerank candidates live on the same heap block.

## Files Changed

- `src/am/ec_ivf/scan.rs`
  - replaced the count-only distinct heap-block scan with `candidate_heap_blocks`
  - records `Heap Blocks Fetched` from that distinct block list length
  - prefetches one entry per distinct heap block
  - added a unit helper test for adjacent sorted block collapse

## Validation

Packet-local artifact metadata is in `artifacts/manifest.md`.

- `cargo check --lib --no-default-features --features pg18`: passed
- `rustfmt --check src/am/ec_ivf/scan.rs`: passed
- `git diff --check -- src/am/ec_ivf/scan.rs`: passed
- focused Rust unit command: blocked before the test body by the existing local `BufferBlocks` symbol issue in the lib-test harness
- isolated PG18 smoke: passed with `ec_ivf` RaBitQ + `heap_f32`; key counters:
  - `Rerank Rows: 3`
  - `Heap Blocks Fetched: 1`

## Notes

No AWS was used. This is a local-only IVF/RaBitQ checkpoint.
