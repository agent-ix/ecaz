# Manifest: Task 91 Packet 013 SPIRE Selected-Row QuantCodec

- Head SHA: `3909a8511d2fc369596930a91750ae48e8b454a7`
- Task bucket: `reviews/task-91/`
- Packet path: `reviews/task-91/013-spire-selected-row-quantcodec/`
- Lane: PG18 focused Rust unit tests
- Fixture: SPIRE V2 selected-row scoring helper and adjacent selected-block routing tests
- Storage format: SPIRE TurboQuant and RaBitQ assignment payloads
- Rerank mode: quantized SPIRE leaf candidate scoring
- Isolated surface: helper-level selected-row scoring plus adjacent scan tests

## Artifacts

- `validation.md`: command log summary for this checkpoint

## Commands

- `cargo test --lib am::ec_spire::scan::tests::selected_row_quant_codec_helper_matches_prepared_assignment_scorer --no-default-features --features pg18`
- `cargo test --lib am::ec_spire::scan::tests::collect_quantized_routed_probe_candidates_matches_prepared_assignment_scorer --no-default-features --features pg18`
- `cargo test --lib am::ec_spire::scan::tests::select_leaf_block_row_ranges --no-default-features --features pg18`
- `git diff --check`

## Key Results

- Selected-row QuantCodec helper parity test: `1 passed; 0 failed`
- Quantized routed candidate scorer parity test: `1 passed; 0 failed`
- Selected leaf-block row-range tests: `2 passed; 0 failed`
- `git diff --check`: passed
