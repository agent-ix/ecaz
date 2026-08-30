# Task 231 Packet 003 artifact manifest

- Head SHAs:
  - format: `1e0d5906abfa5a586091ca51b4ccf1a48690f37f`
  - persisted selector: `c644b3fb0cc7bad7027bd51d277a6578e69b81c1`
  - block-zero metadata: `95c974ec61c1918d84136daafdfbe040f8f6ed6d`
  - raw relation/WAL reader: `ee25eb7d92112697badc87944dd92ea1ee4a38e3f`
  - handoff, Ready admission, batched production reader, and seq-01 fixes:
    `65f166bf64664127ed7dfe52db9999145576c081`
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

## Sequence 05 reviewer-finding evidence

- Timestamp: `2026-08-29T21:53:40-07:00`
- Head SHA: `65f166bf64664127ed7dfe52db9999145576c081`
- Lane: local Intel development host, PostgreSQL 18 / pgrx 0.17
- Fixture/storage format: isolated Task 231 fixed-stride V1 packed and aligned
  multiblock relations plus the end-to-end three-node handoff fixture.
- Isolation: one relation per raw-store test and one generation fixture; no
  shared-table benchmark surface. These are correctness/cost gates, not the
  Packet 005 10k/50k/100k A/B evidence.

### `fixed-stride-decode-microbench.log`

- Command: `cargo test --release -p ecaz fixed_stride_decode_verification_cost_report --lib -- --ignored --nocapture`
- SHA-256: `154c0b199a3747713816fd3016ca4d1c855c994edea31199fc74790d9ac94827`
- Result: `1 passed; 0 failed`.
- Key line: dimensions 768, degree 64, code length 96, stride 9,904 bytes,
  10,000 iterations: fast `1,174 ns/node`; fully verified `27,497 ns/node`;
  verified/default ratio `23.42x`.

### `fixed-stride-receipt-manifest-tests.log`

- Command: `cargo test -p ecaz --lib fixed_stride`
- SHA-256: `62de8a3b028e05da56444ff49ce72c976f95e14fe4f29d77e8357825d6358f2d`
- Result: `7 passed; 0 failed; 1 ignored`.
- Covered result lines: all pure fixed-stride format/descriptor/EFM1 gates,
  Ready receipt V4 and manifest V5 binding, and the PG18 stage/seal/receipt/
  topology fixture.

### `fixed-stride-handoff-pg18.log`

- Command: `cargo pgrx test pg18 fixed_stride`
- SHA-256: `cfff0f8ba6771021da36db335cfb7c2fd561ac34b797bdc2ea46f913dc7cf05b`
- Result: `9 passed; 0 failed; 1 ignored` in the filtered library surface.
- Covered result lines: packed and multiblock store round-trip and batched
  coalescing; zero-line-pointer heapam `SELECT`/`ANALYZE`; relation-level
  corruption/truncation fail-closed matrix; end-to-end relation creation,
  handoff, seal, Ready admission, topology, abort, and node-store drop.

### `fixed-stride-store-clippy.log`

- Command: `cargo clippy --lib --no-default-features --features pg18 -- -D warnings -A clippy::collapsible-if -A clippy::unnecessary-unwrap`
- SHA-256: `f539e5706209d44540b5587578cb13ba4cdf9bc6defb4662f97350afacb4b160`
- Result: PASS on the repository MSRV-aware PG18 library surface.
- Exceptions: the two allows are the same pre-existing unrelated lints noted
  above; no Task 231 warning is suppressed.
