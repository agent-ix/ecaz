# Task 213: ec_distann Fused Head Hop

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: **complete** (2026-08-02). Priority: P1.

P0 spec landed as
`spec/functional/distann/read/FR-090-distann-fused-head-hop.md` (hardened:
fused request defined as an ordinary FR-079 expansion whose requested
vec_ids are the crown-ranked seed candidates, seed_count-bounded first
round with NFR-019 accounting, mid-request failure semantics, exact-policy
claimability condition); packet
`reviews/task-213/001-fused-head-hop-spec/` open. The fused consumer is
implemented in
`reviews/task-213/002-fused-head-hop-implementation/`, with fused-hop
activation counters and measured recall across 10k/50k/100k. The shared
capacity matrix selected 2048 entries for the opt-in fused configuration.
Defaults remain opt-in because all measured fused arms are labeled
`seed_set_change=true`; this preserves the existing default recall policy.

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
