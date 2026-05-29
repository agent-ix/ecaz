# Review Request: Storage Result Raw Bytes

## Scope

This checkpoint tightens structured benchmark output for the Task 60 benchmark-host run without adding comparisons or automated ratio gates.

`ecaz bench storage` renders human-readable sizes such as `13.0 MiB` and `494.0 B`. The suite result parser now preserves those fields and adds raw numeric byte fields when the value has a known byte unit:

- `storage_field.value_bytes`
- `storage_index.size_bytes`
- `storage_index.per_row_bytes`

This lets the benchmark packet record the 1M shipping decision from structured rows without reparsing pretty strings or adding a suite-level comparison mechanism.

## Validation

Artifacts are under `reviews/task-60/009-storage-result-bytes/artifacts/`.

- `cargo-test-ecaz-cli-bench-suite.log`: focused suite tests passed, 29 tests.
- `suite-audit.log`: Task 60 suite audit still passes with 24 steps.

## Remaining Task 60 Gate

The actual 100k/1M benchmark suite still runs on the benchmark host. This checkpoint only makes the resulting storage rows cleaner to consume once that run is complete.
