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
- Both checked-in suite configs pass `ecaz bench suite audit`.
- Live three-owner PG18 10k smoke: one succeeded step, no missing/stale
  artifacts, all six thresholds pass.
- Installed release extension provenance is unanimous on three nodes at
  `174da94ef...`; topology totals 10,000 disjoint rows/records with zero
  non-owned rows/orphans, remote engagement passes, and all four diagnostic
  modes emit recall/latency/storage/head rows.

The smoke intentionally uses only two queries and two measured latency
iterations. Its measurements are path-validation only and MUST NOT be used for
candidate selection or the Task 180 verdict.

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
6. Check the registered `screen-a-suite.json` matrix against Task 180 Phase 1
   steps 1–3. The width-selected seed-count sweep will be checked in only after
   screen A identifies the pre-registered best width.

## Next action while review is open

Run screen A at 100k (200 held-out queries / 2,000 membership trials; 50 warm
latency iterations after 10 warmups), then register the best-width seed-count
sweep. Conditional cap-growth and exact-neighbor arms will run only if their
task triggers fire.
