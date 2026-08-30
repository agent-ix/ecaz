# Task 231 Packet 003 artifact manifest

- Head SHAs:
  - format: `1e0d5906abfa5a586091ca51b4ccf1a48690f37f`
  - persisted selector: `c644b3fb0cc7bad7027bd51d277a6578e69b81c1`
  - block-zero metadata: `95c974ec61c1918d84136daafdfbe040f8f6ed6d`
- Task/packet: `reviews/task-231/003-format-and-reader/`
- Timestamp: `2026-08-29T19:34:24-07:00`
- Lane: local Intel development host, Rust unit format gate
- Fixture/storage format: pure fixed-stride V1 packed/one-page/multi-block
  byte fixtures; no PostgreSQL relation or benchmark corpus
- Isolation: format-only; no index/table fixture and no shared-table surface

## `fixed-stride-format-tests.log`

- Command: `cargo test -p ecaz fixed_stride`
- SHA-256: `6bf444e6691b10d29987ddd8165832c15148d25399e26987d8e4defb058a183f`
- Result: `5 passed; 0 failed; 2635 filtered out`
- Covered result lines: packed/one-page/multi-block arithmetic; persisted
  layout re-derivation; generation tag binding; node round-trip and corruption;
  packed and every multi-block page-envelope segment.

## `fixed-stride-descriptor-tests.log`

- Command: `cargo test -p ecaz --lib fixed_stride`
- Timestamp: `2026-08-29T19:46:19-07:00`
- SHA-256: `800ab526c185536e106494e37cb3c16b91715550e81a7c463486a4a7f26eb431`
- Result: `6 passed; 0 failed; 2635 filtered out`
- Covered result lines: the five format tests above plus generation descriptor
  V5 round-trip, digest corruption, layout/codec re-derivation, V3 graph-record
  binding, and layout mutual exclusion.

## `fixed-stride-metadata-tests.log`

- Command: `cargo test -p ecaz --lib fixed_stride`
- Timestamp: `2026-08-29T19:53:02-07:00`
- SHA-256: `af1ecd96db316ad6ac39232e82f33b142683f6f656896708eaeb45faf5e23893`
- Result: `6 passed; 0 failed; 2635 filtered out`
- Covered result lines: all prior format and descriptor gates plus EFM1
  block-zero metadata round-trip and digest-corruption rejection.
