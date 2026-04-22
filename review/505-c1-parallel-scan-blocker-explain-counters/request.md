# Review Request: Parallel Scan Blocker EXPLAIN Counters

Current head: `bee4cdf`

Scope:
- `src/am/common/explain.rs`
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged owned-output blocker states (`ForeignSelectedPending`,
  `ForeignAdmittedHead`, `AdmissionWindow`) were visible in local scan control
  flow and, after packet 504, in shared worker-runtime snapshots.
- They still were not visible in standard EXPLAIN-facing scan diagnostics, so a
  blocked parallel-bound worker looked the same as an idle one from the staged
  EXPLAIN counter surface.

What changed:
- Extended `TqExplainCounters` with dedicated counters for:
  - foreign selected pending stalls
  - foreign admitted head stalls
  - admission-window stalls
- Extended the staged EXPLAIN counter definition and property tables to expose
  those counters under the `TQVector Stats` group.
- `try_take_parallel_scan_next_output(...)` now increments the matching
  EXPLAIN counter when owned-output state resolves to `Blocked(...)` before the
  blocker snapshot is republished.
- Added focused coverage that blocked materialized and blocked prefetched emit
  paths increment the expected counter while leaving unrelated blocker counters
  at zero.
- Updated Task 18 notes to record that blocked-owner stalls now surface in
  EXPLAIN diagnostics.

Why this matters:
- It makes the remaining ownership/handoff seam visible through the same
  operator-facing EXPLAIN diagnostics used for other staged scan counters.
- That gives the next multi-worker ownership work a sharper diagnostic signal
  without pretending the real handoff contract is already complete.

Still intentionally deferred:
- the real multi-worker output handoff / ownership contract
- planner-visible parallel execution and `amcanparallel = true`
- the LWLock-based serializer replacement before real parallel enablement
- `n = 2/4/8` correctness coverage once the real multi-worker path lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether blocked owned-output stalls belong on the staged EXPLAIN surface now,
  or should stay visible only through worker-runtime snapshots until the final
  handoff contract lands
- Whether incrementing the counters in the scan-side `Blocked(...)` branch is
  the right boundary for these diagnostics
