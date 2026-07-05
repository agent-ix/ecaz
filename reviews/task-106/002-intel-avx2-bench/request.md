# Task 106 packet 002 — Intel AVX2 bench and AWS prep

Status: review request (2026-06-13). Coder lane.

## Summary

This packet validates the Task 106 multi-bit RaBitQ route on the Intel AVX2
lane. AVX2 compiled and ran cleanly. The data does **not** justify changing the
M5 routing decision: bits=2 should use the block kernel, but bits=4 should stay
on the arithmetic estimator.

The suite used the existing local `tqhnsw_real_10k` DBpedia fixture. Its corpus
and query hashes match the m5 Task 106 baseline exactly; no new corpus is used
as evidence.

## Outcomes

- **AVX2 compile/static:** release build, all-target bench check, and clippy
  passed. A warning-only `mb_neon.rs` import cfg fix was added so x86 builds
  are clean.
- **Kernel microbench:** at 1536 dims, bits=2 block dispatch is faster than
  scalar estimate (`69.377 us` vs `138.88 us` median); bits=4 block dispatch is
  much slower than scalar estimate (`72.900 us` vs `12.810 us` median).
- **Index-level suite:** bits=1 and bits=2 emitted AVX2 block-kernel counters;
  bits=4 and bits=8 emitted no block-kernel counter rows and remained on the
  estimator route.
- **PG18 smoke:** focused IVF RaBitQ roundtrip and recall smoke tests passed.

## Validation

- `cargo build --release`
- `cargo check --all-targets --features bench`
- `cargo clippy --lib`
- `cargo bench --features bench --bench quant_score -- rabitq32_multibit`
- `cargo test --lib rabitq32`
- `cargo test --lib ec_ivf::quantizer`
- `cargo test --lib ec_ivf::scan`
- `cargo test --lib ec_spire::options`
- `cargo pgrx test pg18 test_ec_ivf_rabitq_storage_build_scan_insert_vacuum`
- `cargo pgrx test pg18 test_ec_ivf_recall_smoke_compares_exact_hnsw_ivf`
- `ecaz bench suite audit/run/report` using
  `task106-intel-ivf-rabitq-multibit.json`

## Artifacts

- `artifacts/manifest.md` — provenance, commands, fixture hashes, key result
  lines, AWS prep notes.
- `artifacts/raw-*.log` — raw build/test/bench/pgrx logs.
- `artifacts/suite/` — suite manifest, per-step logs, and `results.jsonl`.
- `artifacts/suite-report.log` — normalized suite report.

## Follow-up

AWS should use this packet's suite shape but bind fixture paths to the existing
AWS/staged DBpedia 10k fixture. Before comparing numbers, verify corpus/query
hashes match `c67c5810...a35e75` and `a2c191bb...04ae8`.
