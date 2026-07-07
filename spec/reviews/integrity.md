---
id: SR-003
type: SpecReview
analysis: integrity
scope: "StR-008; FR-075..FR-083 (spec/functional/index/distann); NFR-017..NFR-020; ADR-085; spec/tests.md TC-037..TC-044, EC-019..EC-023; re-reviewed against revision d25ea9e0c (ADR-085 D11 — lean node records + co-placed heap rerank: FR-076/FR-078/FR-079, NFR-018)"
review_set: all
title: "Integrity Analysis: ec_distann Spec Batch (StR-008 / FR-075..083 / NFR-017..020 / ADR-085)"
---
# SR-003: Integrity Analysis — ec_distann Spec Batch

## Summary

Quality-gate (completeness / consistency / atomicity) analysis of the
ec_distann specification batch: StR-008, FR-075..FR-083, NFR-017..NFR-020,
ADR-085, and the TC-037..TC-044 / EC-019..EC-023 rows in `spec/tests.md`.

**Re-review context (d25ea9e0c).** The original 13 findings (FND-001..013)
were assessed against the first-pass batch (`3c4a22b26`). They were published
together with a round of consolidated fixes (`98b40e961`), so most were
already addressed in the tree at the moment the review landed. This re-review
then evaluates the batch after revision **d25ea9e0c**, which replaces inline
full-precision vectors with **lean node records + a co-placed heap rerank
tier** (ADR-085 decision D11): FR-076 drops the record's `vector` field for a
coarse `search_code`; FR-078 co-places each record's full-precision heap row
on the same `hash(vec_id)`-owned node; FR-079 computes `exact_dist` from that
node-local heap read; NFR-018 makes the heap tier the 1.0× ratio denominator;
ADR-085 D1 drops ~5.0× → ~4.0×.

**Prior findings — net state.** Of the 13, twelve are now resolved and one
still stands:
- Resolved by the consolidated fixes (`98b40e961`, pre-revision): FND-001
  (the sole high-severity contradiction — reclaim/repair are now atomic
  epoch-build operations, so no mid-epoch reclaim window exists), FND-002,
  FND-003, FND-005, FND-006, FND-008, FND-010, FND-011, FND-012, and
  partially FND-009.
- Resolved / advanced by revision **d25ea9e0c**: FND-004 (storage arithmetic
  now internally coherent at ~4.0×, downgraded — see below) and FND-007's
  FR-076→FR-078 edge.
- Still standing: FND-013 (FR-083 remains a three-behavior bundle).

**FND-004 is downgraded, not fully closed.** d25ea9e0c removes the 1.0×-raw
inline vector from the record, so the self-inconsistent arithmetic (30 KB /
5.0× vs a stated 24.6 KB) is gone: the record is now the ~24.6 KB code block
≈ **~4.0× raw**. But ~4.0× is *at* NFR-018's threshold, not under it, and
still over the ≤3.0× target; the binding term is the untouched R×
neighbor-code block, and the D7 `GroupedPq` default code size is still
unstated — so the *default* configuration's budget fit remains undemonstrated
(M0 measurement decides). Kept as a low-severity open item.

**Completeness — one real gap the revision introduces.** The co-placed heap
tier is now epoch-critical exact-rerank fidelity state, but two seams are
unspecified: FR-083's incremental-insert path writes the new record without
its co-placed heap row (FND-014), and FR-082's epoch mutation model /
fingerprint never bring the heap tier under epoch immutability or cover
`heap_tid` staleness (FND-015).

**Consistency — no residual record-read contradiction.** The revision is
internally coherent on the record-read language: no FR still says
"self-sufficient", "exactly one record read", or stores a "full-precision
vector" in the record; `exact_dist` remains in the FR-079 wire schema; beam
ordering (coarse `search_code`/neighbor codes) and result scoring (heap
`exact_dist`) are cleanly separated. The one loose thread is that NFR-019's
touch bound and counters do not name the per-expansion heap read that ADR-085
D11's affordability argument depends on (FND-016).

**Atomicity.** FR-078-AC-4 is a clean single obligation. FR-076-AC-5 and
FR-079-AC-5 each bundle two claims (FND-017).

No blocking contradiction remains. Recommend resolving FND-014 and FND-015
before task generation (spec-to-plan) for FR-082/FR-083, and closing the
low-severity residuals (FND-004, FND-007, FND-009, FND-013, FND-016, FND-017)
opportunistically.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | low | RESOLVED (consolidated fixes 98b40e961): the vacuum-reclaim vs missing-record contradiction is closed by making physical reclaim and adjacency repair atomic **epoch-build** operations (FR-082 D10 mutation model; FR-083 "Physical reclaim" + AC-2). Within a published epoch no record is ever physically reclaimed, tombstoned records stay traversable with intact adjacency, and FR-079 case (c) owned-but-absent is now defined as "corruption or placement drift, never a vacuum race" — so no request must both error and not-error. d25ea9e0c did not affect this | FR-083 (Physical reclaim, AC-2), FR-079 (outcome c), FR-082 (D10), NFR-020, EC-023 |
| FND-002 | low | RESOLVED (consolidated fixes 98b40e961): FR-079 now enumerates exactly three outcomes — (a) present, (b) not owned → placement error, (c) owned-but-absent → structural fault error — cleanly separating the non-owned and owned-but-absent (`missing_node_record`) cases the original conflated. d25ea9e0c added AC-5 but did not alter this block | FR-079 (Behavior three-outcome list, AC-3), NFR-020, FR-078 |
| FND-003 | low | RESOLVED (consolidated fixes 98b40e961): FR-083 interim-insert now states the single chosen ADR-085 D5 posture ("spool to a bounded exact-scan delta buffer … merged into results with same-statement visibility; drained by the next epoch build"), no longer "either error or spool" — the FR no longer re-opens a decision the ADR fixed | FR-083 (Interim insert, AC-3), ADR-085 D5 |
| FND-004 | low | DOWNGRADED by d25ea9e0c (partially resolved): dropping the inline vector (D11) removes the self-inconsistent 30 KB/5.0× vs 24.6 KB figures — the record is now the ~24.6 KB neighbor-code block ≈ **~4.0× raw**, internally coherent and matching NFR-018's refreshed note. **Residual (open, low):** ~4.0× is *at* the 4.0× threshold, not under it (and over the ≤3.0× target); the binding amplifier is the untouched R× neighbor-code block; and the D7 `GroupedPq` *default* code size is still unstated, so the default configuration's budget fit is still not demonstrated (M0 storage measurement decides) | ADR-085 D1/D7/D11, NFR-018, FR-076-CON-1, TC-044 |
| FND-005 | low | RESOLVED (consolidated fixes 98b40e961): FR-080 replaced the undefined HNSW "top layers" term with a defined single-layer procedure — a bounded BFS from each build shard's entry medoid over that shard's Vamana graph, per-shard samples unioned ("Vamana graphs are single-layer; 'entry region' means BFS-near the medoid") — so FR-080-AC-3 is now constructible from the spec text | FR-080 (Behavior bullet 1, AC-3), FR-077 |
| FND-006 | low | RESOLVED (consolidated fixes 98b40e961): the misplaced FR-055 dependency was relocated from FR-076 to FR-078 (the actual `SpirePlacementDirectory` consumer) — FR-076 frontmatter now carries only FR-075; FR-078 frontmatter carries FR-055. d25ea9e0c did not touch this. Residual body/frontmatter mismatch on FR-078 (below) folds into FND-007 | FR-076 frontmatter, FR-078 frontmatter/Behavior bullet 3, FR-055 |
| FND-007 | low | PARTIALLY RESOLVED (d25ea9e0c): the FR-076→FR-078 downstream edge is now present ("co-places the heap row"). Remaining asymmetries still stand: FR-075 lists FR-081 downstream but FR-081 omits FR-075 upstream; FR-078 **body** Dependencies Upstream lists only FR-076 while its frontmatter now also declares FR-055 and FR-077; FR-079 downstream omits FR-082; FR-081 lists FR-083 downstream but FR-083 upstream omits FR-081 | FR-075..FR-083 frontmatter `relationships` + body Dependencies |
| FND-008 | low | RESOLVED (consolidated fixes 98b40e961): NFR-017 Verification now cites ADR-085 D2 explicitly — the informational injected-latency (netem) run accompanying the gate and the H×RTT sensitivity "reported, not gated" — and TC-044 carries the netem H×RTT run, so the D2 evidence can no longer be silently omitted | ADR-085 D2, NFR-017 (Verification), TC-044 |
| FND-009 | low | PARTIALLY RESOLVED (consolidated fixes 98b40e961): FR-075-AC-4 now gates on `distinct_recall@10`, and NFR-017 defines a precise pre-registered "matched-recall comparison rule", removing the "at matched recall" ambiguity. **Residual:** FR-077-AC-1 still gates on bare `recall@10` while the rest of the batch uses `distinct_recall@10` — name one metric | FR-077-AC-1, StR-008, NFR-017 |
| FND-010 | low | RESOLVED (consolidated fixes 98b40e961): the test-matrix slips are corrected — EC-020 (vec_id hash collision) now maps to FR-076/FR-083 and TC-037/TC-043 (not TC-038), and the `closure_epsilon` config row now sits under TC-038 (stitch, M1) where the stitch-output invariants belong | spec/tests.md (EC-020, config rows, TC-037/TC-038), FR-076, FR-077 |
| FND-011 | low | RESOLVED (consolidated fixes 98b40e961): FR-081 now specifies the under-filled result set — beam exhaustion before k accumulates returns the fewer-than-k results as a complete (non-fault) result, empty index → zero rows, and results are drawn only from expanded records (head-index candidates count only via their own expansion, and thus only once they carry an exact distance) | FR-081, FR-080, NFR-019, FR-075 |
| FND-012 | low | RESOLVED (consolidated fixes 98b40e961): `plan/design/distann-global-graph-architecture.md` is now the normative home of the M0–M5 milestone definitions, with a "Milestone definitions (normative)" section and an explicit milestone→task mapping (M0=162 … M5=167), so the load-bearing milestone references across the batch now have a defining artifact | FR-080-AC-4, FR-083 (Dependencies), ADR-085, spec/tests.md TC-037..TC-044, plan/design/distann-global-graph-architecture.md |
| FND-013 | low | STILL STANDS (softened): FR-083 remains a single FR bundling three separately-milestoned obligations (delete/vacuum; interim insert, read-path milestones; incremental distributed insert, M5). A "Milestone slicing" note now clarifies which behaviors land when, but the FR is not split, so the early (M0–M3) slices still carry the open M5 incremental-insert criteria inside the same requirement. Splitting would also isolate the FND-014 fix | FR-083, TC-043, ADR-085 D5 |
| FND-014 | medium | NEW (d25ea9e0c completeness gap): the co-placed heap row is now required for exact rerank (FR-078/FR-079), but FR-083's incremental-insert path writes only "the new record to its hash-owned node" and applies back-edges — it never writes the co-placed full-precision heap row that FR-078's build→publish hand-off co-places. An inserted vec_id would then have an index record with no co-placed heap row, so FR-079's `exact_dist` heap read (FR-079-AC-5) has nothing to read. The insert path must co-place the vector alongside the record, parallel to FR-078 | FR-083 (Incremental insert, Remote write endpoint), FR-078 (build→publish co-placement), FR-079-AC-5, FR-076 (heap_tid) |
| FND-015 | medium | NEW (d25ea9e0c consistency/completeness gap): FR-082's epoch model does not bring the co-placed heap tier under epoch consistency. The D10 mutation model enumerates only "graph-node records and adjacency" as the within-epoch immutable set, and the epoch fingerprint "attests to roster, placement, format version, and the build-time record set" — the co-placed vector/heap tier is named nowhere as epoch-versioned, immutable, or fingerprint-covered. If a heap row moves or changes within a published epoch (HOT update, `heap_tid` staleness — the exact hazard EC-009 flags for SPIRE's stored heap TID), FR-079's `exact_dist` can silently read a stale/wrong tuple, undetected by the fingerprint. No distann edge-case (EC-019..EC-023) covers co-placed-heap immutability or `heap_tid` staleness | FR-082 (D10 mutation model, fingerprint), FR-078, FR-079 (exact_dist, AC-5), FR-076 (heap_tid), spec/tests.md EC-009 analog |
| FND-016 | low | NEW (d25ea9e0c consistency gap): ADR-085 D11's justification for co-placed heap over an index-resident tier rests on the equality "records read == nodes expanded == nodes exact-reranked == materialized set (all ≤ BW×H)", but the normative touch-bound NFR-019 only bounds and counts *expanded graph-node records* — its statement, metric table, and EXPLAIN/pipeline counters never name the one-per-expansion co-placed heap read that FR-079 now mandates. The equality that makes D11 affordable is asserted in the ADR/design doc but pinned by no normative NFR/AC | NFR-019 (Statement, Measurement, counters), ADR-085 D11, FR-079 (expansion = record read + heap read), FR-081-AC-2 |
| FND-017 | low | NEW (d25ea9e0c atomicity nit): two of the three new ACs bundle obligations. FR-076-AC-5 couples a structural claim ("record carries no full-precision vector field") with a derived-property claim ("encoded record bytes at fixed R are independent of vector dimension"); FR-079-AC-5 couples a value equality ("exact_dist == full-precision distance to the co-placed heap vector") with a negative implementation assertion ("no vector is read from the index record") that is not independently observable through the FR-079 wire contract. FR-078-AC-4 (record and heap row co-resolve to one node) is, by contrast, a single testable obligation | FR-076-AC-5, FR-079-AC-5, FR-078-AC-4 |
</content>
</invoke>
