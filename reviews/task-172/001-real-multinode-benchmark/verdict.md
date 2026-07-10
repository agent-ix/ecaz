# Task 172 packet 001 — VERDICT: SHELVED (blocked on FR-078 physical sharding)

Date: 2026-07-10. Decision: **shelve the Task 172 benchmark gate; shift to option
(a) — build the FR-078 real sharded build/publish path first, then benchmark.**

This aligns with reviewer feedback `2026-07-10-03` (P0: Task 172 must benchmark
the physically sharded index; physical FR-078 placement is a fail-closed
prerequisite for all gate measurements) and the operator's direction.

## Why we are shelving rather than pushing the fixture to finish

The multi-instance fixture is a **replicated-serving control**, not a sharded
index: it builds the complete global Vamana graph on every node and partitions
only serving ownership. Consequently the benchmark quantities do not measure the
ec_distann design:

- **Storage (NFR-018):** replicated index ≈ 3× → measured cluster amplification
  5.65×. A real record-level shard would be ≈1.9× (each node ~1/3). So the
  number fails the 4.0× gate as an artifact of replication; the sharded design
  likely passes. Not a valid distributed-storage result.
- **Recall (NFR-017):** single==multi identity is real but trivial here — every
  node holds the whole graph, so "distributed" recall never exercises the
  cross-shard hop-round traversal (FR-079/080/081) that is the actual risk.
- **Latency (NFR-017):** the graph search runs locally on the full replicated
  graph; only final materialization goes remote. The measured cost is "local
  full-graph search + eager remote row fetch," not per-hop network cost. This is
  also why the latency sweep did not complete — it repeatedly timed out
  (900s → 700s → 600s) shipping full effective-sized result sets from remote
  nodes (the 011-P2 eager-materialization cost). Pushing it to finish would only
  produce a misleading replicated-control number.

## Do we have enough benchmark data? No — and more is not worth collecting here

- Have (10k, replicated control only): distinct-recall identity `mismatched_ids=0`,
  absolute recall@10 `0.999`, cluster storage summation `5.65×`.
- Missing: any distributed latency (all runs timed out), throughput, 50k/100k,
  remote-engagement counters, telemetry, capacity model.
- Even the two numbers we have are replicated-control values. Collecting the rest
  on this fixture would not answer the gate. The blocker is FR-078 sharding, not
  fixture tuning.

## Retained as valid (functional, NON-gate) evidence

Packet 001's read-path, fanout/merge, 12-drill NFR-020 fault matrix, recall
oracle, and single-vs-multi identity results stand as
**replicated-serving-control** evidence. They must never be promoted as Task 172
distributed latency/storage/scaling numbers.

## Next: option (a)

Build the FR-078 record-level hash-shard build/publish path (one logical global
graph; each graph record + co-placed full-precision vector published to exactly
one `hash(vec_id) mod roster` owner; no non-owner residue) plus the suite-driven
topology audit the reviewer specified (exact coverage, empty pairwise
intersection, one physical record per vec_id, correct owner, co-placement, no
residue, per-node byte counts, FR-078 100k balance). Only after that audit is
green does the 10k/50k/100k recall/latency/throughput/storage/capacity matrix run.

Whether that lands as a dedicated task or folds into the ec_distann program
(161–167) is for the operator to decide; not filing a new task unilaterally.
