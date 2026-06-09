# Task 94 Local Readiness Refresh

Generated: 2026-06-09

Head: `a2a65a4dbe4b81614e52dac6a94572a077599999`

This refresh supersedes the packet 011 inventory by including packets 012 and
013. It does not claim final Task 94 completion because reviewer acceptance for
later packets and approved Graviton 4 / benchmark evidence remain pending.

## Packet Index

| Packet | Scope | Status |
| --- | --- | --- |
| 001 | Phase 1 design, layout audit, bench-suite emitter plan | Reviewer approved |
| 002 | Scalar grouped-PQ block reference | Reviewer approved |
| 003 | NEON backend | Reviewer approved |
| 004 | SVE/SVE2 backend | Reviewer approved |
| 005 | AVX2 backend | Reviewer approved |
| 006 | IVF grouped-PQ candidate-batch registration | Request pending reviewer feedback |
| 007 | DiskANN and HNSW grouped-PQ codec registration | Request pending reviewer feedback |
| 008 | Suite result extraction for `[block-kernel-counters]` rows | Request pending reviewer feedback |
| 009 | DiskANN traversal-level grouped-PQ prefilter batching | Request pending reviewer feedback |
| 010 | Task file status and module-path cleanup | Request pending reviewer feedback |
| 011 | Local readiness matrix through packet 010 | Request pending reviewer feedback |
| 012 | Shared AM grouped-PQ prevalidation before score writes | Request pending reviewer feedback |
| 013 | Local readiness/status refresh through packet 012 | This packet |

## Local Acceptance Evidence

| Requirement | Local evidence | Status |
| --- | --- | --- |
| ADR-076 module layout with scalar, NEON, SVE/SVE2, AVX2 | `src/quant/grouped_pq_block/{mod,scalar,neon,sve,avx2}.rs`; packets 002-005 | Locally implemented; packets 002-005 approved |
| Scalar reference bit-exact vs pre-kernel scorer | Packet 002 `cargo test grouped_pq_block --lib`; packet 007 broader `cargo test grouped_pq --lib` | Locally satisfied |
| SIMD backend parity/tolerance | Packet 003 NEON, packet 004 SVE/SVE2, packet 005 AVX2; local Intel AVX2 executes real AVX2 assertions | Locally satisfied where host supports ISA; Graviton runtime evidence pending |
| Width gating: batches >=32 use block kernel, tails scalar | `score_grouped_pq_batch_for` and grouped-PQ batch tests in packets 006, 007, 009, 012 | Locally satisfied |
| Shape mismatches fail before scoring and before counter increments | Packet 012 malformed candidate-33 regression keeps all output scores at sentinel and records no counters | Locally satisfied |
| Counter attribution under `(surface, quant, isa)` plus scalar tails | Candidate-batch tests in packets 006, 007, 009, 012 verify block rows and scalar-tail rows | Locally satisfied for local/fallback rows |
| IVF grouped-PQ registration through `QuantCodec::score_ip_batch` | Packet 006 | Locally implemented; review pending |
| DiskANN grouped-PQ registration through `QuantCodec::score_ip_batch` | Packet 007 codec path and packet 009 traversal prefilter path | Locally implemented; review pending |
| HNSW grouped-PQ disposition | Packet 007 registers existing HNSW grouped-PQ scan codec batch path under `surface=hnsw` | Locally implemented beyond the original IVF/DiskANN minimum; review pending |
| Suite latency extraction preserves `[block-kernel-counters]` rows | Packet 008 parser test emits metric `block_kernel_counters` | Locally satisfied; review pending |
| Task-file path and packet-range reconciliation | Packets 010 and 013 update task/index pointers to approved `grouped_pq_block` path and latest packet range | Locally satisfied; review pending |
| Existing local grouped-PQ tests pass together | Packet 012 `cargo test grouped_pq_batch --lib` -> 7 passed; packet 009 broader `cargo test grouped_pq --lib` -> 34 passed | Locally satisfied |

## Pending External / Approved Evidence

These gates are intentionally still open because CI/AWS and benchmark runs
require explicit operator approval:

| Gate | Needed evidence |
| --- | --- |
| Graviton 4 SVE2 runtime dispatch | Real Graviton 4 packet showing grouped-PQ SVE/SVE2 dispatching `Isa::Sve2` and reporting measured runtime vector length verbatim |
| Graviton 4 direct counter row | `[block-kernel-counters]` row under `isa=sve2` for grouped-PQ plus scalar-tail row under `isa=scalar` |
| NEON forced-path runtime evidence | ARM packet showing NEON backend path when forced or otherwise selected |
| Benchmark recall equality | Approved `ecaz bench suite` packet comparing kernel-on/off recall for IVF and DiskANN grouped-PQ |
| Scoring-share latency gate | Approved benchmark packet with scoring-share speedup matrix by AM and ISA |
| End-to-end p50/p95/p99 closeout | Approved benchmark packet with latency deltas and no regression claim |
| Reviewer acceptance | Outside reviewer feedback/acceptance for packets 006-013 |
| Final status flip to complete | Only after reviewer acceptance and required host/benchmark evidence exists |

## Current Recommendation

Treat Task 94 as code-side locally ready for reviewer inspection, not complete.
Do not start Task 96 on this lane until Task 94 is accepted/landed or the
operator explicitly changes the one-branch-per-task sequencing rule.
