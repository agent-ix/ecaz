# Review Request: Parallel Scan Admission Probe

Current head: `25e9273`

Scope:
- `plan/tasks/18-parallel-index-scan.md`
- `src/am/common/parallel.rs`

Problem:
- The coordinator can already expose the selected pending output and maintain
  the admitted window, but workers still had no narrow read-only seam for
  "would this selected pending output actually enter the admitted window?"
- Without that probe, the next admission path has to take the coordinator
  serializer just to discover obvious rejects like duplicate heap TIDs or
  candidates that do not beat the full-window tail.

What changed:
- Added `EcParallelCoordinatorAdmissionProbe` as a claim-safe snapshot for:
  - coordinator selection state
  - selected staged result slot state
  - selected pending output
  - current admitted-window summary
  - `would_admit`
- Added
  `read_parallel_scan_selected_pending_output_admission_probe(...)`.
- The probe follows the staged optimistic-read contract:
  - read the selected pending output through the existing fast path
  - inspect the admitted prefix up to `min(result_limit, admitted_result_count)`
  - reject duplicates by heap TID
  - compare against the admitted tail when the window is full
  - locked refresh and retry if the admitted generation changed or the admitted
    prefix was stale while probing
- Added focused regression coverage for:
  - below-capacity admission
  - duplicate rejection
  - full-window tail comparison for both improving and worsening candidates
- Updated Task 18 notes to record the new admission-probe fast path.

Still intentionally deferred:
- `amcanparallel` remains `false`
- no planner-visible parallel execution yet
- no final shared top-K drain path yet
- the coordinator serializer is still the staged raw lock word rather than the
  later LWLock follow-up

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 17`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether this probe is the right narrow seam before wiring the mutating
  coordinator admission path
- Whether the admitted-generation retry contract is sufficient for the staged
  concurrency model
- Whether the duplicate and tail-comparison rules match the intended admitted
  window semantics
