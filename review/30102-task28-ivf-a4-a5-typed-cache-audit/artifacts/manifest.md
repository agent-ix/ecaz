# Artifact Manifest

Packet: `review/30102-task28-ivf-a4-a5-typed-cache-audit`

This packet has no benchmark measurement artifacts. It records a code audit and focused test evidence for Task 28 A4/A5.

## Audit

- Head SHA: `a990d57e`
- Timestamp: `2026-04-28T10:07:25-07:00`
- Command: `rg -n "exact_score_mode_name\\(|mse_no_qjl_4bit" src/am/ec_ivf/quantizer.rs`
- Result: no matches

- Head SHA: `a990d57e`
- Timestamp: `2026-04-28T10:07:25-07:00`
- Command: `rg -n "ExactScoreMode::MseNoQjl4Bit|cached_quantizer_reuses_instances|test_ec_ivf_rescan_reuses_cached_prod_quantizer|type QuantizerKey|static CACHE" src/am/ec_ivf/quantizer.rs src/quant/prod.rs src/lib.rs`
- Key lines:
  - `src/quant/prod.rs:70:type QuantizerKey = (usize, u8, u64);`
  - `src/quant/prod.rs:73: static CACHE: OnceLock<Mutex<HashMap<QuantizerKey, Arc<ProdQuantizer>>>>`
  - `src/am/ec_ivf/quantizer.rs:158: ExactScoreMode::MseNoQjl4Bit`
  - `src/quant/prod.rs:2156: fn cached_quantizer_reuses_instances()`
  - `src/lib.rs:2824: fn test_ec_ivf_rescan_reuses_cached_prod_quantizer()`

## Validation

- Head SHA: `a990d57e`
- Command: `cargo test -p ecaz --lib cached_quantizer_reuses_instances`
- Result: passed

- Head SHA: `a990d57e`
- Command: `cargo test -p ecaz --lib cached_with_presence_reports_whether_entry_already_existed`
- Result: passed

- Head SHA: `a990d57e`
- Command: `cargo test -p ecaz --lib turboquant_dispatch_uses_lut_for_no_qjl_4bit_lane`
- Result: passed

- Head SHA: `a990d57e`
- Command: `cargo pgrx test pg18 test_ec_ivf_rescan_reuses_cached_prod_quantizer`
- Result: passed
