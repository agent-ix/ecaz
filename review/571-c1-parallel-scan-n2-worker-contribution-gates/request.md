# Review Request: Parallel Scan N=2 Worker Contribution Gates

Current head: `08b198d`

Scope:
- `src/lib.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged PG18 `n=2` round-robin gates already asserted that the combined
  two-worker stream matched the serial stream.
- That still left a blind spot: a degenerate staged pass could preserve the
  combined serial order while effectively collapsing the fixture onto one
  worker's output stream.
- For the current branch state, that would hide exactly the kind of staged
  ownership regression these fixtures are supposed to catch.

What changed:
- Tightened both real PG18 `n=2` round-robin regressions:
  - the unique-row fixture now requires both worker streams to emit output
  - the coalesced-duplicate fixture now requires both worker streams to emit
    output too
- Kept the existing byte-identical serial-stream assertion and layered the new
  worker-contribution assertions on top.
- Updated the Task 18 notes to record that the staged `n=2` gates now reject a
  single-worker degenerate pass.

Why this matters:
- The staged `n=2` coverage is now checking ownership shape, not just final
  aggregate ordering.
- That makes the current two-worker correctness gates more sensitive to staged
  ownership collapse before the branch tackles the remaining live blocked
  unique-output transfer seam.
- It narrows the remaining uncertainty: if these fixtures pass, both workers
  are now contributing real output rather than one worker merely replaying the
  serial stream alone.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely live-blocked
  unique outputs
- planner-visible enablement and `amcanparallel = true`
- broader `n=4/8` correctness and measurement once the final contract lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether “both worker streams must contribute output” is the right staged
  ownership contract for these PG18 `n=2` fixtures
- Whether the unique-row and duplicate-drain fixtures are sufficient to reject
  a degenerate single-worker staged pass without over-constraining the final
  implementation
