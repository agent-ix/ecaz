# Artifact Manifest

## Packet

- packet: `600-c1-pg18-ecaz-explain-counters`
- head SHA: `000d3b9ae515fab51a20c375b6d6d42e0d92c239`
- timestamp: `2026-04-24T19:19:53-07:00`

## Artifacts

### `pg18-parallel-ecaz-explain-default.log`

- head SHA: `000d3b9ae515fab51a20c375b6d6d42e0d92c239`
- packet/topic: `600-c1-pg18-ecaz-explain-counters`
- lane: PG18 planner-visible parallel scan, default elected visible tuple emitter
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default scalar `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-ecaz-explain-default.log`
- timestamp: `2026-04-24T19:18:56-07:00`
- key lines:
  - `Index Searches: 0`
  - `Bootstrap Expansions: 17`
  - `Elements Scored: 17`
  - `Heap TIDs Returned: 16`
  - `candidate_duplicate_ids=[]`
  - `candidate_missing_serial_ids=[]`
  - `candidate_extra_ids=[]`
  - `planner-visible Parallel Index Scan validation passed`

### `pg18-parallel-ecaz-explain-multi-emitter.log`

- head SHA: `000d3b9ae515fab51a20c375b6d6d42e0d92c239`
- packet/topic: `600-c1-pg18-ecaz-explain-counters`
- lane: PG18 diagnostic direct multi-emitter output
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default scalar `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-ecaz-explain-multi-emitter.log`
- timestamp: `2026-04-24T19:19:05-07:00`
- result: expected validation failure
- key lines:
  - `Index Searches: 0`
  - `Bootstrap Expansions: 67`
  - `Elements Scored: 67`
  - `Heap TIDs Returned: 2`
  - `candidate_duplicate_ids=[82x2]`
  - `candidate_missing_serial_ids=[71, 459, 284]`
  - `candidate_extra_ids=[387, 82, 165]`
  - `validation failed`

## Validation

- `cargo fmt`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli`
- `cargo clippy -p ecaz-cli --all-targets -- -D warnings`
- `git diff --check`
