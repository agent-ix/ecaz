# Task 91 Review Request: DiskANN TurboQuant Search Codec

## Summary

This checkpoint adds DiskANN `storage_format='turboquant'` as a direct search-code format and routes its prefilter scoring through `QuantCodec`.

Changes:

- Adds `VAMANA_SEARCH_CODEC_TURBOQUANT = 4` metadata support.
- Adds `StorageFormat::TurboQuant` reloption parsing and help text.
- Adds `DiskannBuildBinding::TurboQuant` using 4-bit `ProdQuantizer` MSE-packed bytes as the direct DiskANN search code.
- Adds a DiskANN TurboQuant `QuantCodec` adapter for no-QJL 4-bit LUT scoring.
- Adds explicit rejection for TurboQuant build/query paths on QJL-active dimensions, since DiskANN stores only the MSE-packed direct search code in this slice.
- Extends the SQL storage-format smoke to build and scan `pq_fastscan`, `rabitq`, and `turboquant` indexes. The TurboQuant fixture uses the 1536-dimensional no-QJL lane.

This absorbs the remaining Task 90 DiskANN TurboQuant search-code work into Task 91.

## Validation

Packet-local validation summary:

- `artifacts/manifest.md`
- `artifacts/validation.md`

Commands passed:

- `cargo test --lib am::ec_diskann::quantizer::tests --no-default-features --features pg18`
- `cargo test --lib am::ec_diskann::build::tests::turboquant_build_params_use_direct_search_code_without_sidecar_flags --no-default-features --features pg18`
- `cargo test --lib am::ec_diskann::page::tests --no-default-features --features pg18`
- `cargo test --lib am::ec_diskann::options::tests::diskann_storage_format_parse_accepts_rabitq_and_turboquant --no-default-features --features pg18`
- `cargo test --lib am::ec_diskann::routine::tests::pg_test_ec_diskann_storage_formats_build_and_scan_sql_surface --no-default-features --features pg18`
- `git diff --check`

## Notes

Full broad DiskANN unit filtering was not used as evidence because that filter also runs unrelated pgrx FFI/GUC tests in parallel and previously tripped the known “postgres FFI may not be called from multiple threads” harness constraint.
