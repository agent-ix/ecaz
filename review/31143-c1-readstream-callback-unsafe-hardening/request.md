# Review Request: ReadStream Callback Unsafe Hardening

Head: `ef554f4e7445da3842217ad1ee2ac4cdd10cabae`

Scope:
- `scripts/unsafe_comment_baseline.txt`
- `src/am/common/stream.rs`

What changed:
- Made `write_stream_block` a safe nullable helper. It now owns the only raw
  per-buffer write and documents the PostgreSQL ReadStream storage invariant.
- Added fail-closed null handling for `callback_private_data` in graph, linear,
  and block-sequence ReadStream callbacks. A null private pointer now returns
  `InvalidBlockNumber` instead of being dereferenced.
- Kept the PostgreSQL C callback boundary guarded with `pgrx_extern_c_guard`,
  but documented that boundary and the callback-private state type invariant at
  each remaining unsafe access.
- Removed all four `src/am/common/stream.rs` entries from the unsafe-comment
  baseline.

Baseline result:
- Start: 4,799 entries across 113 files.
- End: 4,795 entries across 112 files.
- Net reduction: 4 entries and 1 file.

Review focus:
- Confirm returning `InvalidBlockNumber` is the right fail-closed behavior for
  unexpected null ReadStream callback-private data.
- Confirm `write_stream_block` is safe to expose as a normal helper because it
  handles null and only writes a single `BlockNumber` to PostgreSQL-owned
  per-buffer storage.
- Confirm the remaining unsafe comments describe real ReadStream callback
  invariants rather than merely restating the operation.

Validation:
- `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-903.txt`
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/unsafe_baseline_report.sh`
  - artifact: `artifacts/unsafe-baseline-after.log`
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/audit-unsafe.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check HEAD^ HEAD`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`

Notes:
- `cargo check` passed with existing warnings from PostgreSQL headers and
  currently unused SPIRE re-exports in `src/am/mod.rs`.
