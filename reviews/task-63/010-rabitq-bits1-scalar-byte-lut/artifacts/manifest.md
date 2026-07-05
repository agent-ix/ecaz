# Artifact Manifest

- head SHA: `34e8be200156a1aea606bd60ff96307d02673daf`
- task bucket: `reviews/task-63/010-rabitq-bits1-scalar-byte-lut/`
- lane: HNSW RaBitQ storage format / common RaBitQ scorer
- fixture/storage format/rerank mode: compile validation only; no benchmark
  fixture
- timestamp: 2026-05-26 America/Los_Angeles

## Artifacts

### `cargo-check-lib.log`

- command: `cargo check -q --lib`
- result: passed; log is empty because `-q` emitted no warnings or errors

### `cargo-test-bits1-byte-lut-no-run.log`

- command:
  `cargo test -q --lib bits1_byte_lut_scalar_sum_matches_per_bit_decode --no-run`
- result: passed compile/no-run validation
- key result: command exited 0
- notes: log contains pre-existing unused/unsafe warnings

### `cargo-test-bits1-byte-lut-runtime.log`

- command:
  `cargo test -q --lib bits1_byte_lut_scalar_sum_matches_per_bit_decode`
- result: local runtime execution failed before the test body with dynamic
  symbol error
- key result: `undefined symbol: LockBuffer`
- notes: this is the same local pgrx-linked runtime limitation seen in adjacent
  HNSW work; use the no-run compile result for this packet
