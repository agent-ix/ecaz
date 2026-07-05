# Task 91 Packet 016 Artifact Manifest

- Head SHA: `3fc5a3aee0e0cfb59d094162e1bd3d7757c7f345`
- Task bucket: `reviews/task-91/`
- Packet path: `reviews/task-91/016-quantcodec-cutoff-scoring/`
- Timestamp: `2026-06-09T07:10:19Z`
- Scope: QuantCodec cutoff scoring API and SPIRE/IVF routing
- Storage / index surfaces: SPIRE V2 RaBitQ cutoff helper; IVF grouped-PQ/RaBitQ min-bound codec hook
- Benchmark lane / fixture / rerank mode: not applicable; this is a code-review and focused validation packet
- Isolated one-index-per-table vs shared-table surfaces: not applicable

## Artifacts

### `cargo-fmt.log`

- Command: `cargo fmt`
- Result: passed
- Key lines: emitted existing stable-rustfmt warnings that `imports_granularity = Crate` and `group_imports = StdExternalCrate` require nightly

### `ivf-grouped-pq-cutoff-test.log`

- Command: `cargo test --lib am::ec_ivf::quantizer::tests::common_quant_codec_grouped_pq_cutoff_prunes_through_trait --no-default-features --features pg18`
- Result: passed
- Key lines: `test am::ec_ivf::quantizer::tests::common_quant_codec_grouped_pq_cutoff_prunes_through_trait ... ok`; `1 passed; 0 failed`

### `spire-active-cutoff-test.log`

- Command: `cargo test --lib am::ec_spire::scan::tests::rabitq_cutoff_helper_routes_active_cutoff_through_quant_codec --no-default-features --features pg18`
- Result: passed
- Key lines: `test am::ec_spire::scan::tests::rabitq_cutoff_helper_routes_active_cutoff_through_quant_codec ... ok`; `1 passed; 0 failed`

### `spire-fallback-cutoff-test.log`

- Command: `cargo test --lib am::ec_spire::scan::tests::rabitq_cutoff_helper_uses_quant_codec_before_cutoff_is_available --no-default-features --features pg18`
- Result: passed
- Key lines: `test am::ec_spire::scan::tests::rabitq_cutoff_helper_uses_quant_codec_before_cutoff_is_available ... ok`; `1 passed; 0 failed`

### `ivf-quantizer-tests.log`

- Command: `cargo test --lib am::ec_ivf::quantizer::tests --no-default-features --features pg18`
- Result: passed
- Key lines: `25 passed; 0 failed`

### `spire-scan-direct-scorer-audit.log`

- Command: `rg -n 'score_payload_ip\(|try_score_payload_ip\(|score_batch_ip\(' src/am/ec_spire/scan/candidates.rs`
- Result: no matches
- Key lines: none; empty output is the expected result

### `quantcodec-cutoff-api-audit.log`

- Command: `rg -n 'try_score_ip_candidate' src/am src/quant`
- Result: passed
- Key lines: trait method in `src/am/common/quant_codec.rs`; SPIRE and IVF implementations; SPIRE scan callsite in `src/am/ec_spire/scan/candidates.rs`

### `git-diff-check.log`

- Command: `git diff --check`
- Result: passed
- Key lines: none; empty output is the expected result
