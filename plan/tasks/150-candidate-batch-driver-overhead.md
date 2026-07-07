# Task 150: candidate_batch driver overhead strip (per-block clocks, ISA resync, pointer gather)

Status: **proposed** (2026-07-04). Owner: unassigned. Priority: P2

## Why

The shared batch-scoring driver taxes every 32-block on the production scan
path of all four AMs, for every quantizer:

- `score_width_cascade` calls `Instant::now()` twice per 32-block plus
  `record_run` counter updates (`src/am/common/candidate_batch/drivers.rs:57,65`),
  and every partial flush does the same (`mod.rs:756,822`). This is
  instrumentation, not scoring, and it is unconditional.
- `sync_session_isa_cap()` runs once per cascade flush (`drivers.rs:51`).
- `current_isa()` re-runs `HostIsaFeatures::detect()` once per 32-block and
  once per partial (`src/quant/rabitq32/mod.rs:67,95,243,262`) instead of
  caching the selected `Isa` the way `simd::backend()` caches its OnceLock.
- Each batch flush builds a fresh `Vec<&[u8]>` of code refs
  (`src/am/common/candidate_batch/mod.rs:736-740`), and the IVF entry builds a
  `CandidateBatch` with a per-candidate `push` loop
  (`src/am/ec_ivf/quantizer.rs:583-586`).

Individually small, but they sit inside the hottest loop in the extension and
apply to RaBitQ, TurboQuant, and every other quant family simultaneously.

## Scope

- Gate the per-block/per-partial `Instant::now()` timing + width/kernel counter
  attribution behind a session GUC (default off) or compile-time feature, so
  the stage-profile workflow can still enable it. Keep coarse per-flush
  counters if they are needed by existing tests/dashboards — the target is the
  per-32-block clock reads, not observability as such.
- Cache the capped `Isa` selection so per-block dispatch is a load, not a
  re-detection; keep `ecaz.isa_cap` semantics (sync once at the flush
  chokepoint remains fine).
- Eliminate the per-flush `Vec<&[u8]>` gather where the payloads are already a
  contiguous slab (IVF SoA, SPIRE columnar): iterate `chunks_exact` directly.
- A/B before/after per CLAUDE.md: 10k/50k/100k recall+latency+storage on at
  least the IVF RaBitQ bits=1 and TQ lanes (both consume this driver), recall
  must be byte-identical (no scoring change).

## Out of Scope (hard)

- No kernel changes, no scoring/dispatch semantic changes, no format changes.
- Do not delete the width-bucket counter machinery — Tasks 152/159 need it.

## Gate / Exit Criteria

- Byte-identical recall, measured latency delta (win or honest null) at
  10k/50k/100k on both lanes, and the instrumentation still reachable via the
  gate for profiling tasks. Closes on the A/B evidence landing.
