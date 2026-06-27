# Task 111 Packet 005 Artifact Manifest

- head SHA: `5e9e9fecb004b79cf40b3bb7c63c715bcec9196a`
- task bucket: `reviews/task-111/`
- packet path: `reviews/task-111/005-dense-scan-scratch-buffers/`
- lane / fixture / storage format / rerank mode: focused scanner allocation/correctness validation for gated dense IVF posting blocks; storage formats `turboquant` and `rabitq`; rerank off/auto
- command used:
  - `script -q -c 'cargo check -q --lib' reviews/task-111/005-dense-scan-scratch-buffers/artifacts/cargo-check-lib.log`
  - `script -q -c 'cargo test -q dense_posting --lib' reviews/task-111/005-dense-scan-scratch-buffers/artifacts/cargo-test-dense-posting.log`
- timestamp: `2026-06-16T20:11:18-07:00`
- isolated one-index-per-table or shared-table surfaces: focused isolated test tables per pg_test fixture plus unit-level scratch coverage
- layout gate: `dense_posting_blocks = 1` reloption; default remains off

## Artifacts

### `cargo-check-lib.log`

Library compile check for dense scan scratch-buffer reuse.

Key result lines: command exited successfully with no compiler diagnostics.

### `cargo-test-dense-posting.log`

Focused unit + PG18 fixture validation. The filter covers:

- dense posting block codec/build staging unit tests
- dense scan scratch capacity reuse
- gated dense build scan over build-time rows
- gated dense RaBitQ build scan over build-time rows
- mixed dense build + row insert scan
- dense build-row vacuum deletion

```text
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 2102 filtered out; finished in 49.98s
```
