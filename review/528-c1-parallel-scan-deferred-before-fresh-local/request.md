# Review Request: Parallel Scan Deferred Before Fresh Local Work

Current head: `ea834b8`

Scope:
- `src/am/ec_hnsw/scan.rs`

Problem:
- The staged deferred-preference seam only activated when there was already an
  active local row to compare against.
- Once the active local cursor was empty, the scan ignored ready deferred rows
  and went back to fresh graph/linear phase work first.
- That let already-scored deferred rows wait behind newly produced local work
  even though they were the only staged outputs available at that moment.

What changed:
- `should_prefer_deferred_parallel_blocked_output(...)` now prefers the best
  deferred row when no active local row is currently staged.
- Added focused coverage showing that:
  - the best deferred row is preferred when the active local cursor is empty
  - a ready deferred row drains before the scan looks for fresh local work
    when there is no active local row
- This keeps the existing staged ownership rules intact:
  - blocked deferred rows still stay deferred
  - ready deferred rows still drain through the existing deferred-take helper

Why this matters:
- It reduces another source of ordering drift without pretending the final
  cross-worker ownership-transfer seam is done.
- The scan now treats “already staged deferred work” as higher priority than
  “go search for a brand-new local row” when there is no active local row in
  hand.

Still intentionally deferred:
- final cross-worker ownership transfer instead of deferred local retention
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement after the remaining ownership seam
  lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether preferring deferred rows before fresh local work is the right staged
  ordering contract when the active local cursor is empty
- Whether limiting the change to the deferred-preference gate, without changing
  the fallback ownership rules, is the right boundary for this slice
