# Task 231 Packet 003 artifact manifest

- Head SHAs:
  - format: `1e0d5906abfa5a586091ca51b4ccf1a48690f37f`
  - persisted selector: `c644b3fb0cc7bad7027bd51d277a6578e69b81c1`
  - block-zero metadata: `95c974ec61c1918d84136daafdfbe040f8f6ed6d`
  - raw relation/WAL reader: `ee25eb7d92112697badc87944dd92ea1ee4a38e3f`
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

## `fixed-stride-store-pg18.log`

- Command: `cargo pgrx test pg18 fixed_stride_store_round_trips_packed_and_multiblock_nodes`
- Timestamp: `2026-08-29T20:18:43-07:00`
- SHA-256: `569f42daf7c6d5c031849f255a4ea6a821c1e4c0dc8e9fac46ff6bdfbf15d804`
- Result: `1 passed; 0 failed; 2650 filtered out`
- Fixture/storage format: isolated one-relation-per-layout PG18 fixtures;
  fixed-stride V1 packed and aligned multi-block raw main forks.
- Covered result lines: EFM1 initialization/admission, autovacuum disabled,
  GenericXLog append, tail retry, node read/identity, and multiblock assembly.

## `fixed-stride-store-clippy.log`

- Command: `cargo clippy --lib --no-default-features --features pg18 -- -D warnings -A clippy::collapsible-if -A clippy::unnecessary-unwrap`
- Timestamp: `2026-08-29T20:18:43-07:00`
- SHA-256: `7ab699a6d110581fc3d9a07867fd4a9dbe12de2bfc359e05c8805616a09f623a`
- Result: PASS.
- Exceptions: the two explicit allows are for pre-existing unrelated warnings
  in `ambuild.rs` and the Task 230 head-sizing descriptor path; neither file is
  changed by raw relation checkpoint `ee25eb7d9`.
