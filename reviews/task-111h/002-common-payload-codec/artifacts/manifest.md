# Task 111h / 002 Common Payload Codec Artifacts

- Head SHA: `1ba94bdee716ed04a131154afee75cab009d0b5c`
- Task bucket: `reviews/task-111h/002-common-payload-codec/`
- Timestamp: `2026-06-19T20:55:34-07:00`
- Scope: common rerank payload codec and legacy `0x2A` sidecar correctness; no
  benchmark matrix in this packet.
- Storage surface: `rerank_placement = 'index'` persisted compact payloads on
  the legacy direct-TID sidecar baseline.
- Formats covered: f16, RaBitQ-4, RaBitQ-8, TurboQuant.
- Isolated one-index-per-table vs shared-table: focused Rust unit tests and
  pg_test fixtures create their own test relations under the pgrx harness; no
  benchmark tables or shared suite surfaces.

## Artifacts

### `cargo-check-pg18.log`

- Command:
  `script -q -c "cargo check --no-default-features --features pg18" reviews/task-111h/002-common-payload-codec/artifacts/cargo-check-pg18.log`
- Purpose: compile validation for the common codec refactor.
- Key result:
  `Finished dev profile ... target(s) in 9.91s`

### `cargo-test-codec.log`

- Command:
  `script -q -c "cargo test --no-default-features --features pg18 compact_payload_codecs_score_source_and_sidecar_consistently --lib" reviews/task-111h/002-common-payload-codec/artifacts/cargo-test-codec.log`
- Purpose: unit-level source-diagnostic vs persisted sidecar differential for
  RaBitQ-4, RaBitQ-8, and TurboQuant, plus scalar-vs-batch sanity.
- Key result:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2195 filtered out`

### `cargo-test-coarse-rerank.log`

- Command:
  `script -q -c "cargo test --no-default-features --features pg18 coarse_rerank --lib" reviews/task-111h/002-common-payload-codec/artifacts/cargo-test-coarse-rerank.log`
- Purpose: option resolution and coarse_rerank pg_test fixtures, including auto
  placement resolving compact f16/RaBitQ-4/RaBitQ-8/TurboQuant to `index`.
- Key result:
  `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 2173 filtered out`

### `cargo-test-index-placement.log`

- Command:
  `script -q -c "cargo test --no-default-features --features pg18 index_placement --lib" reviews/task-111h/002-common-payload-codec/artifacts/cargo-test-index-placement.log`
- Purpose: index-placement option, admin, bytes, insert, and vacuum coverage
  after the codec refactor.
- Key result:
  `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 2186 filtered out`

### `cargo-test-index-quant-formats.log`

- Command:
  `script -q -c "cargo test --no-default-features --features pg18 index_quant_formats_top_neighbor --lib" reviews/task-111h/002-common-payload-codec/artifacts/cargo-test-index-quant-formats.log`
- Purpose: pg_test scan fixture proving RaBitQ-4, RaBitQ-8, and TurboQuant
  index-side persisted sidecar formats can scan and rank an exact neighbor first
  on a separable corpus.
- Key result:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2195 filtered out`
