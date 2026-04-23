# Review Request: Parallel Scan Local-only Obsolete Drop

Current head: `0631e6a`

Scope:
- `src/am/ec_hnsw/scan.rs`

Problem:
- Hidden local-only rows already had duplicate suppression and wakeup retry
  behavior, but they were still missing the same obsolete-row guard that the
  active and deferred blocked paths already enforce.
- If a foreign owner already held the same element, the hidden local-only row
  could wake back up and keep carrying stale local state instead of dropping
  entirely as obsolete.

What changed:
- `resolve_local_only_parallel_scan_duplicate(...)` now checks whether the
  retained foreign blocker already owns the same element as the hidden
  local-only current row.
- When that happens, it clears the entire local result state, clears the
  local-only fallback flags and retained blocker, republishes the worker slot
  snapshot, and returns `Exhausted`.
- Added focused regression coverage proving that the obsolete hidden row drops
  fully, including clearing pending heap TIDs instead of only clearing the
  current element.

Why this matters:
- Hidden local-only rows now follow the same obsolete-row rule as the active
  and deferred blocked paths.
- This narrows one more local-fallback corner case before the remaining
  cross-worker ownership-transfer contract lands.

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
- Whether this is the right place to treat a hidden local-only row as obsolete
  once a foreign owner already holds the same element
- Whether any other hidden local-only wakeup path should use the same full-row
  drop rule
