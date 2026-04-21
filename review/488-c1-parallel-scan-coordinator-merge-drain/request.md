# Review Request: Parallel Scan Coordinator Merge Drain

Current head: `a017d75`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`

Problem:
- The coordinator could already:
  - read the admitted head
  - probe whether the selected pending output would admit
  - fast-reject obvious losers
- But it still had no single staged helper for "what is the next output right
  now?" across those two surfaces.

What changed:
- Added `take_parallel_scan_next_output_snapshot(...)`.
- The staged merge helper now:
  - returns `None` when there is no admitted head and no selected pending work
  - drains the admitted head directly when no better selected pending output is
    present
  - admits the selected pending output first when it beats the admitted head
  - admits and drains the selected pending output when the admitted window is
    empty
- Added focused regression coverage for:
  - empty coordinator merge state
  - selected-only admit-and-drain
  - admitted-head drain before a worse selected pending output
  - better selected pending output draining before the existing admitted head
- Updated Task 18 notes to record the new staged coordinator merge seam.

Important staging note:
- This helper is still a staged merge surface, not the final planner-visible
  parallel execution path.
- It composes the existing take/admit helpers to establish ordered coordinator
  behavior before the later fully integrated coordinator/worker execution loop.

Still intentionally deferred:
- `amcanparallel` remains `false`
- no planner-visible parallel execution yet
- no final scan-side wiring into `ec_hnsw` query execution yet
- the coordinator serializer is still the staged raw lock word rather than the
  later LWLock follow-up

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 17`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether this staged merge helper is the right coordination seam before wiring
  scan-side consumption against it
- Whether the output-order tests capture the intended admitted-head vs selected
  pending precedence
- Whether the staging note is clear enough that this is not yet the final
  parallel execution contract
