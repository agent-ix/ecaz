# Artifact Manifest

Task bucket: `reviews/task-142`
Packet: `reviews/task-142/006-coordinator-fanout-manifest-cache`
Head SHA: `b59470cc35759fe4001da0b6854746ed9f3b3b88`
Timestamp: `2026-07-05T08:58:17Z`
Lane / fixture / storage format / rerank mode: local Rust unit-test validation;
no corpus fixture, no storage benchmark lane, no rerank mode.
Surface: SPIRE coordinator production fanout manifest caching.
Isolation: not applicable to benchmark table isolation; no benchmark matrix was
run in this packet.

## Artifacts

### `artifacts/cargo-test-coordinator-fanout-manifest-cache.log`

Command:

```sh
script -q -e -c "cargo test coordinator_fanout_manifest_cache_reuses_epoch_manifests -- --nocapture" reviews/task-142/006-coordinator-fanout-manifest-cache/artifacts/cargo-test-coordinator-fanout-manifest-cache.log
```

Key result:

```text
test am::ec_spire::production_executor_state_tests::coordinator_fanout_manifest_cache_reuses_epoch_manifests ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2257 filtered out; finished in 0.00s
```

### `artifacts/cargo-test-production-read-profile-rollup.log`

Command:

```sh
script -q -e -c "cargo test production_read_profile_row_preserves_metric_rollup -- --nocapture" reviews/task-142/006-coordinator-fanout-manifest-cache/artifacts/cargo-test-production-read-profile-rollup.log
```

Key result:

```text
test am::ec_spire::production_executor_state_tests::production_read_profile_row_preserves_metric_rollup ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2257 filtered out; finished in 0.00s
```

### `artifacts/cargo-test-routing-hierarchy-cache-regression.log`

Command:

```sh
script -q -e -c "cargo test collect_cached_resolved_scan_plan_selection_reuses_epoch_hierarchy -- --nocapture" reviews/task-142/006-coordinator-fanout-manifest-cache/artifacts/cargo-test-routing-hierarchy-cache-regression.log
```

Key result:

```text
test am::ec_spire::scan::tests::collect_cached_resolved_scan_plan_selection_reuses_epoch_hierarchy ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2257 filtered out; finished in 0.00s
```

## Non-Cited Local Checks

- `git diff --check` passed.
- `cargo fmt --check` and touched-file `rustfmt --check` were attempted, but
  reported pre-existing formatting drift outside this slice; no formatting
  changes were applied.
