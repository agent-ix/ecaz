# Artifact Manifest: HNSW Successor Prefetch Safety

Head SHA: `2757268ed97d6b27ed0623bbe5ccb28a1bfeca11`

Task bucket: `reviews/task-35`
Packet path: `reviews/task-35/014-hnsw-successor-prefetch-safety/`

Slice scope: HNSW PG18 graph prefetch and successor traversal scoring in
`src/am/ec_hnsw/scan.rs`; baseline entries for that file in
`scripts/unsafe_comment_baseline.txt`.

Surface: PG18, ec_hnsw access method. No corpus, no benchmark.
This is a doc/refactor slice; no isolated/shared-table choice applies.
Timestamps are local Pacific time from artifact mtimes.

## Artifacts

- `unsafe-baseline-before.log`
  - command: `make unsafe-baseline-report`
  - timestamp: `2026-05-18 21:17:46 -0700`
  - key line: `entries: 3212`, `files: 102`.
- `unsafe-audit-before-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - timestamp: `2026-05-18 21:17:58 -0700`
  - result: expected failure from line drift before updating the baseline.
- `scan-rs-diff-before-baseline.patch`
  - command: `git diff -- src/am/ec_hnsw/scan.rs`
  - timestamp: `2026-05-18 21:17:46 -0700`
  - result: retained patch snapshot before baseline rewrite.
- `cargo-fmt.log`
  - command: `cargo fmt --all`
  - timestamp: `2026-05-18 21:18:06 -0700`
  - result: passed; only stable-toolchain warnings for unstable
    `rustfmt.toml` options.
- `unsafe-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - timestamp: `2026-05-18 21:18:12 -0700`
  - key line: `wrote scripts/unsafe_comment_baseline.txt with 3180 entries`.
- `unsafe-baseline-update-after-fmt.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - timestamp: `2026-05-18 21:18:42 -0700`
  - key line: `wrote scripts/unsafe_comment_baseline.txt with 3180 entries`.
- `unsafe-audit-after.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - timestamp: `2026-05-18 21:18:50 -0700`
  - result: passed with no output.
- `unsafe-baseline-final.log`
  - command: `make unsafe-baseline-report`
  - timestamp: `2026-05-18 21:18:50 -0700`
  - key line: `entries: 3180`, `files: 102`.
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - timestamp: `2026-05-18 21:19:04 -0700`
  - key line: `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 15.40s`.
  - result: passed; pre-existing unused-import warnings in
    `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `git-diff-check.log`
  - command: `git diff --check`
  - timestamp: `2026-05-18 21:19:15 -0700`
  - result: passed with no output.
- `unsafe-baseline-after.log`
  - command: `make unsafe-baseline-report`
  - timestamp: `2026-05-18 21:19:32 -0700`
  - key line: `entries: 3180`, `files: 102`.

## Key result lines cited by `request.md`

- Baseline start: 3,212 entries, 102 files.
- Baseline end: 3,180 entries, 102 files.
- Net reduction: 32 baseline entries.
- `src/am/ec_hnsw/scan.rs` start: 141 entries.
- `src/am/ec_hnsw/scan.rs` end: 109 entries.
