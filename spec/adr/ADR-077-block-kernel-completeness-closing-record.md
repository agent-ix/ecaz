---
id: ADR-077
title: "Block Kernel Completeness Closing Record"
status: PROPOSED
impact: Affects Tasks 87, 91, 92, 93-99, 101, 102, FR-014, FR-030, FR-032, FR-035, FR-038, NFR-007, and NFR-015.
date: 2026-06-10
---
# ADR-077: Block Kernel Completeness Closing Record

## Context

Since the last functional spec sync, Ecaz moved from a single TurboQuant
candidate-batch scorer toward a cross-AM, cross-quant block-kernel program:

- Task 87 introduced candidate batching and the first LUT32 TurboQuant scorer.
- Task 91 accepted `QuantCodec` as the shared scoring interface.
- Task 92 accepted the universal block-kernel layout, runtime ISA detection,
  counter surface, and scalar counter calibration methodology.
- Tasks 93-98 extend the pattern across TurboQuant, HNSW exact modes, RaBitQ,
  grouped-PQ, binary, IVF, and DiskANN scoring surfaces.
- Task 101 adds the universal width cascade so production dispatch accounts
  for real AM batch distributions instead of only exact block32 flushes.
- Task 102 tracks the remaining real-SIMD LUT32 completion boundary.

The spec needs one closing record that explains what is architectural policy,
what is evidence-backed, and what remains a deferred or pending measurement
cell.

## Decision

Use `QuantCodec` plus candidate batches plus the ADR-076 block-kernel pattern
as the accepted architecture for compressed-domain scan scoring.

The final target surface is:

1. every AM scan loop routes compressed-domain scoring through an
   index-local `QuantCodec` adapter;
2. every block-capable quant family owns scalar reference scoring, runtime ISA
   dispatch, width-cascade routing, shape prevalidation, and counter
   attribution under its codec implementation;
3. dispatch is reported by surface, quant kind, ISA label, and width bucket;
4. scalar anchors remain available for correctness, insert/link paths,
   disabled kernels, and unsupported widths; and
5. benchmark suites and reports preserve backend build profile, ISA, width
   bucket, scoring-share, and end-to-end latency fields before making
   performance claims.

This ADR does not claim that every kernel cell is complete. It records the
common architecture and the closing criteria for promoting the block-kernel
program from partial to complete.

## Completion Criteria

Task 99 or a successor packet may mark this ADR `ACCEPTED` only when the
evidence matrix records, for every current quant family and AM surface:

- scalar reference parity;
- supported ISA dispatch and explicit absent/deferred cells;
- width-cascade attribution for exact block32, partial/octet, and scalar
  remainder paths where the family supports them;
- recall preservation or an explicitly scoped non-recall scoring lane;
- scoring-share and end-to-end latency attribution; and
- backend build-profile provenance for every latency or recall claim.

Missing hardware, debug builds, non-production prototype kernels, and
unmeasured vector lengths must be labeled directly. Nearby measured cells must
not be used as substitutes.

## Current Evidence Boundaries

| Area | Current interpretation |
| --- | --- |
| `QuantCodec` architecture | Accepted by ADR-071 and ADR-072; active AMs route compressed-domain scoring through AM-local adapters. |
| Universal kernel pattern | Accepted by ADR-076; block32 base width, runtime ISA detection, scalar anchors, and counter attribution are architectural policy. |
| Width cascade | Accepted as production dispatch direction by Task 101; specs require width buckets and prevalidation before output mutation. |
| Task 99 matrix | Required before this ADR can become accepted; it is the row-level completeness gate for quant kind, AM surface, ISA, and width bucket. |
| Task 102 LUT32 | Real AVX2/NEON/SVE kernels landed; AVX2 evidence is packeted and approved (Task 102 packets 001-002). The remaining gate is Graviton 4 NEON/SVE2 hardware evidence and Task 102 closeout before LUT32 completion claims. |
| Task 103 Intel lane | Required before Intel completeness claims: int8_approx32 AVX2 kernel (AC1), tiled_lut32 retire/deprioritize disposition (AC2), hamming32 documented AVX2 skip (AC3), and rabitq32 Intel validation (AC4) per packet 001. |
| Graviton 4 evidence | Required for ARM production claims. Reports must include measured runtime vector-length labels such as `sve2-128` when making width-specific claims. |
| AVX-512 and Apple silicon | Deferred; not part of the Task 92/99 acceptance surface. |
| ADR-025 state | Remains outside this closure unless a successor ADR explicitly reopens that decision with benchmark evidence. |

## Consequences

- Functional specs describe compressed-domain scoring consistently across HNSW,
  IVF, DiskANN, SPIRE, and shared quantizer code.
- Benchmark packets must separate scoring-share wins from end-to-end latency
  wins and must preserve absent/deferred cells instead of leaving blank rows.
- AM code should not grow direct calls to ISA-specific scoring functions.
- New quant families enter through the same scalar-reference, batch-dispatch,
  width-cascade, counter, and reporting gates.
- This ADR stays `PROPOSED` until the Task 99-style matrix and remaining
  deferred cells are packeted.

## Related Decisions

- ADR-071: Unified quantizer interface across access methods.
- ADR-072: Index-local quantized codec adapters.
- ADR-076: Universal block kernel pattern.
- FR-014: SIMD acceleration and block-kernel scoring surface.
- FR-038: Configured benchmark suite runner.
- NFR-007: Benchmark provenance.
- NFR-015: Benchmark reporting standard.
