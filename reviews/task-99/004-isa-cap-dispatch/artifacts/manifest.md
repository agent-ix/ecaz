# Manifest — Task 99 packet 004: `ecaz.isa_cap` dispatch cap

- Task bucket / packet: `reviews/task-99/004-isa-cap-dispatch/`
- Code: `src/quant/isa.rs` (capped selector, session cap, env fallback),
  `src/am/common/isa_cap.rs` (GUC + sync), driver chokepoint sync in
  `src/am/common/candidate_batch/drivers.rs`, registration in
  `src/am/mod.rs` / `src/am/common/mod.rs`.
- Validation (local Intel desktop, head = this commit):
  - `cargo-test-quant-isa.log` — 8 passed (incl. the new
    `capped_selection_limits_but_never_fakes`, `cap_encoding_roundtrips`,
    parse tests; pre-existing selector tests unchanged).
  - `cargo-test-candidate-batch.log` — 19 passed (driver sync compiled
    in, `cfg(test)` no-op stub).
  - `cargo-test-lut32.log` — 11 passed.
  - `cargo-clippy.log` — clean under `-D warnings`
    (`--all-targets --no-default-features --features pg18`).
  - Filter safety: all runs `cargo test --lib` (no pg_test selection);
    release backend re-installed afterwards regardless (below).
- End-to-end smoke (`smoke-isa-cap-scalar-latency.log`):
  - Backend rebuilt + installed: release, sha256 `b106bcd4…`;
    `ecaz_build_profile()` probe = `release` after restart.
  - `bench latency --prefix t99_qjl_hnsw_1024 --sweep 32
    --iterations 50 --session-guc ecaz.isa_cap=scalar
    --task87-candidate-batch-counters`
  - Counter row: `surface=hnsw quant=turboquant_qjl isa=scalar`,
    scalar_candidates=34,513 of 35,281 (~1,091 ns/c) — versus the same
    cell uncapped in packet 003: `isa=avx2`, 263 ns/c. Zero avx2
    attribution under the cap → the GUC reaches block dispatch.
  - Functional smoke only — not a benchmark claim (50 iterations,
    single sweep).
- G4 supplemental config: `gen_t99_g4_neon_cap.py` →
  `t99-g4-neon-cap-suite.json` (32 steps, derived from packet 002's
  kernel-on cells, excludes kernel_status/no_kernel cells); dry-run
  clean (`suite-dry-run.log`, `dry-run-manifest.json`).
- Timestamp: 2026-06-12 (PDT)
