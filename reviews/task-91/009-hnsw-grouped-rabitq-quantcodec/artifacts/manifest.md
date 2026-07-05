# Artifact Manifest

- Head SHA: `7727b882b`
- Task bucket: `reviews/task-91/009-hnsw-grouped-rabitq-quantcodec`
- Timestamp: `2026-06-09T05:31:47Z`
- Lane / fixture / storage format / rerank mode: HNSW unit validation, in-process fixtures, grouped-PQ and RaBitQ search-code scoring helpers
- Shared-table vs isolated one-index-per-table surface: not applicable; unit tests only

## Commands

### Focused HNSW codec filter

Command:

```sh
cargo test --lib am::ec_hnsw::scan::tests::hnsw_ --no-default-features --features pg18
```

Key result lines:

```text
running 3 tests
test am::ec_hnsw::scan::tests::hnsw_grouped_pq_scan_codec_matches_direct_search_code_score ... ok
test am::ec_hnsw::scan::tests::hnsw_rabitq_scan_codec_matches_direct_search_code_score ... ok
test am::ec_hnsw::scan::tests::hnsw_turboquant_scan_codec_matches_direct_exact_modes ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2022 filtered out
```

### Full HNSW scan unit-test filter

Command:

```sh
cargo test --lib am::ec_hnsw::scan::tests --no-default-features --features pg18
```

Key result lines:

```text
running 77 tests
test result: ok. 77 passed; 0 failed; 0 ignored; 0 measured; 1948 filtered out
```

### Whitespace check

Command:

```sh
git diff --check
```

Key result lines:

```text
passed with no output
```
