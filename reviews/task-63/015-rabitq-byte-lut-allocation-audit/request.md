# Review Request: RaBitQ Byte-LUT Allocation Audit

- task: `plan/tasks/63-hnsw-rabitq-storage-format.md`
- branch: `task/60-diskann-rabitq`
- packet: `reviews/task-63/015-rabitq-byte-lut-allocation-audit/`

## Summary

This packet audits the common RaBitQ 1-bit byte-LUT scorer path after external
feedback noted that the optimization must not allocate an 8 KB byte LUT for
non-1-bit prepared queries.

Current code already has the intended shape:

- `src/quant/rabitq.rs::bits1_byte_lut` is `Option<Box<[[f32; 8]; 256]>>`.
- `build_bits1_byte_lut_boxed` returns `None` unless `bits_per_dim == 1`.
- `prepared_queries_only_keep_bits1_byte_lut_for_bits1` asserts a bits=1
  prepared query has the byte LUT and a bits=4 prepared query does not.

No code change was needed in this packet.

## Validation

Packet-local logs:

- `artifacts/cargo-test-prepared-query-byte-lut-no-run.log`
  - `cargo test -q --lib prepared_queries_only_keep_bits1_byte_lut_for_bits1 --no-run`
  - passed compile/no-run validation.
- `artifacts/cargo-test-prepared-query-byte-lut-runtime.log`
  - `cargo test -q --lib prepared_queries_only_keep_bits1_byte_lut_for_bits1`
  - blocked locally before the test body by the known pgrx-linked runtime
    symbol issue: `undefined symbol: LockBuffer`.

No benchmarks were run.
