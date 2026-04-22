# Review Request: Parallel Scan Foreign-Head Handoff

Current head: `4f5360d`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged scan-side ownership seam still degraded too early into local-only
  fallback.
- Even when a worker was blocked only by a foreign row that was already in the
  admitted window, the scan layer still treated that as a blocker to work
  around locally instead of consuming the globally admitted row.

What changed:
- Added `try_take_parallel_scan_handoff_output(...)` on the scan side.
- When the blocker kind is `ForeignAdmittedHead`, that helper now drains the
  next globally admitted row through
  `take_parallel_scan_next_output_snapshot(...)` instead of forcing immediate
  local-only fallback.
- The handoff path clears the staged local blocker state, consumes the admitted
  row through the existing scan-side projection helper, and republishes the
  worker snapshot afterward.
- Added focused regression coverage showing that:
  - a foreign admitted head can be drained through the new handoff helper
  - draining that foreign admitted row does not advance the local worker's own
    duplicate-drain cursor
- Task 18 notes now record that already-admitted foreign rows can hand off
  through the shared merge path, while foreign selected-pending cursors are
  still intentionally deferred.

Why this matters:
- This is the first concrete scan-side handoff step that moves beyond
  diagnostics and local-only fallback.
- It narrows the remaining ownership problem to foreign selected-pending state,
  rather than treating the admitted window and the selected-pending seam as the
  same blocker.

Still intentionally deferred:
- foreign selected-pending handoff / ownership transfer
- planner-visible parallel execution and `amcanparallel = true`
- `n = 2/4/8` correctness coverage once the real multi-worker path lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether consuming an already-admitted foreign head at the scan layer is the
  right staged handoff boundary before full foreign selected-pending ownership
  transfer lands
- Whether the helper cleanly preserves the local worker's own staged cursor and
  blocker state while draining the shared admitted row
