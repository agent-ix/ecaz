---
id: FR-090
title: DistANN Fused Head Hop
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-089"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-079"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-090: DistANN Fused Head Hop

## Description

With a populated crown ([FR-089](./FR-089-distann-crown-cache.md)), the
coordinator MAY fuse seed selection into the first traversal expansion,
removing the dedicated head fan-out round trip: the coordinator selects
approximate seed candidates locally from cached crown codes, and exact seed
distances return with the first owner expansion — a fan-out that was
happening regardless. This is the
[FR-086](./FR-086-distann-gateway-copies.md) candidate/result split applied
one layer up: the candidate half (which landmarks look promising) is
answered at the coordinator from bounded cached codes; the result half
(exact distances, actual data) always comes from the owner holding the
vector. The head hop is removed by fusing it with the next hop, never by
answering from resident state — the conformance distinction that keeps this
outside FR-084 territory. The target quantity is the sharded head's
round-trip cost: +8.6% @10k / +5.1% @100k
(`reviews/task-210/006-zero-byte-head/`).

## Behavior

- **Fused request.** When the crown is populated for the pinned epoch, the
  scan MAY skip the dedicated
  [FR-080](./FR-080-distann-coordinator-head-index.md) head fan-out and
  SHALL instead carry the seed work in the first
  [FR-079](./FR-079-distann-remote-expansion-protocol.md) expansion
  request: crown-code-ranked seed candidates are requested alongside the
  first frontier expansion, and each owner returns exact distances for the
  seed candidates it owns within that response.
- **Positional contract.** The fused first request SHALL preserve
  FR-079-AC-1: one response row per requested id, in request order, across
  all owners.
- **Threshold semantics.** The fused expansion SHALL preserve the
  Algorithm-1 candidate/result split and the Task 205 pushdown semantics
  (code threshold from the L-th retained candidate; batch candidate limit
  applied once across the merged batch) exactly as an unfused expansion
  would.
- **Seed exactness.** Seed candidates selected from crown codes are
  approximate; their exact distances SHALL be established by the owning
  node in the fused response before any result-half use. The seed set
  policy SHALL either reproduce the unfused path's seed set exactly (exact
  policy — the fixture's seed-digest check holds) or the arm SHALL be
  labeled a seed-set change and measured as one — never silently both.
- **Fallback.** When the crown is off, unpopulated, or misses, the scan
  SHALL use the unfused two-phase path (dedicated head fan-out, then
  traversal) with identical results. The fused path is an accelerator with
  a correct slow path, never the only path.
- **Distribution invariant.** The fused hop SHALL add nothing resident at
  the coordinator beyond the FR-089 crown
  ([NFR-021](../../../non-functional/NFR-021-distann-distribution-invariant.md)).
- **Observability.** The extension SHALL expose a `fused_head_hops`
  activation counter; the candidate arm of any A/B SHALL assert it
  non-zero. Hop/RTT counters SHALL be reported alongside latency so the
  mechanism (one fewer round trip) is visible in evidence, not inferred
  from the mean.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-090-CON-1 | The fused path SHALL NOT change per-query results where the exact seed policy holds; any recall movement under approximate seeding is measured and labeled, never silent | Integrity | Test + bench |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-090-AC-1 | The fused first expansion returns one row per requested id in request order across owners (FR-079-AC-1 preserved) | Test |
| FR-090-AC-2 | Threshold and batch-limit semantics on the fused expansion equal the unfused path's (Task 205 contract) | Test |
| FR-090-AC-3 | Crown miss/off falls back to the unfused two-phase path with identical results | Test |
| FR-090-AC-4 | Under the exact seed policy the fixture seed-digest check holds; otherwise the arm is labeled a seed-set change | Test + fixture |
| FR-090-AC-5 | Fused vs unfused A/B at 10k/50k/100k (both arms crown-on, so the delta attributes to fusion alone) reports non-zero `fused_head_hops`, hop/RTT counter deltas, and recall; predicted win ~one RTT | Analysis (bench) |

## Dependencies

- **Upstream**: [FR-089](./FR-089-distann-crown-cache.md) (entry gate:
  crown exists with proven activation),
  [FR-079](./FR-079-distann-remote-expansion-protocol.md) (expansion wire),
  [FR-080](./FR-080-distann-coordinator-head-index.md) (unfused path),
  [FR-086](./FR-086-distann-gateway-copies.md) (candidate/result split
  precedent).
- **Downstream**:
  [NFR-021](../../../non-functional/NFR-021-distann-distribution-invariant.md)
  (nothing new resident);
  [NFR-017](../../../non-functional/NFR-017-distann-latency-recall-gate.md)
  (latency posture).
