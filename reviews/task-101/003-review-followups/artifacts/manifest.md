# Task 101 Packet 003 Artifact Manifest

- head SHA: `a808ee5c0c6ecd7a3fac9d8fbcf38bfd77dfa3cf`
- task bucket: `reviews/task-101/`
- packet path: `reviews/task-101/003-review-followups/`
- timestamp: `2026-06-10T16:23:30-07:00`
- lane: local PG18 / Intel AVX2
- surface isolation: code checks are source-level; width evidence copied from Task 94 packet 027 two-step rerun

## Code Checks

Artifacts:
- `cargo-test-candidate-batch.log`
- `cargo-test-grouped-pq.log`
- `cargo-test-lut32.log`

Commands:
- `script -q -c "cargo test --lib candidate_batch --no-default-features --features pg18" reviews/task-101/003-review-followups/artifacts/cargo-test-candidate-batch.log`
- `script -q -c "cargo test --lib grouped_pq --no-default-features --features pg18" reviews/task-101/003-review-followups/artifacts/cargo-test-grouped-pq.log`
- `script -q -c "cargo test --lib lut32 --no-default-features --features pg18" reviews/task-101/003-review-followups/artifacts/cargo-test-lut32.log`

Key results:
- `candidate_batch`: `19 passed; 0 failed; 2067 filtered out`
- `grouped_pq`: `35 passed; 0 failed; 2051 filtered out`
- `lut32`: `6 passed; 0 failed; 2080 filtered out`

## Width Histogram Evidence

Copied artifacts from Task 94 packet 027:
- `task94-027-latency-ivf-pqfastscan-10k-batch-on.log`
- `task94-027-latency-diskann-pqfastscan-50k-grouped-pq.log`
- `task94-027-suite-report.log`
- `task94-027-results-report.jsonl`

Source packet:
- `reviews/task-94/027-latency-width-rerun/`

Key result lines:
- IVF grouped-PQ, `nprobe=32`: `width_lt8=15 width_8_15=20 width_16_31=40 width_ge32=9605`, `scalar_candidates=0`
- IVF grouped-PQ, `nprobe=64`: `width_lt8=0 width_8_15=0 width_16_31=500 width_ge32=19500`, `scalar_candidates=0`
- DiskANN grouped-PQ, `list_size=64`: `width_lt8=970 width_8_15=2531 width_16_31=4873 width_ge32=201`, `scalar_candidates=0`
- DiskANN grouped-PQ, `list_size=128`: `width_lt8=3003 width_8_15=5295 width_16_31=7515 width_ge32=206`, `scalar_candidates=0`

## Task 87 Compat Line

Task 101 packet 002 already contains direct Task 87 compat lines in the exact-mode latency logs:
- `reviews/task-101/002-hnsw-exact-mode-counter-evidence/artifacts/latency-full_lut-1k-kernel-on.log`
- `reviews/task-101/002-hnsw-exact-mode-counter-evidence/artifacts/latency-tiled_lut-1k-kernel-on.log`
- `reviews/task-101/002-hnsw-exact-mode-counter-evidence/artifacts/latency-int8_approx-1k-kernel-on.log`

Relevant observations:
- `quant=turboquant` contributes to `lut32_flushes` under the Task 87 compat line.
- `quant=turboquant_tiled_lut` and `quant=turboquant_int8` have direct block-kernel rows but keep `lut32_flushes=0`, preserving the Task 87 compat aggregation boundary.
