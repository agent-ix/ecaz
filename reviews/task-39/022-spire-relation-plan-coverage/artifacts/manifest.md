# Artifact Manifest

Task bucket: `reviews/task-39/022-spire-relation-plan-coverage`

Code checkpoint: `4b21c27eb5170c1087395034e566a3a8ac399da1`

Timestamp: 2026-05-19 America/Los_Angeles / 2026-05-19 UTC

Surface: Task 39 Spire relation-plan coverage through the pgrx-free
`hardening/careful` harness.

Storage / index isolation: not applicable. This packet is pure Rust harness
coverage; it does not create PostgreSQL indexes or shared-table benchmark
surfaces.

## Artifacts

| Artifact | Command | Key Result |
| --- | --- | --- |
| `careful-spire-relation-plan-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib local_store_relation -- --nocapture` | 3 passed, 0 failed. |
| `coverage/summary.txt` | `make coverage COVERAGE_OUTPUT_DIR=reviews/task-39/022-spire-relation-plan-coverage/artifacts/coverage` | `am/ec_spire/storage/relation_plan.rs`: 82.98% line coverage. |
| `coverage/careful-summary.txt` | same coverage run | careful-only coverage summary. |
| `coverage/coverage.json` | same coverage run | merged raw cargo-llvm-cov JSON. |
| `coverage/careful-coverage.json` | same coverage run | careful-only raw cargo-llvm-cov JSON. |
| `coverage/root-summary.txt` | same coverage run | root `ecaz-cli` coverage summary. |
| `changed-files.txt` | packet-local ratchet input list | Lists relation-plan and adjacent local-store rows considered by this ratchet. |
| `coverage-baseline-check.log` | `make coverage-baseline-check` | `coverage baseline complete for 40 critical paths`. |
| `cargo-check-pg18-bench.log` | `cargo check --all-targets --no-default-features --features pg18,bench` | Passed with pre-existing warnings. |
| `git-diff-check.log` | `git diff --check` | No whitespace errors. |

## Key Lines Cited

```text
am/ec_spire/storage/relation_plan.rs  94  16  82.98%
```

```text
coverage baseline complete for 40 critical paths
```
