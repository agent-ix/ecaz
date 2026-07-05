# Packet 044 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `helpers-expansion-focused-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | `test result: ok. 528 passed; 0 failed` |
| `coverage/summary.txt` (+ JSON) | `make coverage` | `diagnostics_helpers.rs 100%` (220 lines), `routine_helpers.rs 100%` (127 lines), `relation_store.rs 58.66%` |
| `coverage-delta-check.log` | `scripts/check_coverage_delta.sh …` | every ratcheted row green |
| `coverage-baseline-check.log` | `scripts/check_coverage_baseline_complete.sh` | `coverage baseline complete for 42 critical paths` |
| `changed-files.txt` | hand-written | five source paths whose baseline this packet ratchets |
