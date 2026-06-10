# Manifest: Task 98 Packet 002 Phase A Measurement (instrumentation finding)

- Head SHA: `96c6e3476`
- Task bucket: `reviews/task-98/`; packet path: `reviews/task-98/002-phase-a-measurement/`
- Lane: local PG18 pgrx, Apple M5 Pro; database `task93_bench`
- Extension install sha256:
  `fb54dc17b41277e08747669b2371103523e81ee978aaa50915ebd662384af226`
  (verified on disk before the run; `install-ecaz-pg18.log`)
- Fixtures: dbpedia real10k/50k/100k, `ec_hnsw` `storage_format=turboquant`
  (m=16, ef_construction=128); prefixes `task98_pa_hnsw_tq_real{10k,50k,100k}`
- Suite config: `crates/ecaz-cli/suites/task98-phase-a-hnsw-exact-modes.json`
- Cells: {tiled_lut, int8_approx} × {kernel-on, kernel-off} × 3 corpora,
  via `ec_hnsw.turboquant_exact_score_mode` + `ec_hnsw.candidate_batch_scoring`,
  with `ec_hnsw.disable_binary_prefilter=on` on every cell (see findings).

## Findings

1. **Binary-prefilter shadowing (run 1, discarded):** on default settings
   HNSW TurboQuant scans run the binary prefilter branch, where exact-mode
   scoring barely participates in traversal — the first run produced zero
   exact-mode counter rows for that reason. The meaningful Phase A cell
   isolates the modes with `disable_binary_prefilter=on`. This materially
   contextualizes Task 98's end-to-end framing: TiledLut/Int8Approx carry
   traversal scoring only when the binary sidecar lane is unavailable or
   disabled.
2. **Open instrumentation gap (run 2, cited):** with the prefilter
   disabled and the GUCs verified in `suite-manifest.json`, the suite still
   records **zero** `surface=hnsw` block-kernel rows in every cell — the
   widened batch arm (`96c6e3476`) is not on the executed path. Prime
   suspects for the next slice: (a) the `TurboQuantHotCold` storage
   descriptor yields `LoadedElementState::None` at element load, sending
   candidates to the per-candidate `exact_score_cached_graph_element` path
   that bypasses payload batching; (b) score-at-load via
   `live_loaded_state_from_exact_payload`'s non-deferred branch. This is
   the same class of gap the task's references predicted ("if counters
   still do not fire on that surface, Phase A first resolves
   instrumentation") and that Task 87 packets 020/022-024 hit for FullLut.
   Width-histogram data is therefore not yet citable for the scope-down
   decision.

## What the run does establish

- Recall is identical between kernel-on and kernel-off in every cell (e.g.
  real10k: tiled_lut 0.9672/0.9672, int8_approx 0.9656/0.9656) — expected,
  since both cells executed the same per-candidate path.
- The mode GUC works end-to-end (modes produce distinct recall values:
  0.9672 vs 0.9656 vs the FullLut-family baselines), so the packet-001 GUC
  slice is functioning.
- Latency tables exist for all 24 cells as a per-candidate-path baseline
  for the eventual kernel-on comparison.

## Artifacts

Suite outputs, per-cell load/recall/latency logs, install log, shared
truth caches.

## Addendum: root cause confirmed (static)

`graph.rs` `Tuple::exact_payload()` returns `None` for `TurboHot` tuples —
the V3 hot/cold TurboQuant layout does not carry exact payloads inline, so
`live_loaded_state_from_exact_payload` never produces
`LoadedElementState::ExactPayload` on modern `storage_format=turboquant`
indexes, and every exact-mode batching arm (Task 87's FullLut included) is
dead on this surface; scoring happens per candidate inside the cold-read
path (`exact_score_cached_graph_element`). This retroactively explains the
Task 87 packet 020/022-024 zero-counter record. The fix slice batches at
the cold-payload read: accumulate cold reads per frontier and score
through the mode-dispatched wrappers.
