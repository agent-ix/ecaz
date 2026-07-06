# Task 145 / Packet 010 Artifacts

- Head SHA: `6eb1e46ab2f88a3aadce316dd0eac6994a4d999a`
- Task bucket: `reviews/task-145/010-bound-prune-engagement-counter/`
- Scope: code/instrumentation only; no benchmark conclusions.
- Reason: packet 008's bound-prune A/B is faulty/null evidence because it did not prove the pre-materialization prune mechanism engaged. This packet adds the dedicated engagement counter required before any future bound-prune A/B can support a conclusion.

## Code Change

- Added `pre_materialization_pruned_candidate_row_count` to routed scan diagnostics, selected-leaf scan profiles, public SQL scan-profile functions, remote profile payload decode, and `ecaz bench spire-pipeline` production scan-profile aggregation.
- The counter increments only through `SpireRoutedScanObserver::pre_materialization_pruned_candidate`, called from the true pre-materialization branches before row materialization.
- Existing `truncated_candidate_row_count` remains inclusive for compatibility; the new counter distinguishes bound-prune engagement from other truncation causes.

## Validation

- `cargo test --no-default-features --features pg18 collect_quantized_selected_leaf_scan_profile_reports_scan_counters --lib`
  - Log: `artifacts/cargo-test-selected-leaf-profile-counter.log`
  - Result: `1 passed; 0 failed`
- `cargo test --no-default-features --features pg18 collect_scan_placement_diagnostics_reports_candidate_truncation --lib`
  - Log: `artifacts/cargo-test-placement-diagnostics-counter.log`
  - Result: `1 passed; 0 failed`
- `cargo test -p ecaz-cli spire_pipeline_renders_production_scan_profile`
  - Log: `artifacts/cargo-test-cli-production-scan-profile-counter.log`
  - Result: `1 passed; 0 failed`
- `cargo test -p ecaz-cli spire_pipeline_sql_uses_public_snapshot_contracts`
  - Log: `artifacts/cargo-test-cli-sql-contract-counter.log`
  - Result: `1 passed; 0 failed`

## Follow-Up Required

- This packet does not claim bound-prune performance or recall impact.
- The next valid bound-prune evidence must run `ecaz bench suite` with this counter in the production scan-profile output and must show a real off=0/on>0 engagement delta before any latency/recall conclusion is considered.
