# Artifact Manifest

Task bucket: `reviews/task-142`
Packet: `reviews/task-142/009-production-prepared-statements`
Head SHA: `29eca4133da99622a65fb72899052af767f4394f`
Timestamp: `2026-07-05T09:48:42Z`
Lane / fixture / storage format / rerank mode: local Rust unit-test validation;
no corpus fixture, no storage benchmark lane, no rerank mode.
Surface: SPIRE async production pooled remote candidate/heap query execution.
Isolation: not applicable to benchmark table isolation; no benchmark matrix was
run in this packet.

## Artifacts

### `artifacts/cargo-test-production-read-profile-rollup-r3.log`

Command:

```sh
script -q -e -c "cargo test production_read_profile_row_preserves_metric_rollup -- --nocapture" reviews/task-142/009-production-prepared-statements/artifacts/cargo-test-production-read-profile-rollup-r3.log
```

Key result:

```text
test am::ec_spire::production_executor_state_tests::production_read_profile_row_preserves_metric_rollup ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2259 filtered out; finished in 0.00s
```

### `artifacts/cargo-test-endpoint-identity-cache-regression.log`

Command:

```sh
script -q -e -c "cargo test cached_production_endpoint_identity_requires_matching_identity -- --nocapture" reviews/task-142/009-production-prepared-statements/artifacts/cargo-test-endpoint-identity-cache-regression.log
```

Key result:

```text
test am::ec_spire::production_executor_state_tests::cached_production_endpoint_identity_requires_matching_identity ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2259 filtered out; finished in 0.00s
```

## Non-Cited Local Checks

- `git diff --check` passed.
- Earlier `cargo-test-production-read-profile-rollup*.log` attempts in this
  packet were failed compile iterations and are not committed.
