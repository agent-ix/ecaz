# Review Request: Parallel Scan Runtime Blocker Clarification

Current head: `0f0c026`

Scope:
- `src/am/ec_hnsw/shared.rs`
- `src/lib.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The readiness snapshot still claimed there was no merged runtime blocker left
  on main.
- That is no longer true for Task 18. `n=1` parity is live, but the remaining
  blocker is the multi-worker output ownership contract at the scan layer.
- Leaving the stale message in place makes `ec_hnsw_planner_integration_snapshot(...)`
  actively misleading while `amcanparallel` is still intentionally `false`.

What changed:
- Updated `next_runtime_blocker` in the planner integration snapshot to say the
  real blocker:
  `parallel scan still needs a real multi-worker output ownership contract before amcanparallel can turn on`
- Updated the PG test expectation for that snapshot text.
- Added the same blocker note to Task 18 so the task file and runtime snapshot
  say the same thing.

Why this matters:
- This keeps the runtime diagnostics honest while the branch stays in staged
  shared-infra mode.
- It also records the concrete next design seam before the planner-visible
  `amcanparallel` flip: workers currently do not have a final output-ownership
  contract for the multi-worker path.

Still intentionally deferred:
- the actual multi-worker ownership implementation
- planner-visible parallel costing and `amcanparallel = true`
- correctness harness for `n = 2/4/8`

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the runtime blocker wording is precise enough for the current Task 18
  state
- Whether the task note and snapshot text are aligned on the actual remaining
  blocker before `amcanparallel` can turn on
