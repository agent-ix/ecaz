# Task 167 packet 014 manifest

- head SHA: `ff9b9ae8e`
- task bucket: `reviews/task-167/`
- packet: `reviews/task-167/014-physical-mid-insert-fixture/`
- lane: PG18 physical-generation multinode fault drill
- fixture: `drive_physical_fixture` with isolated one-owner `mi` table
- storage format: Task 179 physical generation format
- rerank mode: co-located row-tier exact rerank
- command: `cargo check -p ecaz-cli`
- command: `cargo check --no-default-features --features pg18`
- timestamp: `2026-08-11` America/Los_Angeles
- shared-table/isolated-table surface: isolated physical drill table; no live
  fixture execution in this checkpoint

No benchmark, TC-043, or NFR-020 runtime result is claimed by this packet.
