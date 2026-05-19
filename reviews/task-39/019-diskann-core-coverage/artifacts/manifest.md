# Artifact Manifest

Task bucket: `reviews/task-39/019-diskann-core-coverage`

Code checkpoint: `3a2c6f86aa1f85524a8c64ef6bdc0f5acfc56717`

Timestamp: 2026-05-19 America/Los_Angeles / 2026-05-19 UTC

Surface: Task 39 DiskANN core coverage through the pgrx-free
`hardening/careful` harness.

Storage / index isolation: not applicable. This packet is pure Rust harness
coverage; it does not create PostgreSQL indexes or shared-table benchmark
surfaces.

## Artifacts

| Artifact | Command | Key Result |
| --- | --- | --- |
| `careful-diskann-core-tests-rerun.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib diskann -- --nocapture` | 111 passed, 0 failed. |
| `careful-diskann-core-tests.log` | same focused test command before the final no-warning harness cleanup | 111 passed, 0 failed. Superseded by the rerun log above. |
| `coverage/summary.txt` | `make coverage COVERAGE_OUTPUT_DIR=reviews/task-39/019-diskann-core-coverage/artifacts/coverage` | `am/ec_diskann/build.rs`: 96.69% line coverage; `am/ec_diskann/scan.rs`: 96.95% line coverage. |
| `coverage/careful-summary.txt` | same coverage run | careful-only coverage summary. |
| `coverage/coverage.json` | same coverage run | merged raw cargo-llvm-cov JSON. |
| `coverage/careful-coverage.json` | same coverage run | careful-only raw cargo-llvm-cov JSON. |
| `coverage/root-summary.txt` | same coverage run | root `ecaz-cli` coverage summary. |
| `changed-files.txt` | `printf 'src/am/ec_diskann/build.rs\nsrc/am/ec_diskann/scan.rs\n'` | Ratchet input list for the two Task 39 baseline rows updated by this packet. |
| `coverage-baseline-check.log` | `make coverage-baseline-check` | `coverage baseline complete for 40 critical paths`. |
| `cargo-check-pg18-bench.log` | `cargo check --all-targets --no-default-features --features pg18,bench` | Passed with pre-existing warnings. |
| `git-diff-check.log` | `git diff --check` | No whitespace errors. |

## Key Lines Cited

```text
am/ec_diskann/build.rs  272  9  96.69%
am/ec_diskann/scan.rs   884  27 96.95%
am/ec_diskann/routine.rs 1544 1544 0.00%
```

```text
coverage baseline complete for 40 critical paths
```
