# Review Request: C1 Task17 Shared AM Module Split

Current head at execution: `672a0b5`

## Context

This checkpoint folds coder-2's shared task17 refactor branch into the native
HNSW lane and then lands the ADR-041 stage-2/3 structure needed for a second
access method.

The goal is not just to reshuffle files. The tree now has explicit homes for:

- shared AM helpers under `src/am/common/`
- tqhnsw-specific implementation under `src/am/tqhnsw/`
- cross-AM storage primitives under `src/storage/`
- shared build/training primitives under `src/am/common/training.rs`

This is the shape DiskANN will build on for its own `ambuild` / `amscan`
plumbing.

## What changed

### 1. Merged the shared task17 seam branch

Pulled in coder-2's four shared commits:

1. `3e43131` Quantizer + `QueryScorer` traits
2. `bc5afc3` `PqFastScanQuantizer`
3. `71d523a` grouped-PQ LUT scoring routed through `QueryScorer`
4. `6423425` `storage::page` + `storage::wal`

The native-build lane now carries those shared seams instead of diverging from
them.

### 2. Split `src/am/` into `common/` and `tqhnsw/`

- moved planner/explain/stats/stream helpers into `src/am/common/`
- moved tqhnsw build/scan/insert/vacuum/graph/page/options/routine/shared code
  into `src/am/tqhnsw/`
- kept the crate-level `am::*` surface stable through re-exports so existing
  callers and SQL-facing tests did not need broad path churn

This is the module boundary ADR-041 calls for before a sibling DiskANN AM
lands.

### 3. Lifted shared training primitives out of `build.rs`

Added `src/am/common/training.rs` with reusable helpers for:

- SRHT forward transform application
- grouped-PQ4 codebook training
- grouped-PQ4 source-code derivation
- persisted binary-sidecar derivation / word-count calculation

`tqhnsw::build` now uses that shared module through narrow wrappers, so the
native build path stays stable while `tqdiskann` can call the same primitives.

### 4. Floated quantizer-family naming up to `crate::quant`

`storage_format` now resolves through `crate::quant::Family` rather than a
tqhnsw-only enum definition. That keeps reloption naming aligned with the
cross-AM quantizer surface DiskANN will also need.

## Validation

Green validation for this checkpoint:

```bash
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

## Review focus

1. Are the new boundaries in `src/am/common/` vs `src/am/tqhnsw/` the right
   split for a sibling `tqdiskann` module, or is there still tqhnsw-specific
   code stranded in `common/` or vice versa?
2. Is `src/am/common/training.rs` the right shared API for DiskANN build
   consumers, especially the SRHT / grouped-PQ4 / binary-sidecar helpers?
3. Is the `crate::quant::Family` move the right stopping point for this
   checkpoint, or do you want additional cross-AM reloption cleanup before
   DiskANN starts wiring on top?
