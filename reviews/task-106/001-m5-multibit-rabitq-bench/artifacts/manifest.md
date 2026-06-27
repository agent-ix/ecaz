# Task 106 packet 001 — provenance manifest

- Head SHA: `f422aeec4342389d34dfdc52e6671f8783efa3f6` (branch
  `task-106-unified-driver-closeout`)
- Host / lane: Apple M5, `Darwin 25.4.0 arm64` (aarch64, NEON). Dev lane.
  AVX2 is built but cfg-gated to x86 (not executable here); Intel/G4 are
  separate lanes.
- PG: PostgreSQL 18.3 (Homebrew), pgrx 0.17 managed instance, port 28818,
  ecaz 0.1.1.
- Corpus: `ec_hnsw_real_10k` (real DBpedia, 10000 rows, dim 1536, 200
  queries), staged at
  `/Users/peter/dev/tqvector/data/task31_m5_dbpedia_staged/`
  (sha256 `c67c5810…a35e75`). IVF nlists=64, training_sample_rows=10000,
  rerank default.
- Dates: build/microbench/index-bench 2026-06-12; reviewer remediation
  2026-06-13.

## Commands (reproduce)

Build / static:
- `cargo build --release` → `target/release/libecaz.dylib`
- `cargo check --all-targets --features bench`
- `cargo clippy --lib`
- `cargo test --lib rabitq32 ec_ivf::quantizer ec_ivf::scan ec_spire::options`

pg smoke (real PG, cargo-pgrx managed test instance):
- `cargo pgrx test pg18 test_ec_ivf_rabitq_storage_build_scan_insert_vacuum`
  → raw: `raw-pgtest-ivf-rabitq-roundtrip.log`
- `cargo pgrx test pg18 test_ec_ivf_recall_smoke_compares_exact_hnsw_ivf`
  → raw: `raw-pgtest-ivf-recall-smoke.log`
- `cargo pgrx test pg18 pq_fastscan` / `… test_ec_spire_options_snapshot_sql`
  → raw: `raw-pgtest-spire-pqfastscan.log`

Kernel microbench (criterion):
- `cargo bench --features bench --bench quant_score -- rabitq32_multibit`
  → raw: `raw-criterion-multibit-sweep.log`

Index-level bench:
- Suite runner (reproducible): `ecaz bench suite run --config
  crates/ecaz-cli/suites/task106-m5-ivf-rabitq-multibit.json`
  → `suite/` (suite-manifest + results.jsonl + per-step logs)
- The ad-hoc Auto-gate and SPIRE GUC A/B used the direct command surface
  (`ecaz corpus load` + `ecaz bench latency --task87-candidate-batch-counters`,
  toggling `ec_ivf.scratch_soa_batch_decode` / `ec_spire.candidate_batch_scoring`
  via `ALTER DATABASE`); raw capture in `raw-index-ivf-rabitq-sweep.log`. A
  suite-config form for those two A/B lanes is a follow-up.

## Key cited result lines

Microbench (32-block, median; `raw-criterion-multibit-sweep.log`):
- `block_dispatch_bits2/1024  time: [11.363 µs 11.372 µs 11.383 µs]`
- `scalar_estimate_bits2/1024 time: [34.004 µs 34.140 µs 34.294 µs]` (2.66×)
- `block_dispatch_bits4/1024  time: [12.848 µs 12.853 µs 12.857 µs]`
- `scalar_estimate_bits4/1024 time: [4.6136 µs 4.6153 µs 4.6171 µs]` (block 2.8× slower)

Index-level routing proof (`raw-index-ivf-rabitq-sweep.log`):
- bits=2: `surface=ivf quant=rabitq isa=neon kernel_candidates=530088 scalar_candidates=0 width_ge32=2148` (block kernel engages)
- bits=4: no `quant=rabitq` block-kernel line (arithmetic estimator)

Auto-gate (`raw-index-ivf-rabitq-sweep.log` companion run):
- scratch_soa off: `surface=ivf flushes=0 candidates=0`
- scratch_soa on: `quant=turboquant isa=neon kernel_candidates=530085`

## Artifacts in this packet

- `m5-multibit-rabitq-bench.md`, `m5-index-level-bench.md` — analysis
- `raw-*.log` — raw command/test/bench output
- `suite/` — `ecaz bench suite` run for the IVF rabitq bit sweep
- `request.md`, `feedback/` — review request + reviewer feedback
