# Packet 036 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `coverage-pushes-focused-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | `test result: ok. 488 passed; 0 failed` |
| `coverage/summary.txt` | `make coverage COVERAGE_OUTPUT_DIR=…/artifacts/coverage` | `vec_id.rs 94.64%`, `leaf_v2.rs 95.29%`, `local_store_set.rs 88.89%` |
| `coverage-delta-check.log` | `scripts/check_coverage_delta.sh …/summary.txt fixtures/quality/coverage-baseline.tsv …/changed-files.txt` | all three baseline rows green |
| `coverage-baseline-check.log` | `scripts/check_coverage_baseline_complete.sh fixtures/quality/coverage-baseline.tsv` | `coverage baseline complete for 40 critical paths` |
| `changed-files.txt` | hand-written | three source paths whose baseline this packet ratchets |

Provenance: task bucket `reviews/task-39/`, packet
`036-coverage-pushes`, head SHA at packet commit; surface = pure-Rust
tests in `hardening/careful` over the production sources, no live
PostgreSQL.
