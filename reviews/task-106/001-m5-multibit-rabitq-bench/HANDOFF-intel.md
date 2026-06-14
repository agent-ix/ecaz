# Task 106 — handoff to the Intel (AVX2) lane

Branch: `task-106-unified-driver-closeout` (head `0aadd6239` at handoff).
M5/NEON dev lane is done and pushed. This note is what the Intel agent picks up.

## The one open routing question for Intel

On M5/NEON the multi-bit RaBitQ **block kernel loses at bits=4** to the
per-candidate `NeonBits4` (microbench: 12.9 vs 4.6 µs), so IVF bits=4 routes
to the arithmetic estimator, not the block kernel. **AVX2 may flip this** —
`mb_avx2.rs` gathers the dequant LUT with `permutevar8x32` (hardware gather),
which the NEON path lacks (it does a per-dim scalar gather). If the AVX2 block
kernel beats `Avx2Bits4` at bits=4, change the IVF dispatch to block-route
bits=4 on AVX2 (`src/am/ec_ivf/quantizer.rs`, the `rabitq_bits == 4` arm) and
update the matrix/ADR.

bits=2 is expected to win on AVX2 too (no per-candidate bits=2 SIMD kernel
exists), but confirm.

## What to run on Intel

1. **Build/compile the AVX2 path** (it is cfg-gated to x86 and was never
   compiled on M5): `cargo build --release`, `cargo check --all-targets
   --features bench`, `cargo clippy --lib`. `mb_avx2.rs` uses only intrinsics
   already shipping in `rabitq.rs`/`qjl32/avx2.rs`, but this is its first real
   compile.
2. **Kernel microbench**: `cargo bench --features bench --bench quant_score --
   rabitq32_multibit` → bits 2/4 × 5 dims, `block_dispatch` vs
   `scalar_estimate`. Compare to the M5 table in
   `artifacts/m5-multibit-rabitq-bench.md`.
3. **Index-level suite** (reproducible, real 10k): install a **release**
   backend (`cargo pgrx install --release` — the suite rejects debug), then
   `ecaz bench suite run --config
   crates/ecaz-cli/suites/task106-m5-ivf-rabitq-multibit.json`. The config's
   `corpus_file` points at the staged DBpedia 10k; repoint it to the Intel
   lane's staged path. Confirm the per-bit counter engagement
   (`isa=avx2`, not neon) and the p50 deltas.
4. **pg smoke**: `cargo pgrx test pg18 test_ec_ivf_rabitq_storage_build_scan_insert_vacuum`
   and `..._recall_smoke...`.

## M5 baseline to compare against (release)

| bits | M5 p50 | M5 routing |
| ---- | ------ | ---------- |
| 1 | 0.67 ms | block kernel |
| 2 | 2.09 ms | block kernel |
| 4 | 1.15 ms | estimator (block 2.8× slower on NEON) |
| 8 | 0.96 ms | estimator |

IVF Auto-gate (release): scratch_soa off 2.77 ms / on 1.16 ms (batch-on wins).

## Still open (not Intel's job unless scoped)

- HNSW grouped-PQ flush-width histogram (the deciding measurement; gap is
  OPEN in ADR-077 §9.2 / matrix).
- G4 (SVE2/NEON-cap) lane.
- Production-scale recall@k vs ground truth.

## Drop a packet

Add the Intel results as `reviews/task-106/002-intel-avx2-bench/` with the
same artifact discipline (manifest.md, raw logs, suite results.jsonl).
