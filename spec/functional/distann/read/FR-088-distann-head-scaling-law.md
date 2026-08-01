---
id: FR-088
title: DistANN Head Scaling Law
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-080"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-082"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-088: DistANN Head Scaling Law

## Description

Head capacity SHALL be a sampling-rate law resolved at epoch build, not a
fixed constant: `C = clamp(ceil(rate × N), floor, ceiling)`, where N is the
build's captured record count. The head's job is seed quality; with C fixed
and N growing, each landmark covers N/C vectors, seeds land farther from the
query's true neighborhood, and the deficit is paid in extra hop rounds —
each a full owner fan-out round trip
([FR-081](./FR-081-distann-query-orchestration.md)). The reference design's
head is a scaled-down index over a sample of the corpus (a rate, not a cap).
Since the sharded membership-only head
([FR-080](./FR-080-distann-coordinator-head-index.md)) costs the
coordinator O(C) identifiers only, head growth lands on the owners, where it
belongs (Task 210, `reviews/task-210/006-zero-byte-head/`).

## Inputs

- Reloptions on the control index:
  - `head_sampling_rate` (float; default 0 = law disabled — see override
    below),
  - `head_cap_floor` (int; default 4096, the ADR-085 D3 measured retention),
  - `head_cap_ceiling` (int; default 1,048,576, the frozen v1 upper bound),
  - `head_index_cap` (int; the pre-existing explicit cap, retained as the
    fixture/pin override).
- N: the build's cumulative captured record count at T2 seal time
  ([FR-078](../build/FR-078-distann-hash-placement.md) owner-stream
  accounting).

## Behavior

- When `head_sampling_rate` > 0, the T2 build SHALL resolve
  `C = clamp(ceil(rate × N), floor, ceiling)` from the captured record
  count and use that C for head selection (FR-080 Selection). Resolution
  SHALL be deterministic: identical build inputs yield identical C.
- When an explicit `head_index_cap` is set (fixture pin) or
  `head_sampling_rate` is 0, the build SHALL use the explicit cap unchanged
  (the pre-law behavior). The explicit cap takes precedence over the law.
- The epoch manifest SHALL attest the sizing decision: resolved C, the law
  inputs (rate, floor, ceiling, N), and whether an explicit override was in
  force. Attestation SHALL be bound into the manifest digest chain so a
  replayed build cannot silently resolve a different head size.
- The trained-head policy's exact-cap requirement
  (`training_landmarks_exact` requires C = 4096) SHALL be reconciled: a
  trained generation either pins the explicit cap or the law's resolved C
  must satisfy the policy's validity domain; an inconsistent combination
  SHALL fail the build with `EC_HEAD_TRAINING`
  ([FR-078](../build/FR-078-distann-hash-placement.md) error class), not
  silently re-size.
- The default law (the shipped rate) SHALL be chosen from measured A/B
  evidence per the benchmark gate below; until a rate is landed as the
  default, the shipped default SHALL remain the explicit cap (law
  implemented, default unchanged).
- Growth SHALL respect the FR-080 distribution invariant: coordinator state
  remains O(C) membership identifiers; landmark vectors and per-shard
  graphs grow only on owners and attested replicas.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-088-CON-1 | Resolved C SHALL always lie within [floor, ceiling] and within the frozen v1 head-cap validity domain (16..=1,048,576) | Integrity | Unit test |
| FR-088-CON-2 | The sizing attestation SHALL be digest-bound in the epoch manifest; two builds over identical inputs attest identical sizing | Determinism | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-088-AC-1 | With a rate configured, resolved C equals clamp(ceil(rate × N), floor, ceiling) for the build's captured N, deterministically across replays | Test |
| FR-088-AC-2 | The epoch manifest attests resolved C, rate, floor, ceiling, N, and override status; tampering with any attested field breaks the digest chain | Test |
| FR-088-AC-3 | An explicit head_index_cap (or rate = 0) bypasses the law and is attested as an override | Test |
| FR-088-AC-4 | A trained-head generation with a law-resolved C incompatible with its policy fails the build with a stable error | Test |
| FR-088-AC-5 | Rate sweep A/B at 10k/50k/100k reports recall, latency, storage, and per-arm hop/frontier counters (`traversal_hop_rounds` non-zero); an arm improving latency without moving hop counts is flagged in the packet, not promoted | Analysis (bench) |
| FR-088-AC-6 | Coordinator head-relation derived bytes remain zero as C grows (membership-only persistence) | Test + storage audit |

## Dependencies

- **Upstream**: [FR-080](./FR-080-distann-coordinator-head-index.md)
  (selection, membership persistence, sharded serving);
  [FR-078](../build/FR-078-distann-hash-placement.md) (captured record
  count); [FR-082](../lifecycle/FR-082-distann-epoch-lifecycle.md)
  (manifest digest chain); ADR-087 (zero-byte coordinator head).
- **Downstream**: [FR-089](./FR-089-distann-crown-cache.md) (Task 212 crown
  cache: capacity independent of C — the crown's bound must not inherit the
  law) and [FR-090](./FR-090-distann-fused-head-hop.md) (Task 213 fused head
  hop: seed-quality assumptions).

## Verification scope (deliberate)

Sweeps stop at 100k for now: the build-cost backlog makes larger heads at
1M+ not worth paying yet, and the law's shape (not its asymptote) is
decidable at the staged scales. Re-validating the chosen rate at 1M+ is an
explicit later gate before any 10M+ claim (user ruling, 2026-08-01). If no
swept rate beats the fixed cap at these scales, the honest outcome is: law
implemented, default unchanged, re-sweep at 1M+.
