# Review Request: Parallel Scan Deferred Before Local-Only Wakeup

Current head: `31c46c6`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The previous slice fixed the local-only wakeup bug: a hidden local-only row
  now republishes correctly when its foreign blocker clears.
- But `produce_next_scan_heap_tid(...)` still tried that wakeup path before
  checking whether a better ready deferred row was already waiting.
- That meant a concealed local-only row could jump ahead of a better deferred
  row solely because wakeup happened earlier in the control flow.

What changed:
- Added `should_prefer_deferred_before_local_only_wakeup(...)`.
- `produce_next_scan_heap_tid(...)` now checks that helper before waking a
  concealed local-only row:
  - only when `parallel_local_only_output_active` is set
  - only when deferred preference logic says a deferred row already outranks the
    hidden local-only row
- The hidden row still wakes back into the shared seam later; this only changes
  which row gets first crack at emission on that turn.
- Added focused coverage proving:
  - a better ready deferred row outranks hidden local-only wakeup
  - the ordering check does not disturb the hidden row
  - the hidden row still wakes into the shared path afterward
- Updated Task 18 notes to record that better deferred rows now outrank hidden
  local-only wakeup.

Why this matters:
- It narrows ordering drift without pretending the final ownership-transfer
  contract is complete.
- The shared/local-only wakeup path no longer gets a free priority boost over
  already-ready deferred work.
- That keeps the remaining blocker focused on genuinely blocked unique rows,
  not on avoidable ordering inversions between two already-staged local options.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique deferred outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining ownership seam lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether the ordering rule belongs exactly at the
  `produce_next_scan_heap_tid(...)` entry point, versus inside the deferred
  helper itself
- Whether the focused helper test is sufficient proof for this ordering seam,
  given that the local-only wakeup path already has its own dedicated republish
  test coverage
