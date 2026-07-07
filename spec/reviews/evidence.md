---
id: SR-005
title: "evidence analysis of the ec_distann spec batch"
type: SpecReview
analysis: evidence
scope: "spec/functional/index/distann/FR-075..FR-083, spec/non-functional/NFR-017..NFR-020, spec/adr/ADR-085, spec/tests.md TC-037..TC-044"
review_set: all
---

## Summary

Verification-and-evidence pass (ISO 29148 methods: Inspection | Analysis |
Demonstration | Test) over every Acceptance Criteria row of FR-075..FR-083,
the Measurement and Evaluation tables and Verification sections of
NFR-017..NFR-020, the TC-037..TC-044 matrix rows, and ADR-085's evidence
citations.

What holds up: every AC row carries an explicit verification method; every
AC traces to a TC in `spec/tests.md`; all four NFRs name a concrete
measurement channel (`ecaz bench suite` steps, pipeline counters, epoch
manifest rows, fixture drills) rather than "TBD"; the release-guard the
NFRs lean on exists in the suite runner
(`crates/ecaz-cli/src/commands/bench/suite.rs`,
`manifest_has_release_guarded_steps`). ADR-085's branch+packet citation
convention is correct and was verified against the live refs:
`reviews/task-144/012-release-matrix-decision` exists on branch
`task-144-spire-closure-ratio-pruning`,
`reviews/task-146/006-anchor-results` exists on branch
`task-146-spire-honest-pareto-confirmation`, and the on-main
`reviews/task-144/` bucket is indeed a colliding unrelated task
(hnsw-scorer-default), confirming the branch-qualified citation rule is
load-bearing, not pedantry.

The findings concentrate on three evidence-feasibility gaps: (a) the gate's
headline metric and its anchor packets live only on unmerged branches, so
NFR-017 is not currently measurable from this worktree; (b) several ACs
labeled `Test` are in substance bench A/B measurements mapped to unit/pg_test
TCs that cannot produce `ecaz bench suite` evidence (FR-075-AC-4,
FR-083-AC-4, and the M0 measurement obligations); (c) two NFR measurement
rows name metrics the current suite runner cannot compute (cross-node
storage summation, per-cell counter-cap assertion) without the suite
extension being landed first, per the "extend the runner, land it as its own
commit" convention.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | NFR-017's gate metric and baselines are branch-resident: `distinct_recall` does not exist anywhere in this worktree's bench code (it lives on branch `task-138-spire-distinct-recall-metric`), and the IVF/HNSW anchor numbers (0.9980 @ 37.6 ms, 0.9795 @ 20.4 ms) trace to `reviews/task-146/006-anchor-results` on branch `task-146-spire-honest-pareto-confirmation`. Until the metric implementation and the anchor packets are merged/reachable from the distann working branch, TC-044 cannot be executed and the gate is unverifiable. Record the merge prerequisite explicitly in NFR-017 or the M4 milestone | NFR-017, TC-044, ADR-085 |
| FND-002 | medium | FR-075-AC-4 (recall@10 within 0.002 of `ec_diskann` at 10k) is labeled `Test` and mapped only to TC-037 (Unit/pg_test, `src/tests/ec_distann_basic.rs`); a cross-AM recall-parity comparison is a bench A/B that per repo convention must run via `ecaz bench suite` with `results.jsonl` provenance. Either relabel as Test (bench A/B) and map to a bench TC, or add the cell to TC-044 | FR-075, TC-037, TC-044 |
| FND-003 | medium | FR-083-AC-4 (post-insert recall parity vs fresh rebuild, "Test (bench A/B)") maps to TC-043, whose type is "pg_test + multinode drills" — not a benchmark TC. The parity number needs a suite-produced artifact; extend TC-044 (or add a TC) with an insert-parity bench cell for M5 | FR-083, TC-043, TC-044 |
| FND-004 | medium | NFR-018's headline metric is "index bytes ÷ raw vector bytes ... summed across nodes", but the existing `ecaz bench suite` storage step measures a single local index relation. Multinode summation requires either a suite storage-step extension or a defined epoch-manifest accounting rule; per convention the extension must land as its own commit before the gate packet uses it. Name the mechanism in NFR-018's Verification section | NFR-018, TC-044 |
| FND-005 | medium | NFR-019's Verification says "the suite asserts the cap per cell", but no counter-assertion capability or distann-pipeline step kind exists in the runner yet (tests.md marks it Planned), and the Measurement table does not state the aggregation for "records expanded per query" — the ≤ BW×H bound must be asserted on the per-query max, not a mean, or the bound is not the hard cap NFR-019 claims. Specify max aggregation and the required suite extension | NFR-019, FR-081, TC-041, TC-044 |
| FND-006 | medium | M0 measurement obligations have no bench-capable TC: FR-080-AC-4 (C recall-sensitivity, "Analysis (bench)", explicitly "at M0") maps to TC-041 which is Planned (M2); ADR-085 D7's codec recall/storage comparison at M0 is parked in TC-037 permutation rows (unit/pg_test surface); D1's storage validation "by M0 storage measurement" has no TC before TC-044 (M4). M0's exit evidence needs a named bench artifact — add an M0 measurement TC or re-milestone the rows | FR-080, ADR-085, TC-037, TC-041 |
| FND-007 | medium | FR-081-AC-1 verifies "2-node top-k results identical to a single-node index built from the same corpus and seed" — result identity across two independent builds is only feasible if the sharded build + stitch is deterministic under a fixed seed, which no FR requires (only FR-080-AC-2 requires head-index determinism). Either add a build-determinism clause to FR-077 or restate the AC as deploying one stitched artifact under 1-node vs 2-node placement (which FR-078 already guarantees is recall-neutral) | FR-081, FR-077, TC-041 |
| FND-008 | low | Recall metric naming is inconsistent across evidence surfaces: NFR-017 gates `distinct_recall@10` while FR-075-AC-4, FR-077-AC-1, and FR-083-AC-4 say "recall@10" with thresholds of 0.001–0.002 — exactly the magnitude where duplicate handling changes the number. ADR-085 argues duplicates cannot occur in this architecture, but the comparison baselines (ec_diskann, monolithic build) are measured by the same tooling; pin each AC to the metric name the bench step emits | FR-075, FR-077, FR-083, NFR-017 |
| FND-009 | low | NFR-017's "p50 latency at matched recall" does not define the operating-point selection rule (interpolation across sweep points vs nearest cell at ≥ 0.999), and the p50/p95 rows are specified only at 100k while the recall row spans 10k/50k/100k — pre-register the matched-recall protocol (the Task 146 packet's rule, if that is the intent) in the Verification section so the four-way table is not assembled ad hoc | NFR-017, TC-044 |
| FND-010 | low | NFR-020's fault taxonomy includes "network partition", but the gate substrate is loopback multi-instance (ADR-085 D2) where a true partition is not injectable with the reused connection-level drill machinery (reset/timeout/termination are); name the injection mechanism for the partition case or scope it to connection-level manifestations so the "100% drill matrix" row is executable | NFR-020, TC-042 |
| FND-011 | low | FR-077-CON-4 / ADR-085 D8 bound stitch memory by "one vec_id group plus prune working set" with validation "Analysis + build instrumentation", but no numeric bound and no named analysis artifact path — non-test verification methods must name their artifact (analysis doc + which epoch-manifest/instrumentation field). As written the CON is not falsifiable | FR-077, ADR-085 |
| FND-012 | low | ADR-085 cites the DistributedANN paper by a local home-directory path (`~/dev_bak/papers/distributedann-2509.06046.pdf`) — a non-durable, single-machine reference; the arXiv id (2509.06046) alone is the durable citation. Minor because it is background literature, not gate evidence, but the packet-discipline rule against citing local-only paths applies to ADRs too | ADR-085 |
