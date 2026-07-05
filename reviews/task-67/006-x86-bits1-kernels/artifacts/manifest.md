# Artifact Manifest

- head SHA: `664376fd478b0d7fc5dd813f314c1a6561f09732`
- task bucket: `reviews/task-67/006-x86-bits1-kernels`
- lane: Task 67 x86 bits=1 scoring kernels
- fixture: none
- storage format: RaBitQ bits=1 scoring path
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surfaces: not applicable
- timestamp: `2026-05-30T00:51:17Z`

## Artifacts

- `validation.log`
  - command: `cargo fmt`
  - command: `cargo test -p ecaz quant::rabitq::tests::x86_sum_query_dequant_bits1_matches_scalar_when_available --no-run`
  - key lines:
    - focused bits=1 x86 test build exited 0 and produced test executables
    - runtime execution remains locally blocked by unresolved PostgreSQL symbol `LockBuffer`
