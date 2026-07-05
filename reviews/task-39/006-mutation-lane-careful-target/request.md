# Review request: pure-Rust mutation lane target fix

## Scope

Follow-up to packet `005`: the successful SIMD mutation command targeted `ecaz-careful-hardening` directly, but `scripts/hardening.sh mutants --file src/quant/simd.rs` still treated the file as part of the root `ecaz` package. On this host that falls back into the pgrx test binary and aborts at load.

Code checkpoint: `19de557cf1f0c168f2df3c71cbd0408454ec4197`

Change under review:

- For `src/quant/*` and `src/storage/page.rs`, `run_mutants_lane` now maps the file to the careful package path (`hardening/careful/src/../../../...`) and passes `--package ecaz-careful-hardening`.

## Validation

- `artifacts/bash-n-hardening.log`: `bash -n scripts/hardening.sh`
- `artifacts/make-n-mutants-simd.log`: confirms the Make lane still calls `scripts/hardening.sh mutants --file src/quant/simd.rs`
- `artifacts/careful-simd-mutants-list.log`: confirms cargo-mutants sees the 9 SIMD mutants through the careful package path

## Notes

I did not rerun the full mutation execution here because packet `005` already contains the successful final run:

`9 mutants tested in 19s: 6 caught, 3 unviable`
