# Artifact Manifest

- head SHA: `434a2da562e7e3547b3e154d97a80ad2e98ba845`
- task bucket: `reviews/task-67/002-x86-bits8-kernels`
- lane: Task 67 x86 bits=8 arithmetic-dequant kernels
- fixture: none
- storage format: RaBitQ bits=8 scoring path
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surfaces: not applicable
- timestamp: `2026-05-30T00:21:40Z`

## Artifacts

- `validation.log`
  - command: `cargo fmt`
  - command: `cargo test -p ecaz quant::rabitq::tests::x86_sum_query_dequant_bits8_matches_scalar_when_available --no-run`
  - key lines:
    - focused test build exited 0 and produced test executables
    - runtime execution remains locally blocked by unresolved PostgreSQL symbol `LockBuffer`
