# Task 208 review request: arm-blind storage retrospective

## Outcome

The Task 203 `T4 = ?` cells are closed. The sweep re-read the committed
ec_distann packet corpus and separates three materially different cases:

1. **INVALID:** Tasks 198/199 claimed identical arm storage while the candidate
   added a 1,659,518,976-byte coordinator traversal relation that the old
   pre-arm metric could not observe.
2. **QUALIFIED:** the other “identical/unchanged” statements describe a shared
   immutable physical generation, a bounded query/session cache, or a
   query-only switch. Those statements can remain as generation-scoped claims,
   but they are not evidence of equal total resident state and cannot satisfy
   NFR-021/NFR-022.
3. **SOUND:** Tasks 197, 204, and 205 contain storage evidence capable of
   representing the property they assert. Task 197 emitted the historical
   amplification row; Task 204 repaired per-arm/per-node attribution; Task 205
   consumed that corrected emitter across 10k/50k/100k. Task 205's original
   raw fixed-roster NFR-021 judgement is separately superseded by the normalized
   gate in packet 001.

The detailed packet-level classifications and the complete task-level T4 ledger
are in `artifacts/retrospective-sweep.md`.

## Recommendation

Retain the qualified packets as evidence about their shared base generation,
but do not cite their arm-blind `physical_benchmark_storage` rows as proof that
total resident state was equal or NFR-021-conforming. Keep the Task 198/199
promotion invalid. Use the Task 204 per-arm/per-node emitter and the Task 208
pre-registered conformance row for all future decision-bearing comparisons.

## Validation

This is a documentary audit; no benchmark was run. The sweep covered 426
committed `request.md`, `manifest.md`, `verdict.md`, and `disposition.md` files
under the ec_distann task buckets listed in the artifact. Search commands and
the audited head are recorded in `artifacts/manifest.md`.

This request remains open for outside review.
