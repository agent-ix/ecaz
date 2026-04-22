# Review Request: Parallel Scan Blocker Generations

Current head: `e395360`

Scope:
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Packet 504 exposed blocker kind and blocker slot in shared worker snapshots,
  but a blocked worker still could not tell whether it was blocked on the same
  foreign state or whether the foreign owner had already advanced.
- That left the staged ownership seam visible but not versioned.

What changed:
- Extended `EcParallelOwnedOutputBlocker` with a blocker generation.
- Extended shared worker-slot runtime snapshots with
  `owned_output_blocker_generation`.
- Bumped the parallel descriptor version for the worker-slot layout change.
- `read_parallel_scan_owned_output_state(...)` now tags blockers with the
  relevant coordinator generation:
  - `result_publish_generation` for `ForeignSelectedPending`
  - `admitted_result_generation` for `ForeignAdmittedHead`
  - `admitted_result_generation` for `AdmissionWindow`
- Scan-side worker snapshot publication now mirrors that blocker generation.
- Added focused coverage for:
  - owned-output blocker reads carrying the expected generation
  - blocked materialized/prefetched scan-side staging publishing the generation
    into the shared worker snapshot

Why this matters:
- It turns blocker state from a static label into a versioned seam.
- The next owner-handoff slice can distinguish “same blocker, nothing changed”
  from “foreign owner advanced, retry path should re-evaluate” without having
  to infer movement indirectly from other shared fields.

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
- Whether the chosen generation source per blocker kind is the right one for
  the staged handoff seam
- Whether publishing blocker generation in the shared worker snapshot is the
  right boundary, versus keeping it local until the final ownership contract
