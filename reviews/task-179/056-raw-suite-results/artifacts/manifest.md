# Artifact manifest

- Head SHA: `0474ef90983de8acfc64022e3d548ec0bcae7062`
- Implementation commit: `0474ef90983de8acfc64022e3d548ec0bcae7062`
- Task bucket / packet: `reviews/task-179/056-raw-suite-results`
- Branch: `task-179-ec-distann-physical-shards`
- Lane: local `ecaz bench suite` result normalization
- Run: `2026-07-13T06:07:36-07:00` through
  `2026-07-13T06:08:38-07:00`
- Fixture / storage / rerank mode: not applicable; parser/static validation
- Isolation surface: no PostgreSQL relation or benchmark table used

This is runner correctness evidence, not benchmark evidence. It makes no
latency, recall, storage, or promotion claim.

## Commands

```text
cargo test -p ecaz-cli raw_suite_result_rows_are_structured -- --nocapture
cargo check -p ecaz-cli
```

## Artifact index

- `parser-test.log`: focused accepted/malformed marker regression, exit code 0.
- `cargo-check.log`: CLI compile validation, exit code 0; includes one
  pre-existing unrelated dead-code warning.

## Key cited results

```text
test commands::bench::suite::tests::raw_suite_result_rows_are_structured ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 433 filtered out

Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.93s
```
