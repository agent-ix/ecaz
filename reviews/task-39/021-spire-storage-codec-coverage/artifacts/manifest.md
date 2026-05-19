# Artifact Manifest

Task bucket: `reviews/task-39/021-spire-storage-codec-coverage`

Code checkpoint: `a0afb04fe25d7087fb9cc7215704792f5e58133c`

Timestamp: 2026-05-19 America/Los_Angeles / 2026-05-19 UTC

Surface: Task 39 Spire storage codec coverage through the pgrx-free
`hardening/careful` harness.

Storage / index isolation: not applicable. This packet is pure Rust harness
coverage; it does not create PostgreSQL indexes or shared-table benchmark
surfaces.

## Artifacts

| Artifact | Command | Key Result |
| --- | --- | --- |
| `careful-spire-storage-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib spire -- --nocapture` | 101 passed, 0 failed. |
| `coverage/summary.txt` | `make coverage COVERAGE_OUTPUT_DIR=reviews/task-39/021-spire-storage-codec-coverage/artifacts/coverage` | 11 Spire storage codec rows raised above 0%. |
| `coverage/careful-summary.txt` | same coverage run | careful-only coverage summary. |
| `coverage/coverage.json` | same coverage run | merged raw cargo-llvm-cov JSON. |
| `coverage/careful-coverage.json` | same coverage run | careful-only raw cargo-llvm-cov JSON. |
| `coverage/root-summary.txt` | same coverage run | root `ecaz-cli` coverage summary. |
| `changed-files.txt` | packet-local ratchet input list | Lists the 11 Spire storage rows updated by this packet. |
| `coverage-baseline-check.log` | `make coverage-baseline-check` | `coverage baseline complete for 40 critical paths`. |
| `cargo-check-pg18-bench.log` | `cargo check --all-targets --no-default-features --features pg18,bench` | Passed with pre-existing warnings. |
| `git-diff-check.log` | `git diff --check` | No whitespace errors. |

## Key Lines Cited

```text
am/ec_spire/storage/assignment.rs      140  18  87.14%
am/ec_spire/storage/header.rs          100  18  82.00%
am/ec_spire/storage/helpers.rs         406  67  83.50%
am/ec_spire/storage/leaf_v1.rs          87   2  97.70%
am/ec_spire/storage/leaf_v2.rs          85  24  71.76%
am/ec_spire/storage/leaf_v2_parts.rs   427  96  77.52%
am/ec_spire/storage/local_store.rs     537 117  78.21%
am/ec_spire/storage/local_store_set.rs 171 100  41.52%
am/ec_spire/storage/routing_delta.rs   390  45  88.46%
am/ec_spire/storage/top_graph.rs       279  27  90.32%
am/ec_spire/storage/vec_id.rs          168  52  69.05%
```

```text
coverage baseline complete for 40 critical paths
```
