# Review Request: Parallel Scan Guarded Foreign-Selected Handoff

Current head: `d254eeb`

Scope:
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Foreign selected-pending handoff still went through the generic
  `take_parallel_scan_next_output_snapshot(...)` path.
- That meant a stale foreign-selected blocker could drain whatever the current
  global next output was, instead of only the specific blocked foreign slot
  that the blocker described.

What changed:
- Added
  `take_parallel_scan_foreign_selected_pending_output_snapshot(...)` in the
  shared parallel layer.
- That helper now:
  - runs under the coordinator lock
  - reaps dead roots and refreshes selection
  - only advances the selected foreign slot while both:
    - `selected_result_slot_index == blocker.slot_index`
    - `result_publish_generation == blocker.generation`
- Scan-side handoff now uses:
  - the new guarded helper for `ForeignSelectedPending`
  - direct admitted-head take for `ForeignAdmittedHead`
- Updated the foreign-selected handoff tests to source the blocker from
  `read_parallel_scan_owned_output_state(...)` instead of hard-coded
  generations.
- Added a new regression showing that a stale foreign-selected blocker returns
  `None` and leaves a newer selected foreign slot intact.
- Task 18 notes now record the guarded foreign-selected handoff seam.

Why this matters:
- It narrows the remaining handoff path to the blocker the worker actually saw,
  instead of letting stale state consume a newer unrelated row.
- This keeps the staged ownership model coherent while the broader
  worker-to-worker handoff contract is still under construction.

Still intentionally deferred:
- full multi-worker ownership transfer beyond staged foreign-head and
  foreign-selected handoff
- planner-visible parallel execution and `amcanparallel = true`
- `n = 2/4/8` correctness coverage once the full multi-worker path lands
- the LWLock release-path follow-up noted in packet 511

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether guarding foreign-selected handoff by slot plus
  `result_publish_generation` is the right staged boundary
- Whether the new stale-blocker regression is the right protection against
  draining a newer selected row through a stale handoff path
