# Packet 040 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `diagnostics-helpers-extract-focused-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | `test result: ok. 508 passed; 0 failed` |
| `coverage/summary.txt` (+ JSON) | `make coverage COVERAGE_OUTPUT_DIR=…/artifacts/coverage` | `diagnostics_helpers.rs 100.00%`, `diagnostics.rs 0.00%` |
| `coverage-delta-check.log` | `scripts/check_coverage_delta.sh …` (no `--changed-files`, so every baseline row checked) | both diagnostics paths green at baselines |
| `coverage-baseline-check.log` | `scripts/check_coverage_baseline_complete.sh` | `coverage baseline complete for 41 critical paths` |
| `changed-files.txt` | hand-written | two source paths whose baseline this packet ratchets |
