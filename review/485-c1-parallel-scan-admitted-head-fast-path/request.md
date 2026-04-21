# Review Request: Parallel Scan Admitted-Head Fast Path

Current head: `6431eb4`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`

Problem:
- The staged admitted-result window could already admit and consume ordered
  outputs, but later Task 18 slices still had no direct coordinator fast path
  for "what is the next admitted output right now?"
- Reading `admitted_result[0]` directly works, but it forces every later reader
  through the shared admitted array instead of the coordinator snapshot state
  that already exists for the selected worker-result path.
- The coordinator flag word was also still reusing result-slot validity names in
  its pending-output fast path, which made the admission and selection flag
  ownership harder to reason about.

What changed:
- Bumped the shared parallel DSM version for the larger coordinator state.
- Added coordinator-owned admitted-head cached fields:
  - heap TID
  - score
  - optional approx/comparison/rank metadata
- Split coordinator validity bits away from result-slot flag names:
  - pending-output validity now uses coordinator-scope constants
  - admitted-head validity uses its own coordinator-scope constants
- Added coordinator fast-path helpers for:
  - loading cached selected pending output
  - loading cached admitted head
  - storing/clearing each cached surface under the coordinator lock
- Admission refresh now updates both:
  - admitted worst-score summary
  - admitted-head fast path
- Added a direct read surface:
  - `read_parallel_scan_admitted_head_snapshot(...)`
- That reader follows the staged two-pass contract:
  - optimistic coordinator read first
  - validate against admitted slot zero
  - locked refresh and retry on mismatch
- Tightened coordinator flag stores so selection refresh preserves non-selection
  validity bits and admission refresh preserves non-admission validity bits.
- Added focused coverage for:
  - empty admitted-head reads
  - admitted-head read after two admits
  - admitted-head advance after taking the current admitted head

Still intentionally deferred:
- `amcanparallel` remains `false`
- no shared final output heap beyond the ordered admitted window
- no planner-visible parallel execution yet
- no worker-owned traversal scratch in DSM yet
- the serializer is still the staged raw lock word and still needs the later
  LWLock follow-up before `amcanparallel = true`

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 17`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether caching the admitted head in coordinator state is the right narrow
  seam before the later final-output drain slices
- Whether the new coordinator-scope flag split is the right cleanup boundary for
  pending/admitted fast-path metadata
- Whether the admitted-head reader's optimistic-read then locked-refresh retry
  matches the intended staged concurrency contract
