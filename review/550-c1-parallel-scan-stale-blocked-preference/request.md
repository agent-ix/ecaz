# Review Request: Parallel Scan Stale-Blocked Preference

Current head: `4a0738c`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The previous stale-blocker slice cleared dead retained blocker metadata at
  deferred/local-only drain time.
- But the deferred-preference gate still treated a stale-blocked deferred row
  as "blocked" until that drain path ran.
- That let worse active or hidden local-only work go first even though the
  foreign blocker was already gone.

What changed:
- `deferred_parallel_blocked_output_preference_score(...)` now treats a
  deferred row with dead retained blocker metadata as ready again and uses the
  row's own score for preference decisions.
- Added focused coverage proving:
  - a stale-blocked deferred row now outranks a worse active local row
  - a stale-blocked deferred row now outranks a worse hidden local-only wakeup
    path
- Updated Task 18 notes to record that stale-blocked rows re-enter deferred
  preference before drain time.

Why this matters:
- Dead blocker generations no longer delay a ready deferred row behind worse
  local work.
- The scan discovers stale blocker readiness at the ordering seam, not only at
  the eventual drain seam.
- This keeps the staged ordering path tighter while the final ownership-transfer
  contract for genuinely blocked unique outputs remains deferred.

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
- Whether stale-blocked deferred rows now regain ready-row ordering at the
  right seam
- Whether any remaining preference gate still leaves a dead-blocker row behind
  worse active or hidden local work until drain time
