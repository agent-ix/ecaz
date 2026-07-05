# Artifact Manifest: HNSW Traversal State Safety

Head SHA: `e88aeffd86c225ba51887cd8df13346bba16e26a`

Task bucket: `reviews/task-35`
Packet path: `reviews/task-35/015-hnsw-traversal-state-safety/`

Slice scope: HNSW graph traversal cursor, grouped/windowed result
materialization, scan-owned traversal state reset/free helpers, and PG18
read-stream lifecycle in `src/am/ec_hnsw/scan.rs`; baseline entries for that
file in `scripts/unsafe_comment_baseline.txt`.

Surface: PG18, ec_hnsw access method. No corpus, no benchmark.
This is a doc/refactor slice; no isolated/shared-table choice applies.
Timestamps are local Pacific time from artifact mtimes.

## Artifacts

- `unsafe-baseline-before.log`
  - command: `make unsafe-baseline-report`
  - timestamp: `2026-05-18 21:22:57 -0700`
  - key line: `entries: 3180`, `files: 102`.
- `unsafe-audit-before-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - timestamp: `2026-05-18 21:23:08 -0700`
  - result: expected failure from line drift before updating the baseline.
- `scan-rs-diff-before-baseline.patch`
  - command: `git diff -- src/am/ec_hnsw/scan.rs`
  - timestamp: `2026-05-18 21:22:57 -0700`
  - result: retained patch snapshot before baseline rewrite.
- `cargo-fmt.log`
  - command: `cargo fmt --all`
  - timestamp: `2026-05-18 21:23:15 -0700`
  - result: passed; only stable-toolchain warnings for unstable
    `rustfmt.toml` options.
- `unsafe-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - timestamp: `2026-05-18 21:23:36 -0700`
  - key line: `wrote scripts/unsafe_comment_baseline.txt with 3154 entries`.
- `unsafe-audit-after.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - timestamp: `2026-05-18 21:23:43 -0700`
  - result: passed with no output.
- `unsafe-baseline-after.log`
  - command: `make unsafe-baseline-report`
  - timestamp: `2026-05-18 21:23:43 -0700`
  - key line: `entries: 3154`, `files: 102`.
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - timestamp: `2026-05-18 21:23:58 -0700`
  - key line: `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 16.38s`.
  - result: passed; pre-existing unused-import warnings in
    `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `git-diff-check.log`
  - command: `git diff --check`
  - timestamp: `2026-05-18 21:24:07 -0700`
  - result: passed with no output.

## Key result lines cited by `request.md`

- Baseline start: 3,180 entries, 102 files.
- Baseline end: 3,154 entries, 102 files.
- Net reduction: 26 baseline entries.
- `src/am/ec_hnsw/scan.rs` start: 109 entries.
- `src/am/ec_hnsw/scan.rs` end: 83 entries.
