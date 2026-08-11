# Task 167 packet 015 manifest

- head SHA: `d38abfa44`
- task bucket: `reviews/task-167/`
- packet: `reviews/task-167/015-physical-concurrency-fixture/`
- lane: PG18 physical-generation concurrent insert/query drill
- fixture: published `dm_idx` physical generation
- storage format: Task 179 physical generation format
- rerank mode: co-located row-tier exact rerank
- commands: `cargo check -p ecaz-cli`; `cargo check --no-default-features --features pg18`
- timestamp: `2026-08-11` America/Los_Angeles
- shared-table/isolated-table surface: shared published `dm_idx` fixture; no
  live fixture execution in this checkpoint

No benchmark, TC-043, or NFR-020 runtime result is claimed by this packet.
