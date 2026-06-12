# Review request — Task 99 packet 004: `ecaz.isa_cap` block-kernel dispatch cap

- Task: 99 (measurement infrastructure for the G4 lane; operator
  decision 2026-06-11 — "we are data driven! lets gather the stats")
- Coder: Task 102/103 author lane
- Date: 2026-06-11

## Why

On Graviton 4 the dispatcher always selects SVE2 (highest available
ISA), so the NEON kernels for lut32 / qjl32 / grouped-pq can never
execute there — and G4's SVE2 is 128-bit, the same vector width as
NEON, so "SVE2 beats NEON on G4" is an assumption, not a measurement.
Task 94's Phase B ("Graviton 4 measurement with SVE disabled or the
NEON dispatch path forced") was un-executable: the only forcing
mechanism (`ECAZ_SIMD`) targets the legacy `quant::simd::SimdBackend`
layer, not the block-kernel `Isa` dispatcher.

## What

1. `src/quant/isa.rs`: `select_highest_isa_capped` — caps dispatch
   preference, never fakes an unavailable ISA (capping below the host's
   SIMD tiers lands on scalar). Session cap global (atomic, sentinel
   255) + `ECAZ_ISA_CAP` env fallback for non-PG consumers (criterion);
   invalid env value panics, mirroring the `ECAZ_SIMD` precedent.
   `current_isa()` now applies the effective cap.
2. `src/am/common/isa_cap.rs` (new): `ecaz.isa_cap` enum GUC
   (none|scalar|neon|sve|sve2|avx2, Userset, default none), registered
   from `am::register_gucs()`; synced into the quant layer at the
   single batch-dispatch chokepoint (`drivers::score_width_cascade`),
   so per-step `session_gucs` in suite configs Just Work.
3. Counter attribution unchanged and truthful: dispatch arms return
   the ISA that actually ran, so capped cells report `isa=neon`.
4. `artifacts/gen_t99_g4_neon_cap.py` + `t99-g4-neon-cap-suite.json`:
   the G4 supplemental pass — every kernel-on bench cell from the
   packet 002 profile (excluding kernel_status-marked cells), re-tagged
   with `ecaz.isa_cap=neon` (32 steps). Dry-run clean
   (`artifacts/suite-dry-run.log`).

## Audit notes for the reviewer

- Every production dispatch site routes through `current_isa()`
  (verified by grep across all seven family `mod.rs` + the driver; the
  only direct `is_*_feature_detected!` probes at dispatch level are
  test helpers), so the cap is complete for block/octet/partial paths.
- Deliberately NOT covered: the legacy `quant::simd` layer (`ECAZ_SIMD`
  exists for it) and non-batch one-off scoring rows.
- No unit test mutates the session-cap global — concurrent per-family
  parity tests dispatch through `current_isa()`, and a transient cap
  would race them (same isolation class as
  `CANDIDATE_BATCH_COUNTER_TEST_LOCK`). Cap semantics are covered by
  pure-function tests; the GUC-to-kernel path is validated end-to-end
  by the capped bench cells' `isa=neon` counter rows.

## Validation (complete — see artifacts/manifest.md)

- `cargo test --lib quant::isa` 8 passed; `candidate_batch` 19 passed;
  `lut32` 11 passed; clippy clean under `-D warnings` (logs packeted;
  run after the packet 003 suite finished to avoid polluting its
  latency cells).
- End-to-end smoke: release backend `b106bcd4…` installed + probed;
  `bench latency` on `t99_qjl_hnsw_1024` with `ecaz.isa_cap=scalar`
  flips the cell's counter attribution from `isa=avx2` (263 ns/c,
  packet 003) to `isa=scalar` (34,513/35,281 candidates scalar,
  ~1,091 ns/c) — zero avx2 rows under the cap.
