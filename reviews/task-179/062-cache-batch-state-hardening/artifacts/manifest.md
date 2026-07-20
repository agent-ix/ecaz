# Artifact manifest

- Head SHA: `0b2d4fbabedb4caa59535b875b81359b4dd6f91c`
- Task bucket / packet: `reviews/task-179/062-cache-batch-state-hardening/`
- Timestamp: `2026-07-13T19:54:07-07:00`
- Host / PostgreSQL: local Intel x86_64 / PostgreSQL 18.3
- Lane: local PG18 focused correctness validation
- Fixture / storage format / rerank mode: not applicable
- Isolation: focused callback tests; no benchmark tables or indexes

## Artifacts

| Artifact | Command | Result | SHA-256 |
| --- | --- | --- | --- |
| `stage-batch-pg18.log` | `cargo pgrx test pg18 test_distann_stage_batch_atomic_replay_and_directory` | 1 passed, 0 failed, 2508 filtered | `852eb6d6c11b8740145d9836dca70f8b1e97adfb63bd44e4042715b0595b7170` |
| `materialize-heap-identity-pg18.log` | `cargo pgrx test pg18 test_ec_distann_materialize_rows_ships_heap_identity` | 1 passed, 0 failed, 2508 filtered | `098ba1486760c0267032a4b977ab5995b43f271d3c7bb265885cc40a75a4d358` |
| `clippy-pg18.log` | `cargo clippy --lib --no-default-features --features pg18 -- -D warnings` | pass | `d57102dc4312c0aed4dc25fc188cbda466d3bddc542b823c61fd77c73ffabdd9` |

The logs were captured with `script -q -e -c`, retain their command exit code,
and were normalized to LF before hashing. No corpus, benchmark output,
PostgreSQL operational log, polling snapshot, or generated cache is included.
