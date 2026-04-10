# Task 05: Graph Scan

Status: A3 complete, A4 in progress; 1536 tiled-FWHT quantizer path landed, fixture-backed 10K gate helpers landed, larger rerun still pending

Progress notes:
- Build path is complete.
- Scan lifecycle, query validation, metadata/prepared-query caching, and bootstrap linear scan are implemented.
- AM module split is complete: scan, insert, build, options, cost, vacuum, routine, shared, and
  search all have dedicated modules.
- Planner-facing `ef_search` control-surface groundwork can land independently of ordered scan
  enablement.
- Graph now owns layer-0 neighbor loading and visible-seed expansion helpers.
- Search now owns the bootstrap visible-frontier protocol: trace seeding, discovered-candidate
  registration, scheduler-aware selection/consumption, post-consume progression, visible-seed
  top-up, and single-source refill.
- A3 is closed: graph/search traversal is now the primary ordered scan path, and the linear path
  is the explicit fallback shell only when graph traversal cannot produce an initial ordered result.
- Planner enablement remains gated by ADR-011 and is not part of this task until recall clears A4.
- Recent A4 reference baselines now show the current synthetic fixtures are not a credible gate
  surface by themselves; Task 12 tracks the required real-corpus lane consistent with `NFR-003`.

## Scope

Complete the HNSW scan path from module split through validated recall measurement.

## Subtasks

- [x] **A1: Finish am split.** Extract insert and scan into `am/insert.rs` and `am/scan.rs`. Mechanical refactor, no logic changes.
- [x] **A2: Graph traversal helpers.** The shared layer-0 traversal seam is now in place:
  graph owns neighbor loading and visible-seed expansion, and search owns the bootstrap
  visible-frontier protocol around that traversal.
- [x] **A3: Wire scan.** Graph-first ordered scan runtime is cursor-owned end-to-end on `main`.
  1. `amrescan` prefills the first ordered graph result through `GraphTraversalCursor`.
  2. `amgettuple` drains prefetched graph results and refreshes through the graph cursor.
  3. Linear scan remains the explicit fallback shell only when graph traversal cannot produce the
     initial ordered result.
  4. Retired bootstrap helpers are gated to test/debug surfaces.
- [ ] **A4: Recall gate.** Measure Recall@10 on synthetic data (10K+ vectors, 1536-dim, 4-bit). Brute-force fp32 ground truth. Test at (m=8,ef=40), (m=8,ef=128), (m=8,ef=200), (m=16,ef=200). Gate: Recall@10 >= 89% at m=8 ef=128. If not met, investigate before proceeding.
  - 2026-04-08 evidence on the repaired regular-table 10K x 1536 x 4-bit synthetic fixture harness:
    - `(m=8, ef=40)`: Recall@10 = `8.4%`
    - `(m=8, ef=128)`: Recall@10 = `21.8%` (`FAIL`, required `>= 89%`)
    - `(m=8, ef=200)`: Recall@10 = `26.8%`
    - `(m=16, ef=200)`: Recall@10 = `35.3%`
  - Investigation notes:
    - The original row-at-a-time SPI fixture loader made 10K reruns impractical; batched inserts reduced `1k` fixture reset time from roughly `92s` to roughly `6-7s` and made repaired 10K reruns tractable.
    - An `UNLOGGED`-table optimization attempt produced zero emitted tuples because the resulting 10K index surfaced `dimensions=0` / `tree_height=0` in metadata snapshots. That harness regression was corrected before accepting any recall result.
    - On the repaired regular-table 10K fixture, the exact `tqvector` scorer itself overlaps only `43.1%` with brute-force fp32 truth, and a build-code proxy overlaps `39.4%`. The live graph path at the required budgets remains materially below both (`21.8%` at `ef=128`, `26.8%` at `ef=200`), but rises to `39.1%` by `ef=800`.
    - That means the current A4 failure has two parts:
      1. a real graph traversal / runtime budget gap at the required `ef_search` settings
      2. a larger dataset/config mismatch against the `89%` gate, because the exact quantized path on this corpus is far below the gate even without graph approximation
  - 2026-04-09 evidence after switching the production `1536` quantizer path to tiled FWHT:
    - Exact-only `1k` Recall@10 moved to `77.0%` on the uniform fixture and `81.5%` on the clustered fixture without changing payload layout.
    - A new ignored live `1k` graph-fixture pg-test now passes with `exact Recall@10 >= 70%` and `graph Recall@10 >= 70%` at `(m=8, ef=128)`.
    - The next required A4 step is still a larger rerun on the real gate path; the tiled quantizer change is strong evidence, but it does not by itself clear the `10K` gate.
  - 2026-04-09 harness practicality follow-up:
    - New SQL/debug surfaces now split the `10K` gate into a one-time fixture reset and reusable gate reports: `tqhnsw_graph_scan_recall_fixture_gate_reset(...)` and `tqhnsw_graph_scan_recall_fixture_gate_report(...)`.
    - The first implementation duplicated the `10K` corpus per `m` value; an ignored pg-test stayed in `pg_stat_progress_create_index` phase `building index` for more than `21m` and never reached the reusable report phase.
    - The revised helper now shares one `10K` corpus table across the `m=8` and `m=16` indexes, removing duplicated load work while preserving separate gate reports.
    - Even after the shared-corpus fix, the ignored `10K` timing probe still spent more than `10m` in `building index` and did not yet reach the first reusable report, so the next harness bottleneck is clearly one-time index build cost, not repeated report computation.
  - 2026-04-09 reference-baseline follow-up:
    - Raw `hnsw-rs` on the current deterministic synthetic fixtures is also weak:
      - source graph, uniform `10K`, `(m=8, ef_search=128)`: `29.0%`
      - source graph, clustered `10K`, `(m=8, ef_search=128)`: `26.0%`
      - source graph, uniform `10K`, `(m=16, ef_search=200)`: `66.5%`
    - That means the current synthetic fixtures are useful for debugging, but not sufficient to
      carry the A4 gate alone.
    - Task 12 (`12-real-corpus-recall.md`) now tracks the real-corpus benchmark lane required by
      `NFR-003` so A4 can be evaluated on DBpedia OpenAI embeddings or a documented equivalent.

## Owns

- `FR-009`
- Initial `NFR-003` validation (recall gate only)

## Dependencies

- Tasks 01-04 (all complete)

## Unblocks

- Task 06 (graph-aware insert needs traversal helpers from A2)
- Task 07 (vacuum needs traversal helpers from A2)
- Task 10 (full benchmarks need working scan)
- Task 11 / D2 (planner wiring gated on A4 recall gate)
- End-to-end indexed ANN search

## Deliverables

- `am/scan.rs` and `am/insert.rs` extracted modules
- `hnsw_search` shared traversal helper
- `amgettuple` returning distance-ordered results
- `tqhnsw.ef_search` GUC
- Recall@10 measurement harness and initial results

## Primary Tests

- `TC-113`, `TC-120`, `TC-121`, `TC-131`: scan behavior
- Recall@10 benchmark at multiple (m, ef) configurations
- Buffer pin bound check during scan of 50K-row index

## Notes

- A2 is the hardest subtask. Reference pgvector `hnsw_search_layer` in `hnswscan.c` (~150 lines) but expect more complexity due to raw page tuple decoding.
- The traversal helper should be generic over scoring function signature to support both scan (LUT `score_ip_encoded`) and insert (code-to-code `score_ip_codes_lite`).
- A4 is a gate: if recall fails, all downstream work (insert, vacuum, benchmarks) is premature.
- Ordered result buffering / graph-first execution are now in place on `main`.
- A4 currently remains a stop-ship gate. Planner activation, insert, vacuum, and SIMD merge should stay blocked until the project decides whether to change the measurement path / dataset assumptions or to raise recall on the current quantized path.
- Post-v0.1 follow-up notes for the A3 runtime arc:
  1. The raw `self as *mut Self` aliasing pattern in `GraphTraversalPrefetchContext::run` is
     contained and approved, but a future search API that takes a visitor/trait object would be
     cleaner.
  2. Graph exhaustion currently transitions to `Exhausted`, not back to linear fallback. Keep that
     contract for v0.1 and revisit after A5 introduces live-write connectivity gaps.
  3. `with_visible_frontier_mut_and_bootstrap_expansion` is still a borrow-splitting helper that
     could disappear if graph-phase state is later separated from `TqScanOpaque`.
  4. `GraphTraversalCursor` is reconstructed several times per call site; likely inlined away, but
     could be held for readability in a future optimization pass.
  5. A4 should leave behind a persistent integration-level ordered-result regression test, not just
     one-off recall numbers, so graph-first scan ordering stays protected after planner/insert work
     starts touching adjacent code.
  6. If A5/A6 increase graph-phase state further, split the graph runtime state inside
     `TqScanOpaque` into its own sub-struct so cursor/frontier code can borrow directly without the
     current helper-based field splitting.
  7. After A4 fixes the recall/exhaustion behavior in practice, write down the chosen
     graph-exhaustion policy explicitly in the spec/ADR surface so planner and insert work do not
     have to infer it from scan implementation details.
