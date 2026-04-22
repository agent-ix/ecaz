# Review Request: Parallel Scan Deferred Drain Coverage

Current head: `938911e`

Scope:
- `src/am/ec_hnsw/scan.rs`

Problem:
- The deferred obsolete-drop slice already had focused predicate coverage, but
  it did not yet pin the actual end-to-end deferred drain behavior.
- That left room for a future refactor to keep the predicate green while still
  accidentally reintroducing local emit in the real drain path.

What changed:
- Added end-to-end tests for `emit_next_deferred_parallel_blocked_output(...)`:
  - when the best deferred row is obsolete, drain skips it and emits the next
    eligible deferred row instead
  - when the only deferred row is obsolete, drain returns `false` and emits
    nothing
- The tests also verify the two key scan-side invariants:
  - obsolete dropped rows disappear from `staged_or_emitted_contains_element`
  - `xs_heaptid` only advances when a real eligible deferred row is emitted

Why this matters:
- It locks the deferred drain contract to the real scan output path instead of
  only testing the helper predicate in isolation.
- That makes the next ownership/handoff refactors safer, because a regression
  back into stale local emit will now fail in the real drain seam.

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
- Whether these tests pin the right observable drain-path behavior for obsolete
  deferred rows
- Whether the asserted scan-side invariants are the right ones to preserve
  while the remaining ownership-transfer seam is still in flight
