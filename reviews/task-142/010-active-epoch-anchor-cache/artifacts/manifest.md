# Artifact Manifest

Task bucket: `reviews/task-142`
Packet: `reviews/task-142/010-active-epoch-anchor-cache`
Head SHA: `bda865ffb9c06886a14ae3c4ed24411b096bf79f`
Timestamp: `2026-07-05T10:05:26Z`
Lane / fixture / storage format / rerank mode: local Rust unit-test validation;
no corpus fixture, no storage benchmark lane, no rerank mode.
Surface: SPIRE active epoch anchor manifest cache for remote/session reuse.
Isolation: not applicable to benchmark table isolation; no benchmark matrix was
run in this packet.

## Artifacts

### `artifacts/cargo-test-active-epoch-anchor-cache.log`

Command:

```sh
script -q -e -c "cargo test active_epoch_anchor_cache_reuses_epoch_anchor -- --nocapture" reviews/task-142/010-active-epoch-anchor-cache/artifacts/cargo-test-active-epoch-anchor-cache.log
```

Key result:

```text
test am::ec_spire::production_executor_state_tests::active_epoch_anchor_cache_reuses_epoch_anchor ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2260 filtered out; finished in 0.00s
```

### `artifacts/cargo-test-production-read-profile-rollup.log`

Command:

```sh
script -q -e -c "cargo test production_read_profile_row_preserves_metric_rollup -- --nocapture" reviews/task-142/010-active-epoch-anchor-cache/artifacts/cargo-test-production-read-profile-rollup.log
```

Key result:

```text
test am::ec_spire::production_executor_state_tests::production_read_profile_row_preserves_metric_rollup ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2260 filtered out; finished in 0.00s
```

### `artifacts/cargo-test-fanout-manifest-cache-regression.log`

Command:

```sh
script -q -e -c "cargo test coordinator_fanout_manifest_cache_reuses_epoch_manifests -- --nocapture" reviews/task-142/010-active-epoch-anchor-cache/artifacts/cargo-test-fanout-manifest-cache-regression.log
```

Key result:

```text
test am::ec_spire::production_executor_state_tests::coordinator_fanout_manifest_cache_reuses_epoch_manifests ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2260 filtered out; finished in 0.00s
```

## Non-Cited Local Checks

- `git diff --check` passed.
