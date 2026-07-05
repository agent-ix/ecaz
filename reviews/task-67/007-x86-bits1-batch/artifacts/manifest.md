# Artifact Manifest

- head SHA: `b4757c3472875fd07e0ecade945c8672dc4be702`
- task bucket: `reviews/task-67/007-x86-bits1-batch`
- lane: Task 67 x86 bits=1 batched scoring kernels
- fixture: none
- storage format: RaBitQ bits=1 scoring path
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surfaces: not applicable
- timestamp: `2026-05-30T00:57:30Z`

## Artifacts

- `validation.log`
  - command: `cargo fmt`
  - command: `cargo test -p ecaz quant::rabitq::tests::x86_sum_query_dequant_bits1_pair_matches_scalar_when_available --no-run`
  - key lines:
    - focused bits=1 x86 pair test build exited 0 and produced test executables
    - runtime execution remains locally blocked by unresolved PostgreSQL symbol `LockBuffer`
