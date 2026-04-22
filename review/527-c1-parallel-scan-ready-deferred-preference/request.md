# Review Request: Parallel Scan Ready Deferred Preference

Current head: `889f0fb`

Scope:
- `src/am/ec_hnsw/scan.rs`

Problem:
- The staged “prefer deferred over active local” path still stopped at the
  first blocked deferred row.
- That meant a best deferred row blocked by a live foreign owner could prevent
  the scan from emitting a second deferred row that was already ready and still
  better than the current active local row.
- In that case the scan would fall back to the worse active local row too
  early, even though a ready deferred row should have won on score.

What changed:
- `take_next_deferred_parallel_blocked_output(..., allow_local_emit = false)`
  now keeps scanning past blocked deferred rows instead of bailing out
  immediately.
- Blocked deferred rows are still preserved in the fallback stash and restored
  intact after the search.
- Added focused coverage showing that:
  - a blocked best deferred row does not prevent a ready next deferred row from
    outranking the active local row
  - the blocked best deferred row remains deferred afterward
  - the active local row remains intact for the next turn

Why this matters:
- It tightens score ordering inside the staged ownership contract without
  pretending the final cross-worker transfer protocol is done.
- The scan now behaves more like “emit the best deferred row that is actually
  ready” instead of “give up on deferred preference as soon as the top deferred
  row is blocked”.

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
- Whether “ready deferred row beats active local row even when a better
  deferred row is still blocked” is the right staged ordering contract
- Whether limiting the change to the non-local-emit deferred path is the right
  boundary before the remaining ownership-transfer seam lands
