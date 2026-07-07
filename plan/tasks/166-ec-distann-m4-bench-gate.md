# Task 166: ec_distann M4 — Bench Gate vs Anchors (Program Gate)

Status: proposed (2026-07-06). Depends on: Task 165. Prerequisite merges:
`task-138-spire-distinct-recall-metric` (metric emitter) and the Task 146
anchor evidence branch (`task-146-spire-honest-pareto-confirmation`) must be
on the measuring branch; record merge SHAs in the packet manifest.
Owner: coder (to be assigned). One coder, one branch.
Priority: P0 gate — NFR-017 is the program kill criterion.

## Why

StR-008's satisfaction bar. Everything before this is enablement; this
packet decides promote / iterate / shelve, written into ADR-085 status.

## Goal

Pre-registered four-way comparison (ec_distann / IVF / HNSW / best-SPIRE) on
the Task 146 host/corpus/query protocol, 10k/50k/100k, release build.

## Scope

- `const EC_DISTANN: IndexProfile` in `crates/ecaz-cli/src/profiles.rs` +
  REGISTRY (sweep axis: BW or H per pre-registration).
- New `SuiteStep` `distann-pipeline` kind in
  `crates/ecaz-cli/src/commands/bench/suite.rs` emitting per-round counters
  (rounds, records expanded per-query max, code-scored candidates, per-node
  batch sizes, pool reuse) **and** added to the release-guard whitelist
  (Task 141 rule: every latency-emitting step kind is guarded).
- Multinode storage summation in the suite report (NFR-018 mechanism) —
  lands as its own commit before the gate run.
- Gate matrix per NFR-017 (matched-recall rule), NFR-018 ratio rows, NFR-019
  per-query max + min-BW×H-for-0.999 rows, informational netem H×RTT run
  (ADR-085 D2).

## Required Evidence

Pre-registered criteria committed before the matrix runs; suite manifests
with per-node `ecaz_build_profile()`; every cited number traces to
`results.jsonl`; four-way table in the packet.

## Non-Goals

New mechanisms. Incremental insert (167) — though its FR-083-AC-4 bench cell
reuses this packet's protocol later.

## Acceptance Criteria

1. Gate: distinct_recall@10 ≥ 0.999 at 10k/50k/100k; 3-worker p50 ≤ IVF
   anchor at matched recall per the NFR-017 rule; NFR-018 ≤ 4.0×; NFR-019
   rows within threshold.
2. Verdict (promote / iterate / shelve) recorded in ADR-085 status with the
   numbers.
3. If iterate: the named lever comes from ADR-085's decision menu (D1
   fallback, D4 reopen, C/BW/H retuning), not ad-hoc scope growth.

## References

- NFR-017, NFR-018, NFR-019, StR-008; TC-044
- `plan/design/distann-global-graph-architecture.md` (M4)
