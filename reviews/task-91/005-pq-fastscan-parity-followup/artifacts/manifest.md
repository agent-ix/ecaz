# Task 91 Packet 005 Artifact Manifest

- head SHA: `a3ea53d7160ffe32df34838b634b202b0a5c0ecc`
- task bucket: `reviews/task-91`
- packet path: `reviews/task-91/005-pq-fastscan-parity-followup`
- timestamp: `2026-06-08T21:22:50-07:00`
- lane / fixture / storage format / rerank mode: IVF unit tests,
  `PqFastScan` direct-path parity
- isolated one-index-per-table or shared-table surface: not applicable; unit
  validation only

## Artifacts

### `artifacts/cargo-test-ivf-quantizer.log`

- command:
  `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
- result:
  `test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1996 filtered out; finished in 0.19s`
- coverage:
  IVF quantizer unit module including the new
  `common_quant_codec_pq_fastscan_batch_is_bit_exact_with_direct_path` test.

### `artifacts/git-diff-check.log`

- command: `git diff --check`
- result: passed with no output.
