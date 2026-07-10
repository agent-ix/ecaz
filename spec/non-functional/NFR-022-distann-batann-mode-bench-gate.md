---
id: NFR-022
title: Distann BatANN Coordination-Mode Bench Gate
type: NFR
status: PROPOSED
quality_attribute: performance_efficiency
relationships:
  - target: "ix://agent-ix/ecaz/FR-084"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-087"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-088"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-089"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/NFR-017"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/NFR-007"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/StR-008"
    type: "constrains"
    cardinality: "N:1"
---
# NFR-022: Distann BatANN Coordination-Mode Bench Gate

## Statement

The BatANN program SHALL close on a pre-registered three-way coordination-mode
benchmark: `coordinator` vs `batann_stack` vs `batann_direct` on the same
index, epoch, corpus, and host, at 10k / 50k / 100k, under the NFR-017
protocol (real staged corpora, release build verified, `ecaz bench suite`
runner, matched-recall comparison). Every mode SHALL meet the bench bar
(ADR-086 D9b: distinct_recall@10 ≥ coordinator mode − 0.001, one-sided, at
matched BW/H); per-mode latency p50/p95 and the FR-084 relay counters —
`relay_hops`, `relay_depth_histogram`, `state_bytes_max`,
`state_bytes_total`, `drains_executed`, `fallback_resumed`,
`relay_journeys` — SHALL be emitted to results.jsonl, with
relay-rate-per-hop-round computed as `relay_hops ÷ drains_executed`
(pre-registered formula) and summarized in the gate packet. The gate
packet SHALL record which direct-mode forwarding variant ran
(send-and-abandon vs direct-lite, ADR-086 D4) and the ADR-086 D7 finding
explicitly: the measured
relay rate under hash placement, cited against the paper's 10–30%
locality-partitioned reference, as the evidence input to any locality-aware
placement follow-up. Throughput/QPS is explicitly out of gate scope
(single-query relay cannot reproduce the paper's inter-query-balanced
throughput deltas; ADR-086 Alternatives item (a) is the reopen trigger for
any throughput claim).

## Scope

- The multinode cells run on the 3-worker fixture/topology of the M3/M4
  lane (real multi-instance PG18, distann-local-multinode / task-172
  protocol); mode is the only axis varied within a cell (per-change A/B
  discipline).
- BW/H per cell follow the FR-081 defaults or the M4 gate settings; both
  batann modes run with the D6 default depth and, as an informational row,
  a pinned reduced-depth setting: `relay_max_depth = 4`, both batann modes,
  100k scale, reporting `fallback_resumed` rate, `relay_depth_histogram`,
  and the parity delta. The D9b bench bar applies to the reduced-depth row
  (FR-089-AC-3 requires resumed results to hold parity); its latency result
  is informational, non-gating.
- BW/H stay pinned per cell (mode must be the only axis); BW sensitivity
  relative to the paper's W=64 operating point is deferred per ADR-086
  Alternatives (e) — an informational BW row may be added but does not
  gate.
- 1m is encouraged when 100k shows promise; not required for the gate.
- Prerequisites (state in the gate packet): the task-165 distann lane
  merged (or B-lane residency declared), the relay-counter-emitting suite
  step kind landed as its own commit (FR-038 rule; the `distann-pipeline`
  step kind cited by NFR-017/TC-044 does not exist yet), and the task-172
  real-multinode protocol pinned by a landed packet (ADR-086 Measurement
  Requirements).

## Rationale

The entire point of the mode GUC is an honest A/B of the coordination
strategies (ADR-085 D4 reopen). Hash placement makes ecaz's relay rate
structurally higher than the paper's; without the pre-registered counter
rows, a latency win or loss cannot be attributed between per-hop RTT saved
and relay serialization/occupancy added, and the follow-up decision
(locality-aware placement) would rest on speculation.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| distinct_recall@10 per mode, per scale | = coordinator mode | ≥ coordinator − 0.001 at matched BW/H | suite recall step per mode |
| latency p50/p95 per mode, per scale | batann modes ≤ coordinator | report (gate verdict weighs it; no auto-fail) | suite latency step per mode |
| relay-rate-per-hop-round (batann modes) | measured | report + D7 evidence row | pipeline counters in results.jsonl |
| relay hops/query, depth histogram, state bytes max/total, fallback resumes | measured | report | pipeline counters in results.jsonl |
| NFR-019 cap + NFR-021 envelope per cell | hold in every mode | never exceeded | counter assertions |

## Verification

One `ecaz bench suite` config checked into the owning packet drives the full
matrix (mode × scale × recall/latency; storage is mode-invariant — no
on-disk change per ADR-086 — and runs once per scale); results trace to
suite-manifest.json + results.jsonl per NFR-007, with the relay-counter
field schema pre-registered in the packet config. The gate packet's manifest
carries the pre-registered table and the D7 relay-rate row; the
promote/iterate/shelve verdict is written into ADR-086's status.

## Dependencies

- **Upstream**: [FR-084](../functional/index/distann/FR-084-distann-coordination-mode-selection.md),
  [NFR-017](./NFR-017-distann-latency-recall-gate.md),
  [NFR-019](./NFR-019-distann-per-query-touch-bound.md),
  [NFR-021](./NFR-021-distann-relay-resource-bounds.md),
  [NFR-007](./NFR-007-benchmark-provenance.md),
  [StR-008](../stakeholder/StR-008-distributed-search-single-instance-economics.md)
