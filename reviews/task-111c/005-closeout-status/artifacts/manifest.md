# Task 111c Packet 005 Artifact Manifest

- Task bucket: `reviews/task-111c/`
- Packet: `reviews/task-111c/005-closeout-status/`
- Head SHA: `c3ae49cd586d0451083aa743ed55d145a78465d9`
- Timestamp: 2026-06-17
- Lane: local PG18 build validation
- Scope: closeout/status update plus `ec_ivf.columnar_page_scatter` default-off
  runtime gate

## Code Artifact

### `c3ae49cd586d0451083aa743ed55d145a78465d9`

- Commit: `Task 111c: default page scatter off`
- Changed file: `src/am/ec_ivf/options.rs`
- Behavior: `ec_ivf.columnar_page_scatter` now defaults to `off` outside tests.
  The GUC remains `USERSET`, so scatter can still be enabled explicitly for
  diagnostics and equivalence work.
- Rationale: packet 004 measured the requested page-run locality lever and still
  found page scatter slower than the copy fallback on the reference TQ cell.

## Validation Artifact

### `cargo-build-pg18.log`

- Command: `script -q -c "cargo build --no-default-features --features pg18" reviews/task-111c/005-closeout-status/artifacts/cargo-build-pg18.log`
- Result: `Finished dev profile [unoptimized + debuginfo] target(s) in 1m 13s`

## Decision Evidence Reused From Prior Packets

### `reviews/task-111c/001-page-borrowed-tq-scatter/`

- Result: reviewer verified the page-scatter mechanism is true payload
  zero-copy, with borrowed payload slices tied to pinned page lifetime.
- Carry-forward gap: needed bit-exact equivalence and benchmark proof.

### `reviews/task-111c/002-page-scatter-explain-ab/`

- Result: bit-exact equivalence and counters landed.
- Benchmark signal: page scatter was slower than copy fallback despite removing
  logical payload copies.
- Reviewer gate: do not fan out scatter unless the TQ reference path beats the
  dense/copy baseline; try page-contiguous payload runs first.

### `reviews/task-111c/003-page-scatter-heap-tid-decode/`

- Result: heap-TID decode allocation removed from the scatter hot path.
- Benchmark signal: scatter improved but still trailed copy fallback.
- Reviewer carry-forward: remaining lever was page-contiguous payload access.

### `reviews/task-111c/004-page-run-payload-refs/`

- Result: page-scatter payload refs are derived by contiguous page run and
  accumulated across pages to preserve flush width.
- Focused validation:
  - `test tests::pg_test_ec_ivf_columnar_page_scatter_matches_copy_scan ... ok`
  - `1 passed; 0 failed; 2130 filtered out`
  - release build passed.
- Warmed A/B key result lines:
  - page scatter: `Approximate Scan Elapsed Us`: 30141;
    `Execution Time`: 34.536 ms;
    `Columnar Logical Bytes Copied`: 0;
    `Columnar Payload Bytes Borrowed`: 18358272;
    `Dense Coalesced Flushes`: 109.
  - copy fallback: `Approximate Scan Elapsed Us`: 18986;
    `Execution Time`: 23.199 ms;
    `Columnar Logical Bytes Copied`: 18887163;
    `Columnar Payload Bytes Borrowed`: 0;
    `Dense Coalesced Flushes`: 109.
- Reviewer feedback: `reviews/task-111c/004-page-run-payload-refs/feedback/2026-06-17-01-reviewer.md`
  says the lever is correct but exhausted, recommends stopping scatter fanout,
  keeping the GUC off/default-diagnostic, and marking 111d won't-pursue for this
  line because pre-transpose does not fix scattered-read locality.

## Closeout Decision

The Task 111c reference implementation is correct and observable, but the
promotion gate failed. The page-scatter path remains available only as an
opt-in diagnostic. The default scan path returns to the Task 111b logical-copy
fallback, and codec/ISA fanout is stopped for this access pattern.
Task 111d is marked won't-pursue for this line; future pre-transpose work must
be reopened as a fresh design that can beat the copy fallback directly.
