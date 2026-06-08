# Task 87 Phase 7 Completion Audit

Head SHA: `e6b14dfd68c58f3b785179f730b786bc0599fd40`

Packet path: `reviews/task-87/023-phase7-methodology-and-closeout/`

Packet 023 supersedes packet 015 as final closeout. Packet 015 remains the
Phase 6 plumbing milestone and baseline.

## Reviewer Feedback Addressed

| Feedback | Evidence | Audit result |
| --- | --- | --- |
| Packet 015 seq 02 reopened Task 87 for the 32-block kernel. | `src/quant/lut32.rs`, packet 016 request, packet 016 feedback approval. | Addressed. |
| Route SPIRE and IVF TurboQuant no-QJL 4-bit through the kernel where batch widths justify it. | IVF route through common `CandidateBatch` scorer; SPIRE leaf-level batching in commit `56299f37fdce4300dfba11ab5b63f21284adb6bd`; packet 021 reviewer approval. | Addressed for measured TurboQuant cells. |
| Land scoring-share counters. | Packets 017 and 019 add and expose counters; packets 021/022 cite counter lines. | Addressed for on-path candidate-batch scorer scope. |
| Decide off-path counter vs methodology reframe before closeout. | `artifacts/methodology.md` selects the packet 021 allowed methodology reframe. | Addressed. |
| Investigate HNSW zero counters. | `artifacts/hnsw-reloptions-list.log`, packet 021/022 HNSW counter logs, and `artifacts/methodology.md`. | Addressed with explicit Phase 7 HNSW stop condition. |
| Commit and request packet 022. | `reviews/task-87/022-phase7-50k-100k-counter-suite/`, commit `cbb4d388b`. | Addressed. |

## Acceptance Criteria

| Criterion | Evidence | Audit result |
| --- | --- | --- |
| `CandidateBatch` abstraction lives in a shared module. | `src/am/common/candidate_batch.rs`, packet 003 and later route packets. | Proven. |
| All four AMs route batch-shaped quant scoring or have accepted Stop Condition. | SPIRE packets 003/021; IVF packets 004/011/021/022; HNSW packet 006 plus Phase 7 stop condition in this packet; DiskANN packets 005/009 and Task 91 handoff. | Proven for current Task 87 scope. |
| Per-AM real-corpus suite evidence ships. | Phase 6 packets 012-015; Phase 7 packets 021-023 for SPIRE/IVF/HNSW probe. | Proven for measured AMs; DiskANN covered by accepted Stop Condition. |
| Existing pg_test surfaces pass for touched AMs. | Packet 021 focused tests: SPIRE quantizer `15 passed`, SPIRE scan `99 passed`, common candidate batch `4 passed`; earlier packets cover IVF/HNSW touched code. | Proven for touched Phase 7 code. |
| Closeout packet cites per-AM evidence and aggregate matrix. | This request plus `artifacts/aggregate-matrix.md`. | Proven. |
| Task status flips to complete referencing closeout packet. | `plan/tasks/87-candidate-batched-scoring-across-ams.md` now references `reviews/task-87/023-phase7-methodology-and-closeout/`. | Proven. |

## Phase 7 Validation Gates

| Gate | Evidence | Audit result |
| --- | --- | --- |
| Kernel lives under `src/quant/`. | `src/quant/lut32.rs`; packet 016 reviewer approval. | Proven. |
| Scalar differential coverage. | `lut32_matches_scalar_for_blocks_and_tail`; packet 016 reviewer notes byte-equality. | Proven. |
| No new unsafe without safety docs. | Packet 016 reviewer notes the kernel is safe Rust and no new unsafe. | Proven. |
| SPIRE reaches LUT32 for chunks >= 32. | Packet 021 counters: `4800/4800` LUT32 flushes and `1551640/1551640` candidates on real10k; packet 022 counters: `4800/4800` LUT32 flushes on real50k and real100k. | Proven for measured SPIRE TurboQuant cells. |
| IVF reaches LUT32 for posting-list chunks >= 32. | Packet 021 real10k counters: `7800/8000` LUT32 flushes, `1996800/2000000` LUT32 candidates; packet 022 real100k counters: `78200/78200` LUT32 flushes, `20000000/20000000` candidates. | Proven for measured TurboQuant cells. |
| Recall preserved. | Aggregate matrix recall off/on cells are equal for every measured routed row. | Proven. |
| End-to-end latency improves at touched cells. | Matrix shows p50/p95/p99 improvements for real10k IVF, real100k IVF, real50k SPIRE, real100k SPIRE; real10k SPIRE p50/p95 improve and p99 is flat at +0.4%. | Mostly proven with one documented flat p99 outlier. |
| Scoring-share gate directly measured. | On-path scorer counters are direct; off-path scalar scorer is not instrumented. `artifacts/methodology.md` reframes the gate per packet 021 feedback. | Proven for on-path scope; direct scalar-vs-LUT32 factor is not claimed. |
| HNSW gated by measurement. | HNSW counters are zero on real10k/50k/100k; reloptions show current profiles are not TurboQuant FullLut surfaces. | Closed by explicit Phase 7 stop condition. |
| DiskANN remains handed to Task 91. | Packets 005 and 009; task coordination section. | Proven. |
| Suite-driven per FR-038. | Packet 021 and packet 022 checked-in suite configs and packet-local artifacts. | Proven. |

## Current Completion State

Task 87 is complete against the final scoped claim: shared cross-AM
CandidateBatch plumbing; 32-candidate LUT kernel under `src/quant/`; SPIRE and
IVF TurboQuant no-QJL Phase 7 routing with packet-local real-corpus evidence;
HNSW Phase 7 stop condition for current non-TurboQuant-FullLut profiles; and
DiskANN handed off to Task 91 by accepted prior packets.
