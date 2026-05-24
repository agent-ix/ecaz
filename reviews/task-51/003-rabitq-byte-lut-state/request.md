# Review Request: RaBitQ Byte-LUT State Specialization

- task: Task 51, IVF + RaBitQ optimization round
- packet: `reviews/task-51/003-rabitq-byte-lut-state`
- code commit: `e9933e6b7431f24568ac49efd2c249e5c78efa9f`
- scope: RaBitQ prepared-query state used by `ec_ivf`

## Summary

`PreparedEstimator` and `RaBitQScorer` previously allocated a boxed 256 x 8 f32 byte LUT for every prepared query, even when `bits_per_dim != 1`. The non-bits=1 paths never read that table, so this was an avoidable 8 KB allocation/memset on the default `quant_bits = 4` path.

This change makes the bits=1 byte LUT optional:

- bits=1 prepared state keeps the byte LUT and the aarch64 bits=1 NEON path uses it exactly as before.
- bits=2/4/8 prepared state stores `None` and skips the allocation.
- the scalar helper no longer creates a stack zero table just to satisfy the generic call shape.

The scoring formula, packed code layout, and `ec_ivf` scan behavior are unchanged.

## Files Changed

- `src/quant/rabitq.rs`
  - changed `bits1_byte_lut` fields to `Option<Box<[[f32; 8]; 256]>>`
  - passes `Option<&[[f32; 8]; 256]>` through scorer helpers
  - keeps the bits=1 NEON dispatch guarded on both `bits == 1` and `Some(byte_lut)`
  - added a focused test that bits=1 prepared queries keep the LUT while bits=4 prepared queries do not
  - ran `rustfmt` on the touched file, which also reflowed nearby NEON table literals

## Validation

Packet-local artifact metadata is in `artifacts/manifest.md`.

- `cargo check --lib --no-default-features --features pg18`: passed
- `cargo test --lib prepared_queries_only_keep_bits1_byte_lut_for_bits1 --no-run --no-default-features --features pg18`: passed
- `rustfmt --check src/quant/rabitq.rs`: passed
- `git diff --check -- src/quant/rabitq.rs`: passed
- `cargo pgrx install --test --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`: passed
- isolated PG18 smoke: passed after building both `quant_bits = 1` and `quant_bits = 4` RaBitQ IVF indexes

No AWS was used. This is a local-only IVF/RaBitQ checkpoint.
