# Review Request: RaBitQ 1-Bit Scalar Byte LUT

- task: `plan/tasks/63-hnsw-rabitq-storage-format.md`
- code commit: `34e8be200156a1aea606bd60ff96307d02673daf`
- branch: `task/60-diskann-rabitq`
- packet: `reviews/task-63/010-rabitq-bits1-scalar-byte-lut/`

## Summary

The local 10k and 50k HNSW smoke packets showed binary RaBitQ fixed the storage
regression but remained materially slower than PqFastScan. This checkpoint
removes one obvious implementation cost in the common RaBitQ scorer: the
portable scalar `bits = 1` fallback now consumes the prepared per-query byte
LUT eight dimensions at a time instead of decoding one bit per dimension.

This keeps the quantizer/scorer change in `src/quant/rabitq.rs`, so HNSW,
DiskANN, IVF, and future codec adapters use the same common RaBitQ scoring
surface. It does not change the HNSW on-disk format or any benchmark matrix.

## Touched Behavior

- `sum_query_dequant_with_bf16`
  - dispatches `bits = 1` scoring to the prepared byte-LUT scalar path when
    NEON is unavailable.
- `sum_query_dequant_bits1_byte_lut_scalar`
  - adds the portable eight-lane byte-LUT fallback for 1-bit codes.
- `bits1_byte_lut_scalar_sum_matches_per_bit_decode`
  - covers short tails, byte-aligned lengths, and the production 1536d case
    against the existing per-bit reference.

## Validation

- `cargo check -q --lib`
  - passed; see `artifacts/cargo-check-lib.log`.
- `cargo test -q --lib bits1_byte_lut_scalar_sum_matches_per_bit_decode --no-run`
  - passed compile/no-run validation; see
    `artifacts/cargo-test-bits1-byte-lut-no-run.log`.
- `cargo test -q --lib bits1_byte_lut_scalar_sum_matches_per_bit_decode`
  - blocked locally by the existing pgrx-linked runtime symbol issue:
    `undefined symbol: LockBuffer`;
  - captured in `artifacts/cargo-test-bits1-byte-lut-runtime.log`.

## Notes

This is a code-path improvement for the 1-bit RaBitQ scorer, not a new local
benchmark. The final Task 63 decision still needs the newer-host publishable
50k/100k evidence after this commit is installed.
