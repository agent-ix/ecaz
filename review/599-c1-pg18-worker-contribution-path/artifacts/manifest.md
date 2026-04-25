# Artifact Manifest

## Packet

- packet: `599-c1-pg18-worker-contribution-path`
- head SHA: `019c56d3e8fdab94ca773ed97991098c4dd8681a`
- timestamp: `2026-04-24T19:12:46-07:00`

## Artifacts

### `pg18-parallel-worker-contribution-blocker-default.log`

- head SHA: `019c56d3e8fdab94ca773ed97991098c4dd8681a`
- packet/topic: `599-c1-pg18-worker-contribution-path`
- lane: PG18 planner-visible parallel scan, default elected visible tuple emitter
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default scalar `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-worker-contribution-blocker-default.log`
- timestamp: `2026-04-24T19:10:40-07:00`
- key lines:
  - `next_runtime_blocker=PG18 planner-visible Parallel Index Scan is enabled with one elected visible tuple emitter; next runtime step is shared worker contribution behind that single output stream`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_missing_serial_ids=[]`
  - `candidate_extra_ids=[]`
  - `planner-visible Parallel Index Scan validation passed`

### `pg18-parallel-worker-contribution-blocker-multi-emitter.log`

- head SHA: `019c56d3e8fdab94ca773ed97991098c4dd8681a`
- packet/topic: `599-c1-pg18-worker-contribution-path`
- lane: PG18 diagnostic direct multi-emitter output
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default scalar `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-worker-contribution-blocker-multi-emitter.log`
- timestamp: `2026-04-24T19:11:25-07:00`
- result: expected validation failure
- key lines:
  - `next_runtime_blocker=PG18 diagnostic multi-emitter env is enabled; direct multi-emitter output remains rank-incompatible with Gather Merge and is not the production path`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[379, 177, 472, 473, 378, 165, 172, 93, 280, 57, 366, 258, 176, 82, 71, 377]`
  - `candidate_missing_serial_ids=[459, 284]`
  - `candidate_extra_ids=[165, 377]`
  - `validation failed`

## Validation

- `cargo fmt`
- `git diff --check`
- `cargo test test_ech_planner --lib`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
