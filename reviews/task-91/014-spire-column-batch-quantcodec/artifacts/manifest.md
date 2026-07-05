# Manifest: Task 91 Packet 014 SPIRE Column Batch QuantCodec

- Head SHA: `528f3d2cd90e98c8f6d0d54f2947d0a294418419`
- Task bucket: `reviews/task-91/`
- Packet path: `reviews/task-91/014-spire-column-batch-quantcodec/`
- Lane: PG18 focused Rust unit tests
- Fixture: SPIRE V2 per-column batch helper and adjacent routed candidate tests
- Storage format: SPIRE TurboQuant and RaBitQ assignment payloads
- Rerank mode: quantized SPIRE leaf candidate scoring
- Isolated surface: helper-level V2 column payload batch scoring

## Artifacts

- `validation.md`: command log summary for this checkpoint

## Commands

- `cargo test --lib am::ec_spire::scan::tests::column_payload_quant_codec_batch_helper_matches_prepared_assignment_scorer --no-default-features --features pg18`
- `cargo test --lib am::ec_spire::scan::tests::selected_row_quant_codec_helper_matches_prepared_assignment_scorer --no-default-features --features pg18`
- `cargo test --lib am::ec_spire::scan::tests::collect_quantized_routed_probe_candidates_matches_prepared_assignment_scorer --no-default-features --features pg18`
- `git diff --check`

## Key Results

- Column payload batch QuantCodec helper parity test: `1 passed; 0 failed`
- Selected-row QuantCodec helper parity test: `1 passed; 0 failed`
- Quantized routed candidate scorer parity test: `1 passed; 0 failed`
- `git diff --check`: passed
