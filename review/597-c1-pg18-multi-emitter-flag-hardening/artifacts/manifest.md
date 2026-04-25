# Artifact Manifest

## Packet

- packet: `597-c1-pg18-multi-emitter-flag-hardening`
- head SHA: `a30e0b960c4eb29f1f09c5e16987289aaf0d2ccd`
- timestamp: `2026-04-24T18:30:42-07:00`

## Artifacts

### `pg18-parallel-multi-emitter-env0.log`

- lane: PG18 planner-visible parallel scan, diagnostic flag explicitly disabled with value `0`
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=0 --log-output target/pg18-parallel-multi-emitter-env0.log`
- key lines:
  - `env=["TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC"]`
  - `next_runtime_blocker=PG18 planner-visible Parallel Index Scan is enabled with one elected tuple emitter; rank-compatible multi-emitter Gather Merge output remains the next runtime step`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_duplicate_ids=[]`
  - `candidate_missing_serial_ids=[]`
  - `candidate_extra_ids=[]`
  - `planner-visible Parallel Index Scan validation passed`

### `pg18-parallel-multi-emitter-env1.log`

- lane: PG18 diagnostic direct multi-emitter output
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-multi-emitter-env1.log`
- result: expected validation failure
- key lines:
  - `env=["TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC"]`
  - `next_runtime_blocker=PG18 diagnostic multi-emitter env is enabled; direct multi-emitter output remains rank-incompatible with Gather Merge`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[379, 177, 472, 473, 378, 165, 172, 93, 280, 57, 366, 258, 176, 82, 71, 377]`
  - `candidate_duplicate_ids=[]`
  - `candidate_missing_serial_ids=[459, 284]`
  - `candidate_extra_ids=[165, 377]`
  - `validation failed`

## Validation

- `cargo fmt`
- `cargo test parallel_scan_backend_may_emit_tuples --lib`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`
