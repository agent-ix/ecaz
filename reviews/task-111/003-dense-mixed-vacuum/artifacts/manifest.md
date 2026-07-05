# Task 111 Packet 003 Artifact Manifest

- head SHA: `26e3f443dd5f`
- task bucket: `reviews/task-111/`
- packet path: `reviews/task-111/003-dense-mixed-vacuum/`
- lane / fixture / storage format / rerank mode: focused PG18 correctness fixtures for gated dense IVF posting blocks; storage format `turboquant`; rerank off/auto
- command used:
  - `script -q -c 'cargo check -q --lib' reviews/task-111/003-dense-mixed-vacuum/artifacts/cargo-check-lib.log`
  - `script -q -c 'cargo test -q ivf_explain --lib' reviews/task-111/003-dense-mixed-vacuum/artifacts/cargo-test-ivf-explain.log`
  - `script -q -c 'cargo test -q dense_posting --lib' reviews/task-111/003-dense-mixed-vacuum/artifacts/cargo-test-dense-posting.log`
- timestamp: `2026-06-17T02:51:56Z`
- isolated one-index-per-table or shared-table surfaces: focused isolated test tables per pg_test fixture
- layout gate: `dense_posting_blocks = 1` reloption; default remains off

## Artifacts

### `cargo-check-lib.log`

Library compile check for dense mixed scan and vacuum support.

Key result lines: command exited successfully with no compiler diagnostics.

### `cargo-test-ivf-explain.log`

Focused validation for IVF EXPLAIN counter updates.

```text
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2105 filtered out; finished in 0.00s
```

### `cargo-test-dense-posting.log`

Focused unit + PG18 fixture validation. The filter covers:

- dense posting block codec/build staging unit tests
- gated dense build scan over build-time rows
- mixed dense build + row insert scan
- dense build-row vacuum deletion

```text
running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 2102 filtered out; finished in 47.59s
```
