# Artifact Manifest: HNSW Graph Tuple Loader Safety

Head SHA: `3bea2ceda9b15d115776652aa34074daa0a4ba99`

Task bucket: `reviews/task-35`
Packet path: `reviews/task-35/020-hnsw-graph-tuple-loader-safety/`

Slice scope: HNSW graph tuple loader and tuple-ref callback boundaries in
`src/am/ec_hnsw/graph.rs`; baseline entries for that file in
`scripts/unsafe_comment_baseline.txt`.

Surface: PG18, ec_hnsw access method. No corpus, no benchmark.
This is a documentation-only slice; no isolated/shared-table choice applies.
Timestamps are local Pacific time from artifact mtimes.

## Artifacts

- `unsafe-baseline-before.log`
  - command: `make unsafe-baseline-report`
  - timestamp: `2026-05-18 21:48:54 -0700`
  - key line: `entries: 3071`, `files: 101`.
- `unsafe-audit-before-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - timestamp: `2026-05-18 21:49:05 -0700`
  - result: expected failure from line drift before updating the baseline.
- `graph-rs-diff-before-baseline.patch`
  - command: `git diff -- src/am/ec_hnsw/graph.rs`
  - timestamp: `2026-05-18 21:48:54 -0700`
  - result: retained patch snapshot before baseline rewrite.
- `cargo-fmt.log`
  - command: `cargo fmt --all`
  - timestamp: `2026-05-18 21:49:31 -0700`
  - result: passed; only stable-toolchain warnings for unstable
    `rustfmt.toml` options.
- `unsafe-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - timestamp: `2026-05-18 21:50:03 -0700`
  - key line: `wrote scripts/unsafe_comment_baseline.txt with 3050 entries`.
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - timestamp: `2026-05-18 21:50:07 -0700`
  - key line: `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 13.45s`.
  - result: passed; pre-existing unused-import warnings in
    `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `unsafe-audit-after.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - timestamp: `2026-05-18 21:50:19 -0700`
  - result: passed with no output.
- `unsafe-baseline-after.log`
  - command: `make unsafe-baseline-report`
  - timestamp: `2026-05-18 21:50:19 -0700`
  - key line: `entries: 3050`, `files: 101`.
- `git-diff-check.log`
  - command: `git diff --check`
  - timestamp: `2026-05-18 21:50:19 -0700`
  - result: passed with no output.

## Key result lines cited by `request.md`

- Baseline start: 3,071 entries, 101 files.
- Baseline end: 3,050 entries, 101 files.
- Net reduction: 21 baseline entries.
- `src/am/ec_hnsw/graph.rs` start: 56 entries.
- `src/am/ec_hnsw/graph.rs` end: 35 entries.
