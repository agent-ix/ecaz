# Artifact Manifest

## Packet

- packet: `598-c1-pg18-projected-score-diagnostics`
- head SHA: `09683f38b3410b2c4adc403991987d6cc60b95cb`
- timestamp: `2026-04-24T18:58:28-07:00`

## Artifacts

### `pg18-parallel-projected-score-default.log`

- head SHA: `09683f38b3410b2c4adc403991987d6cc60b95cb`
- packet/topic: `598-c1-pg18-projected-score-diagnostics`
- lane: PG18 planner-visible parallel scan, default elected tuple emitter
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default scalar `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-projected-score-default.log`
- timestamp: `2026-04-24T18:52:55-07:00`
- key lines:
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_missing_serial_ids=[]`
  - `candidate_extra_ids=[]`
  - `serial_projected_orderby_scores=[(177, -1.646769881), (379, -1.742236257), (472, -1.641386509), (473, -1.769334435), (378, -1.614288449), (172, -1.507030725), (93, -1.649150252), (280, -1.575398922), (57, -1.543027639), (366, -1.578914285), (258, -1.510545969), (176, -1.518822074), (82, -1.491723895), (71, -1.584297538), (459, -1.478064418), (284, -1.587190151)]`
  - `candidate_projected_orderby_scores=[(177, -1.646769881), (379, -1.742236257), (472, -1.641386509), (473, -1.769334435), (378, -1.614288449), (172, -1.507030725), (93, -1.649150252), (280, -1.575398922), (57, -1.543027639), (366, -1.578914285), (258, -1.510545969), (176, -1.518822074), (82, -1.491723895), (71, -1.584297538), (459, -1.478064418), (284, -1.587190151)]`
  - `serial_projected_orderby_score_adjacent_inversions=[177(-1.646769881) before 379(-1.742236257), 472(-1.641386509) before 473(-1.769334435), 172(-1.507030725) before 93(-1.649150252), 57(-1.543027639) before 366(-1.578914285), 258(-1.510545969) before 176(-1.518822074), 82(-1.491723895) before 71(-1.584297538), 459(-1.478064418) before 284(-1.587190151)]`
  - `candidate_projected_orderby_score_adjacent_inversions=[177(-1.646769881) before 379(-1.742236257), 472(-1.641386509) before 473(-1.769334435), 172(-1.507030725) before 93(-1.649150252), 57(-1.543027639) before 366(-1.578914285), 258(-1.510545969) before 176(-1.518822074), 82(-1.491723895) before 71(-1.584297538), 459(-1.478064418) before 284(-1.587190151)]`
  - `candidate_exact_score_adjacent_inversions=[177(-1.646769881) before 379(-1.742236257), 472(-1.641386509) before 473(-1.769334435), 172(-1.507030725) before 93(-1.649150252), 57(-1.543027639) before 366(-1.578914285), 258(-1.510545969) before 176(-1.518822074), 82(-1.491723895) before 71(-1.584297538), 459(-1.478064418) before 284(-1.587190151)]`
  - `planner-visible Parallel Index Scan validation passed`

### `pg18-parallel-projected-score-multi-emitter.log`

- head SHA: `09683f38b3410b2c4adc403991987d6cc60b95cb`
- packet/topic: `598-c1-pg18-projected-score-diagnostics`
- lane: PG18 diagnostic direct multi-emitter output
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default scalar `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-projected-score-multi-emitter-rerun.log`
- timestamp: `2026-04-24T18:53:18-07:00`
- result: expected validation failure
- key lines:
  - `next_runtime_blocker=PG18 diagnostic multi-emitter env is enabled; direct multi-emitter output remains rank-incompatible with Gather Merge`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[379, 177, 472, 473, 378, 438, 172, 93, 280, 57, 366, 258, 176, 82, 71, 284]`
  - `candidate_missing_serial_ids=[459]`
  - `candidate_extra_ids=[438]`
  - `serial_projected_orderby_scores=[(177, -1.646769881), (379, -1.742236257), (472, -1.641386509), (473, -1.769334435), (378, -1.614288449), (172, -1.507030725), (93, -1.649150252), (280, -1.575398922), (57, -1.543027639), (366, -1.578914285), (258, -1.510545969), (176, -1.518822074), (82, -1.491723895), (71, -1.584297538), (459, -1.478064418), (284, -1.587190151)]`
  - `candidate_projected_orderby_scores=[(379, -1.742236257), (177, -1.646769881), (472, -1.641386509), (473, -1.769334435), (378, -1.614288449), (438, -1.541159630), (172, -1.507030725), (93, -1.649150252), (280, -1.575398922), (57, -1.543027639), (366, -1.578914285), (258, -1.510545969), (176, -1.518822074), (82, -1.491723895), (71, -1.584297538), (284, -1.587190151)]`
  - `serial_projected_orderby_score_adjacent_inversions=[177(-1.646769881) before 379(-1.742236257), 472(-1.641386509) before 473(-1.769334435), 172(-1.507030725) before 93(-1.649150252), 57(-1.543027639) before 366(-1.578914285), 258(-1.510545969) before 176(-1.518822074), 82(-1.491723895) before 71(-1.584297538), 459(-1.478064418) before 284(-1.587190151)]`
  - `candidate_projected_orderby_score_adjacent_inversions=[472(-1.641386509) before 473(-1.769334435), 172(-1.507030725) before 93(-1.649150252), 57(-1.543027639) before 366(-1.578914285), 258(-1.510545969) before 176(-1.518822074), 82(-1.491723895) before 71(-1.584297538), 71(-1.584297538) before 284(-1.587190151)]`
  - `candidate_exact_score_adjacent_inversions=[472(-1.641386509) before 473(-1.769334435), 172(-1.507030725) before 93(-1.649150252), 57(-1.543027639) before 366(-1.578914285), 258(-1.510545969) before 176(-1.518822074), 82(-1.491723895) before 71(-1.584297538), 71(-1.584297538) before 284(-1.587190151)]`
  - `validation failed`

## Validation

- `cargo fmt`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli`
- `cargo clippy -p ecaz-cli --all-targets -- -D warnings`
- `git diff --check`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
