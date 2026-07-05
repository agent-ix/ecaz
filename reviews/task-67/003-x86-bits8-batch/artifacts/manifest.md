# Artifact Manifest

- head SHA: `e6439ec81ccc84c6907b8268f0510ac402acc684`
- task bucket: `reviews/task-67/003-x86-bits8-batch`
- lane: Task 67 x86 bits=8 batched scoring
- fixture: none
- storage format: RaBitQ bits=8 scoring path
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surfaces: not applicable
- timestamp: `2026-05-30T00:28:14Z`

## Artifacts

- `validation.log`
  - command: `cargo fmt`
  - command: `cargo test -p ecaz quant::rabitq::tests::bits8_batch_estimator_matches_scalar_order --no-run`
  - key lines:
    - focused batch test build exited 0 and produced test executables
    - runtime execution remains locally blocked by unresolved PostgreSQL symbol `LockBuffer`
