---
id: SR-003
type: SpecReview
analysis: integrity
scope: "StR-008; FR-075..FR-083 (spec/functional/index/distann); NFR-017..NFR-020; ADR-085; spec/tests.md TC-037..TC-044, EC-019..EC-023"
review_set: all
title: "Integrity Analysis: ec_distann Spec Batch (StR-008 / FR-075..083 / NFR-017..020 / ADR-085)"
---
# SR-003: Integrity Analysis — ec_distann Spec Batch

## Summary

Quality-gate (completeness / consistency / atomicity) analysis of the
ec_distann specification batch: StR-008, FR-075..FR-083, NFR-017..NFR-020,
ADR-085, and the TC-037..TC-044 / EC-019..EC-023 rows in `spec/tests.md`.

**Completeness — strong.** Every FR chains to StR-008 (FR-075 `implements`,
the rest via the FR dependency DAG); every FR and NFR has acceptance
criteria and a mapped test case (TC-037..TC-044); all four NFRs are
explicitly scoped and referenced by the constrained FRs; the edge-case table
covers the batch's distinctive failure modes (partial hop round, vec_id
collision, batch skew, epoch swap, tombstone/vacuum). ADR-085 sub-decisions
D1/D6/D7→FR-076, D3→FR-080, D5→FR-083, D8→FR-077, D9→FR-081 are each cited
by their consuming FR; D4 is correctly ADR-only with a reopen trigger.

**Consistency — one real contradiction and several coherence gaps.** The
probed FR-079 exact_dist vs FR-081 no-rerank pair is *consistent* (FR-081
explicitly relies on expansion responses carrying exact distances). But the
probed FR-083 vacuum-reclaim vs FR-079 missing-record pair is a genuine
contradiction (FND-001): after vacuum reclaims a tombstone but before
adjacency repair, a frontier request for the reclaimed vec_id must
simultaneously error (FR-079-AC-3, NFR-020 `missing_node_record`) and never
error (FR-083-AC-2). FR-083 also re-opens the interim-insert choice that
ADR-085 D5 already fixed (FND-003), ADR-085 D1's own storage arithmetic
lands above the NFR-018 threshold it claims to satisfy (FND-004), and the
frontmatter relationship graph diverges from body Dependencies in several
places, including a misplaced FR-055 edge (FND-006, FND-007).

**Atomicity — mostly good.** FRs define single, observable, testable
obligations, with one exception: FR-083 bundles three separately-milestoned
behaviors (FND-013), and FR-080's head-sample procedure uses an undefined
term ("shard top layers") that makes it non-implementable as written
(FND-005).

No blocking issue outside FND-001; recommend resolving FND-001..FND-005
before task generation (spec-to-plan) for the affected FRs.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | high | Vacuum edge-repair contradicts missing-record error semantics: FR-083 drops edges to reclaimed records "at expansion time until repaired" and FR-083-AC-2 requires "no expansion ever errors on a reclaimed neighbor", but the coordinator (FR-081) will place a reclaimed neighbor's vec_id in the frontier and request it from its owner, where FR-079 mandates a raised error for a missing record and NFR-020 drills `missing_node_record` as error-or-correct. The same request must both error and not error; needs an explicit mechanism (e.g. repair-before-reclaim ordering, or a `reclaimed` response marker exempting formerly-tombstoned vec_ids from FR-079's error) | FR-083 (Behavior: Delete, AC-2), FR-079 (Behavior bullet 3, AC-3), FR-081, NFR-020 (fault taxonomy), EC-023, TC-042/TC-043 |
| FND-002 | medium | FR-079 conflates non-owned with owned-but-absent: the behavior bullet defines an error only for "a requested vec_id … not owned by this node", yet attaches "distinguishing missing-record from tombstone" to that placement error. Behavior for a vec_id that hashes to this node but has no record (the NFR-020 `missing_node_record` case, and the FND-001 window) is unspecified — two valid interpretations (placement error vs distinct missing-record error vs row with `is_tombstone`) | FR-079 (Behavior, Outputs, AC-3), NFR-020, FR-078 |
| FND-003 | medium | FR-083 re-opens a decision ADR-085 D5 already fixed: FR-083's interim posture is "SHALL either error … or spool to a bounded exact-scan delta buffer; the chosen posture SHALL be a documented reloption default" and later "same-epoch delta visibility vs next-epoch visibility", while D5 states the delta buffer *was chosen* ("chosen over erroring") with same-statement visibility. As written the FR admits two interpretations of an already-made decision; FR-083-AC-3 ("behaves exactly as documented") inherits the ambiguity | FR-083 (Behavior: Interim insert, AC-3), ADR-085 D5 |
| FND-004 | medium | ADR-085 D1 arithmetic conflicts with NFR-018: D1's example (dim=1536, R=32, 4-bit ≈768 B/code) gives 6,144 B vector + 24,576 B codes ≈ 30 KB/record ≈ 5.0× raw — the stated "≈24.6 KB" (4.03×) does not match its own inputs, and either figure exceeds NFR-018's ≤3.0 target and sits at/over the ≤4.0 threshold before metadata and head sample. The arithmetic is also computed for rabitq-class codes while D7 makes GroupedPq the default codec, whose code size is never stated — no demonstration that the *default* configuration fits the budget | ADR-085 D1/D7, NFR-018 (Statement, Measurement), FR-076-CON-1, TC-044 |
| FND-005 | medium | FR-080 head-sample construction uses an undefined concept: "breadth-first sample … union across build shards' top layers". Vamana graphs (FR-077 per-shard builds) are single-layer with a medoid entry point — "top layers" is an HNSW notion with no definition anywhere in the batch, so the sampling procedure has no single valid interpretation and FR-080-AC-3 (per-shard-region reachability) cannot be constructed from the spec text | FR-080 (Behavior bullet 1, AC-3), FR-077, ADR-085 D3 |
| FND-006 | medium | Frontmatter/body dependency mismatch on FR-055: FR-076 frontmatter declares `depends_on` FR-055 (SPIRE topology/placement directory) but its body Dependencies never mentions FR-055 (it cites the ADR-068 source-identity contract instead). The spec that actually consumes FR-055 machinery is FR-078 ("adapted from `SpirePlacementDirectory`"), which carries no FR-055 edge in frontmatter or body — the edge appears to be on the wrong FR | FR-076 (frontmatter vs Dependencies), FR-078 (Behavior bullet 3), FR-055 |
| FND-007 | low | Upstream/downstream edge asymmetries: FR-075 lists FR-081 downstream but FR-081 omits FR-075 upstream (frontmatter and body) despite FR-075 normatively routing multinode scans through FR-081; FR-076 downstream omits FR-078 (which depends on FR-076); FR-077 lists FR-078 downstream but FR-078 upstream omits FR-077 (placement runs on FR-077's stitched output per the workflow diagram); FR-079 downstream omits FR-082; FR-081 lists FR-083 downstream but FR-083 upstream omits FR-081 even though incremental insert "SHALL run the FR-081 beam search" | FR-075..FR-083 frontmatter `relationships` + body Dependencies |
| FND-008 | low | ADR-085 D2 (gate substrate) has no consuming spec text: NFR-017 fixes the loopback multi-instance fixture but never cites D2, and D2's normative companions — one informational netem injected-latency run accompanying the gate, and H×RTT sensitivity reported-not-gated — appear nowhere in NFR-017's Measurement/Verification or in TC-044, so the gate can pass without producing the D2-mandated evidence | ADR-085 D2, NFR-017 (Scope, Verification), TC-044 |
| FND-009 | low | Metric-name and matched-recall drift: FR-075-AC-4 and FR-077-AC-1 gate on "recall@10" while StR-008/NFR-017/tests.md use "distinct_recall@10" (equivalent single-node, but the batch's own rationale is that the distinction was the predecessor's failure mode — name one metric); NFR-017's "at matched recall" is ambiguous between matching the IVF anchor's 0.9980 and the gate's own ≥0.999 floor, two different latency operating points | FR-075-AC-4, FR-077-AC-1, StR-008 (Validation), NFR-017 |
| FND-010 | low | Test-matrix traceability slips: EC-020 (vec_id hash collision, an FR-076/D6 build behavior) is verified by TC-038, whose requirements column lists only FR-077 items and whose harness is the stitch proptest suite; the `closure_epsilon` configuration row is assigned to TC-037 and asserts "stitch output invariants hold", but TC-037 covers FR-075/076/080 (M0) and stitch invariants belong to TC-038 (M1) | spec/tests.md (EC-020, config rows 241–242, TC-037/TC-038), FR-076, FR-077 |
| FND-011 | low | Under-filled result set unspecified: FR-081's result heap is fed only by exact distances from expanded records, and expansion is capped at BW×H (NFR-019), but no requirement relates BW×H (or head-index candidates, which never receive exact distances) to k — behavior when fewer than k records are expanded (tiny corpus, aggressive early-exit, low BW/H GUC settings) is undefined | FR-081, FR-080, NFR-019, FR-075 (GUCs) |
| FND-012 | low | Milestones M0..M5 are load-bearing but undefined in the spec set: FR-080-AC-4 verifies "at M0", FR-083 downstream is "program milestone M5", ADR-085 keys D1/D3/D4/D7 to M0/M2, and every TC row carries a Planned(Mx) status — yet no artifact in scope defines the milestone sequence, entry/exit criteria, or ownership | FR-080-AC-4, FR-083 (Dependencies), ADR-085, spec/tests.md TC-037..TC-044 |
| FND-013 | low | FR-083 is non-atomic: it bundles three separately-verified, separately-milestoned obligations (tombstone delete + vacuum repair; interim insert posture, read-path milestones only; incremental distributed insert, final milestone M5) under one FR. Splitting (delete/vacuum vs interim vs incremental) would let the M0–M3 slices close without carrying open M5 criteria, and would isolate the FND-001/FND-003 fixes | FR-083, TC-043, ADR-085 D5 |
