# Task 224: ec_distann Owner Payload Heap Locality

Status: **ready for its own entry-gate disposition; Task 223 review-closed
STOP** (updated 2026-08-25). Priority: P2 latency. Task 223's accepted
whole-bucket ceiling releases this task and supplies a conservative upper
bound for the heap/TOAST subset; Task 224 must record its own gate outcome.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidates
MAT-25 and MAT-26.

## Why

After projection narrowing and any justified direct tuple path, the remaining
owner cost may be physical heap/TOAST access rather than executor machinery.
Task 218 did not report heap-block dispersion, cache hit state, detoast share,
or per-projection physical read locality, so locality work currently has no
entry evidence.

## Goal

Measure the residual heap/TOAST locality cost and, only if it is material,
advance exactly one of: heap-block/TID-sorted fetch with rank restoration, or
block-batched detoast/binary-send work.

## Entry gate

1. Tasks 222 and 223 are review-closed and define the current control.
2. Owner counters report requested TIDs per block, distinct blocks, rank-order
   displacement, buffer/cache observations where safely available, toasted
   attribute count/bytes, and detoast/send time.
3. No candidate is built unless heap/detoast work is at least 1 ms/scan or 5%
   of warm end-to-end mean at 100k.

## Scope

- Attribute id-only, narrow scalar, vector-bearing, and toasted projections at
  100k without changing production behavior.
- Pre-register and implement at most one candidate from MAT-25/MAT-26.
- Preserve request/result rank ordering after any physical reorder.
- Run a same-generation 100k A/B, then 10k/50k/100k only for a useful result.

## Non-goals

- Combining TID sorting and a new row-tier format.
- Covering sidecars, payload caches, speculative prefetch, or traversal changes.
- Assuming locality from TID distribution without an end-to-end A/B.

## Acceptance

1. Packet-local evidence establishes the heap/TOAST ceiling and candidate
   choice, or closes the family without implementation.
2. Any physical reorder restores exact global result order and passes the full
   materialization semantic/failure matrix.
3. A useful candidate receives 10k/50k/100k recall, latency, and storage
   evidence; otherwise STOP.

## Required review packets

1. `reviews/task-224/001-plan/`
2. `reviews/task-224/002-locality-attribution/`
3. `reviews/task-224/003-isolated-candidate/`
4. `reviews/task-224/004-full-scale-decision/` (only after a useful screen)

## References

- Tasks 222 and 223
- Roadmap MAT-25 / MAT-26

