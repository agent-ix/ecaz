# Artifact Manifest

## Packet

- packet: `606-c1-pg18-contributor-exit-counters`
- head SHA: `2e6198badc3201fd961b133b1fc359243039c453`
- timestamp: `2026-04-24T21:11:12-07:00`

## Artifacts

### `pg18-parallel-contributor-exit-counters-default.log`

- head SHA: `2e6198badc3201fd961b133b1fc359243039c453`
- packet/topic: `606-c1-pg18-contributor-exit-counters`
- lane: PG18 planner-visible parallel scan, default elected visible tuple emitter
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default scalar `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-contributor-exit-counters-default.log`
- timestamp: `2026-04-24T21:00:25-07:00`
- key lines:
  - `Limit (actual time=13.809..15.187 rows=16.00 loops=1)`
  - `Bootstrap Expansions: 17`
  - `Elements Scored: 17`
  - `Heap TIDs Returned: 16`
  - `Parallel Contributor Hidden Publishes: 0`
  - `Parallel Contributor Duplicate Retires: 0`
  - `Parallel Contributor Output Limit Exits: 0`
  - `Parallel Contributor Poll Limit Exits: 0`
  - `candidate_missing_serial_ids=[]`
  - `candidate_extra_ids=[]`
  - `planner-visible Parallel Index Scan validation passed`

### `pg18-parallel-contributor-exit-counters-diagnostic.log`

- head SHA: `2e6198badc3201fd961b133b1fc359243039c453`
- packet/topic: `606-c1-pg18-contributor-exit-counters`
- lane: PG18 diagnostic hidden contributor path behind one elected visible tuple emitter
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default scalar `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-contributor-exit-counters-diagnostic.log`
- timestamp: `2026-04-24T21:00:45-07:00`
- key lines:
  - `Limit (actual time=33.622..35.003 rows=16.00 loops=1)`
  - `Bootstrap Expansions: 17`
  - `Elements Scored: 17`
  - `Heap TIDs Returned: 16`
  - `Parallel Handoffs: Foreign Selected: 0`
  - `Parallel Handoffs: Foreign Head: 0`
  - `Parallel Contributor Hidden Publishes: 8`
  - `Parallel Contributor Duplicate Retires: 4`
  - `Parallel Contributor Output Limit Exits: 0`
  - `Parallel Contributor Poll Limit Exits: 4`
  - `candidate_missing_serial_ids=[]`
  - `candidate_extra_ids=[]`
  - `planner-visible Parallel Index Scan validation passed`

## Validation

- `cargo fmt`
- `cargo test explain_counters --lib`
- `cargo test contributor --lib`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --features pg_test --no-default-features`
- `cargo pgrx test pg18`
- `git diff --check`
