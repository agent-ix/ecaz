# Task 10: Full Benchmark Suite

Status: **infrastructure complete** — A3/A5/A6 are merged on `main`; C1 is now in result-capture mode, starting with trustworthy real-corpus `NFR-001` latency reporting

## Scope

Build the complete quality and performance benchmark infrastructure required by NFR-001, NFR-002, and NFR-003, then capture the end-to-end result artifacts now that scan / insert / vacuum are operational on `main`.

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
- [x] **SQL benchmark CLI surfaces.** `ecaz bench latency`, `ecaz bench storage`, `ecaz bench recall`, and `ecaz corpus generate`.

### Validated

All criterion benchmarks run successfully with `--quick`. Representative results:
- `score_ip_encoded` at 1536/4-bit: ~10.5µs/score, ~95K scores/sec batch throughput
- `decode_approximate` at 1536/4-bit: ~8.8µs
- DataPage element insert: ~68ns, read: ~48ns (code_len=768)
- All proptest, size_of, and recall smoke tests pass.

### Open Result-Capture Work

- [ ] **Latency benchmarks (NFR-001).** Capture warm/cold HNSW p50/p99 on the canonical real corpus, plus sequential-scan throughput and artifact metadata. First slice: harden `ecaz bench latency` real-corpus reporting so `ef_search`, cache state, and host / GUC details are recorded correctly.
- [ ] **Storage accounting (NFR-002).** Capture `pg_relation_size`, `pg_total_relation_size`, and per-datum sizing against the same real-corpus benchmark surfaces.
- [ ] **Full recall suite (NFR-003).** Extend beyond the A4 gate into broader `(m, ef)` SQL reporting, post-insert drift checkpoints, post-vacuum recall refresh, and MSE-only vs MSE+QJL ablations.
- [ ] **BC-001 through BC-016.** Result artifacts now depend on running the scripts against staged benchmark corpora, not on missing scan / insert / vacuum functionality.

## Owns

- `NFR-001` (full)
- `NFR-002` (full)
- `NFR-003` (full, beyond the initial recall gate in Task 05 A4)

## Dependencies

- Task 05 / A3 (working graph scan) — **resolved on `main`**
- Task 06 / A5 (insert-drift observability) — **resolved on `main`**
- Task 07 / A6 (post-vacuum quality baseline) — **resolved on `main`**
- A staged benchmark corpus plus the matching built indexes remain the practical prerequisite for each recorded result artifact

## Unblocks

- Performance and quality sign-off for v0.1

## Deliverables

- ~~Reproducible benchmark scripts~~ **done**
- ~~Benchmark infrastructure and tooling~~ **done**
- Benchmark result artifacts (latency histograms, recall tables, storage breakdown) — **in progress**
- Pass/fail against declared NFR targets — **pending recorded runs**

## Primary Tests

- `BC-001` to `BC-016` as applicable

## Notes

- The initial recall gate (Task 05 A4) provides early signal. This task extends that into the full suite.
- With A5/A6 merged, C1 should start by making the existing SQL benchmark/reporting surfaces trustworthy and self-describing before recording the first durable artifacts.
- Pure-Rust recall harness (tests/recall_integration.rs) can run now for quantizer-level recall independent of HNSW graph quality.
