# Task 224: ec_distann Owner Payload Heap Locality

Status: **packet 003 reviewer seq01 NOT DONE; four blockers corrected at
`7cafbd202`; rereview open; 100k screen remains unauthorized; MAT-25 retired**
(updated 2026-08-25). Request
`reviews/task-224/003-isolated-candidate/request.md`; NOT DONE verdict
`reviews/task-224/003-isolated-candidate/feedback/2026-08-25-01-reviewer.md`.
Priority: P2 latency. The
vector-bearing binary-send bucket is 6.967996 ms/scan / 24.709206% summed owner
work, while the endpoint critical path bounds any serial saving to at most
5.148990 ms / 18.258830%. MAT-25 is retired: 6.785 requested TIDs occupy 6.770
blocks and sorting moves 72% of rows for essentially no coalescing. Final
verdict:
`reviews/task-224/002-locality-attribution/feedback/2026-08-25-02-reviewer.md`.

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
   of warm end-to-end mean at 100k. Summed owner work establishes bucket size,
   but the independently measured endpoint critical path bounds the achievable
   serial share and must be printed beside it.

## Scope

- Attribute id-only, narrow scalar, vector-bearing, and toasted projections at
  100k without changing production behavior.
- Pre-register and implement at most one candidate from MAT-25/MAT-26.
- Preserve request/result rank ordering after any physical reorder.
- Run a same-generation 100k A/B with both arms in the same instrumentation
  state, preferably unprofiled production SQL; never reuse packet 002's
  profiled warm means as a baseline. Run 10k/50k/100k only for a useful result.

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
