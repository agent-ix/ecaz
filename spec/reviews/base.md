---
id: SR-001
title: "base checklist review of the ec_distann physical hash-shard specification"
type: SpecReview
analysis: base
scope: "spec/spec.md; FR-075..FR-083; NFR-014, NFR-016..NFR-020; spec/tests.md TC-037..TC-044, TC-049..TC-050"
review_set: base
---

## Summary

Base re-review of the physical FR-078 handoff and FR-082 publication contract,
including automated ID/link/EARS checks and the six test-coverage rules. Quire
reports 244/244 grammar-clean documents, zero EARS findings, and no structural or
link errors; FR/NFR artifact IDs are unique and contiguous through FR-083 and
NFR-020. The normative contract defects found during review were repaired in
place. The requested `/spec-matrix` pass then added criterion-level traces,
boundary/error/state/fault coverage, a true physical-topology integration row,
and unique test identifiers. Implementation and decision-grade evidence remain
open; the specification review itself has no unresolved physical-contract defect.

## Findings

| ID      | Severity | Summary                          | Refs   |
| ------- | -------- | -------------------------------- | ------ |
| FND-001 | medium | ACCEPTED GAP: the research lane still traces StR-008 directly to FR-075..FR-083 without a US layer; this pass was explicitly scoped to the physical FR/NFR contract, so no retroactive user-story family was invented | StR-008, FR-075 |
| FND-002 | low | RESOLVED: FR-078 and FR-079 now enumerate stable handoff, placement, schema, generation, materialization, and internal error categories with mutation/partial-row outcomes | FR-078, FR-079 |
| FND-003 | medium | RESOLVED BEFORE THIS PASS: FR-080 documents the single-shard degenerate head-sample case; no new action | FR-080, FR-077 |
| FND-004 | low | PARTIALLY RESOLVED: FR-078 is now an explicit prerequisite of NFR-017/NFR-018/NFR-020, while the family-root StR-008 edge remains intentionally compact | StR-008, FR-078, NFR-017..NFR-020 |
| FND-005 | low | OPEN, PRE-EXISTING: TC-043 still spans M3 and M5 DML behavior; it does not block the physical handoff task but weakens milestone-specific DML closeout | spec/tests.md, FR-083 |
| FND-006 | high | RESOLVED: the former one-line handoff now defines locator-free canonical entry/batch formats, exact endpoint signatures, SHA-256 identities, owner-only streaming, an 8 MiB/one-in-flight bound, idempotent receipts, WAL resume, full frozen source-row storage, and replica/tombstone rejection | FR-076, FR-078 |
| FND-007 | high | RESOLVED: query-visible cluster publication now has Building/Ready/Published/Retired/Aborted states, a receipt-bearing canonical manifest, commit-only decision, participant-first/coordinator-pointer-last linearization, and deterministic recovery at every crash boundary | FR-082, NFR-020 |
| FND-008 | high | RESOLVED BY `/spec-matrix`: TC-040/TC-042/TC-044/TC-050 now enumerate every amended AC and constraint plus stable errors, 8 MiB boundaries, sequence/replay permutations, topology shapes, lifecycle transitions, crash boundaries, format fixtures, and the true three-instance integration surface; all remain Planned until packeted evidence lands | FR-076..FR-079, FR-082, NFR-014, NFR-016, NFR-020, spec/tests.md |
| FND-009 | medium | RESOLVED BY `/spec-matrix`: the benchmark-suite case is now uniquely TC-049, SPIRE retains TC-020, TC-050 owns DistANN format discipline, and TC-045..TC-048 remain reserved for Task 173 | spec/tests.md |
| FND-010 | medium | RESOLVED: row materialization no longer trusts caller-selected send functions or live remote base-table TIDs; schema fingerprints, local catalog resolution, full source-row payloads, unsupported type/system-column outcomes, and endpoint privileges are normative | FR-076, FR-078, FR-079, NFR-014 |
| FND-011 | medium | RESOLVED: stable-vec_id UPDATE now appends a complete replacement row/record and atomically redirects the owner directory, reconciling DML with frozen build-time row-tier immutability | FR-082, FR-083 |
| FND-012 | low | RESOLVED: NFR-014 was returned from APPROVED to PROPOSED because this pass materially broadened its security and operations contract to EC_DISTANN | NFR-014 |
| FND-013 | high | RESOLVED DURING IMPLEMENTATION DECOMPOSITION: the original handoff carried only a codec kind, leaving trained GroupedPQ owners unable to prepare/score identically; FR-078 now defines and digests a versioned generation descriptor carrying the complete trained codec artifact and row-schema descriptor, with owner-side no-retraining parity in TC-040/TC-050 | FR-078, TC-040, TC-050 |
| FND-014 | high | RESOLVED DURING IMPLEMENTATION DECOMPOSITION: retention counts had no cluster scan-pin protocol, so old generations could be reclaimed between remote calls; FR-082 now defines durable idempotent UUID pin/unpin, partial-pin cleanup, cancel/restart cleanup, participant-restart behavior, pin conflicts, and audited wedged-token force-retire | FR-082, NFR-020, TC-042 |
| FND-015 | high | RESOLVED DURING IMPLEMENTATION DECOMPOSITION: the specs lacked a metadata-only control-index mode, coordinator build entry, durable secret-referenced roster registry, and physical topology inspection endpoint; FR-075/FR-078 now pin those surfaces and forbid legacy/replicated fallback | FR-075, FR-078, NFR-014, TC-040, TC-044 |
| FND-016 | high | RESOLVED DURING IMPLEMENTATION DECOMPOSITION: a fingerprint-only topology endpoint could not inspect Ready because the final receipt-bearing manifest/fingerprint does not exist until all owners seal; FR-078 now separates build-id-selected generation topology for the pre-decision gate from fingerprint-selected epoch topology for Published/Retired diagnostics and benchmarks | FR-078, FR-082, TC-040, TC-044 |
| FND-017 | medium | RESOLVED DURING IMPLEMENTATION DECOMPOSITION: FR-076-AC-6 incorrectly claimed dimension independence while every current codec stride grows with dimension; the AC now pins the actual `20 + S + R×8 + R×S` formula and the intended invariant (no additional full-precision `4×dimension` field) | FR-076-AC-6, TC-037 |
| FND-018 | high | RESOLVED DURING IMPLEMENTATION DECOMPOSITION: ADR-085/NFR-018 used a 4-bit/768-byte code to describe the landed 1-bit RaBitQ default, overstating the default graph record from the actual 7,008 bytes (~1.14× raw at dim=1536/R=32) to ~4×; the arithmetic now matches `DISTANN_RABITQ_BITS=1` while leaving the measured physical storage gate authoritative | ADR-085 D1/D7, NFR-018, FR-076-AC-6 |
| FND-019 | high | RESOLVED DURING IMPLEMENTATION DECOMPOSITION: NFR-019 and ADR-085 claimed records, exact reranks, and final materializations were equal, contradicting FR-079's tombstone skip and its separate final payload endpoint; the bound is now graph expansions ≤BW×H, exact-vector reads ≤live expansions, payload reads ≤k, and total row-tier reads ≤BW×H+k with separate counters | FR-079, NFR-019, ADR-085 D11, TC-041/TC-044 |
