# Packet 039 Artifacts Manifest

| Artifact | Command | Key result |
| --- | --- | --- |
| `closeout-focused-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | **500 passed, 0 failed** |
| `coverage/summary.txt` (+ JSON) | `make coverage COVERAGE_OUTPUT_DIR=/tmp/task39-closeout-cov` copied here | full coverage summary at closeout head |
| `coverage-delta-check.log` | `scripts/check_coverage_delta.sh /tmp/.../summary.txt fixtures/quality/coverage-baseline.tsv` (no `--changed-files`, so checks every baseline path) | every baseline path green |
| `coverage-baseline-check.log` | `scripts/check_coverage_baseline_complete.sh fixtures/quality/coverage-baseline.tsv` | `coverage baseline complete for 40 critical paths` |
| `test-quality-ci-audit.log` | `make test-quality-ci-audit` | `Task 39 CI audit passed` |

Provenance: task bucket `reviews/task-39/`, packet
`039-task39-closeout`, head SHA at packet commit time. Surface =
shadow-careful + ecaz-cli coverage lane plus `scripts/check_task39_quality_ci.py`
audit against `.github/workflows/ci.yml`.
