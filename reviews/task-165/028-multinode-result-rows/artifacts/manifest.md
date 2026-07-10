# Packet 165/028 artifacts manifest

- task bucket / packet: reviews/task-165/028-multinode-result-rows
- head SHA: f98e84775 (suite parser) — see `git log` for exact
- surface: `ecaz-cli` bench suite result emitter (host tool; no PG runtime)
- change under review: `crates/ecaz-cli/src/commands/bench/suite.rs`
  (`parse_distann_multinode_rows` + `parse_result_rows` arm + 2 unit tests)

## Artifacts

- `parser-unit-tests.log` — `cargo test -p ecaz-cli distann_multinode`. Key
  line: `test result: ok. 2 passed`. Pure parser unit test; no benchmark
  measurement, so NFR-007 A/B evidence is not implicated by this packet (the
  real multi-instance suite run that exercises this parser end-to-end is the
  Task 172 deliverable).

## Cited result lines (request.md)

- `distann_multinode_rows_parse_recall_identity_gate_and_drills ... ok`
- `distann_multinode_recall_mismatch_sets_identity_not_ok ... ok`
- `test result: ok. 2 passed`
