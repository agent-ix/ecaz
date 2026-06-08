# Task 87 Completion Audit

Head SHA: `b71376d81225a896b388010dcfb83e489613e98e`

Packet path: `reviews/task-87/015-phase6-closeout/`

This audit maps Task 87's explicit closeout and acceptance requirements to
current checked-in evidence. It treats packet 015 as a closeout request awaiting
outside reviewer response; it does not substitute for reviewer approval.

## Phase 6 Closeout Requirements

| Requirement | Evidence | Audit result |
| --- | --- | --- |
| All four AM slices reviewer-approved | SPIRE packet 003 shipped with tests. IVF packet 004 shipped with tests. DiskANN packet 005 Stop Condition is reinstated and accepted by packet 009 reviewer feedback. HNSW packet 006 shipped with tests. Packet 009 reviewer feedback says original phasing resumes and lists packets 003/004/006 as already shipped, with DiskANN packet 005 standing. | Mostly proven by packet 009 feedback for scope and prior slice artifacts. Packet 015 itself still awaits reviewer response. |
| Aggregate measurement comparison | `artifacts/aggregate-matrix.md` aggregates real10k, real50k, and real100k off/on results for SPIRE, IVF, and HNSW; DiskANN is represented by accepted Stop Condition. | Proven for the reviewer-approved post-walk-back matrix scope. |
| Closeout packet citing per-AM evidence | `request.md`, `manifest.md`, and `aggregate-matrix.md` cite packets 012, 013, 014 for matrix evidence, packet 005 for DiskANN, and packet 009 for Task 91 handoff. | Proven. |
| Status flip to complete referencing the closeout packet | `plan/tasks/87-candidate-batched-scoring-across-ams.md` status line references `reviews/task-87/015-phase6-closeout/`. | Proven in the working tree. |

## Acceptance Criteria

| Criterion | Evidence | Audit result |
| --- | --- | --- |
| 1. `CandidateBatch` abstraction lives in a shared module | `src/am/common/candidate_batch.rs`; exported through `src/am/common/mod.rs`. Packet 003 request and manifest record the shared abstraction and focused unit tests. | Proven. |
| 2. All four AMs either route batch-shaped quant scoring or have accepted Stop Condition | SPIRE route: packet 003 and `src/am/ec_spire/quantizer/mod.rs`. IVF route: packets 004 and 011 and `src/am/ec_ivf/quantizer.rs` / `src/am/ec_ivf/scan.rs`. HNSW route: packet 006 and `src/am/ec_hnsw/scan.rs`. DiskANN Stop Condition: packet 005, reinstated/accepted by packet 009 feedback. | Proven for current Task 87 scope after packet 009 walk-back. |
| 3. Per-AM real-corpus suite evidence ships in each slice packet | Phase 6 evidence is split across packet 012 real10k, packet 013 real50k, and packet 014 real100k. Packet 015 aggregates them. DiskANN uses packet 005 Stop Condition and Task 91 handoff. | Proven for measured AMs; DiskANN covered by accepted Stop Condition. |
| 4. Existing pg_test surfaces pass across all four AMs | Packet 003 includes CandidateBatch and SPIRE quantizer focused tests. Packet 004 includes IVF quantizer and scan tests. Packet 006 includes HNSW scan tests. Packet 010/011 include later toggle/reachability focused tests. DiskANN packet 005 is source-audit only because no code changed for DiskANN. | Proven for code-touched AM slices; no DiskANN test evidence required by the Stop Condition packet because no DiskANN code changed. |
| 5. Closeout packet cites per-AM evidence and aggregate matrix | Packet 015 request and manifest cite the aggregate matrix and source packets. | Proven. |
| 6. Task status flips to complete only referencing the closeout packet | Status line is `complete (2026-06-08; see reviews/task-87/015-phase6-closeout/)`. | Proven in the working tree. |

## Validation Gates

| Gate | Evidence | Audit result |
| --- | --- | --- |
| Recall@10 byte-equal | Packet 015 aggregate matrix shows unchanged recall in all nine measured SPIRE/IVF/HNSW off/on cells. | Proven for measured AMs; DiskANN covered by Stop Condition. |
| Scoring-share latency measurably faster | SPIRE pipeline emits pipeline counters and shows consistent end-to-end gains. HNSW/IVF instrumentation does not emit isolated scoring-share counters; packet 015 explicitly does not claim the original universal 2x scoring-share gate. | Partially proven. The closeout relies on reviewer-approved structural-slice carve-outs and measurement transparency. |
| End-to-end p50/p95/p99 improves at every cell | SPIRE improves all cells. IVF real50k p50/p95 is flat/slightly worse. HNSW real100k p50 is effectively flat. | Not universally met. Packet 015 documents these misses and asks reviewer to accept the structural-slice closeout under the packet 001 B4 carve-out and packet 009 scope walk-back. |
| Storage unchanged | Off/on cells use same indexes and flip only session GUCs; aggregate matrix records one storage value per AM/corpus. | Proven. |
| Existing pg_test surfaces pass for touched AMs | Packet-local test logs in packets 003, 004, 006, 010, and 011. | Proven for touched code slices. |
| No new unsafe outside existing SIMD kernel boundary | CandidateBatch routes are safe Rust over borrowed/owned payload views; no packet claims new SIMD kernel or new unsafe. | Supported by packet design and code shape; no new unsafe audit artifact was generated in packet 015. |
| Suite-driven per FR-038 | Packet 012 checked in `phase6-suite.json` and suite audit/dry-run artifacts; packets 012-014 include suite-run evidence. | Proven for Phase 6 measurement. |

## Current Completion State

The implementation and closeout evidence are pushed. Packet 015 is the current
closeout request and contains the task status flip. The remaining governance
step is outside reviewer response to packet 015, especially because the original
universal scoring-share and every-cell latency gates are not universally met and
the closeout intentionally relies on accepted structural carve-outs rather than
claiming those gates as fully satisfied.
