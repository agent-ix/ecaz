# Artifact Manifest

- head SHA: `9747b56627dd5564133cf15592433d552427e7a6`
- task bucket: `reviews/task-67/005-x86-bits4-batch`
- lane: Task 67 x86 bits=4 batched scoring kernels
- fixture: none
- storage format: RaBitQ bits=4 scoring path
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surfaces: not applicable
- timestamp: `2026-05-30T00:45:21Z`

## Artifacts

- `validation.log`
  - command: `cargo fmt`
  - command: `cargo test -p ecaz quant::rabitq::tests::bits4_batch_estimator_matches_scalar_order --no-run`
  - key lines:
    - focused bits=4 batch test build exited 0 and produced test executables
    - runtime execution remains locally blocked by unresolved PostgreSQL symbol `LockBuffer`
