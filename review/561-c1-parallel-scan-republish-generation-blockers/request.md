# Review Request: Parallel Scan Republish-Generation Blockers

Current head: `8b7f094`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Retained blockers still treated the same foreign owner row as stale if that
  row republished or re-admitted on the same worker slot with a newer
  generation.
- That meant hidden and deferred rows could start preferring local work too
  early even though the same foreign row was still live and still owned the
  next shared output.

What changed:
- `live_foreign_blocker_heap_tid(...)` now extends same-row continuity across
  generation churn:
  - `ForeignSelectedPending` falls back from exact generation match to
    same-slot + same-element continuity
  - `ForeignAdmittedHead` falls back from exact admitted generation match to
    same-element continuity using the admitted result snapshot
- `deferred_parallel_blocked_output_preference_score(...)` now uses that same
  republish/readmit continuity so deferred ordering keeps honoring the foreign
  row's latest score instead of treating it as stale local work.
- Added focused regressions proving:
  - a retained selected blocker keeps tracking the same foreign row after that
    row republishes with a newer generation
  - a retained admitted-head blocker keeps tracking the same foreign row after
    that row re-admits with a newer generation
- Updated Task 18 notes to record same-row generation continuity.

Why this matters:
- This closes another false-ready gap in the staged ownership model.
- Hidden and deferred rows now track the same foreign owner row through
  selected/admitted/hidden transitions and through generation churn on that
  same row.
- It narrows the remaining ownership-transfer gap without claiming the final
  cross-worker contract is complete.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo test --lib live_foreign_blocker_heap_tid_tracks_selected_blocker_across_republish_generation -- --nocapture`
  - `cargo test --lib deferred_parallel_blocked_output_preference_score_tracks_admitted_blocker_across_readmit_generation -- --nocapture`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether retained blockers now stay live for the same foreign owner row across
  selected/admitted generation churn on the same slot
- Whether deferred ordering now uses the foreign row's latest shared score
  instead of going falsely stale on republish/readmit
- Whether the new continuity remains scoped to the same owner row instead of
  broadening blockers across unrelated same-slot publications
