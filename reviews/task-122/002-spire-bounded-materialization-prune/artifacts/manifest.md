# Artifact Manifest

- Head SHA: `6efa5997de16463f03eb3ddd382a12b2ff00c389`
- Task bucket: `reviews/task-122`
- Packet path: `reviews/task-122/002-spire-bounded-materialization-prune`
- Timestamp: 2026-06-26
- Lane / fixture / storage format / rerank mode: SPIRE local unit validation; TurboQuant bounded candidate retention path.
- Isolation: unit test only; no index/table benchmark run.
- Commands:
  - `cargo test -p ecaz --lib --no-default-features --features pg18 collect_scan_placement_diagnostics_reports_candidate_truncation`
- Key result lines:
  - `running 1 test`
  - `test am::ec_spire::scan::tests::collect_scan_placement_diagnostics_reports_candidate_truncation ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2219 filtered out; finished in 0.05s`
