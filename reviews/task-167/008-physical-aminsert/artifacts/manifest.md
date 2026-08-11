# Task 167 packet 008 artifacts

- Head SHA: `64d940f87`
- Task bucket: `reviews/task-167/008-physical-aminsert/`
- Lane: PG18 local compile and physical-generation lifecycle test
- Fixture: `test_distann_generation_relations_replay_abort_and_privileges`
- Storage format: Published physical generation; graph store with retained
  historical rows and an `is_current` partial unique directory index
- Rerank mode: exact source-vector planning for incremental owner writes
- Command: `cargo check --no-default-features --features pg18`
- Command: `cargo pgrx test pg18 --no-default-features --features pg18 test_distann_generation_relations_replay_abort_and_privileges`
- Timestamp: 2026-08-11 America/Los_Angeles
- Surface: isolated one-generation lifecycle fixture; no shared-table benchmark
- Result: compile exit 0; lifecycle test `1 passed, 0 failed`
- Evidence: `validation.log`

This packet contains validation evidence only. No corpus, polling output, or
resident PostgreSQL cluster is included.
