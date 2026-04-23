# Review Request: Parallel Scan Selected-To-Admitted Blockers

Current head: `56768c9`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- A retained `ForeignSelectedPending` blocker was treated as stale as soon as
  the foreign row left the selected fast path, even if that same foreign row
  had simply moved into the coordinator admitted head.
- That meant hidden local-only rows and deferred blocked rows could start
  treating themselves as ready too early and drift toward local fallback even
  though the same foreign owner was still the next shared output.

What changed:
- Extended selected-blocker liveness to recognize the selected-to-admitted
  transition for the same foreign element.
- `live_foreign_blocker_heap_tid(...)` now:
  - still uses the selected fast path when slot/generation match
  - falls back to the admitted head when the blocker element is the same row
    that just moved into admitted head
- `deferred_parallel_blocked_output_preference_score(...)` now carries that
  same continuity into deferred ordering, so a selected blocker that became the
  admitted head still contributes the correct blocking score instead of looking
  stale.
- Added focused regressions proving both major scan surfaces:
  - hidden local-only wakeup keeps tracking the foreign blocker after selected
    pending becomes admitted head
  - deferred blocked drain keeps tracking the same foreign blocker through that
    transition too
- Updated Task 18 notes to record the new selected-to-admitted blocker
  continuity.

Why this matters:
- This closes a real false-ready gap in the ownership logic.
- Hidden and deferred rows now keep honoring the same foreign owner while it
  moves between coordinator fast paths, instead of prematurely degrading into
  local progress.
- It narrows the remaining ownership-transfer gap without claiming the final
  cross-worker contract is done.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo test try_take_republished_local_only_parallel_output_tracks_selected_blocker_into_admitted_head -- --nocapture`
  - `cargo test take_next_deferred_parallel_blocked_output_tracks_selected_blocker_into_admitted_head -- --nocapture`
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether selected-blocker liveness now correctly persists while the same
  foreign row transitions from selected pending to admitted head
- Whether hidden local-only and deferred blocked rows both preserve that
  continuity instead of falsely treating the blocker as stale
- Whether the new continuity logic stays narrowly scoped to the same foreign
  element rather than broadening selected blockers into unrelated admitted rows
