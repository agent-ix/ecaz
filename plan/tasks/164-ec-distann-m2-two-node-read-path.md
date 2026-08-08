# Task 164: ec_distann M2 — Hash Placement, Remote Expansion, Two-Node Read Path

Status: **complete — superseded by the accepted physical Task 179 lane**
(2026-08-08). The original two-node transport/orchestration and identity
evidence remains a replicated-serving control in
`reviews/task-164/002-two-node-loopback/`; the revised physical FR-078/FR-079
read/materialization contract is accepted in Task 179 packets 059–060, with
the final physical gate accepted in
`reviews/task-172/011-final-gate/feedback/2026-08-08-01-claude.md`.
No distributed performance promotion is implied. Depends on: Tasks 162, 163.
Owner: coder (to be assigned). One coder, one branch.
Priority: P0 — first distributed milestone; produces the D4 baton-passing
reopen-trigger measurement.

## Why

FR-078/FR-079/FR-081: the network protocol and coordinator loop. The 2-node
result-identity test (enabled by 163's determinism) proves the distributed
read path is exactly the single-node algorithm with placement underneath.

## Goal

2-node reads byte-identical to single-node; per-query expansion provably
≤ BW×H; the H×RTT delta measured.

## Corrective boundary (2026-07-10)

The landed two-node identity, transport, dedupe, and BW×H evidence remains a
valid replicated-control baseline. The physical FR-078/FR-079 materialization
and read path is now accepted under Task 179; this task's packet must not be
reused as physical-owner latency or storage evidence. Task 179 replaces
GUC-derived epoch identity, caller-selected materialization functions, and
live base-heap TIDs with manifest-selected physical generations and AM-owned
frozen row tiers.

## Scope

- FR-078 hash placement + adapted placement directory (topology-only) +
  build→publish record hand-off.
- FR-079 `ec_distann_expand_nodes` SQL fn (three-outcome contract, epoch
  fingerprint validation, fixed wire contract, `code_threshold` default
  NULL) over the lifted async transport
  (`ec_spire/coordinator/remote_candidates/dispatch.rs`, post-142 pooling).
- FR-081 remote loop in the lifted CustomScan shell
  (`ec_spire/custom_scan/mod.rs` pattern): per-round per-node parallel
  batches, vec_id visited dedupe, convergence early-exit, sub-k exhaustion
  semantics, EXPLAIN counters.
- FR-082 M2 subset: publish/fingerprint semantics + epoch-mismatch full-scan
  restart (single-epoch fixture; full lifecycle is 165).
- Tests: TC-040, TC-041 (`src/tests/ec_distann_remote.rs` + 2-node fixture).

## Required Evidence

2-node vs 1-node result identity on same corpus/seed; NFR-019 per-query max
counter ≤ BW×H per cell; measured 2-node latency delta vs single-node at
matched BW/H (the D4 reopen-trigger number), release build, packet-local.

## Non-Goals

3-worker lifecycle/fault drills (165); bench gate (166); insert (167).

## Acceptance Criteria

1. FR-078/FR-079 ACs green; FR-081 AC-1/2/3/5 green.
2. Expansion counter assertions wired into the test/bench path.
3. Hop-RTT share of p50 reported; if ≥50% of multinode p50 at gate-relevant
   BW/H, file the baton-passing reopen per ADR-085 D4 (do not implement it).

## References

- FR-078, FR-079, FR-081, FR-082 (M2 subset); ADR-085 D4/D9
- `plan/design/distann-global-graph-architecture.md` (M2)
