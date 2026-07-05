# Task 50 Packet 002: Unsafe Block Count Tooling

## Code Under Review

- Commit: `621b5749b550347d24e7d4912ae14ee8721e87a8`.
- Task: `plan/tasks/50-unsafe-structural-reduction.md`.
- Scope: add the direct unsafe-block count tool required before Slice 1a.

## Changes

- Added `scripts/unsafe_block_count.sh`.
- Added `make unsafe-block-count`.
- The script defaults to `src`, accepts optional scoped path arguments, uses
  `rg --count-matches --with-filename` when available, and falls back to
  `grep -R -E -H -c` when `rg` is absent.
- Output is a stable per-file table sorted by descending count, then path:

```text
 134 src/am/ec_ivf/page.rs
 102 src/am/ec_ivf/scan.rs
```

## Validation

Artifacts are under `artifacts/`:

- `unsafe-block-count-src.log`: full `src/` count.
- `unsafe-block-count-scoped.log`: scoped count for `src/am/ec_ivf/page.rs`
  and `src/am/ec_ivf/scan.rs`.
- `unsafe-block-count-grep-fallback.log`: fallback path run with a `PATH` that
  excludes the Codex-provided `rg`.
- `script-syntax.log`: `bash -n` validation.
- `manifest.md`: command and result metadata.

Top full-count rows:

```text
 356 src/am/ec_hnsw/scan_debug.rs
 226 src/am/ec_hnsw/scan.rs
 203 src/am/ec_hnsw/build_parallel.rs
 160 src/am/ec_spire/dml_frontdoor/mod.rs
 134 src/am/ec_ivf/page.rs
```

## Tests / Benches

- Runtime tests: skipped. This is shell/Make tooling only.
- Benchmarks: skipped. No hot Rust path is touched.

## Follow-Up

Task 50 implementation packets should now capture:

```sh
make unsafe-block-count > reviews/task-50/NNN-topic/artifacts/block-count-before.log
make unsafe-block-count > reviews/task-50/NNN-topic/artifacts/block-count-after.log
```

For narrow packets, use `PATHS='file1 file2 ...'` to capture touched-file-only
counts as supporting evidence.
