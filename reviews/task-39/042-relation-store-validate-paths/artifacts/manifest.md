# Packet 042 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `validate-paths-focused-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | `test result: ok. 513 passed; 0 failed` |
| `coverage/summary.txt` (+ JSON) | `make coverage` | `relation_store.rs 58.52%` |
| `coverage-delta-check.log` | `scripts/check_coverage_delta.sh …` | relation_store row green at new baseline |
| `coverage-baseline-check.log` | `scripts/check_coverage_baseline_complete.sh` | `coverage baseline complete for 42 critical paths` |
| `changed-files.txt` | hand-written | one source path whose baseline this packet ratchets |
