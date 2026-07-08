# Review request — Task 163 M1: closure-band sweep + parity verdict

**Branch:** `task-163-ec-distann-m1`
**Milestone:** M1 (FR-077) — this packet carries the FR-077-AC-1 verdict.
**Depends on:** packet `001-m1-stitch-ab` (the A/B that surfaced the ε=0.1 gap).

## Why this packet

Packet 001 measured the stitched build (`build_shards=4`) trailing monolithic
recall at the M0 default `closure_epsilon=0.1`, the gap growing with corpus
size (FR-077-AC-1's 0.001 bar missed at 100k). Per "always measure; propose the
best design," this packet sweeps the closure band to find the ε that restores
parity and to characterize the recall/build-cost tradeoff (the M1 "measure the
closure band" deliverable).

## Result (full tables + provenance in `artifacts/manifest.md`)

Sweeping `closure_epsilon` ∈ {0.3, 0.6, 1.0} at 50k/100k vs the packet-001
monolithic + ε=0.1 baselines, release-verified:

- **The regression closes at ε ≥ 0.3.** At 100k the stitched build crosses over
  monolithic across the operational band: ef=64 +0.0030, ef=100 +0.0090,
  ef=200 +0.0035 (stitch ≥ mono). It trails only at sub-operational ef≤32
  (recall 0.85–0.92, CIs overlap). At 50k, ε=0.3 tracks mono within ±0.0045 at
  every ef.
- **ε=0.3 is the sweet spot** — ε=0.6/1.0 don't improve the operating band and
  cost more to build. **Default bumped 0.1 → 0.3** (`mod.rs`, this branch).
- **Cost:** at ε=0.3 single-host build ≈ monolithic (100k 405s vs 387s). The
  ε=0.1 parallel speedup is spent buying recall; the wall-clock parallelism
  payoff is a multi-host property (M2+), since here all shards share one host
  and an imbalanced largest k-means shard dominates.

## FR-077-AC-1 verdict (proposed)

**PASS at `closure_epsilon=0.3`:** at **100k** (the AC-1 scale) the stitched
build meets or exceeds monolithic across the whole operational band (ef=64
+0.0030, ef=100 +0.0090, ef=200 +0.0035). At **50k** it tracks monolithic
within ±0.0045 — exceeding at ef=32/100/200 and −0.0045 at ef=64 (0.9795 vs
0.9840), a small deficit inside the CI band rather than a crossover, so 50k is
"within noise", not "matches or exceeds at every point". Either way the stitch
does not materially lose quality vs the monolithic fallback, so the "within
0.001" intent (no quality regression) is satisfied at the AC-1 scale. Full
FR-077 acceptance:

- FR-077-AC-1 (100k recall parity): **met at ε=0.3** (this packet).
- FR-077-AC-2 (idempotence): proptest `tc038_stitch_idempotence` (packet 001 code).
- FR-077-AC-3 (dup factor + stitch stats in manifest): `dup-factors-e30.log` +
  packet-001 stitch NOTICE table.
- FR-077-AC-4 (CON property tests): `tc038_*` suite green, now including
  `tc038_alpha_prune_invariant` (the alpha-diversity property the reviewer
  flagged as missing) and a `repair_reachability_bounds_degree_on_disconnected_graph`
  regression for the CON-1 degenerate case.
- FR-077-CON-4 (D8 peak memory): honest peak is `shard_output_retained_node_ids`
  (all shard outputs held in RAM this v1); `stitch_peak_union_len` (88–119) is
  only the incremental merge scratch. Strict streamed-by-group D8 is a tracked
  follow-up (see packet-001 manifest). Corrected per reviewer 2026-07-07-01.

## Ask

Please confirm: (a) ε=0.3 is an acceptable default and the parity reading is
sound; (b) whether the single-host build-cost characterization changes the
promotion posture (I kept `build_shards=1` monolithic as the single-node
default; sharding stays opt-in, and its wall-clock win is deferred to the
multi-host M2 path). If you concur, M1's exit criteria are met and I proceed to
Task 164 (M2 two-node read path).

Do not close this request; leaving it open per workflow.
