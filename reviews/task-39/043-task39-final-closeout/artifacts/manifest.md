# Packet 043 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `closeout-focused-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | **513 passed, 0 failed** |
| `coverage/summary.txt` (+ JSON) | `make coverage COVERAGE_OUTPUT_DIR=…/artifacts/coverage` | full coverage snapshot at session close |
| `coverage-delta-check.log` | `scripts/check_coverage_delta.sh …/summary.txt fixtures/quality/coverage-baseline.tsv` (full baseline) | every baseline row green |
| `coverage-baseline-check.log` | `scripts/check_coverage_baseline_complete.sh` | `coverage baseline complete for 42 critical paths` |
| `test-quality-ci-audit.log` | `make test-quality-ci-audit` | `Task 39 CI audit passed` |

Provenance: task bucket `reviews/task-39/`, packet
`043-task39-final-closeout`, head SHA at packet commit. Surface =
shadow-careful + ecaz-cli coverage lane plus
`scripts/check_task39_quality_ci.py` audit against
`.github/workflows/ci.yml`.
