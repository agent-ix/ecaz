# Task 31 IVF Rerank Loop Cleanup

Reviewer: please review this tiny Task 31 follow-up on top of `c1a761fd`.

## Scope

This checkpoint removes redundant per-row work in the IVF heap-f32 rerank loop.

## Change

In [`src/am/ec_ivf/scan.rs`](/Users/peter/dev/tqvector/src/am/ec_ivf/scan.rs):

- hoist the scan query slice once before the rerank row loop
- stop calling `ExecClearTuple` after each rerank row, since the next
  `fetch_heap_row_version` already clears the slot before reuse

This keeps the same candidate set, heap fetch order, exact scoring math, and
output ordering.

## Validation

Executed:

```text
cargo fmt --package ecaz
cargo check --no-default-features --features pg18
```

No cargo or pgrx tests were run for this checkpoint. The change is local to the
rerank loop and the next validation step is a quality-lane suite rerun.
