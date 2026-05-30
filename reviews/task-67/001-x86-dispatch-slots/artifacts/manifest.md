# Artifact Manifest

- head SHA: `19715a204e16a4b8142f2ad2ed95ebe3dc752647`
- task bucket: `reviews/task-67/001-x86-dispatch-slots`
- lane: Task 67 Slice A, x86 SIMD feature detection and dispatch slots
- fixture: none
- storage format: RaBitQ dispatch plumbing only
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surfaces: not applicable
- timestamp: `2026-05-30T00:06:27Z`

## Artifacts

- `validation.log`
  - command: `cargo fmt`
  - command: `cargo test -p ecaz quant::simd --no-run`
  - command: `cargo test -p ecaz quant::simd`
  - command: `cargo test -p ecaz quant::rabitq::tests::x86_query_dequant_slots_cover_task67_kernels`
  - key lines:
    - `cargo test -p ecaz quant::simd --no-run` exited 0 and produced test executables.
    - Runtime test execution exited 127 before test bodies due to local unresolved PostgreSQL symbol `LockBuffer`.
