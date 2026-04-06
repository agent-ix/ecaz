# Task 10: Full Benchmark Suite

Status: **infrastructure complete** — NFR benchmark runs blocked on Task 05 (scan), Task 06 (insert drift), Task 07 (vacuum quality)

## Scope

Build the complete quality and performance benchmark infrastructure required by NFR-001, NFR-002, and NFR-003, then run the full suite once scan/insert/vacuum are operational.

## Infrastructure Status

### Complete

- [x] **Cargo.toml setup.** `bench` and `dhat-heap` feature flags, dev-dependencies (criterion 0.5, proptest 1.5, iai-callgrind 0.14, dhat 0.3), `[profile.bench]` with debug symbols, 8 criterion + 3 iai `[[bench]]` entries, 2 dhat `[[bin]]` entries.
- [x] **bench_api module.** Feature-gated `pub mod bench_api` in `src/lib.rs` re-exporting all quantizer, hadamard, rotation, codebook, MSE, QJL, and page codec internals needed by benchmarks.
- [x] **Shared data generators (benches/helpers.rs).** `random_unit_vector`, `random_corpus`, `random_clustered_corpus` (Gaussian mixture), `near_duplicate_pairs` (controlled angular distance).
- [x] **Criterion microbenchmarks (8 suites).**
  - `quant_score.rs`: `score_ip_encoded` (6 dim/bit configs), `score_ip_codes_lite` (3), `score_ip_from_parts` (4), `score_ip_encoded_lite` (3), `decode_approximate` (2), batch-1000 throughput.
  - `quant_encode.rs`: Full encode pipeline (9 configs) + encode_pack (3).
  - `quant_prepare.rs`: `prepare_ip_query` (6 configs).
  - `hadamard.rs`: `fwht_in_place` (5 sizes 64-4096), `orthonormal_fwht` (2), `srht` (4), `inverse_srht` (3).
  - `codebook.rs`: `lloyd_max` (4 configs) + dimension sensitivity (5-dim sweep).
  - `bitpack.rs`: pack/unpack MSE (5 configs), pack/unpack QJL (3 dims).
  - `page_codec.rs`: element/neighbor encode/decode, metadata decode, DataPage insert/read element (2 code_lens), DataPage insert/read neighbor (2 counts).
  - `text_io.rs`: `parse_text` and `format_text` (2 dim/bit configs each).
- [x] **iai-callgrind instruction-count benchmarks (3 suites).** `score_ip_encoded`, `score_ip_codes_lite`, `score_ip_from_parts` at 1536/4-bit. `fwht_in_place` at 2048/4096. `pack_mse_indices`, `unpack_mse_indices`, `pack_qjl_signs` at 1536/3-bit.
- [x] **dhat heap profiling (2 binaries).** `dhat_encode` (1000x encode at 1536/4-bit). `dhat_score` (10Kx100 score_ip_encoded, profiler starts after pre-encoding to verify zero-allocation).
- [x] **Property tests (10 quant + 5 page properties).**
  - Quant: SRHT norm preservation, SRHT roundtrip (generic + real-world dims), MSE pack/unpack, QJL pack/unpack, encode determinism, score symmetry, payload_len, score consistency, decode_approximate bounded error.
  - Page: element/neighbor/metadata/ItemPointer roundtrips, encoded_len correctness.
- [x] **Size-of assertions (13 tests).** Payload lengths at 5 bitwidths, MSE/QJL code lengths, struct sizes, page header, HEAPTID capacity, element tuple encoded len, compression ratio bound.
- [x] **Recall integration harness.**
  - Uniform corpus: 50K vectors / 100 queries (primary), bitwidth sweep (2-8), dimension sweep (128-1536).
  - Clustered corpus: 10K vectors / 50 clusters / spread=0.3 (primary), clustered bitwidth sweep.
  - Near-duplicate stress test: ranking preservation at angular distances 0.01-0.2 radians.
  - Metrics: Recall@1, Recall@10, Recall@100, NDCG@10, MAE, Spearman rho, top-k overlap.
- [x] **Fuzz targets (4).** parse_text, unpack_mse, element_tuple_decode, neighbor_tuple_decode.
- [x] **Miri tests (11).** In prod.rs (5): encode/decode, pack/unpack MSE, pack/unpack QJL, score_ip_encoded, score_ip_codes_lite. In hadamard.rs (2): fwht, orthonormal_fwht. In page.rs (4): ItemPointer, element tuple, neighbor tuple, metadata.
- [x] **Makefile targets (18).** bench, bench-%, bench-iai, dhat-encode, dhat-score, proptest, layout-check, miri, fuzz-*, recall, bench-sql-*, ci-quick, ci-nightly.
- [x] **CI pipeline updates.** Layout assertions, property tests (256 cases) on PR. Criterion + benchmark-action (110% threshold), miri on main push. Fuzz on nightly.
- [x] **clippy.toml.** cognitive-complexity=30, too-many-arguments=8.
- [x] **BENCHMARKS.md template.** NFR-001/002/003 reporting tables.
- [x] **SQL benchmark scripts.** bench_sql_latency.sh, bench_storage.sh, bench_recall.py, gen_synthetic_data.py.

### Validated

All criterion benchmarks run successfully with `--quick`. Representative results:
- `score_ip_encoded` at 1536/4-bit: ~10.5µs/score, ~95K scores/sec batch throughput
- `decode_approximate` at 1536/4-bit: ~8.8µs
- DataPage element insert: ~68ns, read: ~48ns (code_len=768)
- All proptest, size_of, and recall smoke tests pass.

### Still Blocked (requires scan/insert/vacuum)

- [ ] **Latency benchmarks (NFR-001).** HNSW p50/p99 on 50K vectors. Sequential scan throughput. Warm/cold cache. Requires working graph scan (Task 05).
- [ ] **Storage accounting (NFR-002).** pg_relation_size, pg_column_size. Requires working index build + scan (Task 05).
- [ ] **Full recall suite (NFR-003).** Recall@10 at all (m, ef) configurations via SQL. Post-insert drift (0/5/10/20%). Post-vacuum recall. MSE-only vs MSE+QJL ablation. Requires Tasks 05, 06, 07.
- [ ] **BC-001 through BC-016.** All spec benchmark cases require a working index.

## Owns

- `NFR-001` (full)
- `NFR-002` (full)
- `NFR-003` (full, beyond the initial recall gate in Task 05 A4)

## Dependencies

- Task 05 (working graph scan for any SQL-level benchmark)
- Task 06 (insert-drift benchmarks)
- Task 07 (post-vacuum quality benchmarks)

## Unblocks

- Performance and quality sign-off for v0.1

## Deliverables

- ~~Reproducible benchmark scripts~~ **done**
- ~~Benchmark infrastructure and tooling~~ **done**
- Benchmark result artifacts (latency histograms, recall tables, storage breakdown) — **blocked**
- Pass/fail against declared NFR targets — **blocked**

## Primary Tests

- `BC-001` to `BC-016` as applicable

## Notes

- The initial recall gate (Task 05 A4) provides early signal. This task extends that into the full suite.
- Do not start drift/vacuum benchmarks until Tasks 06 and 07 land — premature measurement wastes time.
- Pure-Rust recall harness (tests/recall_integration.rs) can run now for quantizer-level recall independent of HNSW graph quality.
