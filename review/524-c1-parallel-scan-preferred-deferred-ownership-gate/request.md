# Review Request: Parallel Scan Preferred Deferred Ownership Gate

Current head: `e3bef58`

Scope:
- `src/am/ec_hnsw/scan.rs`

Problem:
- The prior deferred-row score-preference seam would take a better deferred row
  before a worse active local row.
- But if that better deferred row was still blocked by a live foreign owner,
  the scan would still fall through to local emit too early.

What changed:
- Added an explicit ownership gate to the deferred take core:
  - `take_next_deferred_parallel_blocked_output(..., allow_local_emit)`
- The normal phase-exhaustion path still allows local emit when a deferred row
  remains blocked after the shared retry.
- The new preferred-deferred path does not.
- When a better deferred row is still blocked by a live foreign owner, it now
  stays in the deferred stash and returns `None` instead of locally emitting
  early.
- Added focused coverage for:
  - preferring a better deferred row when it is ready
  - keeping a better deferred row stashed when its foreign blocker is still
    live

Why this matters:
- It preserves the score-order improvement from the previous slice without
  weakening the ownership contract.
- A deferred row only overtakes the active local row when it can actually be
  drained safely.

Still intentionally deferred:
- full cross-worker ownership transfer instead of scan-local deferred fallback
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
- Whether the new `allow_local_emit` split is the right staged boundary between
  “phase exhaustion” and “preferred deferred replay”
- Whether keeping a still-blocked better deferred row stashed is the correct
  ownership-preserving behavior before the final handoff contract lands
