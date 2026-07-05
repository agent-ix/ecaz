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

## Local 10k HNSW Smoke

After installing the updated PG18 extension locally, I ran a HNSW-only 10k
`ecaz bench suite` pass for `turboquant`, `pq_fastscan`, and `rabitq`; see
`artifacts/local-10k-bin1/`.

- suite status: completed 14, failed 0, skipped 0.
- build index seconds:
  - `turboquant`: 97.93
  - `pq_fastscan`: 109.60
  - `rabitq`: 99.14
- recall@10 at ef_search 40 / 100 / 200:
  - `turboquant`: 0.8845 / 0.9445 / 0.9700
  - `pq_fastscan`: 0.8945 / 0.9635 / 0.9940
  - `rabitq`: 0.8135 / 0.9205 / 0.9365
- latency p50 at ef_search 40 / 100 / 200:
  - `turboquant`: 15.3 ms / 24.6 ms / 38.2 ms
  - `pq_fastscan`: 19.6 ms / 32.3 ms / 44.0 ms
  - `rabitq`: 42.4 ms / 89.0 ms / 156.7 ms
- HNSW index storage:
  - `turboquant`: 13.0 MiB, 1366.4 B/row
  - `pq_fastscan`: 13.1 MiB, 1377.9 B/row
  - `rabitq`: 13.0 MiB, 1366.4 B/row

This confirms the binary RaBitQ change removes the earlier local 4-bit RaBitQ
HNSW storage regression at 10k. It also shows RaBitQ still trails PqFastScan on
local recall and latency at these operating points.

## Notes

This is not the final Task 63 benchmark decision. The canonical
`benchmarks/task63-hnsw-rabitq-format/` suite still needs to be rerun on the
newer Intel and m5 laptop benchmark hosts for publishable 50k/100k evidence.
Older uncommitted local benchmark artifacts under
`benchmarks/task63-hnsw-rabitq-format/artifacts/` are baseline/tuning output,
not accepted post-change evidence for this checkpoint.
