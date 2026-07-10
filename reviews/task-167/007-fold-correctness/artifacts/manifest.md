# Packet 167/007 artifacts manifest

- task bucket / packet: reviews/task-167/007-fold-correctness
- head SHA: 62590b3ea4da63ad8bf0971fcdf3426b42cc86cc
- surface: single-node ec_distann, pgrx test harness (debug `.so`), PG18
- change under review: `src/am/ec_distann/insert.rs` (candidate directory
  filter) + `src/tests/ec_distann_basic.rs`
  (`test_ec_distann_fold_multi_row_clustered_delta`)

## Artifacts

- `multi-row-fold-test.log` — `cargo pgrx test pg18 --no-default-features
  --features pg18 multi_row_clustered`. Key line:
  `test result: ok. 1 passed; 0 failed`. This is a correctness regression, not a
  measurement run (no recall/latency/storage numbers), so a debug `.so` is
  appropriate; NFR-007 A/B benchmark evidence is not implicated by this packet.

## Cited result lines (request.md)

- `test tests::pg_test_ec_distann_fold_multi_row_clustered_delta ... ok`
- `test result: ok. 1 passed; 0 failed`
