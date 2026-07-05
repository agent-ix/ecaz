# Task 31 IVF Heap-Ordered Rerank Fetch

Reviewer: please review this narrow Task 31 rerank follow-up on top of
`422e5ddd`.

## Scope

This checkpoint keeps the existing candidate set and exact-score semantics, but
changes the heap-f32 rerank pass to fetch heap rows in heap-TID order before
sorting the reranked candidates back into score order.

The intent is to improve heap locality in the expensive Task 31 quality lane
(`rerank_width=1000`), where the previous probe-order work already reduced
`postings scored` but did not convert cleanly into latency improvement.

## Change

In [`src/am/ec_ivf/scan.rs`](/Users/peter/dev/tqvector/src/am/ec_ivf/scan.rs):

- add `candidate_heap_tid_cmp` to order candidates by
  `(block_number, offset_number, score)`
- sort the rerank slice by heap TID before calling
  `rerank_probe_candidates_heap_f32`
- keep the existing post-rerank `candidate_cmp` sort, so final result ordering
  and rerank-width truncation semantics stay unchanged
- add a unit test covering the heap-TID comparator

## Validation

Executed:

```text
cargo fmt --package ecaz
cargo test candidate_ --no-default-features --features pg18
```

The Rust unit coverage for the touched comparator and nearby `ec_ivf::scan`
tests passed. The command then entered the `pg_test` install path and failed
when `cargo pgrx` tried to copy into `/opt/homebrew/share/postgresql@18`, which
is the expected sandbox limitation in this environment.

## Why This Slice

The quality lane still spends time in heap-f32 rerank after the scan-phase
pruning win from `422e5ddd`. Fetching exact-source rows in approximate-score
order gives up heap locality unnecessarily. This change is the smallest way to
test whether heap access order is the remaining visible cost.
