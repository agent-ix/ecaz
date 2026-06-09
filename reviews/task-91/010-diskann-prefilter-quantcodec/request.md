# Task 91 Packet 010: DiskANN Prefilter Scoring via `QuantCodec`

## Summary

This packet migrates DiskANN prefilter scoring in `src/am/ec_diskann/quantizer.rs` onto DiskANN-local `QuantCodec` adapters:

- `DiskannBinarySidecarPrefilterCodec` routes binary-sidecar hamming prefilter scoring through `QuantCodec::score_ip_candidate`.
- `DiskannGroupedPqPrefilterCodec` routes grouped-PQ search-code scoring through `QuantCodec::score_ip_candidate`.
- `DiskannRaBitQPrefilterCodec` routes RaBitQ prefilter scoring through `QuantCodec::score_ip_candidate`.
- `DiskannPreparedPrefilter::GroupedPq` now retains `group_size`, so `QuantSearchCodecTag::GroupedPq` matches the existing IVF and HNSW tag semantics.

The public DiskANN prefilter score polarity is unchanged:

- binary sidecar still returns hamming distance directly;
- grouped-PQ and RaBitQ codecs return inner-product estimates, and `DiskannPreparedPrefilter::score` keeps the existing negation into distance scores.

## Scope Notes

- Code commit: `acf2bc0b0 Route DiskANN prefilter scoring through QuantCodec`
- This is Task 91 Phase 5 prefilter scoring migration for `DiskannPreparedPrefilter::{BinarySidecar, GroupedPq, RaBitQ}`.
- This packet does not rename `DiskannBuildCodec`; that remains a separate DiskANN storage-binding cleanup.
- This packet does not land DiskANN TurboQuant search-code support; Task 91 Phase 6 still owns the Task 90 absorption slice.
- No ISA or Graviton target behavior changes are introduced.

## Validation

See `artifacts/manifest.md`.

Commands run:

- `cargo test --lib am::ec_diskann::quantizer::tests --no-default-features --features pg18`
- `cargo test --lib am::ec_diskann::routine::tests --no-default-features --features pg18`
- `git diff --check`

Key results:

- DiskANN quantizer tests: 6 passed, 0 failed.
- DiskANN routine tests: 24 passed, 0 failed.
- `git diff --check`: passed.

## Review Ask

Please review whether DiskANN `BinarySidecar`, `GroupedPq`, and `RaBitQ` prefilter scoring now satisfy the Task 91 `QuantCodec` migration intent without changing distance-score polarity, tag semantics, or SQL-facing DiskANN scan behavior.
