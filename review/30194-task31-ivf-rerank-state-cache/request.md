# Task 31 IVF Rerank State Cache

Reviewer: please review this Task 31 follow-up on top of `79c1a11c`.

## Scope

This checkpoint moves `ec_ivf` heap-f32 rerank setup into scan-owned state,
matching the existing `ec_hnsw` grouped rerank pattern.

## Change

In [`src/am/ec_ivf/scan.rs`](/Users/peter/dev/tqvector/src/am/ec_ivf/scan.rs):

- add scan-owned heap rerank state for:
  - heap relation and ownership bit
  - snapshot and ownership bit
  - tuple slot
  - indexed source attnum/kind
- configure that state once during `amrescan`
- free it during rescan teardown and endscan teardown
- reuse the cached state inside `rerank_probe_candidates_heap_f32`
- allow snapshot fallback to `RegisterSnapshot(GetLatestSnapshot())` when no
  executor or active snapshot is already attached, matching the HNSW helper

This keeps the candidate set, rerank math, and output ordering unchanged. It
only removes per-rerank setup churn.

## Validation

Executed:

```text
cargo fmt --package ecaz
cargo check --no-default-features --features pg18
```

No cargo or pgrx tests were run for this checkpoint. This slice is a local scan
state refactor with no algorithm change, and the next validation step is the
Task 31 suite rerun.

## Why This Slice

After the heap-order fetch checkpoint, the remaining obvious overhead on the IVF
heap-f32 path was repeated setup inside rerank itself. HNSW already avoids that
cost; IVF now does the same.
