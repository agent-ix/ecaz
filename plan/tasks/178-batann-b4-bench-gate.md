# Task 178: BatANN B4 — Three-Way Coordination-Mode Bench Gate

Status: proposed (2026-07-09). Depends on: Tasks 175–177.
Prerequisite merges (state in the gate packet): task-165 distann lane
merged to main (or B-lane residency declared); the relay-counter-emitting
suite step kind landed as its own commit (the `distann-pipeline` step kind
cited by NFR-017/TC-044 does not exist yet); task-172 real-multinode
protocol pinned by a landed packet.
Owner: coder (to be assigned). Bench host: Intel desktop (local lane).
Priority: P0 — program gate; writes the promote/iterate/shelve verdict
into ADR-086.

## Why

NFR-022: the entire point of the mode GUC is an honest A/B of coordinator
vs batann_stack vs batann_direct on the same index/epoch/corpus/host —
including the D7 relay-rate finding under hash placement that decides the
locality-aware-placement follow-up.

## Goal

Pre-registered three-way matrix at 10k/50k/100k with recall parity, per-mode
latency, and relay counters in results.jsonl.

## Scope

- Suite-runner extension (own commit, FR-038 rule): coordination-mode axis
  + FR-084 relay-counter emission with the pre-registered field schema.
- `ecaz bench suite` config in the packet: mode × scale × recall/latency
  (storage once per scale — mode-invariant), 3-worker real multi-instance
  topology per the task-172 protocol; D9b one-sided recall bar; pinned
  reduced-depth informational row (`relay_max_depth=4`, both modes, 100k).
- Gate packet: relay-rate-per-hop-round (relay_hops ÷ drains_executed) D7
  evidence row; direct-mode variant recorded; verdict written into ADR-086
  status.

## Required Evidence

Per repo closeout rules: A/B at 10k/50k/100k minimum, recall + latency (+
storage per scale), release build verified, all rows traced to
suite-manifest.json + results.jsonl (NFR-007). 1m encouraged if 100k shows
promise.

## Non-Goals

Locality-aware placement; throughput/QPS claims (out of gate scope per
NFR-022); any tuning beyond the pre-registered matrix.

## Acceptance Criteria

1. NFR-022 table complete; D9b bar met in every mode incl. reduced-depth
   row.
2. NFR-019 cap + NFR-021 envelope assertions hold per cell in every mode.
3. Promote/iterate/shelve verdict + D7 finding written into ADR-086.

## References

- NFR-022, NFR-017, NFR-007; ADR-086 D7/D9 + Measurement Requirements
- `plan/design/batann-state-passing-coordination.md` (B4)
