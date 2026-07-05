# Artifact Manifest: Task 39 Coverage Lane Portability

- Head SHA: `071479bac0d0ea7b05d6da3f195df88d7f8a2781`
- Task bucket: `reviews/task-39/009-coverage-portability`
- Lane: Task 39 local coverage lane
- Fixture/storage format/rerank mode: not applicable
- Surface isolation: local pure-Rust coverage over `ecaz-cli` and `hardening/careful`
- Timestamp: `2026-05-19T00:28:34Z`

## Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| `make-coverage.log` | `make coverage` | Passed; `ecaz-cli` 355 tests and `hardening/careful` 157 tests completed under `cargo-llvm-cov`; wrote coverage reports. |
| `coverage-summary.txt` | copied from `target/quality/coverage/summary.txt` | Merged summary for root and careful coverage. |
| `coverage-root-summary.txt` | copied from `target/quality/coverage/root-summary.txt` | Root workspace summary. |
| `coverage-careful-summary.txt` | copied from `target/quality/coverage/careful-summary.txt` | `hardening/careful` summary. |
| `coverage-baseline-check.log` | `make coverage-baseline-check` | Passed: `coverage baseline complete for 40 critical paths`. |
| `coverage-delta-check.log` | `scripts/check_coverage_delta.sh target/quality/coverage/summary.txt fixtures/quality/coverage-baseline.tsv` | Passed; all baseline paths at or above allowed floors. |
| `careful-lib-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | Passed: 157 tests. |
| `bash-n-hardening.log` | `bash -n scripts/hardening.sh` | Shell syntax clean. |
| `git-diff-check.log` | `git diff --check` | Clean. |

## Key Result Lines

`make-coverage.log`:

- `test result: ok. 355 passed`
- `test result: ok. 157 passed`
- `coverage summary: target/quality/coverage/summary.txt`
- `coverage json: target/quality/coverage/coverage.json`

`coverage-delta-check.log`:

- `coverage ok: quant/simd.rs actual=95.18 baseline=94.59`
- `coverage ok: storage/page.rs actual=76.57 baseline=76.57`
- `coverage ok: am/ec_diskann/build.rs actual=0.00 baseline=0.00`
- `coverage ok: am/common/cost.rs actual=0.00 baseline=0.00`
