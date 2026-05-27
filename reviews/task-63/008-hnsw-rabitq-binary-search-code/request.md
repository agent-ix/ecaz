# Review Request: HNSW RaBitQ Binary Search Codes

- task: `plan/tasks/63-hnsw-rabitq-storage-format.md`
- code commit: `dd9626447b8f4316052de97f8024253c24a5f36c`
- branch: `task/60-diskann-rabitq`
- packet: `reviews/task-63/008-hnsw-rabitq-binary-search-code/`

## Summary

This checkpoint aligns HNSW RaBitQ with the common RaBitQ codec direction used
by DiskANN: RaBitQ search payloads are now binary 1-bit codes, while the cold
rerank payload remains the existing scalar-quantized code.

The local 10k HNSW-only tuning run showed the previous HNSW RaBitQ layout was
larger than PqFastScan, because it encoded RaBitQ search payloads with
`DEFAULT_QUANT_BITS = 4`. This change introduces `HNSW_RABITQ_BITS = 1` and
uses it for HNSW RaBitQ metadata and build-time search-code encoding.

## Touched Behavior

- `src/am/ec_hnsw/codec.rs`
  - adds `HNSW_RABITQ_BITS = 1`;
  - initializes RaBitQ metadata with `search_bits = 1` and
    `search_subvector_dim = 1`.
- `src/am/ec_hnsw/build.rs`
  - uses `HNSW_RABITQ_BITS` when deriving build-time RaBitQ search codes;
  - writes binary RaBitQ search-code metadata;
  - adds a unit test proving binary search codes plus scalar rerank layout.
- `src/tests/ec_hnsw_storage_lifecycle.rs`
  - extends RaBitQ lifecycle metadata assertions to lock the 1-bit search-code
    layout.

## Validation

- `cargo check -q --lib`
  - passed; see `artifacts/cargo-check-lib.log`.
- `cargo test -q --lib hnsw --no-run`
  - passed compile/no-run validation; see
    `artifacts/cargo-test-hnsw-no-run.log`.
- `cargo test -q --lib rabitq_flush_output_uses_binary_search_codes_and_scalar_rerank`
  - blocked locally by existing pgrx dynamic symbol issue:
    `undefined symbol: LockBuffer`;
  - captured in
    `artifacts/cargo-test-rabitq-binary-search-code-runtime.log`.

## Notes

This is not the final Task 63 benchmark decision. The canonical
`benchmarks/task63-hnsw-rabitq-format/` suite still needs to be rerun on the
newer Intel and m5 laptop benchmark hosts for publishable 50k/100k evidence.
