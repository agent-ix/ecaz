# Artifact Manifest

Task bucket: `reviews/task-142`
Packet: `reviews/task-142/008-production-endpoint-identity-cache`
Head SHA: `9626b54d7e3b2738c4f32ba6a92ae7c154faca91`
Timestamp: `2026-07-05T09:34:44Z`
Lane / fixture / storage format / rerank mode: local Rust unit-test validation;
no corpus fixture, no storage benchmark lane, no rerank mode.
Surface: SPIRE async production pooled connection endpoint identity reuse.
Isolation: not applicable to benchmark table isolation; no benchmark matrix was
run in this packet.

## Artifacts

### `artifacts/cargo-test-production-endpoint-identity-cache-r2.log`

Command:

```sh
script -q -e -c "cargo test cached_production_endpoint_identity_requires_matching_identity -- --nocapture" reviews/task-142/008-production-endpoint-identity-cache/artifacts/cargo-test-production-endpoint-identity-cache-r2.log
```

Key result:

```text
test am::ec_spire::production_executor_state_tests::cached_production_endpoint_identity_requires_matching_identity ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2259 filtered out; finished in 0.00s
```

### `artifacts/cargo-test-production-read-profile-rollup.log`

Command:

```sh
script -q -e -c "cargo test production_read_profile_row_preserves_metric_rollup -- --nocapture" reviews/task-142/008-production-endpoint-identity-cache/artifacts/cargo-test-production-read-profile-rollup.log
```

Key result:

```text
test am::ec_spire::production_executor_state_tests::production_read_profile_row_preserves_metric_rollup ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2259 filtered out; finished in 0.00s
```

## Non-Cited Local Checks

- `git diff --check` passed.
- `artifacts/cargo-test-production-endpoint-identity-cache.log` was a failed
  compile attempt before the mutability fix and is not committed.
