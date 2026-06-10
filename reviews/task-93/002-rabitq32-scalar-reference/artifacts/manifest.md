# Manifest: Task 93 Packet 002 RaBitQ32 Scalar Reference

- Head SHA: `b3dcf46d7b2dbbecc4384894a3cff85b40d75a47` (merge of `origin/main`
  into `task-93-rabitq-block-kernel`; the Phase 2 slice itself landed at
  `5ea1a1945` and was re-applied onto main's structure in the merge)
- Task bucket: `reviews/task-93/`
- Packet path: `reviews/task-93/002-rabitq32-scalar-reference/`
- Lane: local PG18 pgrx fixture, Apple M5 Pro (arm64, macOS)
- Host/socket: `/Users/peter/.pgrx`, port `28818`
- Database: `task93_bench`, dropped and recreated immediately before the run
  so the extension catalog is built from the head-SHA install
- Extension install: `target/debug/ecaz dev install ecaz-pg-test --pg 18`,
  installed backend sha256
  `a7da2411bd5a6ceb9b4a47b3d36dc3aa1ee6cb3297b3070c65901a88c959e164`
  (`install-ecaz-pg18.log`)
- Fixtures: dbpedia real10k (`data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_*`,
  10000 rows) and dbpedia real100k
  (`data/task60_m5_dbpedia_staged/ec_real_100k_*`, 100000 rows), 1536-dim
- Storage format: `ec_ivf`, `storage_format=rabitq`, `quant_bits=1`,
  `rerank=off`; real10k `nlists=64`, real100k `nlists=256`
- Isolation: isolated one-index-per-table prefixes
  `task93_p2_ivf_rabitq_real10k` and `task93_p2_ivf_rabitq_real100k`
- Suite config: `crates/ecaz-cli/suites/task93-phase2-ivf-rabitq.json`
  (committed with this packet)
- Runner command: `target/debug/ecaz bench suite run --config
  crates/ecaz-cli/suites/task93-phase2-ivf-rabitq.json --database task93_bench
  --host /Users/peter/.pgrx --port 28818 --manifest-output
  .../suite-manifest.json --results-output .../results.jsonl --log-file
  .../suite-run.log`
- Timestamp: 2026-06-09 (suite-manifest.json carries per-step timestamps)
- Kernel ISA note: this host reports NEON at runtime; the Phase 2 NEON backend
  is a fallback stub that delegates to scalar and returns `Isa::Scalar`, so
  all `[block-kernel-counters]` rows in this packet report `isa=scalar` by
  design (ADR-076 fallback-attribution contract).
- Kernel-on cell: `--ivf-scratch-soa-batch-decode`
  (GUC `ec_ivf.scratch_soa_batch_decode=on`), routing IVF RaBitQ bits=1
  posting batches through `score_rabitq_bits1_batch_for`.
- Kernel-off cell: default GUCs (`ec_ivf.scratch_soa_batch_decode` is off by
  default — a Task 51 diagnostic mode), i.e. the pre-kernel per-candidate
  production scoring path. Production default behavior is therefore unchanged
  by this slice.

## Validation artifacts (merged HEAD)

### `cargo-clippy.log`

- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: clean.

### `cargo-test-rabitq32.log`

- `cargo test -p ecaz --lib --no-default-features --features pg18 rabitq32 -- --test-threads=1`
- 3 passed: strict `f32::to_bits()` parity vs an independent forced-scalar
  byte-LUT anchor for the scalar tail and the 32-wide block, plus ADR-076
  tolerance vs production-dispatched `estimate_ip_scalar_only` /
  `estimate_ip_bits1_batch` including the one-shot production per-candidate
  vs production batch agreement check.

### `cargo-test-candidate-batch.log`

- Focused `am::common::candidate_batch` run: 10 passed. Includes RaBitQ
  width gates (`<32` scalar-only attribution, `>32` block+tail attribution)
  and shape/meta/count mismatch rejecting before any counter increments.

### `cargo-test-ivf-quantizer.log`

- Focused `ec_ivf::quantizer` run: 27 passed. Includes
  `rabitq_bits1_batch_dispatch_routes_through_block_kernel`, which proves the
  runtime SoA helper's bits=1 arm is bit-exact with the scalar kernel
  (35 payloads: one 32-block plus 3-tail).

### `cargo-test-diskann-quantizer.log` / `cargo-test-hnsw-scan.log`

- Focused runs covering the new `score_ip_batch` overrides: 12 and 81 passed.

## Bench artifacts

### `load-ivf-rabitq-real10k.log` / `load-ivf-rabitq-real100k.log`

- Suite `load` steps (commands embedded in `suite-run.log` and
  `suite-manifest.json`). real10k: corpus sha256 `c67c5810…`, 200-row query
  file sha256 `a2c191bb…`. Index reloptions as listed above.

### `recall-ivf-rabitq-real10k-kernel-{on,off}.log`

Recall byte-equality gate — PASS. Identical recall at every cell:

| nprobe | kernel-on recall@10 | kernel-off recall@10 |
|---|---|---|
| 8 | 0.8953 (ci 0.8692–0.9167, p10 0.80, p50 0.90, p90 1.00, worst 0.60, ndcg 0.9974) | identical |
| 32 | 0.8953 (ci 0.8692–0.9167, worst 0.40, ndcg 0.9975) | identical |

### `recall-ivf-rabitq-real100k-kernel-{on,off}.log`

| nprobe | kernel-on recall@10 | kernel-off recall@10 |
|---|---|---|
| 32 | 0.7719 (ci 0.7228–0.8145, p10 0.61, p50 0.80, p90 0.90, worst 0.50, ndcg 0.9885) | identical |

### `latency-ivf-rabitq-real10k-kernel-{on,off}.log`

Key `[block-kernel-counters]` rows (kernel-on; kernel-off cell emits zero
block-kernel rows, confirming a clean toggle):

```text
[block-kernel-counters] command=latency label=nprobe=8 surface=ivf quant=rabitq isa=scalar flushes=192 candidates=38379 ... kernel_flushes=162 kernel_candidates=37920 kernel_elapsed_ms=30.064352 scalar_flushes=30 scalar_candidates=459 scalar_elapsed_ms=0.363331
[block-kernel-counters] command=latency label=nprobe=32 surface=ivf quant=rabitq isa=scalar flushes=651 candidates=155298 ... kernel_flushes=619 kernel_candidates=154784 kernel_elapsed_ms=79.645346 scalar_flushes=32 scalar_candidates=514 scalar_elapsed_ms=0.274374
```

Wall latency (32 iterations, concurrency 1):

| nprobe | kernel-on p50/p95/p99 | kernel-off p50/p95/p99 |
|---|---|---|
| 8 | 1.84 / 2.35 / 2.94 ms | 1.05 / 1.23 / 1.78 ms |
| 32 | 3.71 / 4.67 / 5.78 ms | 1.45 / 2.47 / 2.75 ms |

### `latency-ivf-rabitq-real100k-kernel-{on,off}.log`

```text
[block-kernel-counters] command=latency label=nprobe=32 surface=ivf quant=rabitq isa=scalar flushes=1646 candidates=410113 ... kernel_flushes=1614 kernel_candidates=409568 kernel_elapsed_ms=149.232572 scalar_flushes=32 scalar_candidates=545 scalar_elapsed_ms=0.199375
```

| nprobe | kernel-on p50/p95/p99 | kernel-off p50/p95/p99 |
|---|---|---|
| 32 | 5.87 / 10.4 / 15.2 ms | 3.43 / 5.75 / 7.94 ms |

### `suite-manifest.json` / `results.jsonl` / `suite-run.log`

- Structured runner outputs; `suite-manifest.json` records per-step commands
  including the `--ivf-scratch-soa-batch-decode` toggle per cell.

### `truth-cache/`

- Exact-scan ground-truth caches keyed by corpus rows/queries/dim/k, shared by
  the kernel-on and kernel-off recall cells of each fixture.

## Interpretation

- Recall byte-equal at every (corpus × nprobe) cell — the Task 93 per-AM
  validation gate 1 passes for the IVF surface.
- Direct `[block-kernel-counters]` rows attribute 98.6–99.9% of batch
  candidates to the 32-wide kernel path with the remainder on scalar tails,
  and drop to zero in the kernel-off cell.
- Kernel-on wall latency is slower than kernel-off, as expected for Phase 2:
  the kernel-on cell replaces the NEON-dispatched per-candidate production
  scorer with the deliberately deterministic forced-scalar block kernel, and
  it only engages behind the default-off `ec_ivf.scratch_soa_batch_decode`
  diagnostic GUC. The ≥2× scoring-share gate is a per-ISA gate that applies
  when the real NEON/SVE/AVX2 backends land (Phases B–D); per the task's stop
  conditions this scalar baseline is documented, not backed out.
