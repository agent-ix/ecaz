# Task 213: ec_distann Fused Head Hop

Status: **ready** (2026-08-01). Priority: P1.

Entry gate: Task 212 P1 (the crown exists and its counters prove activation).

## Why

The dedicated head fan-out costs one full round trip before traversal begins.
Task 212's crown can prune its width, but the round trip itself survives as
long as seed selection is a separate protocol phase. The saved RTT — the
actual +8.6%@10k / +5%@100k quantity from
`reviews/task-210/006-zero-byte-head/` — requires fusing the phases.

## What changes

With a populated crown, the coordinator selects **approximate** seed
candidates locally from cached codes and skips the dedicated head fan-out:
the first traversal expansion request (which fans to the owners anyway)
carries the seed work, and exact seed distances return with that expansion.
This is the same candidate/result split TRAV-30 uses, applied one layer up:

- **candidate half** (which landmarks look promising) — answered at the
  coordinator from bounded cached codes;
- **result half** (exact distances, actual data) — always from the owner
  holding the vector, inside a fan-out that was happening regardless.

The head hop is removed by **fusing it with the next hop, never by answering
from resident state** — the conformance distinction that keeps this out of
FR-084 territory.

## Invariants to preserve (simultaneously)

- FR-079-AC-1 positional reassembly across owners — the fused first request
  keeps one response row per requested id, in request order.
- Algorithm 1's candidate/result split and the Task 205 threshold semantics
  on the fused expansion.
- NFR-021: nothing new resident at the coordinator beyond the Task 212 crown.
- Fallback: crown unpopulated/miss ⇒ the unfused two-phase path, identical
  results. The fused path is an accelerator with a correct slow path, never
  the only path.
- Same-seed attribution: the fixture's seed-digest check must either hold
  (exact policy) or the arm must be labeled as a seed-set change and measured
  as one — not silently both.

## Phases

- **P0 — spec first.** `/specify` the fused protocol (request shape, seed
  exactness contract, fallback), `/spec-review` before implementation.
- **P1 — protocol + counters.** Fused expansion carrying seed work;
  `fused_head_hops` activation counter asserted non-zero; unfused fallback
  path preserved and tested.
- **P2 — A/B.** 10k/50k/100k, fused vs unfused (both with crown on, so the
  delta attributes to fusion alone). The predicted win is ~one RTT
  (~2–3 ms at 10k); recall must be unchanged where the exact seed policy
  holds. Report honestly if the approximate seed selection moves recall.

## Benchmark gate

Standard 10/50/100k A/B per the repo rule; hop/RTT counters reported
alongside latency so the mechanism (one fewer round trip) is visible in the
evidence, not inferred from the mean.
