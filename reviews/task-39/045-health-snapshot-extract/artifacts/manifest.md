# Packet 045 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `health-snapshot-extract-focused-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | `test result: ok. 529 passed; 0 failed` |
| `coverage/summary.txt` (+ JSON) | `make coverage` | `diagnostics_helpers.rs 100%` (284 lines), `diagnostics.rs 0%` (485 lines) |
| `coverage-delta-check.log` | `scripts/check_coverage_delta.sh …` | every baseline row green |
| `coverage-baseline-check.log` | `scripts/check_coverage_baseline_complete.sh` | `coverage baseline complete for 42 critical paths` |
| `changed-files.txt` | hand-written | two source paths whose extraction this packet ratchets |
