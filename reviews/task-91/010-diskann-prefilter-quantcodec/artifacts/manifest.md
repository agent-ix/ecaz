# Artifact Manifest

- Head SHA: `acf2bc0b0`
- Task bucket: `reviews/task-91/010-diskann-prefilter-quantcodec`
- Timestamp: `2026-06-09T05:41:48Z`
- Lane / fixture / storage format / rerank mode: DiskANN unit and pg18 routine validation; binary-sidecar, grouped-PQ, and RaBitQ prefilter scoring helpers
- Shared-table vs isolated one-index-per-table surface: pg18 routine tests use their built-in SQL fixtures; quantizer tests are in-process unit fixtures

## Commands

### DiskANN quantizer unit tests

Command:

```sh
cargo test --lib am::ec_diskann::quantizer::tests --no-default-features --features pg18
```

Key result lines:

```text
running 6 tests
test am::ec_diskann::quantizer::tests::diskann_grouped_pq_prefilter_codec_matches_direct_search_code_score ... ok
test am::ec_diskann::quantizer::tests::diskann_binary_sidecar_prefilter_codec_matches_direct_hamming_score ... ok
test am::ec_diskann::quantizer::tests::diskann_rabitq_prefilter_codec_matches_direct_search_code_score ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2022 filtered out
```

### DiskANN routine tests

Command:

```sh
cargo test --lib am::ec_diskann::routine::tests --no-default-features --features pg18
```

Key result lines:

```text
running 24 tests
test am::ec_diskann::routine::tests::pg_test_ec_diskann_storage_formats_build_and_scan_sql_surface ... ok
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 2004 filtered out
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
