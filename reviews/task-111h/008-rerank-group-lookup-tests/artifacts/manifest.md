# Task 111h / 008 Artifacts Manifest

Head SHA: `84a2694580e50650ef6dbe3861efa98c26d73fc8`

Task bucket: `reviews/task-111h/008-rerank-group-lookup-tests/`

Timestamp: `2026-06-20T05:11:23Z`

Scope: private packed rerank group payload lookup tests.

Storage surface: `rerank_placement = 'index'`, compact persisted payloads,
packed rerank group layout with `0x2B` group headers and `0x2C` payload
continuation segments.

Suite surface: no benchmark suite. These are correctness/static validation logs,
not latency/recall/storage measurement evidence. No benchmark tables were
created; no isolated one-index-per-table or shared-table benchmark surfaces were
used.

## Artifacts

- `cargo-test-rerank-group-lookup.log`
  - Command: `script -q -c "cargo test --no-default-features --features pg18 rerank_group_payload_lookup --lib" reviews/task-111h/008-rerank-group-lookup-tests/artifacts/cargo-test-rerank-group-lookup.log`
  - Key result: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2200 filtered out`

- `cargo-check-pg18.log`
  - Command: `script -q -c "cargo check --no-default-features --features pg18" reviews/task-111h/008-rerank-group-lookup-tests/artifacts/cargo-check-pg18.log`
  - Key result: `Finished dev profile [unoptimized + debuginfo] target(s) in 7.25s`
