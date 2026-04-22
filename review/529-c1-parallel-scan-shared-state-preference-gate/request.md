# Review Request: Parallel Scan Shared-State Preference Gate

Current head: `e096232`

Scope:
- `src/am/ec_hnsw/scan.rs`

Problem:
- The deferred-preference gate still decided using the best deferred row's
  local staged score, even when that row was blocked on a foreign owner.
- That could misfire in two directions:
  - a blocked best deferred row could cause the scan to emit a ready next
    deferred row that was actually worse than the active local row
  - a handoffable blocked deferred row had no way to participate using the
    actual shared pending/admitted score it would drain through the handoff
    path

What changed:
- Added a shared-state-aware deferred preference score helper:
  - ready deferred rows still use their local staged score
  - foreign-selected blockers consult the shared selected-pending fast path
  - foreign-admitted-head blockers consult the shared admitted-head fast path
  - obsolete / admission-window-blocked rows do not participate
- `should_prefer_deferred_parallel_blocked_output(...)` now compares the
  active local row against the best actually preferable deferred candidate,
  not simply the best deferred row by local score.
- Added focused coverage for:
  - still preferring a ready deferred row when no active local row exists
  - refusing to emit a ready-but-worse deferred row just because a better
    deferred row is blocked and cannot currently drain

Why this matters:
- It fixes a real ordering bug in the staged deferred-preference seam.
- The preference gate now tracks the score of work that can really drain
  through the current shared state, instead of inferring that from a blocked
  row's local staged score.

Still intentionally deferred:
- final cross-worker ownership transfer instead of deferred local retention
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
- Whether using the shared selected/admitted fast paths is the right staged
  contract for deferred preference gating
- Whether the updated gate fixes the “blocked best row causes worse ready row
  to preempt active local work” bug without overreaching into full ownership
  transfer
