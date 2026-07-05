# Review Request: HNSW Scan Setup and Cache Safety

Head: `ce7858b391c1ba4ff246d91f9c569e49c603171e`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/012-hnsw-scan-setup-cache-safety/request.md`
- `reviews/task-35/012-hnsw-scan-setup-cache-safety/artifacts/*`

What changed:
- Documented HNSW scan callback boundaries for `ambeginscan`, `amrescan`,
  `amgettuple`, and `amendscan`.
- Documented scan setup boundaries for runtime format validation, parallel
  scan worker slot attach/publish/release, heap rerank relation/snapshot
  resolution, source-attribute lookup, EXPLAIN counter extraction, and query
  palloc/pfree ownership.
- Added small scan-owned pointer helpers for `Box::into_raw`/`Box::from_raw`
  and cached quantizer `Arc::into_raw`/`Arc::from_raw` cleanup.
- Added safe wrappers for scan-owned frontier, visited, emitted, quantizer, and
  cache pointer access so repeated raw pointer dereferences are localized.
- Fixed a borrow issue introduced by the cache helper refactor by cloning the
  cached `Arc` before recording a graph-element cache hit.

Baseline result:
- Start: 3,329 entries across 102 files.
- End: 3,264 entries across 102 files.
- Net reduction: 65 baseline entries.
- `src/am/ec_hnsw/scan.rs` start/end: 258 entries to 193 entries.

Review focus:
- Confirm the scan-owned raw pointer helpers match the actual allocation
  sources and are not being used for PostgreSQL-owned memory.
- Confirm the parallel scan comments accurately tie state pointer, worker slot,
  and rescan epoch together.
- Confirm the heap rerank comments distinguish borrowed executor pointers from
  guard-owned fallback relation/snapshot state.
- Confirm the graph-element cache hit fix preserves behavior while satisfying
  Rust's borrow rules.

Validation:
- `make unsafe-baseline-report` before baseline update
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/check_unsafe_comments.sh` before baseline update
  - artifact: `artifacts/unsafe-audit-before-baseline-update.log`
  - result: expected failure due shifted `src/am/ec_hnsw/scan.rs` line numbers
    before the baseline refresh.
- `git diff -- src/am/ec_hnsw/scan.rs`
  - artifact: `artifacts/scan-rs-diff-before-baseline.patch`
- `bash scripts/check_unsafe_comments.sh --update-baseline`
  - artifact: `artifacts/unsafe-baseline-update.log`
- `make unsafe-baseline-report` after initial baseline update
  - artifact: `artifacts/unsafe-baseline-after.log`
- `bash scripts/check_unsafe_comments.sh` after initial baseline update
  - artifact: `artifacts/unsafe-audit-after.log`
  - result: passed with no output.
- `cargo fmt --all`
  - artifact: `artifacts/cargo-fmt.log`
  - result: passed; rustfmt emitted existing stable-toolchain warnings for
    unstable `rustfmt.toml` options.
- `bash scripts/check_unsafe_comments.sh --update-baseline` after formatting
  - artifact: `artifacts/unsafe-baseline-update-after-fmt.log`
- `make unsafe-baseline-report` final
  - artifact: `artifacts/unsafe-baseline-final.log`
- `bash scripts/check_unsafe_comments.sh` final
  - artifact: `artifacts/unsafe-audit-final.log`
  - result: passed with no output.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18-bench.log`
  - result: failed with a borrow error in the graph-element cache helper.
- `cargo fmt --all` after borrow fix
  - artifact: `artifacts/cargo-fmt-after-borrow-fix.log`
  - result: passed; rustfmt emitted existing stable-toolchain warnings for
    unstable `rustfmt.toml` options.
- `bash scripts/check_unsafe_comments.sh --update-baseline` after borrow fix
  - artifact: `artifacts/unsafe-baseline-update-after-borrow-fix.log`
- `make unsafe-baseline-report` after borrow fix
  - artifact: `artifacts/unsafe-baseline-final-after-borrow-fix.log`
- `bash scripts/check_unsafe_comments.sh` after borrow fix
  - artifact: `artifacts/unsafe-audit-final-after-borrow-fix.log`
  - result: passed with no output.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18-bench-after-borrow-fix.log`
  - result: passed with existing warnings from `src/am/common/parallel.rs` and
    `src/am/mod.rs`.
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
  - result: passed with no output.

Tests skipped:
- PostgreSQL runtime tests were not run. This packet changes HNSW scan setup
  documentation and local pointer helpers; compile coverage was run with PG18.
