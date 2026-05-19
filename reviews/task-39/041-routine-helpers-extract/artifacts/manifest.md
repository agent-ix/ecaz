# Packet 041 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `routine-helpers-extract-focused-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | `test result: ok. 511 passed; 0 failed` |
| `coverage/summary.txt` (+ JSON) | `make coverage` | `routine_helpers.rs 100.00%`, `routine.rs 0.00%` |
| `coverage-delta-check.log` | `scripts/check_coverage_delta.sh …` | both routine paths green at baselines |
| `coverage-baseline-check.log` | `scripts/check_coverage_baseline_complete.sh` | `coverage baseline complete for 42 critical paths` |
| `changed-files.txt` | hand-written | two source paths whose baseline this packet ratchets |
