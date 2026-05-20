# Packet 037 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `relation-store-chain-focused-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | `test result: ok. 497 passed; 0 failed` |
| `coverage/summary.txt` | `make coverage COVERAGE_OUTPUT_DIR=…/artifacts/coverage` | `relation_store.rs 56.53%`, `page.rs 83.15%` |
| `coverage-delta-check.log` | `scripts/check_coverage_delta.sh …/summary.txt fixtures/quality/coverage-baseline.tsv …/changed-files.txt` | both ratcheted rows green at new baseline |
| `coverage-baseline-check.log` | `scripts/check_coverage_baseline_complete.sh fixtures/quality/coverage-baseline.tsv` | `coverage baseline complete for 40 critical paths` |
| `changed-files.txt` | hand-written | two source paths whose baseline this packet ratchets |

Provenance: task bucket `reviews/task-39/`, packet
`037-relation-store-chain`, head SHA at packet commit time; surface =
pure-Rust tests in `hardening/careful` driving the Phase-1
backing-page emulator from packet 035.
