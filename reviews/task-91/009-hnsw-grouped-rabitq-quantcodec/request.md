# Task 91 Packet 009: HNSW Grouped-PQ/RaBitQ Search Scoring via `QuantCodec`

## Summary

This packet migrates the remaining HNSW grouped approximate search-code scoring helpers in `src/am/ec_hnsw/scan.rs` onto local `QuantCodec` adapters:

- `HnswGroupedPqScanCodec` routes PqFastScan grouped-PQ LUT scoring through `QuantCodec::score_ip_candidate`.
- `HnswRaBitQScanCodec` routes HNSW RaBitQ search-code scoring through `QuantCodec::score_ip_candidate`.
- `PreparedGroupedScanQuery` now retains the persisted grouped-PQ `group_size`, so its `QuantSearchCodecTag::GroupedPq` matches the existing IVF tag semantics.

The public traversal polarity is unchanged: the codecs return inner product scores, and the existing HNSW helper layer still negates them into distance scores.

## Scope Notes

- Code commit: `7727b882b Route HNSW grouped search scoring through QuantCodec`
- This is Task 91 Phase 4 follow-through after packet 008's TurboQuant adapter.
- This packet does not change storage layout, insert/build encoding, or binary traversal fallback scoring.
- Graviton 4 targeting is unaffected; no ISA or SVE-width assumption is introduced here.

## Validation

See `artifacts/manifest.md`.

Commands run:

- `cargo test --lib am::ec_hnsw::scan::tests::hnsw_ --no-default-features --features pg18`
- `cargo test --lib am::ec_hnsw::scan::tests --no-default-features --features pg18`
- `git diff --check`

Key results:

- Focused codec filter: 3 passed, 0 failed.
- Full HNSW scan test filter: 77 passed, 0 failed.
- `git diff --check`: passed.

## Review Ask

Please review whether the HNSW grouped-PQ and RaBitQ approximate search-code scoring paths now satisfy the Task 91 `QuantCodec` migration intent without changing traversal score polarity or grouped-PQ tag semantics.
