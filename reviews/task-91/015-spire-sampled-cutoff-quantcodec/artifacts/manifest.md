# Manifest: Task 91 Packet 015 SPIRE Sampled Fallback QuantCodec Cleanup

- Head SHA: `7170a2caaae7786ce0ebf2b5fcd1af1f833aa17a`
- Task bucket: `reviews/task-91/`
- Packet path: `reviews/task-91/015-spire-sampled-cutoff-quantcodec/`
- Lane: code-level SPIRE QuantCodec migration cleanup
- Fixture: focused unit tests
- Storage format: SPIRE assignment payload formats `TurboQuant` and `RaBitQ`
- Rerank mode: not applicable
- Isolation: source audit + focused tests; no benchmark run

## Artifacts

This packet has no large generated artifacts. Validation output is summarized
in `request.md`.

## Commands

### Formatting

```text
cargo fmt
```

Result:

- completed successfully;
- emitted the repository's existing warnings that `imports_granularity` and
  `group_imports` are nightly-only rustfmt settings.

### Focused Tests

```text
cargo test --lib am::ec_spire::scan::tests::rabitq_cutoff_helper_uses_quant_codec_before_cutoff_is_available --no-default-features --features pg18
```

Result:

- `1 passed; 0 failed`

```text
cargo test --lib am::ec_spire::scan::tests::selected_row_quant_codec_helper_matches_prepared_assignment_scorer --no-default-features --features pg18
```

Result:

- `1 passed; 0 failed`

```text
cargo test --lib am::ec_spire::scan::tests::column_payload_quant_codec_batch_helper_matches_prepared_assignment_scorer --no-default-features --features pg18
```

Result:

- `1 passed; 0 failed`

### Source Audit

```text
rg -n "score_payload_ip\\(|try_score_payload_ip\\(|score_batch_ip\\(" src/am/ec_spire/scan src/am/ec_spire/quantizer/mod.rs | grep -v '/tests'
```

Key production scan result:

```text
src/am/ec_spire/scan/candidates.rs:2765:            scorer.try_score_payload_ip(column_format, gamma, encoded_payload, min_ip_to_keep)
```

Interpretation:

- Remaining direct scan-side scorer call is the bounded RaBitQ cutoff
  exception only.
- SPIRE sampled-row, selected-row, no-cutoff, and column-batch scoring paths
  now route through `QuantCodec`.

### Whitespace

```text
git diff --check
```

Result:

- passed
