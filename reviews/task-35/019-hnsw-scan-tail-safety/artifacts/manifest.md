# Artifact Manifest: HNSW Scan Tail Safety

Head SHA: `bc51924f88dcce1a005f648acdceb8e204e7585e`

Task bucket: `reviews/task-35`
Packet path: `reviews/task-35/019-hnsw-scan-tail-safety/`

Slice scope: HNSW scan raw query slices, test-only parallel scan storage,
test-only cache reset assertions, and Miri raw opaque scoring helper in
`src/am/ec_hnsw/scan.rs`; baseline entries for that file in
`scripts/unsafe_comment_baseline.txt`.

Surface: PG18, ec_hnsw access method. No corpus, no benchmark.
This is a documentation-only slice; no isolated/shared-table choice applies.
Timestamps are local Pacific time from artifact mtimes.

## Artifacts

- `unsafe-baseline-before.log`
  - command: `make unsafe-baseline-report`
  - timestamp: `2026-05-18 21:42:25 -0700`
  - key line: `entries: 3096`, `files: 102`.
- `unsafe-audit-after-comments-before-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - timestamp: `2026-05-18 21:42:25 -0700`
  - result: passed with no output.
- `scan-rs-diff-before-baseline.patch`
  - command: `git diff -- src/am/ec_hnsw/scan.rs`
  - timestamp: `2026-05-18 21:42:25 -0700`
  - result: retained patch snapshot before baseline rewrite.
- `cargo-fmt.log`
  - command: `cargo fmt --all`
  - timestamp: `2026-05-18 21:42:48 -0700`
  - result: passed; only stable-toolchain warnings for unstable
    `rustfmt.toml` options.
- `unsafe-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - timestamp: `2026-05-18 21:43:12 -0700`
  - key line: `wrote scripts/unsafe_comment_baseline.txt with 3071 entries`.
- `unsafe-audit-final.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - timestamp: `2026-05-18 21:43:25 -0700`
  - result: passed with no output.
- `unsafe-baseline-after.log`
  - command: `make unsafe-baseline-report`
  - timestamp: `2026-05-18 21:43:25 -0700`
  - key line: `entries: 3071`, `files: 101`.
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - timestamp: `2026-05-18 21:43:38 -0700`
  - key line: `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 13.68s`.
  - result: passed; pre-existing unused-import warnings in
    `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `git-diff-check.log`
  - command: `git diff --check`
  - timestamp: `2026-05-18 21:43:57 -0700`
  - result: passed with no output.

## Key result lines cited by `request.md`

- Baseline start: 3,096 entries, 102 files.
- Baseline end: 3,071 entries, 101 files.
- Net reduction: 25 baseline entries.
- `src/am/ec_hnsw/scan.rs` start: 25 entries.
- `src/am/ec_hnsw/scan.rs` end: 0 entries.
