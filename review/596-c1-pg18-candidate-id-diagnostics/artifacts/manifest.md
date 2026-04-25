# Artifact Manifest

## Packet

- packet: `596-c1-pg18-candidate-id-diagnostics`
- head SHA: `12849cde18eac9122c1663251f6022db0bf6cef3`
- timestamp: `2026-04-24T18:15:50-07:00`

## Artifacts

### `pg18-parallel-id-diagnostic-default.log`

- lane: PG18 planner-visible parallel scan, elected tuple emitter
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-id-diagnostic-default.log`
- key lines:
  - `next_runtime_blocker=PG18 planner-visible Parallel Index Scan is enabled with one elected tuple emitter; rank-compatible multi-emitter Gather Merge output remains the next runtime step`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_duplicate_ids=[]`
  - `candidate_missing_serial_ids=[]`
  - `candidate_extra_ids=[]`
  - `planner-visible Parallel Index Scan validation passed`

### `pg18-parallel-id-diagnostic-multi-emitter.log`

- lane: PG18 diagnostic direct multi-emitter output
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-id-diagnostic-multi-emitter.log`
- result: expected validation failure
- key lines:
  - `next_runtime_blocker=PG18 diagnostic multi-emitter env is enabled; direct multi-emitter output remains rank-incompatible with Gather Merge`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[379, 93, 93, 177, 472, 473, 378, 280, 57, 366, 258, 176, 258, 172, 86, 82]`
  - `candidate_duplicate_ids=[93x2, 258x2]`
  - `candidate_missing_serial_ids=[71, 459, 284]`
  - `candidate_extra_ids=[93, 258, 86]`
  - `validation failed`

## Validation

- `cargo fmt`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli`
- `cargo clippy -p ecaz-cli --all-targets -- -D warnings`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`
