# Artifact Manifest

## Packet

- packet: `602-c1-pg18-contributor-findings`
- head SHA: `4e4a9dfa6d9c9e968742af128b2700ad43995405`
- timestamp: `2026-04-24T19:45:37-07:00`

## Artifacts

### `pg18-parallel-contributor-findings-default.log`

- head SHA: `4e4a9dfa6d9c9e968742af128b2700ad43995405`
- packet/topic: `602-c1-pg18-contributor-findings`
- lane: PG18 planner-visible parallel scan, default elected visible tuple emitter
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default scalar `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-contributor-findings-default.log`
- timestamp: `2026-04-24T19:45:13-07:00`
- key lines:
  - `Limit (actual time=13.540..14.488 rows=16.00 loops=1)`
  - `Bootstrap Expansions: 17`
  - `Elements Scored: 17`
  - `Heap TIDs Returned: 16`
  - `Parallel Handoffs: Foreign Selected: 0`
  - `Parallel Handoffs: Foreign Head: 0`
  - `candidate_missing_serial_ids=[]`
  - `candidate_extra_ids=[]`
  - `planner-visible Parallel Index Scan validation passed`

### `pg18-parallel-contributor-findings-diagnostic.log`

- head SHA: `4e4a9dfa6d9c9e968742af128b2700ad43995405`
- packet/topic: `602-c1-pg18-contributor-findings`
- lane: PG18 diagnostic hidden contributor path behind one elected visible tuple emitter
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default scalar `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-contributor-findings-diagnostic.log`
- timestamp: `2026-04-24T19:45:13-07:00`
- key lines:
  - `next_runtime_blocker=PG18 diagnostic contributor env is enabled; non-emitting workers publish hidden coordinator output behind the elected visible tuple emitter`
  - `Limit (actual time=35.245..36.338 rows=16.00 loops=1)`
  - `Bootstrap Expansions: 17`
  - `Elements Scored: 17`
  - `Heap TIDs Returned: 16`
  - `Parallel Handoffs: Foreign Selected: 0`
  - `Parallel Handoffs: Foreign Head: 0`
  - `candidate_missing_serial_ids=[]`
  - `candidate_extra_ids=[]`
  - `planner-visible Parallel Index Scan validation passed`

## Validation

- `cargo fmt`
- `git diff --check`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-contributor-findings-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-contributor-findings-diagnostic.log`
