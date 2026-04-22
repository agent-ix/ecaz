# Review Request: Parallel Scan Staged Vs Emitted Ownership

Current head: `314f2b6`

Scope:
- `src/am/ec_hnsw/scan.rs`

Problem:
- The staged parallel scan seam still marked some candidates as "emitted" too
  early, before a heap TID had actually been returned to Postgres.
- That was especially wrong on blocked-owner paths: a foreign worker could stay
  ahead, the local candidate would stage but not emit, and the scan code could
  still blacklist that element as already emitted.
- The result was a correctness risk for the remaining ownership/handoff work:
  blocked staged rows and truly returned rows were not separated cleanly.

What changed:
- Added `staged_or_emitted_contains_element(...)` so graph/linear candidate
  selection treats current staged scan results as live without conflating them
  with actually returned rows.
- Moved `mark_emitted_element(...)` to actual emit boundaries:
  - shared admitted-result consume
  - graph prefetched emit
  - linear pending/materialized emit
- Removed early emit marking from the earlier materialization/staging seams.
- Updated focused scan tests to assert:
  - successful shared-merge emits do mark the element emitted
  - blocked staging paths do not mark the element emitted
- Fixed the direct helper tests to initialize the emitted-result set the same
  way real scans do.

Why this matters:
- It separates "currently staged by this worker" from "already returned to the
  executor," which is the correct ownership boundary for the remaining Task 18
  handoff work.
- That keeps blocked-owner staging from poisoning duplicate filtering too early.

Still intentionally deferred:
- the actual multi-worker output handoff / ownership contract
- planner-visible parallel execution and `amcanparallel = true`
- `n = 2/4/8` correctness coverage once the real multi-worker path lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the staged-vs-emitted split now sits at the right boundary for scan
  ownership
- Whether any scan path still marks an element emitted before a heap TID is
  actually returned
