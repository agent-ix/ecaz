# Task 28 IVF A4/A5 typed dispatch and cache audit

## Scope

This packet records the A4/A5 merge-gate audit at head `a990d57e`.

No code change was needed in this slice.

## A4: typed exact-score-mode dispatch

Status: satisfied.

`src/am/ec_ivf/quantizer.rs` no longer dispatches on `exact_score_mode_name()` or the literal `"mse_no_qjl_4bit"`. The TurboQuant 4-bit LUT arm now matches the typed discriminator:

- `ExactScoreMode::MseNoQjl4Bit` in `IvfQuantizer::prepare_ip_query`.

The fallback score path is selected by the prepared-query enum arm, so `score_ip_from_parts` also avoids string-mode dispatch.

Audit command:

- `rg -n "exact_score_mode_name\\(|mse_no_qjl_4bit" src/am/ec_ivf/quantizer.rs`

Result: no matches.

## A5: `ProdQuantizer::cached` cache key and IVF scan reuse

Status: satisfied.

`ProdQuantizer::cached(dimensions, bits, seed)` uses a process-global cache keyed by:

- `type QuantizerKey = (usize, u8, u64)`
- cache storage: `OnceLock<Mutex<HashMap<QuantizerKey, Arc<ProdQuantizer>>>>`

That exactly covers the current IVF TurboQuant call sites, which all use the index dimensions plus default bits and seed. The cached value is an `Arc<ProdQuantizer>`, and repeated calls with the same triple return the same allocation.

Existing regression coverage:

- `quant::prod::tests::cached_quantizer_reuses_instances`
- `quant::prod::tests::cached_with_presence_reports_whether_entry_already_existed`
- `tests::pg_test_ec_ivf_rescan_reuses_cached_prod_quantizer`

The PG18 test creates an IVF index, runs two query-prep rescans on the same index, and asserts `debug_ec_ivf_quantizer_cache_ptr(...)` returns the same pointer across scans.

## Validation

Focused validation run:

- `cargo test -p ecaz --lib cached_quantizer_reuses_instances`
- `cargo test -p ecaz --lib cached_with_presence_reports_whether_entry_already_existed`
- `cargo test -p ecaz --lib turboquant_dispatch_uses_lut_for_no_qjl_4bit_lane`
- `cargo pgrx test pg18 test_ec_ivf_rescan_reuses_cached_prod_quantizer`

All passed.

## Conclusion

A4 and A5 are closed for the current branch. The next merge-gate item in sequence is A1, the cost-model constant audit.
