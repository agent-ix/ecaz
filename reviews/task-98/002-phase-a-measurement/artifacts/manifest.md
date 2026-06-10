# Manifest: Task 98 Packet 002 Phase A Measurement (complete)

- Head SHA: `eb7183a65` (cold-read batching fix `a1122aac8` + CLI snapshot
  warning; kernels/routing from `96c6e3476` lineage)
- Task bucket: `reviews/task-98/`; packet: `002-phase-a-measurement/`
- Lane: local PG18 pgrx, Apple M5 Pro; database `task93_bench` (dropped and
  recreated immediately before the cited run — see finding 3)
- Extension install sha256 `4fff2050…` (`install-ecaz-pg18.log`, final
  entry; verified on disk)
- Fixtures: dbpedia real10k/50k/100k, `ec_hnsw` `storage_format=turboquant`
  (m=16, ef_construction=128); suite config
  `crates/ecaz-cli/suites/task98-phase-a-hnsw-exact-modes.json`
- Cells: {tiled_lut, int8_approx} × {kernel-on, kernel-off} × 3 corpora;
  `ec_hnsw.disable_binary_prefilter=on` on every cell (finding 1)

## Findings (chronological; all packeted commits)

1. **Binary-prefilter shadowing**: default HNSW TurboQuant scans take the
   binary-prefilter branch where exact modes barely participate; Phase A
   cells isolate the modes with the prefilter disabled.
2. **Hot/cold payload root cause** (`a1122aac8`): V3 hot/cold TurboHot
   tuples carry no inline exact payload, so every exact-mode batching arm
   (Task 87's FullLut included) was dead code on modern indexes. Fixed by
   cold-loading payloads at accumulation time into the mode-dispatched
   batch flush.
3. **Stale extension catalog masked the fix** (`eb7183a65`): the width-
   column schema change does not propagate to existing databases; the CLI
   silently returned zero block-kernel rows on the mismatch. The bench
   database must be recreated after counter-schema changes; the CLI now
   warns instead of swallowing the error.

## Width distribution (acceptance criterion 4) — decisive

| mode | corpus | flushes | candidates | lt8 | 8-15 | 16-31 | **ge32** |
|---|---|---|---|---|---|---|---|
| tiled_lut | 10k | 6,629 | 16,689 | 6,218 | 317 | 91 | **3 (0.045%)** |
| int8_approx | 10k | 6,656 | 16,733 | 6,244 | 317 | 92 | **3** |
| tiled_lut | 50k | 11,954 | 30,329 | 11,330 | 498 | 123 | **3 (0.025%)** |
| int8_approx | 50k | 11,971 | 30,352 | 11,345 | 500 | 123 | **3** |
| tiled_lut | 100k | 12,353 | 36,573 | 11,358 | 747 | 238 | **10 (0.081%)** |
| int8_approx | 100k | 12,372 | 36,611 | 11,375 | 747 | 240 | **10** |

Mean batch width 2.5–3.0; ≥32 share ≤ 0.08% at every corpus — far below
the task's 20% threshold. **Scope-down decision: skip the Phase C SVE
cloud measurement.** SIMD value on this surface flows through the
partial-width dispatch (Task 93 packet 004 convention), which the
int8_approx cells demonstrate with full `isa=neon` coverage.

## Recall byte-equality — PASS at all six cells

tiled_lut 0.9672 / 0.9417 / 0.9906→0.8906; int8_approx 0.9656 / 0.9375 /
0.8875 — identical kernel-on vs kernel-off at every (mode × corpus).

## Counters and latency

- int8_approx: full NEON coverage (e.g. 100k: 36,611 candidates,
  isa=neon, ~300 ns/cand); kernel-on p50 faster at 100k (5.76 vs 6.92 ms),
  parity at 10k (4.57 vs 4.40).
- tiled_lut: scalar backend by design this phase (isa=scalar rows);
  latency mixed within host noise (10k on dramatically faster 6.12 vs
  15.3 ms; 100k 7.84 vs 6.63) — no consistent regression; the tiled SIMD
  question follows the same partial-width form as int8 if pursued.
- Kernel-off cells emit zero block-kernel rows (clean toggles).

## Artifacts

Suite outputs, 24 per-cell logs, install log (4 installs across the
debugging arc, final one cited), shared truth caches.
