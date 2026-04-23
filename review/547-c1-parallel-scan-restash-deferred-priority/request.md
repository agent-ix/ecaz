# Review Request: Parallel Scan Restash Deferred Priority

Current head: `02b8c26`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The previous slice restashed still-blocked hidden local-only rows back into
  the deferred blocked-output stash.
- But after that restash, both graph and linear wakeup paths immediately
  returned to fresh local work.
- That meant a better ready deferred row could still wait an extra full turn
  even though the hidden blocked row had already been moved out of the way.

What changed:
- After `restash_local_only_parallel_blocked_output(...)`, both:
  - graph traversal wakeup
  - linear fallback wakeup
  now re-check `try_emit_preferred_deferred_parallel_blocked_output(...)`
  before continuing with fresh local work.
- Added focused coverage proving:
  - a blocked hidden local-only row can restash
  - a better ready deferred row can then emit first
  - the restashed blocked row stays deferred with blocker metadata intact
- Updated Task 18 notes to record that restash can hand priority back to
  deferred work immediately.

Why this matters:
- Restash no longer just preserves the blocked row for later.
- It also reopens the staged deferred-priority seam immediately, which keeps
  ordering closer to the shared path instead of forcing an extra round-trip
  through fresh local work.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the wakeup branches should immediately reopen deferred priority after
  restash, or whether any remaining wakeup branch still skips that opportunity
- Whether the new focused test captures the intended ordering handoff clearly
