# Task 111 Packet 002 Artifact Manifest

- head SHA: `a82983bcac9d`
- task bucket: `reviews/task-111/`
- packet path: `reviews/task-111/002-gated-dense-blocks/`
- lane / fixture / storage format / rerank mode: unit-level gated dense-block implementation checks; no benchmark lane or corpus fixture in this packet
- command used:
  - `script -q -c 'cargo check -q --lib' reviews/task-111/002-gated-dense-blocks/artifacts/cargo-check-lib.log`
  - `script -q -c 'cargo test -q dense_posting --lib' reviews/task-111/002-gated-dense-blocks/artifacts/cargo-test-dense-posting.log`
  - `script -q -c 'cargo test -q ivf_explain --lib' reviews/task-111/002-gated-dense-blocks/artifacts/cargo-test-ivf-explain.log`
- timestamp: `2026-06-17T02:33:51Z`
- isolated one-index-per-table or shared-table surfaces: not applicable; unit/compiler checks only
- layout gate: `dense_posting_blocks = 1` reloption; default remains off

## Artifacts

### `cargo-check-lib.log`

Library compile check for the implementation slice.

Key result lines: command exited successfully with no compiler diagnostics.

### `cargo-test-dense-posting.log`

Focused codec/build validation for dense posting blocks.

```text
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2102 filtered out; finished in 0.06s
```

### `cargo-test-ivf-explain.log`

Focused validation for IVF EXPLAIN counter updates.

```text
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2102 filtered out; finished in 0.00s
```
