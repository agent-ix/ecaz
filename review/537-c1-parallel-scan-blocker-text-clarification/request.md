# Review Request: Parallel Scan Blocker Text Clarification

Current head: `200a743`

Scope:
- `src/am/ec_hnsw/shared.rs`
- `src/lib.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged Task 18 work has narrowed the remaining blocker since the older
  snapshot text was written.
- The planner/runtime snapshot and task notes still said parallel scan needed a
  generic "multi-worker output ownership contract", which overstated the
  remaining gap after the recent duplicate suppression, local-only wakeup, and
  deferred ordering slices.

What changed:
- Narrowed the shared snapshot blocker string to:
  - `parallel scan still needs a real ownership-transfer contract for genuinely blocked unique outputs before amcanparallel can turn on`
- Updated the SQL-facing planner integration test to assert the narrower text.
- Updated Task 18 notes so the current blocker matches the runtime snapshot:
  - ownership transfer is the missing piece
  - specifically for genuinely blocked unique outputs

Why this matters:
- The branch is no longer blocked by generic shared-merge scaffolding.
- The remaining engineering gap is now accurately described for reviewers and
  for the SQL/admin snapshot surface.
- That keeps follow-up work focused on the final ownership-transfer seam rather
  than reopening already-landed duplicate or local-only cases.

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
- Whether the narrower blocker wording now matches the actual remaining hazard
  on the branch
- Whether any other live Task 18/status surface still needs the same wording
  adjustment
