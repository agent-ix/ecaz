# Artifact Manifest

## Packet

- packet: `604-c1-pg18-contributor-retire-counter`
- head SHA: `f6d426ee93de3f6dfd206d290ae5d9b393b513b0`
- timestamp: `2026-04-24T20:17:38-07:00`

## Artifacts

### `pg18-parallel-contributor-retire-counter-default.log`

- head SHA: `f6d426ee93de3f6dfd206d290ae5d9b393b513b0`
- packet/topic: `604-c1-pg18-contributor-retire-counter`
- lane: PG18 planner-visible parallel scan, default elected visible tuple emitter
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default scalar `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-contributor-retire-counter-default.log`
- timestamp: `2026-04-24T20:11:30-07:00`
- key lines:
  - `Limit (actual time=14.093..15.152 rows=16.00 loops=1)`
  - `Bootstrap Expansions: 17`
  - `Elements Scored: 17`
  - `Heap TIDs Returned: 16`
  - `Parallel Handoffs: Foreign Selected: 0`
  - `Parallel Handoffs: Foreign Head: 0`
  - `Parallel Contributor Duplicate Retires: 0`
  - `candidate_missing_serial_ids=[]`
  - `candidate_extra_ids=[]`
  - `planner-visible Parallel Index Scan validation passed`

### `pg18-parallel-contributor-retire-counter-diagnostic.log`

- head SHA: `f6d426ee93de3f6dfd206d290ae5d9b393b513b0`
- packet/topic: `604-c1-pg18-contributor-retire-counter`
- lane: PG18 diagnostic hidden contributor path behind one elected visible tuple emitter
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default scalar `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-contributor-retire-counter-diagnostic.log`
- timestamp: `2026-04-24T20:11:37-07:00`
- key lines:
  - `Limit (actual time=34.083..34.948 rows=16.00 loops=1)`
  - `Bootstrap Expansions: 17`
  - `Elements Scored: 17`
  - `Heap TIDs Returned: 16`
  - `Parallel Handoffs: Foreign Selected: 0`
  - `Parallel Handoffs: Foreign Head: 0`
  - `Parallel Contributor Duplicate Retires: 0`
  - `candidate_missing_serial_ids=[]`
  - `candidate_extra_ids=[]`
  - `planner-visible Parallel Index Scan validation passed`

## Validation

- `cargo fmt`
- `cargo test retire_obsolete_non_emitting_parallel_contributor_output --lib`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --features pg_test --no-default-features`
- `git diff --check`
