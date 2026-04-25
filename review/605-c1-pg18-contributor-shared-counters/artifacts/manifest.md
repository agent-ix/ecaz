# Artifact Manifest

## Packet

- packet: `605-c1-pg18-contributor-shared-counters`
- head SHA: `368a31ef56a494c24e57b3eab715f16d986963e8`
- timestamp: `2026-04-24T20:48:58-07:00`

## Artifacts

### `pg18-parallel-contributor-shared-counters-default.log`

- head SHA: `368a31ef56a494c24e57b3eab715f16d986963e8`
- packet/topic: `605-c1-pg18-contributor-shared-counters`
- lane: PG18 planner-visible parallel scan, default elected visible tuple emitter
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default scalar `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-contributor-shared-counters-default.log`
- timestamp: `2026-04-24T20:48:08-07:00`
- key lines:
  - `Limit (actual time=13.478..14.567 rows=16.00 loops=1)`
  - `Bootstrap Expansions: 17`
  - `Elements Scored: 17`
  - `Heap TIDs Returned: 16`
  - `Parallel Contributor Hidden Publishes: 0`
  - `Parallel Contributor Duplicate Retires: 0`
  - `candidate_missing_serial_ids=[]`
  - `candidate_extra_ids=[]`
  - `planner-visible Parallel Index Scan validation passed`

### `pg18-parallel-contributor-shared-counters-diagnostic.log`

- head SHA: `368a31ef56a494c24e57b3eab715f16d986963e8`
- packet/topic: `605-c1-pg18-contributor-shared-counters`
- lane: PG18 diagnostic hidden contributor path behind one elected visible tuple emitter
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default scalar `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-contributor-shared-counters-diagnostic.log`
- timestamp: `2026-04-24T20:48:14-07:00`
- key lines:
  - `Limit (actual time=33.686..34.986 rows=16.00 loops=1)`
  - `Bootstrap Expansions: 17`
  - `Elements Scored: 17`
  - `Heap TIDs Returned: 16`
  - `Parallel Contributor Hidden Publishes: 8`
  - `Parallel Contributor Duplicate Retires: 4`
  - `candidate_missing_serial_ids=[]`
  - `candidate_extra_ids=[]`
  - `planner-visible Parallel Index Scan validation passed`

## Validation

- `cargo fmt`
- `cargo test contributor --lib`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --features pg_test --no-default-features`
- `git diff --check`
