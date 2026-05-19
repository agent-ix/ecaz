# Packet 038 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `relation-store-error-paths-focused-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | `test result: ok. 500 passed; 0 failed` |
| `coverage/summary.txt` | `make coverage COVERAGE_OUTPUT_DIR=…/artifacts/coverage` | `relation_store.rs 58.10%` |
| `coverage-delta-check.log` | `scripts/check_coverage_delta.sh …` | green at new baseline |
| `coverage-baseline-check.log` | `scripts/check_coverage_baseline_complete.sh` | `coverage baseline complete for 40 critical paths` |
| `changed-files.txt` | hand-written | one source path whose baseline this packet ratchets |
