# Task 167 packet 012 manifest

- head SHA: `57476bc2b`
- task bucket: `reviews/task-167/`
- packet: `reviews/task-167/012-physical-tombstone-and-fault-drill/`
- lane: PG18 physical DML and multinode fault-drill code
- fixture: isolated one-owner physical generation in the adapted drill
- storage format: Task 179 physical generation format
- rerank mode: co-located row-tier exact rerank
- commands:
  - `cargo check --no-default-features --features pg18`
  - `cargo check -p ecaz-cli`
  - `cargo pgrx test pg18 --no-default-features --features pg18 test_distann_generation_relations_replay_abort_and_privileges`
- timestamp: `2026-08-11` America/Los_Angeles
- shared-table/isolated-table surface: the drill uses an isolated `mi` table;
  no benchmark matrix was run

The packet does not claim a green TC-043 run or any benchmark result.
