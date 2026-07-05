# Artifact Manifest

- head SHA: `497d2890e277f8fbea7fe68e447c4b68e56f1489`
- task bucket: `reviews/task-67/004-x86-bits4-kernels`
- lane: Task 67 x86 bits=4 scoring kernels
- fixture: none
- storage format: RaBitQ bits=4 scoring path
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surfaces: not applicable
- timestamp: `2026-05-30T00:37:24Z`

## Artifacts

- `validation.log`
  - command: `cargo fmt`
  - command: `cargo test -p ecaz quant::rabitq::tests::x86_sum_query_dequant_bits4_matches_scalar_when_available --no-run`
  - key lines:
    - focused bits=4 test build exited 0 and produced test executables
    - runtime execution remains locally blocked by unresolved PostgreSQL symbol `LockBuffer`
