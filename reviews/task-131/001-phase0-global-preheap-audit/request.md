# Review Request: Phase 0 Global Pre-Heap Audit Metrics

Task: 131 (`plan/tasks/131-spire-streaming-global-topk-pruning.md`)
Head SHA: `2badf60a1f219e515bbb12449eceb072bad5892a`

## Summary

This checkpoint starts Task 131 with the smallest Phase 0 / Phase 1 evidence
surface needed before changing distributed heap behavior.

Changes:

- Added the Task 131 task definition and task-index entry.
- Fixed the draft artifact bucket typo from `reviews/task-122/` to
  `reviews/task-131/`.
- Added production-read profile counters for the coordinator-side global
  compact-candidate merge before heap resolution:
  - `global_pre_heap_input_count`
  - `global_pre_heap_candidate_count`
  - `global_pre_heap_duplicate_vec_id_count`
  - `global_pre_heap_pruned_candidate_count`
- Exposed the counters through
  `ec_spire_remote_search_production_read_profile`.
- Aggregated and rendered the counters in
  `ecaz bench spire-pipeline --include-production-read-profile` reports.

The runtime behavior is intentionally unchanged: remote heap requests still use
the existing per-worker local `top_k` shape. These counters quantify how many
remote heap rows a future global merge-before-heap protocol could avoid.

## Validation

Packet-local artifacts are under
`reviews/task-131/001-phase0-global-preheap-audit/artifacts/`.

- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`
  passed.
- `cargo-test-ecaz-cli-production-read-profile.log`:
  `cargo test spire_pipeline_renders_production_read_profile --package ecaz-cli`
  passed.
- `cargo-test-production-read-profile-timeout.log`: bounded attempt to run the
  extension unit test timed out before test execution in this session; see the
  manifest for details.

## Reviewer Focus

- Whether the new counters are named clearly enough for Task 131 Phase 0/1
  benchmark artifacts.
- Whether computing the limited global merge after candidate receive but before
  applying heap results is the right low-risk diagnostic point.
- Whether the counter semantics are sufficient before implementing a candidate
  subset heap-resolution endpoint.
