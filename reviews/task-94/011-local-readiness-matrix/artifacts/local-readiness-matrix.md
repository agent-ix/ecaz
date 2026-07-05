# Task 94 Local Readiness Matrix

Generated: 2026-06-09

Head: `d8b79b412`

This matrix summarizes local Task 94 evidence after packets 001-010. It does
not claim final Task 94 completion because Graviton 4 runtime/vector-length
evidence and benchmark closeout evidence have not been run.

## Packet Index

| Packet | Scope | Status |
| --- | --- | --- |
| 001 | Phase 1 design, layout audit, bench-suite emitter plan | Reviewer approved |
| 002 | Scalar grouped-PQ block reference | Reviewer approved |
| 003 | NEON backend | Reviewer approved |
| 004 | SVE/SVE2 backend | Reviewer approved |
| 005 | AVX2 backend | Reviewer approved |
| 006 | IVF grouped-PQ candidate-batch registration | Request pending reviewer feedback |
| 007 | DiskANN and HNSW grouped-PQ codec batch registration | Request pending reviewer feedback |
| 008 | Suite result extraction for `[block-kernel-counters]` rows | Request pending reviewer feedback |
| 009 | DiskANN traversal-level grouped-PQ prefilter batching | Request pending reviewer feedback |
| 010 | Task file status and module-path cleanup | Request pending reviewer feedback |

## Acceptance Matrix

| Requirement | Local evidence | Status |
| --- | --- | --- |
| ADR-076 module layout with scalar, NEON, SVE/SVE2, AVX2 | `src/quant/grouped_pq_block/{mod,scalar,neon,sve,avx2}.rs`; packets 002-005 | Locally implemented; packets 002-005 approved |
| Scalar reference bit-exact vs pre-kernel scorer | Packet 002 `cargo test grouped_pq_block --lib`; packet 007 broader `cargo test grouped_pq --lib` | Locally satisfied |
| SIMD backend parity/tolerance | Packet 003 NEON, packet 004 SVE/SVE2, packet 005 AVX2; local Intel AVX2 executes real AVX2 assertions | Locally satisfied where host supports ISA; Graviton runtime evidence pending |
| Width gating: batches >=32 use block kernel, tails scalar | `score_grouped_pq_batch_for` and grouped-PQ batch tests in packets 006, 007, 009 | Locally satisfied |
| Counter attribution under `(surface, quant, isa)` plus scalar tails | Candidate-batch tests in packets 006, 007, 009 verify `kernel_candidates=32`, `scalar_candidates=7` | Locally satisfied for local/fallback rows |
| IVF grouped-PQ registration through `QuantCodec::score_ip_batch` | Packet 006 | Locally implemented; review pending |
| DiskANN grouped-PQ registration through `QuantCodec::score_ip_batch` | Packet 007 codec path and packet 009 traversal prefilter path | Locally implemented; review pending |
| HNSW grouped-PQ disposition | Packet 007 registers existing HNSW grouped-PQ scan codec batch path under `surface=hnsw` | Locally implemented beyond the original IVF/DiskANN minimum; review pending |
| Suite latency extraction preserves `[block-kernel-counters]` rows | Packet 008 parser test emits metric `block_kernel_counters` | Locally satisfied; review pending |
| Task-file path reconciliation | Packet 010 updates task file and README from stale `pq_fastscan32` path to approved `grouped_pq_block` path | Locally satisfied; review pending |
| Existing local grouped-PQ tests pass together | Post-packet 009 local run: `cargo test grouped_pq --lib` -> 34 passed, 0 failed | Locally satisfied |
| Existing grouped-PQ pg_test surfaces pass | Same `cargo test grouped_pq --lib` matched `pg_test_pq_fastscan_binary_score_mode_bypasses_grouped_pq_scoring` and passed | Locally satisfied for matched local PG18 pg_test |

## Pending External / Approved Evidence

These items are intentionally not completed in this branch segment because the
user restricted CI and AWS/bench runs until explicit approval:

| Gate | Needed evidence |
| --- | --- |
| Graviton 4 SVE2 runtime dispatch | Real Graviton 4 packet showing grouped-PQ SVE/SVE2 test dispatching `Isa::Sve2` and reporting measured runtime vector length verbatim |
| Graviton 4 direct counter row | `[block-kernel-counters]` row under `isa=sve2` for grouped-PQ plus scalar-tail row under `isa=scalar` |
| NEON forced-path runtime evidence | ARM packet showing NEON backend path when forced or otherwise selected |
| Benchmark recall equality | Approved local/host `ecaz bench suite` packet comparing kernel-on/off recall for IVF and DiskANN grouped-PQ |
| Scoring-share latency gate | Approved local/host benchmark packet with scoring-share speedup matrix by AM and ISA |
| End-to-end p50/p95/p99 closeout | Approved local/host benchmark packet with latency deltas and no regression claim |
| Final status flip to complete | After reviewer accepts packets 006-010 and required host/benchmark evidence exists |

## Current Recommendation

Treat Task 94 as code-side locally ready for reviewer inspection, not complete.
Do not start Task 96 on this lane until Task 94 is accepted/landed or the
operator explicitly changes the one-branch-per-task sequencing rule.
