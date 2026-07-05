# Artifact Manifest

Task bucket: `reviews/task-39/020-quant-family-coverage`

Code checkpoint: `7817f61d80992c27b78d8b7144209c19c979d498`

Timestamp: 2026-05-19 America/Los_Angeles / 2026-05-19 UTC

Surface: Task 39 quant family coverage through the pgrx-free
`hardening/careful` harness.

Storage / index isolation: not applicable. This packet is pure Rust harness
coverage; it does not create PostgreSQL indexes or shared-table benchmark
surfaces.

## Artifacts

| Artifact | Command | Key Result |
| --- | --- | --- |
| `careful-quant-family-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib quant_ -- --nocapture` | 4 passed, 0 failed. |
| `coverage/summary.txt` | `make coverage COVERAGE_OUTPUT_DIR=reviews/task-39/020-quant-family-coverage/artifacts/coverage` | `quant/mod.rs`: 100.00% line coverage. |
| `coverage/careful-summary.txt` | same coverage run | careful-only coverage summary. |
| `coverage/coverage.json` | same coverage run | merged raw cargo-llvm-cov JSON. |
| `coverage/careful-coverage.json` | same coverage run | careful-only raw cargo-llvm-cov JSON. |
| `coverage/root-summary.txt` | same coverage run | root `ecaz-cli` coverage summary. |
| `changed-files.txt` | `printf 'src/quant/mod.rs\n'` | Ratchet input list for the Task 39 baseline row updated by this packet. |
| `coverage-baseline-check.log` | `make coverage-baseline-check` | `coverage baseline complete for 40 critical paths`. |
| `cargo-check-pg18-bench.log` | `cargo check --all-targets --no-default-features --features pg18,bench` | Passed with pre-existing warnings. |
| `git-diff-check.log` | `git diff --check` | No whitespace errors. |

## Key Lines Cited

```text
quant/mod.rs  17  0  100.00%
```

```text
coverage baseline complete for 40 critical paths
```
