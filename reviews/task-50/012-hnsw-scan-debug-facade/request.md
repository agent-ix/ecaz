# Task 50 Packet 012: HNSW Scan Debug Facade

## Code Under Review

- Code commit: `202f9929f2407f503d3e0b6bd02a182ca1359726`
- File touched: `src/am/ec_hnsw/scan_debug.rs`
- Task: `plan/tasks/50-unsafe-structural-reduction.md`

## Scope

This packet processes the top-density HNSW debug scan module from the Task 50
top-15 map. The change introduces a small debug-only facade for repeated HNSW
AM scan operations and scan descriptor field access:

- `debug_am_begin_scan`
- `debug_am_rescan`
- `debug_am_gettuple`
- `debug_am_end_scan`
- `debug_index_scan_end`
- scan opaque and heap TID accessors

The unsafe contracts for those repeated debug probes now live in one helper
block instead of being repeated at each callsite.

## Unsafe Block Count

| file | before | after | delta | percent | top-15 target status |
| --- | ---: | ---: | ---: | ---: | --- |
| `src/am/ec_hnsw/scan_debug.rs` | 356 | 129 | -227 | -63.8% | Meets >=30% target |

This is not a scoring, traversal, or cache hot-path production change. The file
is gated to tests / `pg_test`, so no benchmark lane is claimed for this packet.

## Validation

- `make unsafe-block-count PATHS='src/am/ec_hnsw/scan_debug.rs'`
  - before: `356 src/am/ec_hnsw/scan_debug.rs`
  - after: `129 src/am/ec_hnsw/scan_debug.rs`
- `rustfmt --edition 2021 --check src/am/ec_hnsw/scan_debug.rs`: passed with
  existing stable-rustfmt warnings about unstable config keys.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with existing warnings outside this slice.
- `git diff --check`: passed.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`:
  still blocked by the existing repo-wide clippy backlog; first failures remain
  outside this slice.
- `cargo fmt --all --check`: still reports pre-existing formatting drift in
  files outside this slice; touched-file rustfmt check passed.
- `cargo test scan_debug --lib --no-default-features --features pg18`: built the
  test binary, then failed to launch outside PostgreSQL with
  `undefined symbol: CacheRegisterRelcacheCallback`.

## Artifacts

- `artifacts/block-count-before.log`
- `artifacts/block-count-after.log`
- `artifacts/rustfmt-touched-check.log`
- `artifacts/cargo-fmt-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-clippy-pg18.log`
- `artifacts/cargo-test-scan-debug.log`
- `artifacts/git-diff-check.log`
- `artifacts/manifest.md`
