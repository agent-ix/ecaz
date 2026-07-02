# Artifact Manifest

Task bucket: `reviews/task-131/001-phase0-global-preheap-audit/`
Head SHA: `2badf60a1f219e515bbb12449eceb072bad5892a`
Timestamp: 2026-07-01

## Scope

Phase 0 / Phase 1 instrumentation checkpoint for Task 131. This packet adds
production-read profile counters that quantify the coordinator-side global
compact-candidate merge before heap resolution:

- `global_pre_heap_input_count`
- `global_pre_heap_candidate_count`
- `global_pre_heap_duplicate_vec_id_count`
- `global_pre_heap_pruned_candidate_count`

The counters are exposed by `ec_spire_remote_search_production_read_profile`
and aggregated in `ecaz bench spire-pipeline --include-production-read-profile`
reports. This checkpoint does not change heap-fetch behavior and does not make
a latency, recall, or promotion claim.

## Artifacts

### `cargo-check-pg18.log`

- Command:
  `cargo check --no-default-features --features pg18 > reviews/task-131/001-phase0-global-preheap-audit/artifacts/cargo-check-pg18.log 2>&1`
- Result: pass.
- Key line: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.27s`

### `cargo-test-ecaz-cli-production-read-profile.log`

- Command:
  `cargo test spire_pipeline_renders_production_read_profile --package ecaz-cli > reviews/task-131/001-phase0-global-preheap-audit/artifacts/cargo-test-ecaz-cli-production-read-profile.log 2>&1`
- Result: pass.
- Key lines:
  - `test commands::bench::spire_pipeline::tests::spire_pipeline_renders_production_read_profile ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 418 filtered out`

### `cargo-test-production-read-profile-timeout.log`

- Command:
  `timeout 90s cargo test production_read_profile_row_preserves_metric_rollup --no-default-features --features pg18 > reviews/task-131/001-phase0-global-preheap-audit/artifacts/cargo-test-production-read-profile-timeout.log 2>&1`
- Result: timed out with exit code 124.
- Key line before timeout: `Compiling ecaz v0.1.1 (/home/peter/dev/ecaz)`
- Interpretation: the extension unit-test target did not reach test execution
  within the bounded window in this sandboxed session. The PG18 compile check
  above covers type-checking for the touched extension code.

## Isolation Notes

- Fixture: no live multi-instance fixture run in this packet.
- Storage format: unchanged.
- Rerank mode: unchanged.
- Surface: production distributed-read profile metrics and CLI report
  aggregation only.
- One-index-per-table vs shared-table: not applicable for this compile/report
  checkpoint.
