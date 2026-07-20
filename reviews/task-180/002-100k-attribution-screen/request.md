---
task: 180
packet: 002-100k-attribution-screen
role: coder
status: open
date: 2026-07-14
---

# Review request: bounded-head attribution surfaces and PG18 smoke

Please review the Task 180 benchmark-only implementation before the Phase 1
100k screen is promoted as decision evidence.

## Commits

- `174da94efc82d1f0ea4d11751eb8a834f4d8c29f` — benchmark-only extension and
  suite surfaces, distinct-recall emitter, and checked-in screen-A config.
- `d9fd2bf79ce80618d52ebccc79bcc30e96fdfa54` — numeric provenance/engagement
  normalization plus compact live PG18 smoke evidence.
- `7c63fb124174ad44e2148c25f05df4419444ea8b` — feature-only retention of
  additional candidates already scored by a bounded head search when the
  requested seed count exceeds the frontier width.
- `2dbd78450f4f94b7c48c60554b1f0db646ff8fe6` — reject an attribution query
  rather than silently emitting fewer seeds than its result provenance claims.

## Scope

- Adds the opt-in `distann-head-attribution-benchmark` feature. Normal builds
  remain hardwired to persisted-head seeding and RaBitQ neighbor scoring.
- Adds runtime diagnostic modes `persisted_head`, `head_sample_exact`, and the
  existing benchmark-only O(N) `owner_scan` oracle.
- Decouples persisted-head graph search width and returned seed count from
  distributed BW/H.
- Adds conditional `exact_neighbor` traversal scoring. It preserves the exact
  same seeds, adjacency, BW/H, and persisted RaBitQ graph; for each bounded
  expansion it fetches the referenced neighbor nodes' already-returned exact
  source distances and substitutes only the traversal scores. Work is bounded
  by requested expansion nodes times graph degree. It remains unavailable in
  normal production builds.
- Lets one `distann-local-multinode` suite step evaluate named variants against
  one immutable physical generation, avoiding repeated identical 100k builds.
- Extends `ecaz bench recall` with distinct recall and Wilson CI fields while
  retaining the legacy membership recall and all previous column positions.
- Emits per-arm seed/search/codec/corpus/query/extension provenance and separate
  head sample/graph/cache byte accounting.

No production seed default, reloption, persisted format, codec, or ordinary
query behavior is intentionally changed.

## Validation

The compact artifact source of truth is
`reviews/task-180/002-100k-attribution-screen/artifacts/manifest.md`.

- Normal PG18 check: pass.
- PG18 plus attribution feature check: pass.
- CLI check: pass (one pre-existing unrelated dead-code warning).
- Focused persisted-head, distinct-recall, multi-variant expansion, and
  provenance-normalization tests: pass.
- All checked-in suite configs pass `ecaz bench suite audit`.
- Live three-owner PG18 10k smoke: one succeeded step, no missing/stale
  artifacts, all six thresholds pass.
- Installed release extension provenance is unanimous on three nodes at
  `174da94ef...`; topology totals 10,000 disjoint rows/records with zero
  non-owned rows/orphans, remote engagement passes, and all four diagnostic
  modes emit recall/latency/storage/head rows.

The smoke intentionally uses only two queries and two measured latency
iterations. Its measurements are path-validation only and MUST NOT be used for
candidate selection or the Task 180 verdict.

## Screen A result

The first decision-grade 100k screen completed successfully with 200 held-out
queries / 2,000 trials, 50 warm latency measurements after 10 warmups, exact
100,000-row disjoint topology, two remote-owner engagement probes, and unanimous
release provenance at extension SHA `174da94ef...`.

| Variant | Distinct recall@10 (95% CI) | Warm p50 | Warm p95 |
| --- | ---: | ---: | ---: |
| production width 32 / seeds 32 | 0.9275 (0.9153-0.9381) | 41.70 ms | 57.30 ms |
| owner-scan oracle | 0.9970 (0.9935-0.9986) | 2467.60 ms | 2515.50 ms |
| exact bounded sample / seeds 32 | 0.9275 (0.9153-0.9381) | 42.20 ms | 55.20 ms |
| width 64 / seeds 32 | 0.9280 (0.9158-0.9385) | 40.00 ms | 52.80 ms |
| width 128 / seeds 32 | 0.9275 (0.9153-0.9381) | 40.80 ms | 53.90 ms |
| width 256 / seeds 32 | 0.9275 (0.9153-0.9381) | 41.20 ms | 55.00 ms |

Exact sample equality and the flat width sweep show that approximate head-graph
search is not the meaningful cap-4096 loss source. Width 64 is selected for the
seed-count sweep by the registered recall/CI/latency order. Exact-sample recall
below 0.9900 fires the registered 8192/16384 cap-growth branch. The owner oracle
attributes most loss to bounded sample coverage but its 0.9970 result also
shows a remaining traversal ceiling below the final 0.9990 gate.

The checked-in `screen-b-seeds-suite.json` and `screen-c-caps-suite.json` both
pass audit and dry-run expansion. They preserve BW4/H100, graph degree 32,
RaBitQ neighbor scoring, query identity, and the three-owner physical topology.

Screen B has now completed under unanimous clean-release extension provenance
at branch head `53b62bbea...`. Returned seed counts 32/64/128 all measured
distinct recall 0.9280 with identical 0.9158-0.9385 confidence intervals. Warm
p50 was 40.30/40.20/41.40 ms respectively. The 128-seed invariant passed, so
the equality is not silent truncation. Seed count is not the cap-4096 limiter;
the required exact-sample cap-growth suite is complete.

Cap 8192 measured recall 0.9250 (CI 0.9126-0.9357), p50 43.5 ms, and
51,791,485 cached-head bytes. Cap 16384 measured recall 0.9440 (CI
0.9330-0.9533), p50 45.2 ms, and 103,590,793 cached-head bytes. Both passed
topology, provenance, and remote engagement. The cap-16384 interval overlaps
the cap-4096 width-64 seed cells, so the registered tie-break chooses the lower
p50 and lower-head-byte cap-4096 / persisted width-64 / 64-seed cell (recall
0.9280, p50 40.2 ms) for Phase 2. The exact-neighbor trigger is false because
0.9280 is 0.0690 below owner recall 0.9970, not within 0.0050.

## Requested review focus

1. Confirm normal builds cannot select owner-scan, exact-sample, independent
   head controls, or exact-neighbor behavior.
2. Check that exact-sample ranks the already-persisted full-precision sample and
   that approximate width and returned seed count are truly independent.
3. Check that exact-neighbor changes only neighbor traversal scores for the
   fixed seed arm and remains bounded/diagnostic.
4. Check multi-variant fixture reuse for cross-arm state leakage, especially
   session GUC propagation and backend-local epoch/head caches.
5. Check result normalization/provenance and shared-build/shared-storage labels
   for misleading attribution.
6. Check Screen A's width-64 selection and the exact-sample-triggered cap branch
   against Task 180 Phase 1.
7. Check that the feature-only scored-candidate pool makes width 64 / seeds 128
   a real returned-seed arm without increasing graph-search width, and that the
   invariant prevents mislabeled short seed sets.

## Next action while review is open

Run the Phase 2 10k/50k/100k confirmation matrix with exactly production
persisted width 32 / seeds 32, owner-scan oracle, and selected bounded persisted
width 64 / seeds 64 against one immutable cap-4096 generation per scale.
