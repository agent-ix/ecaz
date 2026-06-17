# Task 111 Packet 004 Artifact Manifest

- head SHA: `e2e8383916ad706a534a9b45af7b70602fb38240`
- task bucket: `reviews/task-111/`
- packet path: `reviews/task-111/004-rabitq-dense-scan/`
- lane / fixture / storage format / rerank mode: focused PG18 correctness fixture for gated dense IVF posting blocks; storage format `rabitq`; quant bits `1`; rerank off/auto
- command used:
  - `script -q -c 'cargo check -q --lib' reviews/task-111/004-rabitq-dense-scan/artifacts/cargo-check-lib.log`
  - `script -q -c 'cargo test -q dense_posting --lib' reviews/task-111/004-rabitq-dense-scan/artifacts/cargo-test-dense-posting.log`
- timestamp: `2026-06-16T20:01:12-07:00`
- isolated one-index-per-table or shared-table surfaces: focused isolated test tables per pg_test fixture
- layout gate: `dense_posting_blocks = 1` reloption; default remains off

## Artifacts

### `cargo-check-lib.log`

Library compile check for the RaBitQ dense scan fixture.

Key result lines: command exited successfully with no compiler diagnostics.

### `cargo-test-dense-posting.log`

Focused unit + PG18 fixture validation. The filter covers:

- dense posting block codec/build staging unit tests
- gated dense build scan over build-time rows
- gated dense RaBitQ build scan over build-time rows
- mixed dense build + row insert scan
- dense build-row vacuum deletion

```text
running 6 tests
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2102 filtered out; finished in 52.63s
```
