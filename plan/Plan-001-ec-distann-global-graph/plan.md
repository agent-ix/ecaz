---
id: Plan-001
title: "ec_distann global-graph access method (M0–M5)"
type: Plan
status: active
relationships:
  - target: ix://agent-ix/ecaz/StR-008
    type: references
  - target: ix://agent-ix/ecaz/FR-075
    type: references
  - target: ix://agent-ix/ecaz/FR-076
    type: references
  - target: ix://agent-ix/ecaz/FR-077
    type: references
  - target: ix://agent-ix/ecaz/FR-078
    type: references
  - target: ix://agent-ix/ecaz/FR-079
    type: references
  - target: ix://agent-ix/ecaz/FR-080
    type: references
  - target: ix://agent-ix/ecaz/FR-081
    type: references
  - target: ix://agent-ix/ecaz/FR-082
    type: references
  - target: ix://agent-ix/ecaz/FR-083
    type: references
  - target: ix://agent-ix/ecaz/NFR-017
    type: references
  - target: ix://agent-ix/ecaz/NFR-018
    type: references
  - target: ix://agent-ix/ecaz/NFR-019
    type: references
  - target: ix://agent-ix/ecaz/NFR-020
    type: references
---
# Plan-001: ec_distann global-graph access method (M0–M5)

Executable plan for the ec_distann program: a fifth PostgreSQL access method
implementing a DistributedANN-style **single global Vamana graph** with
hash-placed lean records, a co-placed full-precision heap tier for exact
rerank (ADR-085 D11), coordinator head index, and H batched hop-rounds of
remote expansion.

Normative sources: `spec/stakeholder/StR-008-*`, FR-075..FR-083
(`spec/functional/index/distann/`), NFR-017..NFR-020, ADR-085 (D1–D11),
TC-037..TC-044 / EC-019..EC-027 (`spec/tests.md`), and the **M0–M5 milestone
table** in `plan/design/distann-global-graph-architecture.md`. Operational
task files: `plan/tasks/162..167-*.md` (M0=162 … M5=167). Task 168's
batched-beam primitive (`src/am/ec_diskann/scan.rs::greedy_descent_beam_with`)
is already landed on this branch and is the FR-081 local-loop building block.

## Requirements Summary

| Req | Title | Milestone(s) | Tests |
|---|---|---|---|
| FR-075 | ec_distann Access Method Surface | M0 | TC-037 |
| FR-076 | Graph-Node Record Format + Global Identity (lean record, D11) | M0 | TC-037, TC-044 |
| FR-077 | Sharded Clustered Build + Closure Overlap + Stitch | M1 | TC-038, TC-039 |
| FR-078 | Hash Placement + Placement Directory (co-placed heap row) | M2 | TC-040 |
| FR-079 | Remote Expansion Protocol (exact_dist from local heap read) | M2 | TC-040 |
| FR-080 | Coordinator Head Index | M0 (degenerate), M2 | TC-037, TC-041 |
| FR-081 | Query Orchestration + Scan Semantics | M0 (local), M2 (remote) | TC-041 |
| FR-082 | Epoch Lifecycle + Consistency (D10 immutability) | M2 (subset), M3 | TC-042 |
| FR-083 | DML Path (tombstone/delta early; full insert M5) | M3 (early), M5 | TC-043 |
| NFR-017 | Latency + Recall Gate vs release anchors | M4 | TC-044 |
| NFR-018 | Space Amplification ≤ 4× raw | M0 measure, M4 gate | TC-044 |
| NFR-019 | Per-Query Touch Bound (reads==expansions==reranks ≤ BW×H) | M2, M4 | TC-041, TC-044 |
| NFR-020 | Fault Behavior (correct-or-error, no silent partials) | M3 | TC-042, TC-043 |

## Dependency Graph

- `FR-075/FR-076 (M0 scaffold + record) -> everything`
  Reason: AM surface and record format are the substrate for build, scan,
  placement, and DML.
- `FR-076 vec_id identity -> FR-077 (stitch dedup), FR-078 (hash placement), FR-083 (insert collision)`
  Reason: hash64(source_identity) is the global key for placement, stitch
  uniqueness, and insert-time collision errors (ADR-085 D6, ADR-063 via ADR-068).
- `FR-081 local loop (M0) -> FR-081 remote loop (M2)`
  Reason: the expansion-function signature designed at M0 is the seam the
  remote fn implements; Task 168's batched-beam is the loop shape.
- `FR-077 (M1 stitch) -> FR-078/FR-079 (M2)`
  Reason: two-node read path consumes a stitched, seed-deterministic build
  (M2's result-identity test depends on build determinism).
- `FR-078 co-placement -> FR-079 exact rerank`
  Reason: FR-079's local heap read is only sound when the vector tier is
  co-placed by the same hash owner (D11).
- `FR-082 epoch subset (M2 fingerprint) -> FR-082 full (M3) -> FR-083 (M3 early, M5 full)`
  Reason: D10 epoch immutability is what makes owned-but-absent /
  vector-missing hard errors and TID-stability sound; DML semantics sit on it.
- `M0..M3 evidence + task-138 distinct_recall + task-146 anchors -> NFR-017/018/019 gate (M4)`
  Reason: the gate matrix needs the metric emitter and anchor evidence merged
  into the measuring branch (record merge SHAs in the packet).
- `M4 gate verdict -> M5 insert`
  Reason: committed scope, but sequenced after the program gate per the
  milestone table.

## Critical Path

M0 (Task-001) → M1 (Task-002) → M2 (Task-003) → M3 (Task-004) → M4 gate
(Task-005) → M5 (Task-006). Single-coder serial by design (one branch per
milestone task); the only genuinely parallelizable item is the suite-runner
extension (Task-007), which must merge before M4.

## Shared Dependencies (discrete deliverables)

- **Batched-beam primitive** — landed (Task 168); consumed by FR-081 local
  loop at M0 and the hop-round shape at M2.
- **Expansion-function seam** — designed at M0 (`local_expand` signature ==
  future `ec_distann_expand_nodes` wire contract, D1-fallback-safe); M2
  implements the remote twin. Single writer: M0 owns the signature.
- **vec_id identity helper** — hash64(source_identity) + collision error;
  built at M0 (FR-076), reused by M1 stitch, M2 placement, M5 insert.
- **Suite-runner extension** (`distann-pipeline` step kind + multinode
  storage summation + release-guard whitelist entry) — Task-007, lands as
  its own commit before M4 (FR-038 discipline; debug-build trap #4).

## Quality Gates

#### Gate G0: M0 kill-check spike (ADR-085 D2) — program go/no-go
- **Measures:** single-node recall-vs-H curve × measured SPIRE per-round
  transport cost → projected multinode p50.
- **Pass criteria:** projection beats/approaches the 37.6 ms IVF-100k anchor
  envelope (NFR-017 posture); D1 storage measurement decides record-format
  posture (keep R=32 / lower R / smaller codes / D1 fallback).
- **If fails:** stop the program before any remote work; escalate with the
  spike table (early kill criterion is the spike's purpose).

#### Gate G1: M2 result identity
- **Measures:** 2-node top-k vs single-node build (same corpus/seed);
  per-query expansions ≤ BW×H; hop RTT share of multinode p50.
- **Pass criteria:** TC-040/TC-041 green; identity exact; if hop RTT ≥ 50%
  of multinode p50, reopen ADR-085 D4 (baton passing) as a follow-up.
- **If fails:** fix determinism/placement before M3 fault work.

#### Gate G2: M4 bench gate (program gate)
- **Measures:** pre-registered four-way table (ec_distann / IVF / HNSW /
  best-SPIRE) at 10k/50k/100k per NFR-017 matched-recall rule; NFR-018
  ratio rows; NFR-019 min-BW×H row; informational netem run.
- **Pass criteria:** thresholds in NFR-017/018/019; verdict
  (promote/iterate/shelve) written into ADR-085 status.
- **Prerequisite:** task-138 `distinct_recall` emitter + task-146 anchors
  merged into the measuring branch (record merge SHAs); Task-007 landed.

## Test Plan

| Test | What | Harness | Milestone |
|---|---|---|---|
| TC-037 | AM surface, lean record round-trip (no inline vector, FR-076-AC-5), head index determinism; M0 bench cells (parity A/B, C sensitivity, D7 codec, D1 ratio) | pg_test `src/tests/ec_distann_basic.rs` + `ecaz bench suite` | M0 |
| TC-038 | Stitch property suite (degree ≤ R, vec_id uniqueness, medoid reachability, idempotence, α-prune) | proptest in `src/am/ec_distann/` | M1 |
| TC-039 | Stitched-vs-monolithic distinct_recall A/B at 100k (within 0.001) | `ecaz bench suite` | M1 |
| TC-040 | Placement determinism, record/heap co-resolution, expand-protocol four-outcome table, exact_dist == heap distance | pg_test `src/tests/ec_distann_remote.rs` | M2 |
| TC-041 | 2-node result identity, BW×H cap, dedupe, early-exit equivalence, EXPLAIN counters, C sensitivity | pg_test + 2-node fixture + bench counters | M2 |
| TC-042 | Epoch lifecycle + fault drill matrix (incl. hop_round_failure_mid_beam, missing_node_record, placement_drift, missing_heap_row, coplacement_drift; FR-082-AC-5 TID-reuse drill) | multinode fixture drills | M3 |
| TC-043 | DML: tombstone/vacuum, interim posture, insert-then-query parity vs fresh rebuild, mid-insert drills | pg_test + fixture + bench cell | M3 early / M5 |
| TC-044 | Gate matrix (NFR-017/018/019 rows) via release-guarded `distann-pipeline` suite steps | `ecaz bench suite` | M4 |

Edge cases EC-019..EC-027 land inside the TC that owns their surface (see
`spec/tests.md` rows).

## Remaining Work

### Track A: Critical Path (serial, one coder, one branch per task)
Task-001 (M0) → Task-002 (M1) → Task-003 (M2) → Task-004 (M3) →
Task-005 (M4 gate) → Task-006 (M5). Gates G0/G1/G2 as above.

### Track B: Parallel
Task-007 (suite-runner extension) — independent of A until its merge
deadline (before Task-005 starts).

## Task File Mapping

| Task file | Track | Milestone / repo task | Owns | Status |
|---|---|---|---|---|
| Task-001-m0-single-node-parity.md | A | M0 / `plan/tasks/162` | FR-075, FR-076, FR-080(degenerate), FR-081(local), NFR-018 measure, G0 | not_started |
| Task-002-m1-sharded-build-stitch.md | A | M1 / `plan/tasks/163` | FR-077 | not_started |
| Task-003-m2-two-node-read-path.md | A | M2 / `plan/tasks/164` | FR-078, FR-079, FR-081(remote), FR-082(subset), G1 | not_started |
| Task-004-m3-lifecycle-faults.md | A | M3 / `plan/tasks/165` | FR-082(full), FR-083(early), NFR-020 | not_started |
| Task-005-m4-bench-gate.md | A | M4 / `plan/tasks/166` | NFR-017, NFR-018, NFR-019, G2 | not_started |
| Task-006-m5-incremental-insert.md | A | M5 / `plan/tasks/167` | FR-083(full) | not_started |
| Task-007-suite-runner-distann-extension.md | B | pre-M4 / `plan/tasks/166` prereq | distann-pipeline step kind + storage summation + release guard | not_started |

## Coordination Rules

- One coder, one branch per milestone task (`task-16N-…`), off this branch's
  lineage; review packets under `reviews/task-16N/` per repo CLAUDE.md.
- **Freeze the expansion-function signature at M0** (Task-001 design output);
  M2 implements against it without renegotiation unless G1 fails.
- Do not start Task-003+ before G0 passes — the kill-check exists to stop
  remote work early.
- Task-007 merges before Task-005 begins; any new bench step kind MUST be in
  the suite release-guard whitelist (`crates/ecaz-cli/src/commands/bench/suite.rs`)
  before latency evidence is collected.
- M4 prerequisites: merge `task-138-spire-distinct-recall-metric` and
  `task-146-spire-honest-pareto-confirmation` into the measuring branch;
  record merge SHAs in the packet manifest.
- Bench discipline per CLAUDE.md: `ecaz bench suite` only, A/B per change at
  10k/50k/100k, release-verified backend, evidence in packet `artifacts/`.
