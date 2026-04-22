# Review Request: Parallel Scan Foreign-Duplicate Suppression

Current head: `7332ee3`

Scope:
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The staged blocked-owner fallback could still degrade into a second local
  emit even when the foreign owner already held the exact same element.
- That left a duplicate-owner hole in the current handoff seam: same-element
  blockers were treated like any other foreign stall and could still fall back
  to local-only emit.

What changed:
- `EcParallelOwnedOutputBlocker` now carries the blocking element TID in the
  shared readiness probe.
- `read_parallel_scan_owned_output_state(...)` now fills that blocker element
  TID from:
  - the selected foreign worker slot for `ForeignSelectedPending`
  - the admitted head snapshot for `ForeignAdmittedHead`
  - `INVALID` for `AdmissionWindow`
- `blocked_parallel_scan_disposition(...)` now drops the staged local row
  immediately when the foreign blocker already owns the same element TID.
- Added focused coverage for:
  - blocker payload element TIDs in the shared readiness probe
  - scan-side duplicate-owner suppression via the blocked-owner disposition

Why this matters:
- It makes the staged ownership seam less lossy: a foreign worker that already
  owns the same row now suppresses the local duplicate instead of merely
  downgrading it into local-only fallback.
- That is a concrete step toward the real multi-worker handoff contract the
  runtime blocker is still waiting on.

Feedback processed in this slice:
- Retrospective reviewer feedback for packets `490` through `508` was reviewed
  this turn before landing the checkpoint.
- The `507` feedback explicitly called out foreign-duplicate drop as an
  important follow-up seam; this packet is that follow-up.

Still intentionally deferred:
- the real multi-worker output handoff / ownership transfer contract
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
- Whether blocker element TID is the right minimum shared payload for
  duplicate-owner suppression
- Whether dropping same-element foreign blockers at the scan disposition layer
  is the right staged boundary before the full worker/consumer handoff lands
