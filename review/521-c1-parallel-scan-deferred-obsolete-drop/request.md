# Review Request: Parallel Scan Deferred Obsolete Drop

Current head: `54c4882`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Deferred blocked rows already got one last shared-handoff retry before local
  emit.
- But after that retry, the deferred drain path still locally emitted rows that
  were already known to be obsolete:
  - admission-window losers
  - same-element foreign duplicates

What changed:
- Added a narrow deferred-row drop guard in the scan path.
- Deferred drain now drops rows instead of locally emitting them when the
  retained blocker proves the row is obsolete after its final shared retry:
  - `AdmissionWindow` blockers
  - foreign blockers whose published `element_tid` matches the deferred row
- The deferred drain republishes worker snapshot state before continuing so the
  shared blocker view stays aligned after a dropped deferred row.
- Added focused unit coverage for:
  - admission-window deferred drops
  - same-element foreign deferred drops
  - distinct foreign blockers still staying eligible for local fallback
- Updated Task 18 notes to describe the deferred obsolete-drop guard.

Why this matters:
- It closes a correctness hole where deferred rows could still bypass the
  ownership seam on the very last drain.
- The deferred stash now behaves more like a true ownership backstop instead of
  a late local-emit escape hatch for rows the shared state already proved should
  not win.

Still intentionally deferred:
- full cross-worker ownership transfer instead of scan-local deferred fallback
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
- Whether dropping deferred admission losers and same-element foreign duplicates
  is the right terminal behavior after the last shared retry
- Whether the retained-blocker-based drop guard is the right narrow seam before
  the final ownership-transfer contract lands
